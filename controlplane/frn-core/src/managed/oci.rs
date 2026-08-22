//! Read-only OCI registry client to discover deployable chart versions and the
//! form schemas the charts ship under `frn/`.
//!
//! Speaks the Docker Registry v2 API directly with `reqwest` (no `helm` binary
//! in the server image), including the `WWW-Authenticate` Bearer dance. The
//! charts registry is private, so a read credential is required
//! ([`RegistryCredentials`]); without one, discovery is skipped.

use std::io::Read;

use serde::Deserialize;

use crate::authorization::Authorize;
use crate::managed::{Catalog, ManagedServices};

/// A chart version discovered in the registry, with the metadata needed to
/// register it as a deployable [`super::ManagedServiceVersion`].
#[derive(Debug, Clone)]
pub struct DiscoveredChart {
    /// The chart version (the OCI tag), e.g. `0.1.0`.
    pub chart_version: String,
    /// The application version from `Chart.yaml` (`appVersion`), if declared.
    pub app_version: Option<String>,
    /// Contents of `frn/configurable-values.schema.json`, if the chart ships it.
    pub configurable_values_schema: Option<serde_json::Value>,
    /// Contents of `frn/ui.schema.json`, if the chart ships it.
    pub ui_schema: Option<serde_json::Value>,
    /// Contents of `frn/connection-info.schema.json`, if the chart ships it.
    pub connection_info_schema: Option<serde_json::Value>,
}

/// Read credentials for the charts registry (GitLab Container Registry).
///
/// A GitLab deploy/access token with the `read_registry` scope: `username` is
/// the token's username, `password` the token value.
#[derive(Debug, Clone)]
pub struct RegistryCredentials {
    pub username: String,
    pub password: String,
}

/// Errors raised while discovering charts from the registry.
#[derive(Debug, thiserror::Error)]
pub enum OciError {
    #[error("invalid OCI reference: {0}")]
    InvalidReference(String),
    #[error("registry request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("registry authentication failed: {0}")]
    Auth(String),
    #[error("registry returned {status} for {url}")]
    Status { status: u16, url: String },
    #[error("unexpected registry payload: {0}")]
    Payload(String),
    #[error("failed to unpack chart archive: {0}")]
    Unpack(String),
}

/// A parsed OCI reference: `oci://<registry>/<repository>`.
struct ChartReference {
    registry: String,
    repository: String,
}

impl ChartReference {
    /// Parses an `oci://host/path/to/chart` reference.
    fn parse(oci_reference: &str) -> Result<Self, OciError> {
        let without_scheme = oci_reference
            .strip_prefix("oci://")
            .ok_or_else(|| OciError::InvalidReference(oci_reference.to_owned()))?;
        let (registry, repository) = without_scheme
            .split_once('/')
            .ok_or_else(|| OciError::InvalidReference(oci_reference.to_owned()))?;
        if registry.is_empty() || repository.is_empty() {
            return Err(OciError::InvalidReference(oci_reference.to_owned()));
        }
        Ok(Self {
            registry: registry.to_owned(),
            repository: repository.to_owned(),
        })
    }
}

/// OCI image manifest (subset): its layers reference the chart archive blob.
#[derive(Debug, Deserialize)]
struct OciManifest {
    layers: Vec<OciDescriptor>,
}

#[derive(Debug, Deserialize)]
struct OciDescriptor {
    digest: String,
    #[serde(rename = "mediaType")]
    media_type: String,
}

/// `Chart.yaml` (subset): we only need the application version.
#[derive(Debug, Deserialize)]
struct ChartYaml {
    #[serde(rename = "appVersion")]
    app_version: Option<String>,
}

/// Media type of the Helm chart content layer inside an OCI artifact.
const HELM_CHART_LAYER_MEDIA_TYPE: &str = "application/vnd.cncf.helm.chart.content.v1.tar+gzip";

/// Read-only client for a single chart's OCI repository.
pub struct OciChartClient {
    http: reqwest::Client,
    reference: ChartReference,
    credentials: Option<RegistryCredentials>,
}

impl OciChartClient {
    /// Builds a client for the given `oci://…` chart reference.
    pub fn new(
        oci_reference: &str,
        credentials: Option<RegistryCredentials>,
    ) -> Result<Self, OciError> {
        Ok(Self {
            http: reqwest::Client::new(),
            reference: ChartReference::parse(oci_reference)?,
            credentials,
        })
    }

    /// Base URL of the registry v2 API for this repository.
    fn api_base(&self) -> String {
        format!(
            "https://{}/v2/{}",
            self.reference.registry, self.reference.repository
        )
    }

    /// Obtains a Bearer token for a pull, following the registry's
    /// `WWW-Authenticate` challenge (realm/service/scope) and authenticating
    /// with the configured credentials.
    async fn pull_token(&self) -> Result<String, OciError> {
        // Trigger the challenge with an unauthenticated request to the tags API.
        let probe = self
            .http
            .get(format!("{}/tags/list", self.api_base()))
            .send()
            .await?;

        let challenge = probe
            .headers()
            .get(reqwest::header::WWW_AUTHENTICATE)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                OciError::Auth("registry did not present a Bearer challenge".to_owned())
            })?
            .to_owned();

        let (realm, service, scope) = parse_bearer_challenge(&challenge)
            .ok_or_else(|| OciError::Auth(format!("unparseable challenge: {challenge}")))?;

        let mut request = self
            .http
            .get(&realm)
            .query(&[("service", service.as_str()), ("scope", scope.as_str())]);
        if let Some(credentials) = &self.credentials {
            request = request.basic_auth(&credentials.username, Some(&credentials.password));
        }

        let response = request.send().await?;
        if !response.status().is_success() {
            return Err(OciError::Auth(format!(
                "token endpoint returned {}",
                response.status()
            )));
        }

        #[derive(Deserialize)]
        struct TokenResponse {
            token: Option<String>,
            access_token: Option<String>,
        }
        let body: TokenResponse = response.json().await?;
        body.token
            .or(body.access_token)
            .filter(|t| !t.is_empty())
            .ok_or_else(|| OciError::Auth("registry returned an empty token".to_owned()))
    }

    /// Lists the chart's version tags (unordered; the registry returns them
    /// lexically).
    pub async fn list_versions(&self) -> Result<Vec<String>, OciError> {
        let token = self.pull_token().await?;
        let url = format!("{}/tags/list", self.api_base());
        let response = self.http.get(&url).bearer_auth(&token).send().await?;
        if !response.status().is_success() {
            return Err(OciError::Status {
                status: response.status().as_u16(),
                url,
            });
        }

        #[derive(Deserialize)]
        struct TagList {
            tags: Option<Vec<String>>,
        }
        let body: TagList = response.json().await?;
        Ok(body.tags.unwrap_or_default())
    }

    /// Pulls a chart version and extracts its `appVersion` and any `frn/` schemas.
    pub async fn fetch_chart(&self, chart_version: &str) -> Result<DiscoveredChart, OciError> {
        let token = self.pull_token().await?;
        let manifest = self.fetch_manifest(&token, chart_version).await?;

        let layer = manifest
            .layers
            .iter()
            .find(|l| l.media_type == HELM_CHART_LAYER_MEDIA_TYPE)
            .or_else(|| manifest.layers.first())
            .ok_or_else(|| OciError::Payload("manifest has no layers".to_owned()))?;

        let archive = self.fetch_blob(&token, &layer.digest).await?;
        unpack_chart(chart_version, &archive)
    }

    /// Fetches the image manifest for a tag.
    async fn fetch_manifest(&self, token: &str, tag: &str) -> Result<OciManifest, OciError> {
        let url = format!("{}/manifests/{}", self.api_base(), tag);
        let response = self
            .http
            .get(&url)
            .bearer_auth(token)
            .header(
                reqwest::header::ACCEPT,
                "application/vnd.oci.image.manifest.v1+json",
            )
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(OciError::Status {
                status: response.status().as_u16(),
                url,
            });
        }
        response.json().await.map_err(OciError::from)
    }

    /// Fetches a blob (the gzipped chart archive) by digest.
    async fn fetch_blob(&self, token: &str, digest: &str) -> Result<Vec<u8>, OciError> {
        let url = format!("{}/blobs/{}", self.api_base(), digest);
        let response = self.http.get(&url).bearer_auth(token).send().await?;
        if !response.status().is_success() {
            return Err(OciError::Status {
                status: response.status().as_u16(),
                url,
            });
        }
        Ok(response.bytes().await?.to_vec())
    }
}

/// Parses a `WWW-Authenticate: Bearer realm="…",service="…",scope="…"` header
/// into its `(realm, service, scope)` parts.
fn parse_bearer_challenge(header: &str) -> Option<(String, String, String)> {
    let params = header.strip_prefix("Bearer ")?;
    let mut realm = None;
    let mut service = None;
    let mut scope = None;
    for part in params.split(',') {
        let (key, value) = part.split_once('=')?;
        let value = value.trim().trim_matches('"').to_owned();
        match key.trim() {
            "realm" => realm = Some(value),
            "service" => service = Some(value),
            "scope" => scope = Some(value),
            _ => {}
        }
    }
    Some((realm?, service?, scope?))
}

/// Unpacks a gzipped Helm chart archive and reads the metadata we register.
///
/// Helm packages a chart as `<name>/…` inside the tarball; we read
/// `<name>/Chart.yaml` for the app version and the optional `<name>/frn/*.json`
/// schema files, tolerating their absence.
fn unpack_chart(chart_version: &str, gzipped: &[u8]) -> Result<DiscoveredChart, OciError> {
    let decoder = flate2::read::GzDecoder::new(gzipped);
    let mut archive = tar::Archive::new(decoder);

    let mut app_version = None;
    let mut configurable_values_schema = None;
    let mut ui_schema = None;
    let mut connection_info_schema = None;

    let entries = archive
        .entries()
        .map_err(|e| OciError::Unpack(e.to_string()))?;
    for entry in entries {
        let mut entry = entry.map_err(|e| OciError::Unpack(e.to_string()))?;
        let path = entry
            .path()
            .map_err(|e| OciError::Unpack(e.to_string()))?
            .to_string_lossy()
            .into_owned();

        // Strip the leading `<chart-name>/` component to match on the file's
        // location within the chart, regardless of the packaged name.
        let relative = path.split_once('/').map(|(_, rest)| rest).unwrap_or(&path);

        match relative {
            "Chart.yaml" => {
                let mut contents = String::new();
                entry
                    .read_to_string(&mut contents)
                    .map_err(|e| OciError::Unpack(e.to_string()))?;
                let chart: ChartYaml = serde_yaml::from_str(&contents)
                    .map_err(|e| OciError::Payload(format!("invalid Chart.yaml: {e}")))?;
                app_version = chart.app_version;
            }
            "frn/configurable-values.schema.json" => {
                configurable_values_schema = Some(read_json_entry(&mut entry)?);
            }
            "frn/ui.schema.json" => {
                ui_schema = Some(read_json_entry(&mut entry)?);
            }
            "frn/connection-info.schema.json" => {
                connection_info_schema = Some(read_json_entry(&mut entry)?);
            }
            _ => {}
        }
    }

    Ok(DiscoveredChart {
        chart_version: chart_version.to_owned(),
        app_version,
        configurable_values_schema,
        ui_schema,
        connection_info_schema,
    })
}

impl<A: Authorize + Clone> ManagedServices<A> {
    /// Registers every catalogue chart's versions (idempotently). Best-effort per
    /// chart — a registry error is logged and skipped, never fatal, since a
    /// missing version only makes one service undeployable. Skipped without
    /// credentials.
    pub async fn sync_versions_from_registry(
        &self,
        catalog: &Catalog,
        credentials: Option<&RegistryCredentials>,
    ) {
        let Some(credentials) = credentials else {
            tracing::info!("charts registry credentials absent, skipping version discovery");
            return;
        };

        // Bounded concurrency: charts are independent network round-trips, so a
        // slow or 404 chart must not hold up the others.
        use futures::stream::StreamExt as _;
        const MAX_CONCURRENT_CHARTS: usize = 8;

        futures::stream::iter(catalog.managed_services.iter().filter_map(|service| {
            service
                .chart
                .oci_reference
                .as_deref()
                .map(|oci_reference| (service.slug.as_str(), oci_reference))
        }))
        .for_each_concurrent(MAX_CONCURRENT_CHARTS, |(slug, oci_reference)| async move {
            if let Err(error) = self
                .discover_service_versions(slug, oci_reference, credentials)
                .await
            {
                tracing::warn!(
                    service = %slug,
                    oci_reference,
                    %error,
                    "failed to discover chart versions; service may not be deployable"
                );
            }
        })
        .await;
    }

    /// Discovers and registers all versions for a single chart.
    async fn discover_service_versions(
        &self,
        service_slug: &str,
        oci_reference: &str,
        credentials: &RegistryCredentials,
    ) -> Result<(), OciError> {
        let client = OciChartClient::new(oci_reference, Some(credentials.clone()))?;
        // The DB enforces semver on chart_version; skip non-semver tags (upstream
        // image tags like `5.3.2-debian-12-r12`) rather than failing the insert.
        let versions: Vec<String> = client
            .list_versions()
            .await?
            .into_iter()
            .filter(|tag| {
                let is_semver = semver::Version::parse(tag).is_ok();
                if !is_semver {
                    tracing::debug!(service = %service_slug, tag, "skipping non-semver chart tag");
                }
                is_semver
            })
            .collect();
        tracing::info!(
            service = %service_slug,
            count = versions.len(),
            "discovered deployable chart versions"
        );

        for version in versions {
            let chart = client.fetch_chart(&version).await?;
            let mut conn = match self.db.acquire().await {
                Ok(conn) => conn,
                Err(error) => {
                    tracing::warn!(service = %service_slug, %error, "cannot acquire db connection");
                    continue;
                }
            };
            match self
                .register_version(
                    &mut conn,
                    service_slug,
                    &chart.chart_version,
                    chart.app_version.as_deref(),
                    oci_reference,
                    chart.configurable_values_schema.as_ref(),
                    chart.ui_schema.as_ref(),
                    chart.connection_info_schema.as_ref(),
                )
                .await
            {
                Ok(_) => tracing::info!(
                    service = %service_slug,
                    chart_version = %chart.chart_version,
                    "registered chart version"
                ),
                // Already present: idempotent, nothing to do.
                Err(crate::managed::ManagedServiceError::VersionAlreadyExists(_)) => {}
                Err(error) => tracing::warn!(
                    service = %service_slug,
                    chart_version = %chart.chart_version,
                    %error,
                    "failed to register chart version"
                ),
            }
        }
        Ok(())
    }
}

/// Reads a tar entry as a JSON value.
fn read_json_entry<R: Read>(entry: &mut R) -> Result<serde_json::Value, OciError> {
    let mut contents = String::new();
    entry
        .read_to_string(&mut contents)
        .map_err(|e| OciError::Unpack(e.to_string()))?;
    serde_json::from_str(&contents)
        .map_err(|e| OciError::Payload(format!("invalid frn/ schema: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_valid_oci_reference() {
        let reference = ChartReference::parse("oci://registry.gitlab.com/group/charts/vaultwarden")
            .expect("valid reference");
        assert_eq!(reference.registry, "registry.gitlab.com");
        assert_eq!(reference.repository, "group/charts/vaultwarden");
    }

    #[test]
    fn rejects_a_reference_without_scheme() {
        assert!(matches!(
            ChartReference::parse("registry.gitlab.com/group/chart"),
            Err(OciError::InvalidReference(_))
        ));
    }

    #[test]
    fn rejects_a_reference_without_repository() {
        assert!(matches!(
            ChartReference::parse("oci://registry.gitlab.com"),
            Err(OciError::InvalidReference(_))
        ));
    }

    #[test]
    fn parses_a_bearer_challenge() {
        let header = r#"Bearer realm="https://gitlab.com/jwt/auth",service="container_registry",scope="repository:group/chart:pull""#;
        let (realm, service, scope) = parse_bearer_challenge(header).expect("parsed");
        assert_eq!(realm, "https://gitlab.com/jwt/auth");
        assert_eq!(service, "container_registry");
        assert_eq!(scope, "repository:group/chart:pull");
    }

    #[test]
    fn unpacks_a_chart_with_frn_schemas() {
        // Build a minimal gzipped tar: chart/Chart.yaml + chart/frn/*.json.
        let mut tar_buf = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut tar_buf);
            let chart_yaml = b"apiVersion: v2\nname: demo\nversion: 1.2.3\nappVersion: \"9.9.9\"\n";
            append_file(&mut builder, "demo/Chart.yaml", chart_yaml);
            let cfg = br#"{"type":"object"}"#;
            append_file(
                &mut builder,
                "demo/frn/configurable-values.schema.json",
                cfg,
            );
            let ui = br#"{"ui:order":[]}"#;
            append_file(&mut builder, "demo/frn/ui.schema.json", ui);
            builder.finish().expect("tar finish");
        }
        let mut gz = Vec::new();
        {
            use std::io::Write as _;
            let mut encoder =
                flate2::write::GzEncoder::new(&mut gz, flate2::Compression::default());
            encoder.write_all(&tar_buf).expect("gz write");
            encoder.finish().expect("gz finish");
        }

        let chart = unpack_chart("1.2.3", &gz).expect("unpack");
        assert_eq!(chart.chart_version, "1.2.3");
        assert_eq!(chart.app_version.as_deref(), Some("9.9.9"));
        assert!(chart.configurable_values_schema.is_some());
        assert!(chart.ui_schema.is_some());
        assert!(chart.connection_info_schema.is_none());
    }

    #[test]
    fn unpacks_a_chart_without_frn_schemas() {
        let mut tar_buf = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut tar_buf);
            let chart_yaml = b"apiVersion: v2\nname: bare\nversion: 0.1.0\n";
            append_file(&mut builder, "bare/Chart.yaml", chart_yaml);
            builder.finish().expect("tar finish");
        }
        let mut gz = Vec::new();
        {
            use std::io::Write as _;
            let mut encoder =
                flate2::write::GzEncoder::new(&mut gz, flate2::Compression::default());
            encoder.write_all(&tar_buf).expect("gz write");
            encoder.finish().expect("gz finish");
        }

        let chart = unpack_chart("0.1.0", &gz).expect("unpack");
        assert_eq!(chart.app_version, None);
        assert!(chart.configurable_values_schema.is_none());
        assert!(chart.ui_schema.is_none());
    }

    fn append_file(builder: &mut tar::Builder<&mut Vec<u8>>, path: &str, data: &[u8]) {
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, path, data)
            .expect("append tar entry");
    }

    /// Live check against the real GitLab charts registry. Ignored by default;
    /// run with credentials:
    ///   CHARTS_REGISTRY_USER=… CHARTS_REGISTRY_TOKEN=… \
    ///     cargo test -p frn-core managed::oci::tests::live -- --ignored --nocapture
    #[tokio::test]
    #[ignore = "hits the real registry; needs CHARTS_REGISTRY_USER/TOKEN"]
    async fn live_discovers_vaultwarden() {
        let credentials = match (
            std::env::var("CHARTS_REGISTRY_USER"),
            std::env::var("CHARTS_REGISTRY_TOKEN"),
        ) {
            (Ok(username), Ok(password)) => RegistryCredentials { username, password },
            _ => panic!("set CHARTS_REGISTRY_USER and CHARTS_REGISTRY_TOKEN"),
        };
        let client = OciChartClient::new(
            "oci://registry.gitlab.com/getbunker-france-nuage/france-nuage/charts/vaultwarden",
            Some(credentials),
        )
        .expect("client");

        let versions = client.list_versions().await.expect("list versions");
        println!("versions: {versions:?}");
        assert!(!versions.is_empty());

        let chart = client
            .fetch_chart(versions.first().expect("a version"))
            .await
            .expect("fetch chart");
        println!(
            "chart_version={} app_version={:?} has_cfg_schema={} has_ui_schema={}",
            chart.chart_version,
            chart.app_version,
            chart.configurable_values_schema.is_some(),
            chart.ui_schema.is_some(),
        );
    }
}

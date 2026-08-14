//! Server configuration management for the gRPC server.
//!
//! This module provides the [`Config`] structure that encapsulates all
//! configuration parameters needed for server operation, including network
//! settings, CORS policies, OIDC authentication, and PostgreSQL database connectivity.
//! The configuration system provides sensible defaults suitable for development while
//! remaining flexible for production deployments.

use crate::error::Error;
use frn_core::App;
use frn_crypto::Kek;
use mock_server::MockServer;
use spicedb::SpiceDB;
use sqlx::{Pool, Postgres};
use std::{collections::BTreeMap, env, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, ExposeHeaders};

/// Configuration for the gRPC server with CORS, authentication, networking, and PostgreSQL database settings.
///
/// This structure encapsulates all the necessary configuration parameters for setting up
/// a gRPC server with Cross-Origin Resource Sharing (CORS) support, OIDC JWT authentication,
/// and PostgreSQL database connectivity. It provides sensible defaults suitable for
/// development environments while maintaining flexibility for production deployments.
///
/// # Example
///
/// ```rust,no_run
/// use server::config::Config;
/// use sqlx::PgPool;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = PgPool::connect("postgresql://localhost/db").await?;
/// # let mock = mock_server::MockServer::new().await;
/// let config = Config::test(&pool, &mock).await?;
/// # Ok(())
/// # }
/// ```
///
/// The default configuration binds to all interfaces on port 8080 and allows any origin
/// and HTTP methods, making it suitable for development scenarios.
#[derive(Clone)]
pub struct Config {
    /// The frn core app. Allows for incremental upgrade to the new code structure.
    pub app: App<SpiceDB>,

    /// The socket address where the server will bind and listen for connections.
    ///
    /// This field determines both the IP address and port number that the gRPC server
    /// will use. The default value binds to all available interfaces (`[::]`) on port 8080.
    pub addr: SocketAddr,

    /// CORS configuration specifying which headers are allowed in cross-origin requests.
    ///
    /// This field controls the `Access-Control-Allow-Headers` header in HTTP responses.
    /// The default configuration allows all headers using [`AllowHeaders::any()`].
    ///
    /// [`AllowHeaders::any()`]: https://docs.rs/tower-http/latest/tower_http/cors/struct.AllowHeaders.html#method.any
    pub allow_headers: AllowHeaders,

    /// CORS configuration specifying which HTTP methods are allowed for cross-origin requests.
    ///
    /// This field controls the `Access-Control-Allow-Methods` header in HTTP responses.
    /// The default configuration allows all HTTP methods using [`AllowMethods::any()`].
    ///
    /// [`AllowMethods::any()`]: https://docs.rs/tower-http/latest/tower_http/cors/struct.AllowMethods.html#method.any
    pub allow_methods: AllowMethods,

    /// CORS configuration specifying which origins are allowed to make cross-origin requests.
    ///
    /// This field controls the `Access-Control-Allow-Origin` header in HTTP responses.
    /// The default configuration allows requests from any origin using [`AllowOrigin::any()`].
    ///
    /// [`AllowOrigin::any()`]: https://docs.rs/tower-http/latest/tower_http/cors/struct.AllowOrigin.html#method.any
    pub allow_origin: AllowOrigin,

    /// CORS configuration specifying which response headers are exposed to client scripts.
    ///
    /// This field controls the `Access-Control-Expose-Headers` header in HTTP responses.
    /// The default configuration exposes all headers using [`ExposeHeaders::any()`].
    ///
    /// [`ExposeHeaders::any()`]: https://docs.rs/tower-http/latest/tower_http/cors/struct.ExposeHeaders.html#method.any
    pub expose_headers: ExposeHeaders,

    /// PostgreSQL database connection pool for persistent storage operations.
    ///
    /// This field provides the PostgreSQL connection pool that will be shared across
    /// all services for performing persistent storage operations.
    pub pool: Pool<Postgres>,

    /// Pre-shared token for authenticating the workflow worker.
    pub worker_token: String,

    /// Pre-shared token for authenticating CI service account (managed services version registration).
    pub ci_token: String,

    /// Platform-level configuration injected into managed service Helm values.
    pub managed_platform_config: frn_core::managed::PlatformConfig,

    /// Key Encryption Key used to wrap per-cluster kubeconfig encryption keys.
    pub kubeconfig_encryption_kek: Arc<Kek>,

    /// Stripe secret API key (sk_live_xxx / sk_test_xxx).
    pub stripe_secret_key: Option<String>,

    /// Stripe webhook signing secret (whsec_xxx).
    pub stripe_webhook_secret: Option<String>,

    /// URL to redirect to after successful Stripe checkout.
    pub stripe_checkout_success_url: Option<String>,

    /// URL to redirect to after canceled Stripe checkout.
    pub stripe_checkout_cancel_url: Option<String>,

    /// Whether CORS must allow credentials (cookies). Enabled together with the
    /// BFF so browser gRPC-web calls can carry the httpOnly session cookie.
    pub allow_credentials: bool,

    /// Confidential-client BFF, present only when `OIDC_CLIENT_SECRET` is set.
    ///
    /// `None` means the `/auth/*` routes are **not** mounted. The former SPA/PKCE
    /// frontend has been removed, so in that state the console has no auth path at
    /// all and cannot authenticate — an absent secret is a deployment
    /// misconfiguration to avoid, not a graceful fallback to a legacy flow.
    pub bff: Option<crate::bff::Bff>,
}

/// Deterministic KEK used only by [`Config::test`]. Not a secret: tests run
/// against an isolated database, and a fixed value keeps runs reproducible.
const TEST_KUBECONFIG_ENCRYPTION_KEK: [u8; frn_crypto::KEK_SIZE] = [42u8; 32];

impl Config {
    /// Creates a test configuration with a dynamically allocated port and mock OIDC server.
    ///
    /// This constructor is specifically designed for test environments where:
    /// - A random available port is automatically allocated to avoid conflicts
    /// - OIDC authentication is configured to use the provided mock server
    /// - Database connection pool is cloned from the provided reference
    ///
    /// ## Parameters
    ///
    /// * `pool` - Reference to PostgreSQL connection pool (will be cloned)
    /// * `mock_server` - Mock server instance for OIDC authentication testing
    ///
    /// ## Usage in Tests
    ///
    /// ```
    /// # use server::Config;
    /// # use mock_server::MockServer;
    /// # async fn example(pool: &sqlx::PgPool) -> Result<(), server::error::Error> {
    /// let mock = MockServer::new().await;
    /// let config = Config::test(pool, &mock).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Features
    ///
    /// - **Dynamic Port**: Uses `reserve_socket_addr(None)` to allocate an available port
    /// - **Mock Authentication**: Configures OpenID for the mock server
    /// - **Test Isolation**: Each test gets its own port to avoid interference
    pub async fn test(pool: &Pool<Postgres>, _mock_server: &MockServer) -> Result<Self, Error> {
        let addr = Config::reserve_socket_addr(None).await?;

        let app = App::test(pool.to_owned())
            .await
            .expect("could not bootstrap app");

        Ok(Config {
            app,
            addr,
            allow_headers: AllowHeaders::any(),
            allow_methods: AllowMethods::any(),
            allow_origin: AllowOrigin::any(),
            expose_headers: ExposeHeaders::any(),
            pool: pool.clone(),
            worker_token: "test-worker-token".to_owned(),
            ci_token: "test-ci-token".to_owned(),
            managed_platform_config: frn_core::managed::PlatformConfig {
                default_storage_class: None,
                cnpg_backup_enabled: false,
                deployment_labels: BTreeMap::new(),
                deployment_annotations: BTreeMap::new(),
            },
            kubeconfig_encryption_kek: Arc::new(Kek::from_bytes(TEST_KUBECONFIG_ENCRYPTION_KEK)),
            stripe_secret_key: None,
            stripe_webhook_secret: None,
            stripe_checkout_success_url: None,
            stripe_checkout_cancel_url: None,
            allow_credentials: false,
            bff: None,
        })
    }

    /// Creates a configuration instance from environment variables.
    ///
    /// This method provides a convenient way to initialize server configuration
    /// from environment variables, making it suitable for containerized deployments
    /// and production environments.
    ///
    /// # Environment Variables
    ///
    /// * `DATABASE_URL` - PostgreSQL connection string (required)
    /// * `OIDC_URL` - OIDC provider discovery URL (optional, defaults to GitLab)
    ///
    /// # Default Values
    ///
    /// - **OIDC Provider**: GitLab's OIDC discovery endpoint if `OIDC_URL` not set
    /// - **Server Settings**: Same defaults as [`Config::new()`] for networking and CORS
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `DATABASE_URL` environment variable is not set
    /// - Database connection cannot be established
    /// - OIDC discovery fails or provider is unreachable
    /// - OIDC provider configuration is invalid
    pub async fn from_env() -> Result<Self, Error> {
        let app = App::new().await.expect("could not bootstrap app");
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("could not connect to database");

        let worker_token = env::var("WORKER_TOKEN").expect("WORKER_TOKEN must be set");
        let ci_token = env::var("CI_SERVICE_TOKEN").expect("CI_SERVICE_TOKEN must be set");
        let default_storage_class = env::var("MANAGED_DEFAULT_STORAGE_CLASS").ok();
        let cnpg_backup_enabled = env::var("MANAGED_CNPG_BACKUP_ENABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        let deployment_labels = parse_key_value_pairs(env::var("DEPLOYMENT_LABELS").ok());
        let deployment_annotations = parse_key_value_pairs(env::var("DEPLOYMENT_ANNOTATIONS").ok());
        let kubeconfig_encryption_kek = Arc::new(
            Kek::from_base64(
                &env::var("KUBECONFIG_ENCRYPTION_KEY")
                    .expect("KUBECONFIG_ENCRYPTION_KEY must be set"),
            )
            .expect("KUBECONFIG_ENCRYPTION_KEY must be base64-encoded 32 bytes"),
        );

        // Confidential-client BFF, gated on the presence of the client secret.
        // Absent secret => `bff` is `None`, the `/auth/*` routes are not mounted,
        // and the console cannot authenticate: the SPA/PKCE frontend has been
        // removed, so this is a deployment misconfiguration, not a fallback path.
        let bff = Self::build_bff(&app).await?;

        // CORS: cookies require credentialed CORS with an explicit origin. Only
        // switch to that stricter policy when the BFF is active; otherwise keep
        // the historical `any()` policy used by the bearer-token SPA flow.
        let (allow_headers, allow_methods, allow_origin, expose_headers, allow_credentials) =
            match &bff {
                Some(_) => Self::bff_cors()?,
                None => (
                    AllowHeaders::any(),
                    AllowMethods::any(),
                    AllowOrigin::any(),
                    ExposeHeaders::any(),
                    false,
                ),
            };

        Ok(Config {
            app,
            addr: Config::reserve_socket_addr(env::var("CONTROLPLANE_ADDR").ok()).await?,
            allow_headers,
            allow_methods,
            allow_origin,
            expose_headers,
            pool,
            worker_token,
            ci_token,
            managed_platform_config: frn_core::managed::PlatformConfig {
                default_storage_class,
                cnpg_backup_enabled,
                deployment_labels,
                deployment_annotations,
            },
            kubeconfig_encryption_kek,
            stripe_secret_key: env::var("STRIPE_SECRET_KEY").ok(),
            stripe_webhook_secret: env::var("STRIPE_WEBHOOK_SECRET").ok(),
            stripe_checkout_success_url: env::var("STRIPE_CHECKOUT_SUCCESS_URL").ok(),
            stripe_checkout_cancel_url: env::var("STRIPE_CHECKOUT_CANCEL_URL").ok(),
            allow_credentials,
            bff,
        })
    }

    /// Builds the confidential-client BFF from the environment, or `None` when
    /// no `OIDC_CLIENT_SECRET` is configured (the config gate).
    ///
    /// # Environment Variables (BFF mode only)
    ///
    /// * `OIDC_CLIENT_SECRET` — confidential client secret (its presence toggles
    ///   BFF mode). Comes from a k8s sealed secret; never hardcode it.
    /// * `OIDC_CLIENT_ID` — confidential client id (defaults to the console's
    ///   historical `francenuage` client id).
    /// * `OIDC_REDIRECT_URL` — absolute URL of `/auth/callback`, registered on
    ///   the IdP (required in BFF mode).
    /// * `OIDC_URL` — discovery URL (already required for gRPC auth).
    /// * `CONSOLE_URL` — where the browser lands after login/logout, and the CORS
    ///   allowed origin (already used for Stripe redirects).
    /// * `AUTH_COOKIE_DOMAIN` — optional cookie `Domain`.
    /// * `AUTH_COOKIE_SAMESITE` — `Lax` (default), `Strict`, or `None`.
    /// * `AUTH_COOKIE_INSECURE` — set to `1`/`true` to drop `Secure` (local http
    ///   dev only).
    /// * `AUTH_COOKIE_KEY` — base64 32-byte key sealing the encrypted session
    ///   cookie (required in BFF mode). The gRPC cookie path in `IAM` reads the
    ///   same variable, so both agree on one key.
    /// * `SESSION_MAX_TTL` — session cookie `Max-Age` as a positive integer number
    ///   of **seconds** (the refresh window). Unset => the default
    ///   [`crate::bff::DEFAULT_SESSION_MAX_TTL_SECS`]; set-but-unparseable => fail
    ///   loud at startup (never a silent fallback that ignores the operator's
    ///   value). The inner payload `exp` (short access lifetime) comes from the
    ///   id_token.
    async fn build_bff(app: &App<SpiceDB>) -> Result<Option<crate::bff::Bff>, Error> {
        let client_secret = match env::var("OIDC_CLIENT_SECRET") {
            Ok(secret) if !secret.is_empty() => secret,
            _ => return Ok(None),
        };

        let same_site =
            crate::bff::SameSite::from_env_value(env::var("AUTH_COOKIE_SAMESITE").ok().as_deref());
        let cookie_secure = !matches!(
            env::var("AUTH_COOKIE_INSECURE").ok().as_deref(),
            Some("1") | Some("true")
        );
        let cookie_key = frn_core::identity::SessionKey::from_base64(
            &env::var("AUTH_COOKIE_KEY")
                .expect("AUTH_COOKIE_KEY must be set in BFF mode (OIDC_CLIENT_SECRET present)"),
        )
        .expect("AUTH_COOKIE_KEY must be base64-encoded 32 bytes");
        let session_max_age_secs =
            parse_session_max_ttl(env::var("SESSION_MAX_TTL").ok().as_deref());

        let settings = crate::bff::Settings {
            oidc_url: app.config.oidc_url.clone(),
            client_id: env::var("OIDC_CLIENT_ID").unwrap_or_else(|_| "francenuage".to_owned()),
            client_secret,
            redirect_url: env::var("OIDC_REDIRECT_URL")
                .expect("OIDC_REDIRECT_URL must be set in BFF mode (OIDC_CLIENT_SECRET present)"),
            console_url: env::var("CONSOLE_URL")
                .expect("CONSOLE_URL must be set in BFF mode (OIDC_CLIENT_SECRET present)"),
            cookie_domain: env::var("AUTH_COOKIE_DOMAIN")
                .ok()
                .filter(|d| !d.is_empty()),
            cookie_secure,
            cookie_same_site: same_site,
            cookie_key,
            session_max_age_secs,
        };

        // Deployment-contract guard: a console and control plane on different
        // registrable domains need `SameSite=None; Secure`, otherwise the browser
        // withholds the session cookie on cross-site gRPC-web / `/auth/me` calls
        // and authentication silently breaks. Surfaced loudly, never a panic (the
        // registrable-domain check is a coarse heuristic).
        if let Some(warning) = same_site_cross_site_warning(
            &settings.console_url,
            &settings.redirect_url,
            settings.cookie_same_site,
            settings.cookie_secure,
        ) {
            tracing::error!("{warning}");
        }

        let bff = crate::bff::Bff::discover(app.openid.clone(), app.db.clone(), settings)
            .await
            .map_err(|err| Error::Core(frn_core::Error::Other(err.to_string())))?;

        Ok(Some(bff))
    }

    /// Credentialed CORS policy for BFF mode: explicit console origin, cookies
    /// allowed, and the explicit method/header lists required whenever
    /// `Access-Control-Allow-Credentials` is set (a `*` wildcard is illegal with
    /// credentials).
    fn bff_cors() -> Result<(AllowHeaders, AllowMethods, AllowOrigin, ExposeHeaders, bool), Error> {
        use http::{HeaderName, Method};

        let console_url = env::var("CONSOLE_URL")
            .expect("CONSOLE_URL must be set in BFF mode (OIDC_CLIENT_SECRET present)");
        // Canonical bare origin so the exact-origin CORS match can never be
        // silently defeated by a trailing slash or path in `CONSOLE_URL`.
        let origin = console_cors_origin(&console_url)?;

        let allow_headers = AllowHeaders::list(bff_cors_allow_headers());
        let allow_methods = AllowMethods::list([Method::GET, Method::POST, Method::OPTIONS]);
        let expose_headers = ExposeHeaders::list([
            HeaderName::from_static("grpc-status"),
            HeaderName::from_static("grpc-message"),
            HeaderName::from_static("grpc-status-details-bin"),
        ]);

        Ok((
            allow_headers,
            allow_methods,
            AllowOrigin::exact(origin),
            expose_headers,
            true,
        ))
    }

    /// Reserves a socket address, either from a preset string or by allocating dynamically.
    ///
    /// This method provides flexible address allocation for server binding:
    /// - If a preset address is provided, it parses and validates the address
    /// - If no preset is given, it allocates an available port on the loopback interface
    ///
    /// ## Parameters
    ///
    /// * `preset` - Optional address string (e.g., "127.0.0.1:8080", "[::1]:3000")
    ///
    /// ## Returns
    ///
    /// Returns a `SocketAddr` that can be used for server binding.
    ///
    /// ## Behavior
    ///
    /// - **With preset**: Parses the provided address string
    /// - **Without preset**: Binds to `[::1]:0` to get an OS-allocated port
    ///
    /// ## Usage
    ///
    /// ```
    /// # use server::Config;
    /// # async fn example() -> Result<(), server::error::Error> {
    /// // Use specific address
    /// let addr1 = Config::reserve_socket_addr(Some("127.0.0.1:8080".to_string())).await?;
    ///
    /// // Allocate dynamic port
    /// let addr2 = Config::reserve_socket_addr(None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn reserve_socket_addr(preset: Option<String>) -> Result<SocketAddr, Error> {
        match preset {
            Some(preset) => preset.parse().map_err(Into::into),
            None => TcpListener::bind("[::1]:0")
                .await?
                .local_addr()
                .map_err(Into::into),
        }
    }
}

/// Derives the canonical CORS allow-origin (scheme + host + optional non-default
/// port — no path, no query, no trailing slash) from a `CONSOLE_URL`.
///
/// `CONSOLE_URL` doubles as the post-login redirect `Location` (which needs the
/// full URL) and as the CORS exact-origin (which must equal the browser `Origin`
/// header, a bare scheme+host+port). A value like `https://console.france-nuage.fr/`
/// or one carrying a path parses fine yet never equals that bare `Origin`, so used
/// verbatim it silently blocks every credentialed gRPC-web call. Normalizing here
/// keeps the redirect on the full URL while the CORS layer gets the bare origin.
/// Fails loud (at startup) when `CONSOLE_URL` is not a valid absolute http(s) origin.
pub fn console_cors_origin(console_url: &str) -> Result<http::HeaderValue, Error> {
    let url = reqwest::Url::parse(console_url).map_err(|err| {
        config_error(format!(
            "CONSOLE_URL is not a valid absolute URL ({console_url:?}): {err}"
        ))
    })?;
    // `origin().is_tuple()` alone is not enough: `ws`/`wss`/`ftp` are "special"
    // schemes that also yield a tuple origin, so a mistaken `CONSOLE_URL=ws://…`
    // would pass yet never equal the browser's `https://` `Origin` header —
    // silently blocking every credentialed call. Require http(s) explicitly.
    if !matches!(url.scheme(), "http" | "https") {
        return Err(config_error(format!(
            "CONSOLE_URL must use the http or https scheme, got {:?} in {console_url:?}",
            url.scheme()
        )));
    }
    let origin = url.origin();
    if !origin.is_tuple() {
        return Err(config_error(format!(
            "CONSOLE_URL must be an absolute http(s) origin, got {console_url:?}"
        )));
    }
    http::HeaderValue::from_str(&origin.ascii_serialization())
        .map_err(|err| config_error(format!("CONSOLE_URL yields a non-header-safe origin: {err}")))
}

/// The exact `Access-Control-Allow-Headers` allow-list advertised in BFF mode.
///
/// Single source of truth: the CORS layer builds its allow-list from this, and a
/// black-box test asserts the gRPC-web browser transport's request headers are a
/// subset — so dropping one here (or adding a transport header without adding it
/// here) fails the test instead of silently breaking CORS preflight in the browser.
pub fn bff_cors_allow_headers() -> Vec<http::HeaderName> {
    ["content-type", "x-grpc-web", "x-user-agent", "grpc-timeout"]
        .into_iter()
        .map(http::HeaderName::from_static)
        .collect()
}

/// Parses `SESSION_MAX_TTL` (the session cookie `Max-Age` refresh window) into a
/// positive number of seconds.
///
/// - `None` (unset) => the documented default
///   [`crate::bff::DEFAULT_SESSION_MAX_TTL_SECS`].
/// - `Some(positive integer seconds)` => that value, honored verbatim.
/// - `Some(unparseable / non-positive)` => **fail loud** at startup.
///
/// The fail-loud path is deliberate: a silent fallback to the default would
/// override an operator who set e.g. `"24h"` or `"12h"` (not a valid `i64`) or a
/// negative value, and they would never know their configured TTL was ignored
/// (the 0-silent-fail rule). `panic!` here is startup configuration validation,
/// the one place a hard stop is the correct fail-fast (mirrors the `.expect`s on
/// `AUTH_COOKIE_KEY` / `OIDC_REDIRECT_URL` in [`Config::build_bff`]).
pub fn parse_session_max_ttl(raw: Option<&str>) -> i64 {
    match raw {
        None => crate::bff::DEFAULT_SESSION_MAX_TTL_SECS,
        Some(value) => {
            let secs = value.parse::<i64>().unwrap_or_else(|err| {
                panic!(
                    "SESSION_MAX_TTL must be a positive integer number of seconds, got {value:?}: {err}"
                )
            });
            assert!(
                secs > 0,
                "SESSION_MAX_TTL must be a positive integer number of seconds, got {secs}"
            );
            secs
        }
    }
}

/// Deployment-contract guard for the session cookie's `SameSite` policy.
///
/// The console's gRPC-web and `/auth/me` calls are cross-site *subresource*
/// requests. With `SameSite=Lax`/`Strict` the browser withholds the `frn_session`
/// cookie on those requests whenever the console and the control plane live on
/// different registrable domains — the session then silently never arrives.
/// Cross-site delivery requires `SameSite=None; Secure`. Returns the warning to
/// surface loudly at startup, or `None` when the policy is safe.
///
/// The registrable-domain comparison is a last-two-labels approximation: it does
/// **not** consult the public suffix list, so under a multi-label public suffix it
/// can **under-warn** — `app.co.uk` and `api.co.uk` are different registrable
/// domains, yet both collapse to `co.uk` and would look same-site, suppressing the
/// warning exactly when it is needed. This is a startup guard, not a security
/// boundary, so rather than pull in a PSL we bias toward emitting the warning: a
/// shared site whose shape looks like a ccTLD-style public suffix (see
/// [`looks_like_public_suffix`]) is treated as ambiguous, not same-site. A
/// spurious warning is the safe direction; a suppressed one is not.
pub fn same_site_cross_site_warning(
    console_url: &str,
    controlplane_url: &str,
    same_site: crate::bff::SameSite,
    secure: bool,
) -> Option<String> {
    // `SameSite=None; Secure` is the only policy that delivers cross-site.
    if matches!(same_site, crate::bff::SameSite::None) && secure {
        return None;
    }
    let console_site = registrable_domain(console_url)?;
    let controlplane_site = registrable_domain(controlplane_url)?;
    // Suppress only when the two collapse to the SAME registrable domain AND that
    // domain is not itself a public-suffix shape (in which case the last-two-labels
    // collapse is unreliable and we must not claim same-site — bias to warning).
    if console_site == controlplane_site && !looks_like_public_suffix(&console_site) {
        return None;
    }
    // Two ways to reach here: genuinely different registrable domains, or the
    // same domain whose ccTLD-like shape makes the last-two-labels collapse
    // unreliable. Word the diagnostic for each so it never claims "different"
    // while printing two identical sites.
    let diagnosis = if console_site == controlplane_site {
        format!(
            "share a registrable domain ({console_site}) whose public-suffix-like shape makes \
             its same-site status unreliable"
        )
    } else {
        format!("are on different registrable domains ({console_site} vs {controlplane_site})")
    };
    Some(format!(
        "cookie SameSite policy is cross-site-unsafe: console origin ({console_url}) and \
         control-plane origin ({controlplane_url}) {diagnosis}, but AUTH_COOKIE_SAMESITE is not \
         `none` with Secure. The browser will withhold the frn_session cookie on gRPC-web and \
         /auth/me subresource calls, silently breaking authentication. Set AUTH_COOKIE_SAMESITE=none \
         (served over HTTPS) for cross-site deployments, or host the console and control plane \
         on the same registrable domain."
    ))
}

/// Whether a two-label domain looks like a ccTLD-style multi-label public suffix
/// (`co.uk`, `com.au`, `co.jp`, `gov.uk`, `ac.uk`, …), for which the last-two-labels
/// [`registrable_domain`] approximation is unreliable.
///
/// Cheap shape heuristic (no public-suffix-list dependency): exactly two labels
/// where the first is short (≤ 3 chars, matching `co`/`com`/`gov`/`ac`/`edu`…) and
/// the last is a 2-char ccTLD. This deliberately over-flags rather than under-flags
/// — a false positive only produces a spurious startup warning (safe), whereas a
/// false negative would silently suppress the cross-site cookie warning (the bug).
fn looks_like_public_suffix(domain: &str) -> bool {
    let labels: Vec<&str> = domain.split('.').filter(|label| !label.is_empty()).collect();
    matches!(labels.as_slice(), [sld, tld] if sld.len() <= 3 && tld.len() == 2)
}

/// Registrable domain of a URL, approximated as its last two DNS labels
/// (e.g. `console.france-nuage.fr` -> `france-nuage.fr`). IP-literal or
/// single-label hosts are returned whole; `None` when the URL has no host.
///
/// This is a coarse last-two-labels heuristic that does not consult the public
/// suffix list, so it collapses multi-label public suffixes (`app.co.uk` ->
/// `co.uk`); callers that need same-site certainty must guard with
/// [`looks_like_public_suffix`].
fn registrable_domain(url: &str) -> Option<String> {
    let parsed = reqwest::Url::parse(url).ok()?;
    let host = parsed.host_str()?.to_ascii_lowercase();
    if host.parse::<std::net::IpAddr>().is_ok() {
        return Some(host);
    }
    let labels: Vec<&str> = host.split('.').filter(|label| !label.is_empty()).collect();
    match labels.len() {
        0 => None,
        1 => Some(host),
        n => Some(labels[n - 2..].join(".")),
    }
}

/// Builds a fail-loud configuration error without adding a new `Error` variant —
/// mirrors [`Config::build_bff`]'s use of `frn_core::Error::Other`.
fn config_error(message: String) -> Error {
    Error::Core(frn_core::Error::Other(message))
}

/// Parses a comma-separated list of `key=value` pairs into a map.
///
/// Used for the `DEPLOYMENT_LABELS` and `DEPLOYMENT_ANNOTATIONS` environment
/// variables. Entries without an `=` or with an empty key are ignored; keys and
/// values are trimmed. Returns an empty map when the input is `None`.
fn parse_key_value_pairs(raw: Option<String>) -> BTreeMap<String, String> {
    raw.into_iter()
        .flat_map(|value| {
            value
                .split(',')
                .filter_map(|pair| pair.split_once('='))
                .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
                .filter(|(key, _)| !key.is_empty())
                .collect::<Vec<_>>()
        })
        .collect()
}

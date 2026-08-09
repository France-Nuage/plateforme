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
use std::{env, net::SocketAddr, sync::Arc};
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
    /// `None` keeps the legacy SPA/PKCE flow as the sole auth path (the `/auth/*`
    /// routes are not mounted), so merging this change does not alter the
    /// currently-deployed authentication.
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
        let kubeconfig_encryption_kek = Arc::new(
            Kek::from_base64(
                &env::var("KUBECONFIG_ENCRYPTION_KEY")
                    .expect("KUBECONFIG_ENCRYPTION_KEY must be set"),
            )
            .expect("KUBECONFIG_ENCRYPTION_KEY must be base64-encoded 32 bytes"),
        );

        // Confidential-client BFF, gated on the presence of the client secret.
        // Absent secret => `bff` is `None` and the CORS policy stays permissive:
        // the legacy SPA/PKCE auth path is unchanged.
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
    /// * `SESSION_MAX_TTL` — session cookie `Max-Age` in seconds (the refresh
    ///   window). Defaults to [`crate::bff::DEFAULT_SESSION_MAX_TTL_SECS`]. The
    ///   inner payload `exp` (short access lifetime) comes from the id_token.
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
        let session_max_age_secs = env::var("SESSION_MAX_TTL")
            .ok()
            .and_then(|value| value.parse::<i64>().ok())
            .filter(|secs| *secs > 0)
            .unwrap_or(crate::bff::DEFAULT_SESSION_MAX_TTL_SECS);

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
        use http::{HeaderName, HeaderValue, Method};

        let console_url = env::var("CONSOLE_URL")
            .expect("CONSOLE_URL must be set in BFF mode (OIDC_CLIENT_SECRET present)");
        let origin: HeaderValue = console_url
            .parse()
            .expect("CONSOLE_URL must be a valid origin");

        let allow_headers = AllowHeaders::list([
            HeaderName::from_static("content-type"),
            HeaderName::from_static("x-grpc-web"),
            HeaderName::from_static("x-user-agent"),
            HeaderName::from_static("grpc-timeout"),
        ]);
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

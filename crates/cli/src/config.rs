use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct EnvValue {
    pub env: String,
}

/// A leaf value in the configuration.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum ConfigValue {
    Value(String),
    Environment(EnvValue),
}

impl ConfigValue {
    /// Get the resolved value of the config value.
    ///
    /// Returns the specified value directly or resolves it from the environment
    /// variable specified in the `Environment` variant.
    pub fn value(&self) -> Option<String> {
        match self {
            ConfigValue::Value(value) => Some(value.clone()),
            ConfigValue::Environment(env) => std::env::var(&env.env).ok(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// The host address to bind the server to.
    #[serde(default)]
    pub host: Option<String>,

    /// The port to bind the server to.
    #[serde(default)]
    pub port: Option<u16>,

    /// The backend configuration.
    #[serde(default)]
    pub backend: Backend,

    #[serde(default)]
    pub secret_backend: Option<SecretBackend>,

    /// Upstream Unity Catalog instance to delegate selected surfaces to.
    ///
    /// Required when [`Config::routing`] marks any surface as
    /// [`RoutingMode::Upstream`]; ignored otherwise.
    #[serde(default)]
    pub upstream: Option<UpstreamConfig>,

    /// Per-surface routing: whether each API surface is served locally or
    /// proxied to the [`upstream`](Config::upstream) instance.
    ///
    /// Defaults to all-local, so existing configs behave exactly as before.
    #[serde(default)]
    pub routing: RoutingConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: None,
            port: None,
            backend: Backend::InMemory,
            secret_backend: None,
            upstream: None,
            routing: RoutingConfig::default(),
        }
    }
}

/// Configuration for the upstream Unity Catalog instance.
///
/// The hybrid server proxies selected surfaces to this instance while serving
/// the rest from its local [`Backend`]. The upstream connection is
/// unauthenticated by design: authorization is enforced in *this* server's
/// policy layer before any request is forwarded, which lets the hybrid server
/// act as a test-bed for policy-engine integrations against real upstream data.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct UpstreamConfig {
    /// Base URL of the upstream Unity Catalog REST API, e.g.
    /// `http://uc-java:8080/api/2.1/unity-catalog`.
    pub url: String,
}

/// Whether an API surface is served from the local store or proxied upstream.
#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RoutingMode {
    /// Serve the surface from the local backend store (default).
    #[default]
    Local,
    /// Proxy the surface to the upstream instance.
    Upstream,
}

/// Per-surface routing configuration.
///
/// One field per Unity Catalog surface mounted by the REST server. Each
/// defaults to [`RoutingMode::Local`], so an empty or absent `routing` section
/// preserves the all-local behavior.
#[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct RoutingConfig {
    #[serde(default)]
    pub catalogs: RoutingMode,
    #[serde(default)]
    pub schemas: RoutingMode,
    #[serde(default)]
    pub tables: RoutingMode,
    #[serde(default)]
    pub credentials: RoutingMode,
    #[serde(default)]
    pub external_locations: RoutingMode,
    #[serde(default)]
    pub functions: RoutingMode,
    #[serde(default)]
    pub recipients: RoutingMode,
    #[serde(default)]
    pub shares: RoutingMode,
}

impl RoutingConfig {
    /// Returns `true` if any surface is routed upstream.
    pub fn any_upstream(&self) -> bool {
        self.surfaces()
            .iter()
            .any(|(_, mode)| *mode == RoutingMode::Upstream)
    }

    /// Surfaces that are routed upstream but do not yet have a proxy adapter
    /// implemented. Setting any of these to [`RoutingMode::Upstream`] is a
    /// configuration error and the server should refuse to start.
    ///
    /// v1 implements adapters for catalogs, schemas, and tables only.
    pub fn unsupported_upstream(&self) -> Vec<&'static str> {
        [
            ("credentials", self.credentials),
            ("external-locations", self.external_locations),
            ("functions", self.functions),
            ("recipients", self.recipients),
            ("shares", self.shares),
        ]
        .into_iter()
        .filter(|(_, mode)| *mode == RoutingMode::Upstream)
        .map(|(name, _)| name)
        .collect()
    }

    fn surfaces(&self) -> [(&'static str, RoutingMode); 8] {
        [
            ("catalogs", self.catalogs),
            ("schemas", self.schemas),
            ("tables", self.tables),
            ("credentials", self.credentials),
            ("external-locations", self.external_locations),
            ("functions", self.functions),
            ("recipients", self.recipients),
            ("shares", self.shares),
        ]
    }
}

/// Backend configuration for the unity catalog server.
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case", tag = "engine")]
pub enum Backend {
    /// Postgres backend configuration.
    Postgres(PostgresBackendConfig),

    /// In-memory backend configuration.
    ///
    /// This is useful for testing and development purposes
    /// but should not be used in production.
    #[default]
    InMemory,
}

/// Postgres backend configuration.
#[derive(Debug, Deserialize, Serialize)]
pub struct PostgresBackendConfig {
    /// The host of the server.
    pub host: ConfigValue,

    /// The port of the server.
    pub port: ConfigValue,

    /// The database user.
    pub user: ConfigValue,

    /// Password for the user.
    pub password: ConfigValue,

    /// The database name.
    pub database: ConfigValue,
}

impl PostgresBackendConfig {
    /// Get the full connection string for the Postgres backend.
    pub fn connection_string(&self) -> Option<String> {
        let host = self.host.value()?;
        let port = self.port.value()?;
        let user = self.user.value()?;
        let password = self.password.value()?;
        let database = self.database.value()?;

        Some(format!(
            "postgres://{user}:{password}@{host}:{port}/{database}"
        ))
    }
}

/// Secret backend configuration.
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "engine")]
pub enum SecretBackend {
    /// Postgres secret backend.
    ///
    /// This is used to store secrets in a Postgres database.
    /// This is generally only recommended for evaluation purposes.
    /// For production use, it is recommended to use a dedicated secret store.
    Postgres(PostgresSecretConfig),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PostgresSecretConfig {
    pub encryption_key: ConfigValue,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_config() {
        let config = r#"
            {
                "host": "0.0.0.0",
                "port": 8080,
                "backend": {
                    "engine": "postgres",
                    "database": "postgres",
                    "host": "localhost",
                    "port": "5432",
                    "user": "user",
                    "password": {
                        "env": "PG_PASSWORD"
                    }
                }
            }
        "#;

        let config: Config = serde_json::from_str(config).unwrap();
        assert_eq!(config.host.as_deref(), Some("0.0.0.0"));
        assert_eq!(config.port, Some(8080));
        assert!(matches!(config.backend, Backend::Postgres(_)));
        let backend = match config.backend {
            Backend::Postgres(backend) => backend,
            _ => unreachable!(),
        };
        assert_eq!(backend.host.value().unwrap(), "localhost");
        assert_eq!(backend.port.value().unwrap(), "5432");
        assert_eq!(backend.user.value().unwrap(), "user");
        assert_eq!(
            backend.password,
            ConfigValue::Environment(EnvValue {
                env: "PG_PASSWORD".to_string()
            })
        );
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.host.is_none());
        assert!(config.port.is_none());
        assert!(matches!(config.backend, Backend::InMemory));
        assert!(config.secret_backend.is_none());
        assert!(config.upstream.is_none());
        assert_eq!(config.routing, RoutingConfig::default());
        assert!(!config.routing.any_upstream());
    }

    #[test]
    fn test_minimal_config() {
        let config = r#"{}"#;
        let config: Config = serde_json::from_str(config).unwrap();
        assert!(matches!(config.backend, Backend::InMemory));
        assert!(!config.routing.any_upstream());
    }

    #[test]
    fn test_routing_defaults_to_local() {
        let routing = RoutingConfig::default();
        assert_eq!(routing.catalogs, RoutingMode::Local);
        assert!(!routing.any_upstream());
        assert!(routing.unsupported_upstream().is_empty());
    }

    #[test]
    fn test_hybrid_config_roundtrip() {
        let yaml = r#"
            backend:
              engine: in-memory
            upstream:
              url: "http://uc-java:8080/api/2.1/unity-catalog"
            routing:
              catalogs: upstream
              schemas: local
              tables: upstream
        "#;
        let config: Config = serde_yml::from_str(yaml).unwrap();
        assert_eq!(
            config.upstream,
            Some(UpstreamConfig {
                url: "http://uc-java:8080/api/2.1/unity-catalog".to_string(),
            })
        );
        assert_eq!(config.routing.catalogs, RoutingMode::Upstream);
        assert_eq!(config.routing.schemas, RoutingMode::Local);
        assert_eq!(config.routing.tables, RoutingMode::Upstream);
        assert!(config.routing.any_upstream());
        assert!(config.routing.unsupported_upstream().is_empty());

        // round-trips back to YAML and re-parses identically
        let serialized = serde_yml::to_string(&config).unwrap();
        let reparsed: Config = serde_yml::from_str(&serialized).unwrap();
        assert_eq!(reparsed.routing, config.routing);
        assert_eq!(reparsed.upstream, config.upstream);
    }

    #[test]
    fn test_unsupported_upstream_surfaces_detected() {
        let yaml = r#"
            routing:
              catalogs: upstream
              functions: upstream
              shares: upstream
        "#;
        let config: Config = serde_yml::from_str(yaml).unwrap();
        assert!(config.routing.any_upstream());
        assert_eq!(
            config.routing.unsupported_upstream(),
            vec!["functions", "shares"]
        );
    }
}

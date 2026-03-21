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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: None,
            port: None,
            backend: Backend::InMemory,
            secret_backend: None,
        }
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
    }

    #[test]
    fn test_minimal_config() {
        let config = r#"{}"#;
        let config: Config = serde_json::from_str(config).unwrap();
        assert!(matches!(config.backend, Backend::InMemory));
    }
}

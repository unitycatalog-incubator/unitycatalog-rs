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
    /// This will witerh return the specified value or the value of the environment variable
    /// specified in the `Environment` variant.
    pub fn value(&self) -> Option<String> {
        match self {
            ConfigValue::Value(value) => Some(value.clone()),
            ConfigValue::Environment(env) => std::env::var(&env.env).ok(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub url: String,

    /// The backend configuration.
    pub backend: Backend,

    #[serde(default)]
    pub secret_backend: Option<SecretBackend>,
}

/// Backend configuration for the unity catalog server.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", tag = "engine")]
pub enum Backend {
    /// Postgres backend configuration.
    Postgres(PostgresBackendConfig),

    /// In-memory backend configuration.
    ///
    /// This is useful for testing and development purposes
    /// but should not be used in production.
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
    /// Azure Key Vault secret backend.
    Azure(AzureKeyValut),

    /// Postgres secret backend.
    ///
    /// This is used to store secrets in a Postgres database.
    /// This is generally only recommended for evaluation purposes.
    /// For production use, it is recommended to use a dedicated secret store.
    Postgres(PostgresSecretConfig),
}

/// Azure Key Vault secret backend configuration.
#[derive(Debug, Deserialize, Serialize)]
pub struct AzureKeyValut {
    /// The name of the vault.
    pub vault_name: ConfigValue,

    /// The client ID.
    pub client_id: ConfigValue,

    /// The tenant ID.
    pub tenant_id: ConfigValue,

    /// The client secret.
    pub client_secret: Option<ConfigValue>,

    /// The federated token file.
    pub federated_token_file: Option<ConfigValue>,
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
                "url": "http://localhost:8080",
                "backend": {
                    "engine": "postgres",
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
        assert_eq!(config.url, "http://localhost:8080");
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
}

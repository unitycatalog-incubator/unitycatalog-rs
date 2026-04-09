pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Build error: {0}")]
    Build(String),

    #[error("bindings config is required when python, node, or node_ts output is enabled")]
    MissingBindingsConfig,

    #[error("Missing annotation for {object}: {message}")]
    MissingAnnotation { object: String, message: String },

    #[error("Invalid annotation for {object}: {message}")]
    InvalidAnnotation { object: String, message: String },

    #[error("Invalid models_path template `{template}`: {source}")]
    InvalidModelsPathTemplate {
        template: String,
        #[source]
        source: syn::Error,
    },

    #[error("Missing HTTP rule pattern for method `{method}`")]
    MissingHttpPattern { method: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

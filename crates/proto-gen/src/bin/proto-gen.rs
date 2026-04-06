use std::fs;
use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand};
use protobuf::Message;
use protobuf::descriptor::FileDescriptorSet;

use proto_gen::error::Result;
use proto_gen::{
    BindingsConfig, CodeGenConfig, CodeGenOutput, enrich_openapi, generate_code,
    parse_file_descriptor_set,
};

#[derive(Parser)]
#[command(
    name = "proto-gen",
    about = "Generate Rust/Python/Node.js code from proto descriptors"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate Rust/Python/Node.js code from proto descriptor
    Generate(Box<GenerateArgs>),
    /// Enrich openapi.yaml with validation rules from buf JSON Schema files
    EnrichOpenapi(EnrichOpenApiArgs),
}

#[derive(Args)]
struct GenerateArgs {
    /// Path to a YAML config file; CLI flags override values from the file.
    #[clap(long, short = 'c')]
    config: Option<PathBuf>,

    #[clap(long, env = "UC_BUILD_OUTPUT_COMMON")]
    output_common: Option<String>,

    /// Parent models directory (e.g. `crates/common/src/models`).
    /// Generated files are written into `<output_models>/<models_subdir>/`.
    #[clap(long, env = "UC_BUILD_OUTPUT_MODELS")]
    output_models: Option<String>,

    /// Name of the generated subdirectory inside `output_models`. Defaults to `"_gen"`.
    #[clap(long, env = "UC_BUILD_MODELS_SUBDIR")]
    models_subdir: Option<String>,

    #[clap(long, env = "UC_BUILD_OUTPUT_SERVER")]
    output_server: Option<String>,

    #[clap(long, env = "UC_BUILD_OUTPUT_CLIENT")]
    output_client: Option<String>,

    #[clap(long, env = "UC_BUILD_OUTPUT_PYTHON")]
    output_python: Option<String>,

    #[clap(long, env = "UC_BUILD_OUTPUT_NODE")]
    output_node: Option<String>,

    #[clap(long, env = "UC_BUILD_OUTPUT_NODE_TS")]
    output_node_ts: Option<String>,

    #[clap(long, short, env = "UC_BUILD_DESCRIPTORS")]
    descriptors: Option<String>,

    /// Fully-qualified path to the request context type (e.g. `my_crate::Context`).
    #[clap(long, env = "UC_BUILD_CONTEXT_TYPE")]
    context_type: Option<String>,

    /// Fully-qualified path to the Result alias (e.g. `my_crate::Result`).
    #[clap(long, env = "UC_BUILD_RESULT_TYPE")]
    result_type: Option<String>,

    /// Template for the external model import path. Use `{service}` as placeholder.
    #[clap(long, env = "UC_BUILD_MODELS_PATH_TEMPLATE")]
    models_path_template: Option<String>,

    /// Template for the crate-local model import path. Use `{service}` as placeholder.
    #[clap(long, env = "UC_BUILD_MODELS_PATH_CRATE_TEMPLATE")]
    models_path_crate_template: Option<String>,

    /// Filename for the generated Python typings stub.
    #[clap(long, env = "UC_BUILD_PYTHON_TYPINGS_FILENAME")]
    python_typings_filename: Option<String>,
}

#[derive(Args)]
struct EnrichOpenApiArgs {
    /// Path to a YAML config file; CLI flags override values from the file.
    #[arg(long, short = 'c')]
    config: Option<PathBuf>,

    /// Path to openapi.yaml
    #[arg(long)]
    spec: Option<PathBuf>,
    /// Directory containing *.schema.strict.bundle.json files
    #[arg(long)]
    jsonschema_dir: Option<PathBuf>,
    /// Proto descriptor binary for path/body deduplication (Pass 2); omit to skip
    #[arg(long)]
    descriptors: Option<PathBuf>,
    /// Translate snake_case JSON Schema property names to camelCase in OpenAPI
    #[arg(long)]
    camel_case: Option<bool>,
}

// ---------------------------------------------------------------------------
// Config file types
// ---------------------------------------------------------------------------

/// Compute a POSIX-style relative path from `base` to `target` (both absolute).
///
/// e.g. `relative_path("/a/b/_gen", "/a/b/gen")` → `"../gen"`
fn relative_path(base: &std::path::Path, target: &std::path::Path) -> String {
    let base_components: Vec<_> = base.components().collect();
    let target_components: Vec<_> = target.components().collect();

    let common_len = base_components
        .iter()
        .zip(target_components.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let up_count = base_components.len() - common_len;
    let mut parts: Vec<std::borrow::Cow<str>> = (0..up_count).map(|_| "..".into()).collect();
    for comp in &target_components[common_len..] {
        parts.push(comp.as_os_str().to_string_lossy());
    }

    if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    }
}

/// Top-level config file schema. Both subcommands share a single file, each
/// with its own optional section. CLI flags always override config file values.
#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ProtoGenConfig {
    /// Shared descriptor path used by both `generate` and `enrich_openapi`.
    descriptors: Option<String>,
    /// Path to buf.gen.yaml; used to auto-derive model path templates.
    buf_gen: Option<PathBuf>,
    generate: Option<FileGenerateConfig>,
    enrich_openapi: Option<FileEnrichOpenApiConfig>,
}

#[derive(Debug, serde::Deserialize)]
struct BufGenPlugin {
    remote: Option<String>,
    out: String,
}

#[derive(Debug, serde::Deserialize)]
struct BufGenConfig {
    plugins: Vec<BufGenPlugin>,
}

/// Find the prost plugin's output path in a buf.gen.yaml file.
fn find_prost_out(buf_gen_path: &Path) -> std::result::Result<PathBuf, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(buf_gen_path)?;
    let cfg: BufGenConfig = serde_yaml::from_str(&text)?;
    let prost = cfg
        .plugins
        .iter()
        .find(|p| {
            p.remote
                .as_deref()
                .map(|r| r.contains("prost") && !r.contains("serde") && !r.contains("tonic"))
                .unwrap_or(false)
        })
        .ok_or("no prost plugin found in buf.gen.yaml")?;
    Ok(PathBuf::from(&prost.out))
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FilePythonConfig {
    output: Option<String>,
    error_type: Option<String>,
    result_type: Option<String>,
    typings_package_filter: Option<String>,
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FileNodeConfig {
    output: Option<String>,
    napi_error_ext_trait: Option<String>,
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FileTsConfig {
    output: Option<String>,
    aggregate_client_name: Option<String>,
    client_crate_name: Option<String>,
    error_base_class: Option<String>,
    error_code_prefix: Option<String>,
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FileGenerateConfig {
    output_common: Option<String>,
    /// Parent models directory (e.g. `crates/common/src/models`).
    output_models: Option<String>,
    /// Name of the generated subdirectory inside `output_models`. Defaults to `"_gen"`.
    models_subdir: Option<String>,
    output_server: Option<String>,
    output_client: Option<String>,
    context_type: Option<String>,
    result_type: Option<String>,
    models_path_template: Option<String>,
    models_path_crate_template: Option<String>,
    /// Crate name for the models crate; used to auto-derive `models_path_template`
    /// when `buf_gen` is set in the top-level config.
    models_crate_name: Option<String>,
    python_typings_filename: Option<String>,
    /// Set to `true` to generate `labels.rs` with `Resource` / `ObjectLabel` enums.
    generate_resource_enum: Option<bool>,
    python: Option<FilePythonConfig>,
    node: Option<FileNodeConfig>,
    typescript: Option<FileTsConfig>,
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FileEnrichOpenApiConfig {
    spec: Option<PathBuf>,
    jsonschema_dir: Option<PathBuf>,
    camel_case: Option<bool>,
}

fn load_proto_gen_config(
    path: &Path,
) -> std::result::Result<ProtoGenConfig, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path)?;
    Ok(serde_yaml::from_str(&text)?)
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate(args) => run_generate(*args),
        Commands::EnrichOpenapi(args) => run_enrich_openapi(args),
    }
}

// ---------------------------------------------------------------------------
// generate subcommand
// ---------------------------------------------------------------------------

fn run_generate(mut args: GenerateArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut file_cfg = FileGenerateConfig::default();
    let mut buf_gen_path: Option<PathBuf> = None;

    // Merge config file values (CLI flags win — only fill in fields not already set).
    if let Some(ref config_path) = args.config.clone() {
        let file = load_proto_gen_config(config_path)?;

        // Top-level descriptors (shared with enrich_openapi)
        if args.descriptors.is_none() {
            args.descriptors = file.descriptors.clone();
        }

        buf_gen_path = file.buf_gen.clone();

        let cfg = file.generate.unwrap_or_default();

        macro_rules! fill {
            ($field:ident) => {
                if args.$field.is_none() {
                    args.$field = cfg.$field.clone();
                }
            };
        }

        fill!(output_common);
        fill!(output_models);
        fill!(models_subdir);
        fill!(output_server);
        fill!(output_client);
        fill!(context_type);
        fill!(result_type);
        fill!(models_path_template);
        fill!(models_path_crate_template);
        fill!(python_typings_filename);

        // Fill output paths from nested language sections (CLI wins)
        if args.output_python.is_none() {
            args.output_python = cfg.python.as_ref().and_then(|c| c.output.clone());
        }
        if args.output_node.is_none() {
            args.output_node = cfg.node.as_ref().and_then(|c| c.output.clone());
        }
        if args.output_node_ts.is_none() {
            args.output_node_ts = cfg.typescript.as_ref().and_then(|c| c.output.clone());
        }

        file_cfg = cfg;
    }

    // Auto-derive path templates from buf.gen.yaml when not explicitly set.
    let mut prost_out_abs: Option<PathBuf> = None;
    if args.models_path_template.is_none() {
        if let Some(ref bgp) = buf_gen_path {
            let prost_out = find_prost_out(bgp)?;
            prost_out_abs = Some(fs::canonicalize(&prost_out)?);
            let crate_name = file_cfg
                .models_crate_name
                .as_deref()
                .ok_or("models_crate_name is required in generate config when buf_gen is set")?;
            args.models_path_template = Some(format!("{crate_name}::models::{{service}}::v1"));
        }
    }
    if args.models_path_crate_template.is_none() && buf_gen_path.is_some() {
        args.models_path_crate_template = Some("crate::models::{service}::v1".to_string());
    }
    // Lazily resolve prost_out if not yet resolved (needed for models_gen_dir even when
    // models_path_template was already set).
    if prost_out_abs.is_none() {
        if let Some(ref bgp) = buf_gen_path {
            let prost_out = find_prost_out(bgp)?;
            if prost_out.exists() {
                prost_out_abs = Some(fs::canonicalize(&prost_out)?);
            }
        }
    }

    let descriptors = args.descriptors.as_deref().ok_or(
        "required argument missing: --descriptors (or set it in the config file under generate.descriptors or top-level descriptors)",
    )?.to_owned();
    let output_common = args.output_common.as_deref().ok_or(
        "required argument missing: --output-common (or set it in the config file under generate.output_common)",
    )?.to_owned();

    let descriptor_path = fs::canonicalize(PathBuf::from(&descriptors))?;
    let descriptor_bytes = fs::read(&descriptor_path)?;
    let file_descriptor_set = FileDescriptorSet::parse_from_bytes(&descriptor_bytes)?;

    let metadata = parse_file_descriptor_set(&file_descriptor_set)?;

    let resolve_dir = |p: &str| fs::canonicalize(PathBuf::from(p));

    let output_common = resolve_dir(&output_common)?;
    let output_models = args.output_models.as_deref().map(resolve_dir).transpose()?;
    let models_subdir = args
        .models_subdir
        .clone()
        .unwrap_or_else(|| "_gen".to_string());
    let output_server = args.output_server.as_deref().map(resolve_dir).transpose()?;
    let output_client = args.output_client.as_deref().map(resolve_dir).transpose()?;
    let output_python = args.output_python.as_deref().map(resolve_dir).transpose()?;
    let output_node = args.output_node.as_deref().map(resolve_dir).transpose()?;
    let output_node_ts = args
        .output_node_ts
        .as_deref()
        .map(resolve_dir)
        .transpose()?;

    let python_typings_filename = args
        .python_typings_filename
        .clone()
        .unwrap_or_else(|| "unitycatalog_client.pyi".to_string());

    // Compute relative path from the models subdir to prost_out (for include! paths).
    // e.g. models = ".../models", subdir = "_gen", prost_out = ".../models/gen"
    // → relative_path(".../models/_gen", ".../models/gen") = "../gen"
    let models_gen_dir: Option<String> = output_models
        .as_deref()
        .zip(prost_out_abs.as_deref())
        .map(|(models_dir, prost_out)| {
            let subdir_path = models_dir.join(&models_subdir);
            relative_path(&subdir_path, prost_out)
        });

    let output = CodeGenOutput {
        common: output_common,
        models: output_models,
        models_subdir,
        server: output_server,
        client: output_client,
        python: output_python,
        node: output_node,
        node_ts: output_node_ts,
        python_typings_filename,
    };

    let config = build_config(output, &args, &file_cfg, models_gen_dir);
    generate_code(&metadata, &config)?;

    Ok(())
}

/// Build a [`CodeGenConfig`] from parsed CLI arguments and config file settings.
fn build_config(
    output: CodeGenOutput,
    args: &GenerateArgs,
    file_cfg: &FileGenerateConfig,
    models_gen_dir: Option<String>,
) -> CodeGenConfig {
    let has_bindings_output =
        output.python.is_some() || output.node.is_some() || output.node_ts.is_some();

    let bindings = has_bindings_output.then(|| {
        let py = file_cfg.python.as_ref();
        let node = file_cfg.node.as_ref();
        let ts = file_cfg.typescript.as_ref();
        BindingsConfig {
            aggregate_client_name: ts
                .and_then(|c| c.aggregate_client_name.clone())
                .unwrap_or_default(),
            client_crate_name: ts
                .and_then(|c| c.client_crate_name.clone())
                .unwrap_or_default(),
            py_error_type: py.and_then(|c| c.error_type.clone()).unwrap_or_default(),
            py_result_type: py.and_then(|c| c.result_type.clone()).unwrap_or_default(),
            napi_error_ext_trait: node
                .and_then(|c| c.napi_error_ext_trait.clone())
                .unwrap_or_default(),
            typings_package_filter: py.and_then(|c| c.typings_package_filter.clone()),
            ts_error_base_class: ts
                .and_then(|c| c.error_base_class.clone())
                .unwrap_or_default(),
            ts_error_code_prefix: ts
                .and_then(|c| c.error_code_prefix.clone())
                .unwrap_or_default(),
        }
    });

    let generate_resource_enum = file_cfg.generate_resource_enum.unwrap_or(false);

    CodeGenConfig {
        context_type_path: args
            .context_type
            .clone()
            .unwrap_or_else(|| "crate::api::RequestContext".to_string()),
        result_type_path: args
            .result_type
            .clone()
            .unwrap_or_else(|| "crate::Result".to_string()),
        models_path_template: args.models_path_template.clone().unwrap_or_default(),
        models_path_crate_template: args.models_path_crate_template.clone().unwrap_or_default(),
        output,
        generate_resource_enum,
        bindings,
        models_gen_dir,
    }
}

// ---------------------------------------------------------------------------
// enrich-openapi subcommand
// ---------------------------------------------------------------------------

fn run_enrich_openapi(mut args: EnrichOpenApiArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Merge config file values (CLI flags win).
    if let Some(ref config_path) = args.config.clone() {
        let file = load_proto_gen_config(config_path)?;

        // Top-level descriptors
        if args.descriptors.is_none() {
            args.descriptors = file.descriptors.map(PathBuf::from);
        }

        let cfg = file.enrich_openapi.unwrap_or_default();

        macro_rules! fill {
            ($field:ident) => {
                if args.$field.is_none() {
                    args.$field = cfg.$field;
                }
            };
        }

        fill!(spec);
        fill!(jsonschema_dir);
        fill!(camel_case);
    }

    // Apply defaults for fields not set by either CLI or config.
    let spec = args
        .spec
        .unwrap_or_else(|| PathBuf::from("openapi/openapi.yaml"));
    let jsonschema_dir = args
        .jsonschema_dir
        .unwrap_or_else(|| PathBuf::from("openapi/jsonschema"));
    let camel_case = args.camel_case.unwrap_or(false);

    enrich_openapi(
        &spec,
        &jsonschema_dir,
        camel_case,
        args.descriptors.as_deref(),
    )?;

    Ok(())
}

use std::fs;
use std::path::Path;

use crate::codegen::GeneratedCode;

/// Return the appropriate "do not edit" header for a given file path.
fn generated_header(path: &str) -> &'static str {
    if path.ends_with(".ts") || path.ends_with(".js") {
        "// @generated — do not edit by hand.\n"
    } else if path.ends_with(".py") || path.ends_with(".pyi") {
        "# @generated — do not edit by hand.\n"
    } else {
        // Rust and everything else
        "// @generated — do not edit by hand.\n"
    }
}

pub fn write_generated_code(
    generated_code: &GeneratedCode,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(output_dir)?;

    for (relative_path, content) in &generated_code.files {
        let file_path = output_dir.join(relative_path);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let header = generated_header(relative_path);
        let final_content = if content.starts_with(header) {
            content.clone()
        } else {
            format!("{}{}", header, content)
        };

        fs::write(&file_path, final_content)?;
    }

    tracing::info!("Successfully wrote all generated files");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_generated_code() -> GeneratedCode {
        let mut files = HashMap::new();
        files.insert(
            "test_handler.rs".to_string(),
            "// Test handler content\npub trait TestHandler {}\n".to_string(),
        );
        files.insert(
            "test_routes.rs".to_string(),
            "// Test routes content\npub fn test_route() {}\n".to_string(),
        );

        GeneratedCode { files }
    }

    #[test]
    fn test_write_generated_code() {
        let temp_dir = TempDir::new().unwrap();
        let generated_code = create_test_generated_code();

        let result = write_generated_code(&generated_code, temp_dir.path());
        assert!(result.is_ok());

        assert!(temp_dir.path().join("test_handler.rs").exists());
        assert!(temp_dir.path().join("test_routes.rs").exists());
    }
}

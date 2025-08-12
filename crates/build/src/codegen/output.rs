//! Output module for writing generated code to files
//!
//! This module handles the final step of code generation: writing the generated
//! Rust code to appropriate files in the output directory. It manages:
//!
//! - Creating directory structures
//! - Writing files with proper formatting
//! - Handling file conflicts and overwrites
//! - Organizing generated code into logical modules

use std::fs;
use std::path::Path;

use super::GeneratedCode;

/// Write generated code to the output directory
pub fn write_generated_code(
    generated_code: &GeneratedCode,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "cargo:warning=Writing {} generated files to {}",
        generated_code.files.len(),
        output_dir.display()
    );

    // Ensure output directory exists
    fs::create_dir_all(output_dir)?;

    // Write each generated file
    for (relative_path, content) in &generated_code.files {
        write_file(output_dir, relative_path, content)?;
    }

    // Generate a summary file for debugging
    write_generation_summary(output_dir, generated_code)?;

    println!("cargo:warning=Successfully wrote all generated files");
    Ok(())
}

/// Write a single file to the output directory
fn write_file(
    output_dir: &Path,
    relative_path: &str,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = output_dir.join(relative_path);

    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Format the content before writing
    let formatted_content = format_rust_code(content)?;

    // Write the file
    fs::write(&file_path, formatted_content)?;

    println!(
        "cargo:warning=Wrote generated file: {}",
        file_path.display()
    );

    Ok(())
}

/// Format Rust code (basic formatting)
fn format_rust_code(content: &str) -> Result<String, Box<dyn std::error::Error>> {
    // For now, just ensure consistent line endings and add final newline
    let mut formatted = content.replace("\r\n", "\n").replace('\r', "\n");

    // Ensure file ends with newline
    if !formatted.ends_with('\n') {
        formatted.push('\n');
    }

    // TODO: In the future, we could integrate with rustfmt for proper formatting
    Ok(formatted)
}

/// Write a summary file for debugging and tracking
fn write_generation_summary(
    output_dir: &Path,
    generated_code: &GeneratedCode,
) -> Result<(), Box<dyn std::error::Error>> {
    let summary_path = output_dir.join("GENERATION_SUMMARY.md");

    let mut summary = String::new();
    summary.push_str("# Generated Code Summary\n\n");
    summary.push_str(&format!(
        "Generated {} files at {}\n\n",
        generated_code.files.len(),
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));

    summary.push_str("## Files Generated\n\n");
    let mut file_paths: Vec<_> = generated_code.files.keys().collect();
    file_paths.sort();

    for file_path in file_paths {
        let content = &generated_code.files[file_path];
        let line_count = content.lines().count();
        summary.push_str(&format!("- `{}` ({} lines)\n", file_path, line_count));
    }

    summary.push_str("\n## Code Generation Statistics\n\n");

    let total_lines: usize = generated_code
        .files
        .values()
        .map(|content| content.lines().count())
        .sum();

    let total_chars: usize = generated_code
        .files
        .values()
        .map(|content| content.len())
        .sum();

    summary.push_str(&format!("- Total lines of code: {}\n", total_lines));
    summary.push_str(&format!("- Total characters: {}\n", total_chars));
    summary.push_str(&format!(
        "- Average lines per file: {:.1}\n",
        total_lines as f64 / generated_code.files.len() as f64
    ));

    // Count different file types
    let mut file_types = std::collections::HashMap::new();
    for file_path in generated_code.files.keys() {
        let extension = file_path.split('.').last().unwrap_or("unknown");
        *file_types.entry(extension).or_insert(0) += 1;
    }

    summary.push_str("\n## File Types\n\n");
    for (ext, count) in file_types {
        summary.push_str(&format!("- `.{}`: {} files\n", ext, count));
    }

    // Analyze content patterns
    let handler_files = generated_code
        .files
        .keys()
        .filter(|path| path.contains("handler"))
        .count();
    let route_files = generated_code
        .files
        .keys()
        .filter(|path| path.contains("route"))
        .count();
    let client_files = generated_code
        .files
        .keys()
        .filter(|path| path.contains("client"))
        .count();
    let extractor_files = generated_code
        .files
        .keys()
        .filter(|path| path.contains("extractor"))
        .count();

    summary.push_str("\n## Code Categories\n\n");
    summary.push_str(&format!("- Handler traits: {} files\n", handler_files));
    summary.push_str(&format!("- Route handlers: {} files\n", route_files));
    summary.push_str(&format!("- Client code: {} files\n", client_files));
    summary.push_str(&format!(
        "- Request extractors: {} files\n",
        extractor_files
    ));

    summary.push_str("\n---\n");
    summary.push_str("*This file is automatically generated and should not be edited manually.*\n");

    fs::write(summary_path, summary)?;
    Ok(())
}

/// Validate that all required files were generated
pub fn validate_output(
    output_dir: &Path,
    expected_files: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut missing_files = Vec::new();

    for expected_file in expected_files {
        let file_path = output_dir.join(expected_file);
        if !file_path.exists() {
            missing_files.push(*expected_file);
        }
    }

    if !missing_files.is_empty() {
        return Err(format!(
            "Missing expected generated files: {}",
            missing_files.join(", ")
        )
        .into());
    }

    println!("cargo:warning=Output validation passed: all expected files generated");
    Ok(())
}

/// Clean the output directory before generation
pub fn clean_output_directory(output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if output_dir.exists() {
        println!(
            "cargo:warning=Cleaning output directory: {}",
            output_dir.display()
        );
        fs::remove_dir_all(output_dir)?;
    }

    fs::create_dir_all(output_dir)?;
    Ok(())
}

/// Get information about existing files in output directory
pub fn analyze_existing_files(
    output_dir: &Path,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut existing_files = Vec::new();

    if output_dir.exists() {
        for entry in fs::read_dir(output_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    existing_files.push(file_name.to_string());
                }
            }
        }
    }

    Ok(existing_files)
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

        // Check that files were created
        assert!(temp_dir.path().join("test_handler.rs").exists());
        assert!(temp_dir.path().join("test_routes.rs").exists());
        assert!(temp_dir.path().join("GENERATION_SUMMARY.md").exists());
    }

    #[test]
    fn test_format_rust_code() {
        let input = "pub fn test() {\r\n    println!(\"hello\");\r\n}";
        let result = format_rust_code(input).unwrap();
        assert_eq!(result, "pub fn test() {\n    println!(\"hello\");\n}\n");
    }

    #[test]
    fn test_validate_output() {
        let temp_dir = TempDir::new().unwrap();

        // Create expected files
        fs::write(temp_dir.path().join("expected1.rs"), "test content").unwrap();
        fs::write(temp_dir.path().join("expected2.rs"), "test content").unwrap();

        let result = validate_output(temp_dir.path(), &["expected1.rs", "expected2.rs"]);
        assert!(result.is_ok());

        // Test with missing file
        let result = validate_output(temp_dir.path(), &["expected1.rs", "missing.rs"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_clean_output_directory() {
        let temp_dir = TempDir::new().unwrap();

        // Create some files
        fs::write(temp_dir.path().join("old_file.rs"), "old content").unwrap();
        assert!(temp_dir.path().join("old_file.rs").exists());

        let result = clean_output_directory(temp_dir.path());
        assert!(result.is_ok());
        assert!(!temp_dir.path().join("old_file.rs").exists());
        assert!(temp_dir.path().exists()); // Directory should still exist
    }
}

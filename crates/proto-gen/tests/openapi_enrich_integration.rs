use std::fs;

use tempfile::TempDir;

/// Test that gnostic-style `$ref` values are rewritten to valid OpenAPI component refs.
#[test]
fn test_gnostic_ref_rewriting() {
    let dir = TempDir::new().unwrap();
    let spec_path = dir.path().join("openapi.yaml");
    let jsonschema_dir = dir.path().join("jsonschema");
    fs::create_dir_all(&jsonschema_dir).unwrap();

    // Use concat to avoid Rust 2021 reserved-prefix lint on `json"` in raw strings
    let gnostic_ref = concat!("#/$defs/example.catalog.v1.Catalog.schema.strict.", "json");
    let yaml = format!(
        "openapi: \"3.0.0\"\ninfo:\n  title: Test\n  version: \"1.0\"\ncomponents:\n  schemas:\n    Catalog:\n      type: object\n      properties:\n        id:\n          $ref: \"{gnostic_ref}\"\n"
    );
    fs::write(&spec_path, yaml).unwrap();

    proto_gen::enrich_openapi(&spec_path, &jsonschema_dir, false, None).unwrap();

    let result = fs::read_to_string(&spec_path).unwrap();
    assert!(
        result.contains("#/components/schemas/Catalog"),
        "expected rewritten ref in output:\n{result}"
    );
    assert!(
        !result.contains("#/$defs/"),
        "expected no gnostic refs remaining:\n{result}"
    );
}

/// Test that a valid openapi.yaml with no gnostic refs round-trips without corruption.
#[test]
fn test_round_trip_without_jsonschema() {
    let dir = TempDir::new().unwrap();
    let spec_path = dir.path().join("openapi.yaml");
    let jsonschema_dir = dir.path().join("jsonschema");
    fs::create_dir_all(&jsonschema_dir).unwrap();

    let yaml = r#"
openapi: "3.0.0"
info:
  title: Round Trip Test
  version: "1.0"
paths:
  /catalogs:
    get:
      summary: List catalogs
      responses:
        "200":
          description: OK
components:
  schemas:
    Catalog:
      type: object
      properties:
        name:
          type: string
"#;
    fs::write(&spec_path, yaml).unwrap();

    proto_gen::enrich_openapi(&spec_path, &jsonschema_dir, false, None).unwrap();

    let result = fs::read_to_string(&spec_path).unwrap();
    // Must parse back as valid YAML without error
    let parsed: serde_yaml::Value =
        serde_yaml::from_str(&result).expect("output should be valid YAML after round-trip");
    assert!(
        parsed.get("openapi").is_some(),
        "output should retain 'openapi' key"
    );
}

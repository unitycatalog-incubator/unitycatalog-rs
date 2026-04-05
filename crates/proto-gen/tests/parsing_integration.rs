use proto_gen::parsing::{CodeGenMetadata, parse_file_descriptor_set};
use protobuf::Message;
use protobuf::descriptor::FileDescriptorSet;

fn load_descriptor() -> FileDescriptorSet {
    let bytes = include_bytes!("../proto/example.bin");
    FileDescriptorSet::parse_from_bytes(bytes).expect("valid descriptor")
}

fn parse_meta() -> CodeGenMetadata {
    parse_file_descriptor_set(&load_descriptor()).expect("parse succeeded")
}

#[test]
fn test_parses_services_and_methods() {
    let meta = parse_meta();
    // CatalogService + SchemaService
    assert_eq!(meta.services.len(), 2);

    let catalog = meta
        .services
        .get("CatalogService")
        .expect("CatalogService exists");
    assert_eq!(catalog.methods.len(), 5); // Create/Get/List/Update/Delete

    let schema = meta
        .services
        .get("SchemaService")
        .expect("SchemaService exists");
    assert_eq!(schema.methods.len(), 2); // ListByTags + ListByCatalogType
}

#[test]
fn test_parses_messages() {
    let meta = parse_meta();
    let key = ".example.catalog.v1.Catalog";
    assert!(meta.messages.contains_key(key), "expected key {key}");
    let catalog = &meta.messages[key];
    assert!(
        catalog.resource_descriptor.is_some(),
        "Catalog should have resource descriptor"
    );
}

#[test]
fn test_parses_field_behavior() {
    use proto_gen::google::api::FieldBehavior;
    use proto_gen::parsing::MessageField;

    let meta = parse_meta();
    let catalog = &meta.messages[".example.catalog.v1.Catalog"];

    let name_field: &MessageField = catalog
        .fields
        .iter()
        .find(|f| f.name == "name")
        .expect("name field exists");

    assert!(
        name_field
            .field_behavior
            .contains(&FieldBehavior::OutputOnly),
        "name field should be OUTPUT_ONLY"
    );
}

#[test]
fn test_parses_enums() {
    let meta = parse_meta();
    let key = ".example.catalog.v1.CatalogType";
    let catalog_type = meta.enums.get(key).expect("CatalogType enum exists");
    // UNSPECIFIED + MANAGED + DELTASHARING
    assert_eq!(catalog_type.values.len(), 3);
    assert!(
        catalog_type
            .values
            .iter()
            .any(|v| v.name == "CATALOG_TYPE_UNSPECIFIED")
    );
    assert!(
        catalog_type
            .values
            .iter()
            .any(|v| v.name == "MANAGED_CATALOG")
    );
    assert!(
        catalog_type
            .values
            .iter()
            .any(|v| v.name == "DELTASHARING_CATALOG")
    );
}

#[test]
fn test_http_patterns() {
    let meta = parse_meta();
    let catalog = meta
        .services
        .get("CatalogService")
        .expect("CatalogService exists");

    let get_method = catalog
        .methods
        .iter()
        .find(|m| m.method_name == "GetCatalog")
        .expect("GetCatalog method exists");

    let (http_method, path) = get_method.http_info().expect("http_info present");
    assert_eq!(http_method, "GET");
    assert_eq!(path, "/catalogs/{name}");
    assert_eq!(get_method.path_parameters(), vec!["name"]);
}

#[test]
fn test_post_with_body() {
    let meta = parse_meta();
    let catalog = meta
        .services
        .get("CatalogService")
        .expect("CatalogService exists");

    let create_method = catalog
        .methods
        .iter()
        .find(|m| m.method_name == "CreateCatalog")
        .expect("CreateCatalog method exists");

    let (http_method, path) = create_method.http_info().expect("http_info present");
    assert_eq!(http_method, "POST");
    assert_eq!(path, "/catalogs");
    assert_eq!(create_method.path_parameters(), Vec::<String>::new());
}

#[test]
fn test_patch_method() {
    let meta = parse_meta();
    let catalog = meta
        .services
        .get("CatalogService")
        .expect("CatalogService exists");

    let update_method = catalog
        .methods
        .iter()
        .find(|m| m.method_name == "UpdateCatalog")
        .expect("UpdateCatalog method exists");

    let (http_method, path) = update_method.http_info().expect("http_info present");
    assert_eq!(http_method, "PATCH");
    assert_eq!(path, "/catalogs/{name}");
}

#[test]
fn test_delete_method() {
    let meta = parse_meta();
    let catalog = meta
        .services
        .get("CatalogService")
        .expect("CatalogService exists");

    let delete_method = catalog
        .methods
        .iter()
        .find(|m| m.method_name == "DeleteCatalog")
        .expect("DeleteCatalog method exists");

    let (http_method, path) = delete_method.http_info().expect("http_info present");
    assert_eq!(http_method, "DELETE");
    assert_eq!(path, "/catalogs/{name}");
    assert_eq!(delete_method.path_parameters(), vec!["name"]);
}

#[test]
fn test_repeated_query_param() {
    let meta = parse_meta();
    let schema_svc = meta
        .services
        .get("SchemaService")
        .expect("SchemaService exists");

    let list_by_tags = schema_svc
        .methods
        .iter()
        .find(|m| m.method_name == "ListByTags")
        .expect("ListByTags method exists");

    // tags field should be repeated
    let tags_field = list_by_tags
        .input_fields
        .iter()
        .find(|f| f.name == "tags")
        .expect("tags field exists");

    assert!(tags_field.repeated, "tags should be a repeated field");
}

#[test]
fn test_enum_query_param() {
    let meta = parse_meta();
    let schema_svc = meta
        .services
        .get("SchemaService")
        .expect("SchemaService exists");

    let list_by_type = schema_svc
        .methods
        .iter()
        .find(|m| m.method_name == "ListByCatalogType")
        .expect("ListByCatalogType method exists");

    // catalog_type field should be an enum
    let ct_field = list_by_type
        .input_fields
        .iter()
        .find(|f| f.name == "catalog_type")
        .expect("catalog_type field exists");

    use proto_gen::parsing::types::BaseType;
    assert!(
        matches!(ct_field.unified_type.base_type, BaseType::Enum(_)),
        "catalog_type should be an Enum type"
    );
}

#[test]
fn test_map_field() {
    let meta = parse_meta();
    let catalog = &meta.messages[".example.catalog.v1.Catalog"];

    let props_field = catalog
        .fields
        .iter()
        .find(|f| f.name == "properties")
        .expect("properties field exists");

    use proto_gen::parsing::types::BaseType;
    assert!(
        matches!(props_field.unified_type.base_type, BaseType::Map(_, _)),
        "properties should be a Map type"
    );
}

#[test]
fn test_oneof_field() {
    let meta = parse_meta();
    let storage = &meta.messages[".example.catalog.v1.StorageConfig"];

    let provider_field = storage
        .fields
        .iter()
        .find(|f| f.name == "provider")
        .expect("provider oneof field exists");

    use proto_gen::parsing::types::BaseType;
    assert!(
        matches!(provider_field.unified_type.base_type, BaseType::OneOf(_)),
        "provider should be a OneOf type"
    );
    assert!(
        provider_field.oneof_variants.is_some(),
        "provider should have oneof variants"
    );
    let variants = provider_field.oneof_variants.as_ref().unwrap();
    assert_eq!(variants.len(), 2); // s3, azure
}

#[test]
fn test_resource_descriptor_content() {
    let meta = parse_meta();
    let catalog = &meta.messages[".example.catalog.v1.Catalog"];
    let rd = catalog
        .resource_descriptor
        .as_ref()
        .expect("resource descriptor exists");
    assert_eq!(rd.r#type, "example.io/Catalog");
    assert!(rd.pattern.iter().any(|p| p.contains("catalogs/{catalog}")));
}

mod http_pattern_tests {
    use rstest::rstest;

    // Access HttpPattern via its public type alias through parsing internals.
    // Since HttpPattern is pub(crate), we test through the public API indirectly.
    // These tests verify extract_path_parameters behavior via the public parse API.

    #[rstest]
    #[case("/items", 0)]
    #[case("/items/{id}", 1)]
    #[case("/items/{id}/sub", 1)]
    #[case("/a/{x}/b/{y}", 2)]
    fn test_path_parameter_count(#[case] template: &str, #[case] expected_count: usize) {
        use proto_gen::google::api::{HttpRule, http_rule::Pattern};
        use proto_gen::parsing::MethodMetadata;

        let http_rule = HttpRule {
            pattern: Some(Pattern::Get(template.to_string())),
            ..Default::default()
        };

        let method = MethodMetadata {
            service_name: "TestService".to_string(),
            method_name: "TestMethod".to_string(),
            input_type: ".test.TestRequest".to_string(),
            output_type: ".test.TestResponse".to_string(),
            operation: None,
            http_rule,
            input_fields: vec![],
            documentation: None,
        };

        assert_eq!(method.path_parameters().len(), expected_count);
    }
}

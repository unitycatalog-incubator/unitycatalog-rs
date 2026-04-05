use proto_gen::analysis::{RequestType, analyze_metadata};
use proto_gen::parsing::{CodeGenMetadata, parse_file_descriptor_set};
use protobuf::Message;
use protobuf::descriptor::FileDescriptorSet;
use rstest::rstest;

fn load_descriptor() -> FileDescriptorSet {
    let bytes = include_bytes!("../proto/example.bin");
    FileDescriptorSet::parse_from_bytes(bytes).expect("valid descriptor")
}

fn parse_meta() -> CodeGenMetadata {
    parse_file_descriptor_set(&load_descriptor()).expect("parse succeeded")
}

#[test]
fn test_analyze_metadata_produces_service_plan() {
    let meta = parse_meta();
    let plan = analyze_metadata(&meta).unwrap();
    assert!(!plan.services.is_empty());
}

#[test]
fn test_extract_managed_resources_dedup() {
    let meta = parse_meta();
    let plan = analyze_metadata(&meta).unwrap();
    let catalog_svc = plan
        .services
        .iter()
        .find(|s| s.service_name == "CatalogService")
        .unwrap();
    assert_eq!(catalog_svc.managed_resources.len(), 1);
    assert_eq!(catalog_svc.managed_resources[0].type_name, "Catalog");
}

fn find_method_plan<'a>(
    plan: &'a proto_gen::analysis::GenerationPlan,
    service_name: &str,
    method_name: &str,
) -> &'a proto_gen::analysis::MethodPlan {
    plan.services
        .iter()
        .find(|s| s.service_name == service_name)
        .unwrap_or_else(|| panic!("service {service_name} not found"))
        .methods
        .iter()
        .find(|m| m.metadata.method_name == method_name)
        .unwrap_or_else(|| panic!("method {method_name} not found"))
}

#[rstest]
#[case("GetCatalog", RequestType::Get)]
#[case("CreateCatalog", RequestType::Create)]
#[case("UpdateCatalog", RequestType::Update)]
#[case("DeleteCatalog", RequestType::Delete)]
#[case("ListCatalogs", RequestType::List)]
fn test_catalog_request_type(#[case] method_name: &str, #[case] expected: RequestType) {
    let meta = parse_meta();
    let plan = analyze_metadata(&meta).unwrap();
    let method_plan = find_method_plan(&plan, "CatalogService", method_name);
    assert_eq!(method_plan.request_type, expected);
}

#[rstest]
#[case("ListByTags")]
#[case("ListByCatalogType")]
fn test_schema_service_custom_request_type(#[case] method_name: &str) {
    let meta = parse_meta();
    let plan = analyze_metadata(&meta).unwrap();
    let method_plan = find_method_plan(&plan, "SchemaService", method_name);
    assert!(matches!(method_plan.request_type, RequestType::Custom(_)));
}

#[test]
fn test_get_catalog_has_path_param() {
    let meta = parse_meta();
    let plan = analyze_metadata(&meta).unwrap();
    let method_plan = find_method_plan(&plan, "CatalogService", "GetCatalog");
    let path_params: Vec<_> = method_plan.path_parameters().collect();
    assert_eq!(path_params.len(), 1);
    assert_eq!(path_params[0].name, "name");
}

#[test]
fn test_create_catalog_has_no_path_params() {
    let meta = parse_meta();
    let plan = analyze_metadata(&meta).unwrap();
    let method_plan = find_method_plan(&plan, "CatalogService", "CreateCatalog");
    assert_eq!(method_plan.path_parameters().count(), 0);
}

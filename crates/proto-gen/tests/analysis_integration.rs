use proto_gen::analysis::{MethodPlanner, RequestType, analyze_metadata};
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

#[rstest]
#[case("GetCatalog", RequestType::Get)]
#[case("CreateCatalog", RequestType::Create)]
#[case("UpdateCatalog", RequestType::Update)]
#[case("DeleteCatalog", RequestType::Delete)]
fn test_request_type(#[case] method_name: &str, #[case] expected: RequestType) {
    let meta = parse_meta();
    let catalog_svc = meta.services.get("CatalogService").unwrap();
    let method = catalog_svc
        .methods
        .iter()
        .find(|m| m.method_name == method_name)
        .unwrap();
    let planner = MethodPlanner::try_new(method, &meta).unwrap();
    assert_eq!(planner.request_type(), expected);
}

#[test]
fn test_list_catalogs_request_type() {
    let meta = parse_meta();
    let catalog_svc = meta.services.get("CatalogService").unwrap();
    let method = catalog_svc
        .methods
        .iter()
        .find(|m| m.method_name == "ListCatalogs")
        .unwrap();
    let planner = MethodPlanner::try_new(method, &meta).unwrap();
    assert_eq!(planner.request_type(), RequestType::List);
}

#[rstest]
#[case("ListByTags")]
#[case("ListByCatalogType")]
fn test_schema_service_custom_request_type(#[case] method_name: &str) {
    let meta = parse_meta();
    let schema_svc = meta.services.get("SchemaService").unwrap();
    let method = schema_svc
        .methods
        .iter()
        .find(|m| m.method_name == method_name)
        .unwrap();
    let planner = MethodPlanner::try_new(method, &meta).unwrap();
    assert!(matches!(planner.request_type(), RequestType::Custom(_)));
}

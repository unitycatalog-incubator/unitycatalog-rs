# Unity Catalog Client Test Framework

This directory contains a comprehensive test framework for the Unity Catalog Rust client implementation.
The framework provides both unit tests with mock servers, user journey testing, and integration test
capabilities against real Unity Catalog deployments.

## Overview

The test framework is designed to be:
- **Maintainable**: Uses fixtures and shared utilities to reduce code duplication
- **Comprehensive**: Covers CRUD operations, error handling, edge cases, and multi-step workflows
- **Flexible**: Supports both mock testing and integration testing
- **Extensible**: Easy to add new test scenarios, user journeys, and client types
- **Journey-based**: Tests real-world workflows with dependent API calls

## Architecture

### Core Components

- **`test_utils/`**: Shared utilities and infrastructure
  - `mod.rs`: Main test utilities and mock server setup
  - `fixtures.rs`: Reusable test fixtures using `rstest`
  - `responses.rs`: Pre-defined response data for consistent testing

- **`test_data/`**: JSON response files for different scenarios
  - `catalogs/`: Catalog-specific response data
  - `schemas/`: Schema-specific response data
  - `journeys/`: User journey definitions in JSON format
  - `errors/`: Error response samples

- **Test Files**:
  - `catalog_tests.rs`: Comprehensive catalog client tests
  - `journey_tests.rs`: User journey-based tests
  - `journey_integration_recorder.rs`: Integration test recorder for capturing real responses
  - `integration_test_helper.rs`: Integration test utilities and configuration

## Dependencies

The framework uses the following testing libraries:

```toml
[dev-dependencies]
rstest = "0.18"           # Parameterized tests and fixtures
mockito = "1.4"           # HTTP mocking
tokio = "1.0"             # Async runtime
tokio-test = "0.4"        # Async test utilities
serde_json = "1.0"        # JSON handling
tempfile = "3.1"          # Temporary files for tests
reqwest = "0.11"          # HTTP client for integration recording
chrono = "0.4"            # Date/time handling
uuid = "1.0"              # UUID generation
```

## Running Tests

### Unit Tests (Mock Server)

Run all unit tests with mock servers:

```bash
cargo test --package unitycatalog-client
```

Run specific test categories:

```bash
# Catalog tests only
cargo test catalog_tests

# Journey tests only
cargo test journey_tests

# Error handling tests
cargo test error_handling

# Specific test function
cargo test test_create_catalog_basic
```

### Integration Tests

To run integration tests against a real Unity Catalog server:

1. Set environment variables:
```bash
export UC_SERVER_URL="http://localhost:8080"
export UC_AUTH_TOKEN="your-auth-token"
export RUN_INTEGRATION_TESTS=true
```

2. Run tests:
```bash
cargo test --package unitycatalog-client
```

### Response Capture Mode

To capture responses from a real server for use in mock tests:

```bash
export CAPTURE_RESPONSES=true
export UC_SERVER_URL="http://localhost:8080"
export UC_AUTH_TOKEN="your-auth-token"
cargo test --package unitycatalog-client
```

Captured responses will be saved to `tests/captured_responses/`.

### Journey Response Recording

To record real server responses for journey testing:

```bash
export RECORD_JOURNEY_RESPONSES=true
export UC_SERVER_URL="http://localhost:8080"
export UC_AUTH_TOKEN="your-auth-token"
cargo test --package unitycatalog-client
```

Recorded journey files will be saved to `tests/test_data/journeys/recorded/`.

## Test Structure

### User Journey Testing

The framework supports multi-step user journeys that test dependent API operations:

```rust
#[rstest]
#[tokio::test]
async fn test_catalog_lifecycle_journey(
    #[future] journey_test_setup: (UnityCatalogClient, TestServer),
    journey_variables: HashMap<String, serde_json::Value>,
) {
    let (client, server) = journey_test_setup.await;

    // Load journey definition from JSON
    let journey = JourneyLoader::load_journey("catalog_lifecycle.json")
        .expect("Failed to load journey");

    // Execute journey with mock server
    let mut executor = JourneyExecutor::new(client, Some(server))
        .with_variables(journey_variables);

    let result = executor.execute_journey(journey).await;
    assert!(result.success);
}
```

### Journey Definition Format

Journeys are defined in JSON format with dependent steps:

```json
{
  "name": "catalog_lifecycle",
  "description": "Complete catalog CRUD operations",
  "variables": {
    "catalog_name": "test_catalog",
    "storage_root": "s3://test-bucket/"
  },
  "steps": [
    {
      "id": "create_catalog",
      "description": "Create a new catalog",
      "method": "POST",
      "path": "/catalogs",
      "request_body": {
        "name": "{catalog_name}",
        "storage_root": "{storage_root}"
      },
      "expected_status": 201,
      "extract_variables": {
        "catalog_id": "$.name"
      }
    },
    {
      "id": "get_catalog",
      "description": "Retrieve the created catalog",
      "method": "GET",
      "path": "/catalogs/{catalog_name}",
      "expected_status": 200,
      "depends_on": ["create_catalog"]
    }
  ]
}
```

### Using Fixtures

The framework provides reusable fixtures using `rstest`:

```rust
#[rstest]
#[tokio::test]
async fn test_create_catalog(
    #[future] test_client: (UnityCatalogClient, TestServer),
    catalog_name: String,
    catalog_properties: HashMap<String, String>,
) {
    let (client, mut server) = test_client.await;
    // Test implementation...
}
```

### Mock Server Setup

Each test automatically gets a configured mock server:

```rust
// Setup mock response
let expected_response = CatalogResponses::catalog_info(&catalog_name, Some("Test catalog"));
let mock = server
    .mock_catalog_endpoint("POST", "/api/2.1/unity-catalog/catalogs")
    .with_status(201)
    .with_header("content-type", "application/json")
    .with_body(serde_json::to_string(&expected_response).unwrap())
    .create();

// Execute test
let result = catalog.create(/* parameters */).await;

// Verify
mock.assert();
assert!(result.is_ok());
```

### Test Data Management

Test data is organized in JSON files:

```
test_data/
├── catalogs/
│   ├── get_catalog.json           # Standard catalog response
│   ├── list_catalogs.json         # List response with multiple catalogs
│   ├── sharing_catalog.json       # Sharing catalog response
│   └── catalog_not_found.json     # Error response
├── schemas/
│   └── basic_schema.json
├── journeys/
│   ├── catalog_lifecycle.json     # Complete catalog CRUD journey
│   ├── hierarchical_data_structure.json  # Catalog -> Schema -> Table journey
│   ├── error_handling.json        # Error scenario journey
│   └── recorded/                  # Recorded responses from real server
└── errors/
    └── various error scenarios
```

Load test data in tests:

```rust
let expected_response = TestDataLoader::load_response("catalogs", "get_catalog.json")
    .expect("Failed to load test data");
```

## Test Categories

### CRUD Operations
- Create catalogs (managed and sharing)
- Get catalog information
- Update catalog properties
- Delete catalogs
- List catalogs with pagination

### User Journey Testing
- **Catalog Lifecycle**: Complete CRUD operations with dependencies
- **Hierarchical Data Structure**: Catalog → Schema → Table creation and cleanup
- **Error Handling Journey**: Various failure scenarios and error conditions
- **Multi-step Workflows**: Real-world usage patterns with variable extraction

### Error Handling
- 404 Not Found
- 409 Conflict (already exists)
- 403 Permission Denied
- 400 Bad Request
- 500 Internal Server Error
- 429 Rate Limiting

### Edge Cases
- Invalid catalog names
- Empty responses
- Concurrent operations
- Large property sets
- Unicode handling

### Performance Tests
- Pagination with large datasets
- Concurrent operations
- Timeout handling

## Adding New Tests

### 1. Adding Journey Tests

Create a new journey definition file:

```bash
# Add new journey scenario
cat > test_data/journeys/my_workflow.json << EOF
{
  "name": "my_workflow",
  "description": "Description of the workflow",
  "variables": {
    "resource_name": "test_resource"
  },
  "steps": [
    {
      "id": "create_resource",
      "description": "Create the resource",
      "method": "POST",
      "path": "/resources",
      "request_body": {"name": "{resource_name}"},
      "expected_status": 201,
      "extract_variables": {"resource_id": "$.id"}
    }
  ]
}
EOF
```

### 2. Create Test Data

Add response files to appropriate directory:

```bash
# Add new catalog scenario
echo '{"name": "new_scenario", ...}' > test_data/catalogs/new_scenario.json
```

### 3. Add Fixtures (if needed)

Add new fixtures to `test_utils/fixtures.rs`:

```rust
#[fixture]
pub fn new_test_scenario() -> SomeType {
    // Fixture implementation
}
```

### 4. Write Tests

Add tests to appropriate test file:

```rust
#[rstest]
#[tokio::test]
async fn test_new_functionality(
    #[future] test_client: (UnityCatalogClient, TestServer),
    new_test_scenario: SomeType,
) {
    // Test implementation
}
```

## Test Configuration

Create `tests/test_config.toml` for custom configuration:

```toml
operation_timeout_secs = 30
response_capture_dir = "tests/captured_responses"
test_data_dir = "tests/test_data"
```

## Environment Variables

- `UC_SERVER_URL`: Unity Catalog server URL for integration tests
- `UC_AUTH_TOKEN`: Authentication token for integration tests
- `RUN_INTEGRATION_TESTS`: Set to "true" to enable integration tests
- `CAPTURE_RESPONSES`: Set to "true" to capture real server responses
- `RECORD_JOURNEY_RESPONSES`: Set to "true" to record real responses for journey files
- `OVERWRITE_JOURNEY_RESPONSES`: Set to "true" to overwrite existing recorded journey files
- `RECORD_SUCCESS_ONLY`: Set to "true" to only record successful responses (default)

## Best Practices

### Test Organization
- Group related tests in the same file
- Use descriptive test names that explain the scenario
- Use fixtures for common setup to reduce duplication
- Use journey tests for multi-step workflows and dependent operations

### Journey Design
- Design journeys to test real-world usage patterns
- Use variable extraction to pass data between steps
- Include proper cleanup steps with `continue_on_failure: true`
- Tag steps appropriately (setup, main, cleanup)
- Define clear dependencies between steps

### Mock Responses
- Use realistic response data that matches actual server responses
- Include all relevant fields in mock responses
- Test both success and error scenarios
- Record real responses when possible for accuracy

### Assertions
- Use the provided `TestAssertions` helper for common validations
- Verify both the operation result and that the mock was called
- Test edge cases and boundary conditions
- Validate journey execution results and variable extraction

### Performance
- Use `#[tokio::test]` for async tests
- Set reasonable timeouts for operations
- Test concurrent operations where relevant
- Consider journey execution time when designing complex workflows

## Troubleshooting

### Common Issues

1. **Mock not called**: Ensure the endpoint path and method match exactly
2. **JSON parsing errors**: Verify test data files have valid JSON
3. **Test timeouts**: Check if the operation is hanging; increase timeout if needed
4. **Missing fixtures**: Ensure fixture functions are public and properly annotated
5. **Journey validation errors**: Check step dependencies and variable references
6. **Variable substitution issues**: Ensure variables are properly defined and extracted
7. **Step dependency failures**: Verify dependent steps complete successfully

### Debug Tips

- Use `RUST_LOG=debug` to see detailed logging
- Print mock server URLs to verify endpoint configuration
- Use `--nocapture` flag to see test output: `cargo test -- --nocapture`
- Enable journey recording to see actual server responses: `RECORD_JOURNEY_RESPONSES=true`
- Validate journey files using `JourneyLoader::validate_journey()`
- Check variable substitution in journey execution context

## Future Enhancements

Planned improvements to the test framework:

1. **Advanced JSONPath support**: Full JSONPath library for complex variable extraction
2. **Journey composition**: Ability to compose journeys from smaller reusable components
3. **Performance benchmarks**: Add criterion.rs benchmarks for journey execution
4. **Schema validation**: Validate responses against OpenAPI schemas
5. **Test report generation**: Generate HTML test reports with journey visualization
6. **Parallel journey execution**: Execute independent journey steps in parallel
7. **Journey debugging tools**: Interactive journey execution and step-by-step debugging

## Contributing

When adding new tests:

1. Follow the existing patterns and structure
2. Add appropriate test data files and journey definitions
3. Use fixtures for reusable components
4. Include both positive and negative test cases
5. Design journeys for real-world workflows with proper dependencies
6. Record actual server responses when possible
7. Update this README if adding new concepts or patterns

## Related Documentation

- [Unity Catalog API Documentation](https://docs.databricks.com/en/data-governance/unity-catalog/index.html)
- [rstest Documentation](https://docs.rs/rstest/)
- [mockito Documentation](https://docs.rs/mockito/)
- [Tokio Testing Guide](https://tokio.rs/tokio/topics/testing)
- [JSONPath Specification](https://goessner.net/articles/JsonPath/)
- [Journey Testing Best Practices](docs/journey-testing-guide.md)
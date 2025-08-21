# Unity Catalog Acceptance Testing Framework

A comprehensive testing framework for Unity Catalog that provides journey-based testing, mock server support, and response recording capabilities.

## Overview

The Unity Catalog Acceptance Testing Framework enables you to test complete user workflows through "journeys" - multi-step sequences of API calls with dependency management and variable passing between steps. This approach provides more realistic testing scenarios compared to isolated unit tests.

## Features

- **Journey-Based Testing**: Define multi-step workflows in JSON format
- **Dependency Management**: Automatic step ordering based on dependencies
- **Variable Substitution**: Pass data between steps using extracted variables
- **Mock Server Support**: Fast testing with configurable mock responses
- **Response Recording**: Capture real server responses for test data
- **Integration Testing**: Execute against live Unity Catalog instances
- **Assertion Helpers**: Rich set of assertion functions for validation
- **Test Data Builders**: Convenient builders for creating test data

## Quick Start

### Basic Usage

```rust
use unitycatalog_acceptance::{
    journey::{JourneyExecutor, JourneyLoader},
    mock::TestServer,
};
use std::collections::HashMap;

#[tokio::test]
async fn test_catalog_lifecycle() {
    // Set up mock server
    let server = TestServer::new().await;
    let client = server.create_client();

    // Load journey definition
    let journey = JourneyLoader::load_journey("catalog_lifecycle.json").unwrap();

    // Execute journey
    let mut executor = JourneyExecutor::new(client, Some(server))
        .with_variables(HashMap::new());

    let result = executor.execute_journey(journey).await;
    assert!(result.success);
}
```

### Creating Test Data

```rust
use unitycatalog_acceptance::models::{CatalogBuilder, TestDataUtils};

// Using builders
let catalog = CatalogBuilder::new("test_catalog")
    .with_comment("Test catalog")
    .with_storage_root("s3://test-bucket/catalogs/test")
    .with_property("environment", "test")
    .build_json();

// Using utilities
let unique_name = TestDataUtils::sanitize_name("test-catalog-123");
let timestamp = TestDataUtils::timestamp();
```

### Journey Definitions

Journeys are defined in JSON format and stored in `test_data/journeys/`:

```json
{
  "name": "catalog_lifecycle",
  "description": "Complete catalog CRUD operations",
  "variables": {
    "catalog_name": "test_catalog_{timestamp}"
  },
  "steps": [
    {
      "id": "create_catalog",
      "description": "Create a test catalog",
      "method": "POST",
      "path": "/api/2.1/unity-catalog/catalogs",
      "request_body": {
        "name": "{catalog_name}",
        "comment": "Test catalog"
      },
      "expected_status": 201,
      "extract_variables": {
        "catalog_id": "$.name"
      },
      "tags": ["setup"]
    },
    {
      "id": "get_catalog",
      "description": "Verify catalog creation",
      "method": "GET",
      "path": "/api/2.1/unity-catalog/catalogs/{catalog_name}",
      "expected_status": 200,
      "depends_on": ["create_catalog"],
      "tags": ["verification"]
    },
    {
      "id": "delete_catalog",
      "description": "Cleanup test catalog",
      "method": "DELETE",
      "path": "/api/2.1/unity-catalog/catalogs/{catalog_name}",
      "expected_status": 200,
      "depends_on": ["get_catalog"],
      "continue_on_failure": true,
      "tags": ["cleanup"]
    }
  ]
}
```

## Module Organization

### Core Modules

- **`journey`**: Journey execution engine and data structures
- **`mock`**: Mock server utilities and test fixtures
- **`recorder`**: Response recording for integration testing
- **`models`**: Shared data models and builders
- **`assertions`**: Common assertion helpers

### Journey Module

The journey module provides the core execution engine:

```rust
use unitycatalog_acceptance::journey::{
    JourneyExecutor, JourneyLoader, UserJourney, JourneyStep
};

// Load journey from file
let journey = JourneyLoader::load_journey("my_journey.json")?;

// Create executor
let mut executor = JourneyExecutor::new(client, Some(server));

// Execute journey
let result = executor.execute_journey(journey).await;
```

### Mock Module

Mock server functionality for fast testing:

```rust
use unitycatalog_acceptance::mock::{TestServer, TestFixtures};

// Create mock server
let mut server = TestServer::new().await;

// Set up default mocks
server.setup_default_mocks().await;

// Create custom mock
server.mock_catalog_endpoint("GET", "/api/2.1/unity-catalog/catalogs")
    .with_status(200)
    .with_body(&TestFixtures::catalog_info("test").to_string())
    .create_async()
    .await;
```

### Recorder Module

Record real API responses for test data:

```rust
use unitycatalog_acceptance::recorder::{JourneyRecorder, RecordingConfig};

// Configure recording
let config = RecordingConfig::from_env()?;
let mut recorder = JourneyRecorder::new(config)?;

// Record journey
let recorded = recorder.record_journey(journey).await?;
```

### Models Module

Data builders and test utilities:

```rust
use unitycatalog_acceptance::models::{
    CatalogBuilder, SchemaBuilder, TableBuilder, TestContext
};

// Create test context
let mut context = TestContext::new("test_run_123");
context.set_variable("environment", "test");

// Build test data
let catalog = CatalogBuilder::new("test_catalog")
    .with_comment("Test catalog")
    .build_json();
```

### Assertions Module

Rich assertion helpers:

```rust
use unitycatalog_acceptance::assertions::TestAssertions;

// Journey assertions
TestAssertions::assert_journey_success(&result);
TestAssertions::assert_variables_extracted(&result, &["catalog_id"]);

// JSON assertions
TestAssertions::assert_json_contains_fields(&response, &["name", "created_at"]);
TestAssertions::assert_json_field_equals(&response, "name", &expected_name);

// Unity Catalog specific assertions
TestAssertions::assert_unity_catalog_naming("test_catalog", "Catalog");
TestAssertions::assert_timestamp_is_recent(&response, "created_at");
```

## Testing Modes

### Mock Testing (Default)

Fast testing with mock servers:

```rust
#[tokio::test]
async fn test_with_mocks() {
    let server = TestServer::new().await;
    let client = server.create_client();
    
    // Mock server provides predictable responses
    let mut executor = JourneyExecutor::new(client, Some(server));
    let result = executor.execute_journey(journey).await;
    
    assert!(result.success);
}
```

### Integration Testing

Testing against real Unity Catalog servers:

```bash
# Enable integration testing
export RUN_INTEGRATION_TESTS=true
export UC_SERVER_URL="http://localhost:8080"
export UC_AUTH_TOKEN="your-auth-token"

cargo test
```

```rust
#[tokio::test]
async fn test_integration() {
    let config = IntegrationConfig::from_env();
    
    if !config.enabled {
        return; // Skip when not configured
    }
    
    // Test against real server
    let client = create_integration_client(&config);
    let mut executor = JourneyExecutor::new(client, None);
    let result = executor.execute_journey(journey).await;
    
    assert!(result.success);
}
```

### Response Recording

Capture real responses for mock data:

```bash
# Enable recording
export RECORD_JOURNEY_RESPONSES=true
export UC_SERVER_URL="http://localhost:8080"
export UC_AUTH_TOKEN="your-auth-token"

cargo test
```

Recorded responses are saved to `test_data/journeys/recorded/` and can be used to create more accurate mocks.

## Environment Variables

### Integration Testing
- `RUN_INTEGRATION_TESTS`: Enable integration tests (default: false)
- `UC_SERVER_URL`: Unity Catalog server URL
- `UC_AUTH_TOKEN`: Authentication token for server

### Response Recording
- `RECORD_JOURNEY_RESPONSES`: Enable response recording (default: false)
- `JOURNEY_RECORDING_DIR`: Output directory for recordings (default: test_data/journeys/recorded)
- `RECORD_SUCCESS_ONLY`: Only record successful responses (default: true)
- `OVERWRITE_JOURNEY_RESPONSES`: Overwrite existing recordings (default: false)

### Test Configuration
- `TEST_CATALOG_PREFIX`: Prefix for test catalogs (default: test)
- `TEST_SUFFIX`: Suffix for test resources (default: random)
- `REQUEST_TIMEOUT_SECS`: HTTP request timeout (default: 30)

## Journey Definition Reference

### Journey Structure

```json
{
  "name": "journey_name",
  "description": "What this journey tests",
  "variables": {
    "variable_name": "default_value"
  },
  "steps": [...],
  "metadata": {
    "author": "test_author",
    "tags": ["integration", "smoke"]
  }
}
```

### Step Structure

```json
{
  "id": "unique_step_id",
  "description": "What this step does",
  "method": "GET|POST|PUT|DELETE|PATCH",
  "path": "/api/path/with/{variables}",
  "request_body": {...},
  "expected_status": 200,
  "expected_response": {...},
  "extract_variables": {
    "var_name": "$.json.path"
  },
  "depends_on": ["other_step_id"],
  "continue_on_failure": false,
  "tags": ["setup", "main", "cleanup"]
}
```

### Variable Substitution

Variables use `{variable_name}` syntax and can be:
- Defined in the journey variables section
- Extracted from previous step responses using JSONPath
- Set by the test environment

### Dependency Management

Steps can depend on other steps using the `depends_on` field:
- Dependencies are resolved automatically
- Dependent steps only run if their dependencies succeed
- Circular dependencies are detected and reported

### Tags

Steps can be tagged for organization:
- `setup`: Resource creation steps
- `main`: Core test logic
- `cleanup`: Resource cleanup (usually with `continue_on_failure: true`)
- `verification`: Validation steps

## Best Practices

### Journey Design

1. **Start Simple**: Begin with basic workflows, add complexity gradually
2. **Use Dependencies**: Model real-world step dependencies properly
3. **Extract Variables**: Pass important data between steps
4. **Plan Cleanup**: Always include cleanup steps with error tolerance
5. **Tag Appropriately**: Use tags to organize step purposes

### Test Organization

1. **One Journey Per Test**: Keep journeys focused on single workflows
2. **Reusable Components**: Use builders and utilities for common patterns
3. **Clear Naming**: Use descriptive names for resources and steps
4. **Error Testing**: Include negative test scenarios
5. **Resource Cleanup**: Always clean up test resources

### Performance

1. **Mock by Default**: Use mocks for fast feedback during development
2. **Integration for Validation**: Run integration tests for final validation
3. **Parallel Testing**: Design tests to run independently
4. **Resource Isolation**: Use unique names to avoid conflicts

## Migration from Old Framework

### Before (test_utils)

```rust
mod test_utils;
use test_utils::journeys::{JourneyExecutor, JourneyLoader};
use test_utils::TestServer;

// Old approach with inline test utilities
```

### After (acceptance crate)

```rust
use unitycatalog_acceptance::{
    journey::{JourneyExecutor, JourneyLoader},
    mock::TestServer,
    assertions::TestAssertions,
    models::CatalogBuilder,
};

// New approach with dedicated crate
```

### Migration Steps

1. **Update Dependencies**: Add `unitycatalog-acceptance` to dev-dependencies
2. **Update Imports**: Change imports from `test_utils::` to `unitycatalog_acceptance::`
3. **Use New Assertions**: Replace custom assertions with `TestAssertions`
4. **Use Builders**: Replace inline JSON with builder patterns
5. **Move Journey Files**: Copy journey definitions to acceptance crate

## Examples

See the `tests/` directory for complete examples:
- `acceptance_example_test.rs`: Basic usage patterns
- Journey definitions in `test_data/journeys/`

## Contributing

When adding new functionality:

1. **Add Tests**: Include unit tests for new features
2. **Update Documentation**: Keep README and code comments current
3. **Follow Patterns**: Use existing patterns for consistency
4. **Add Examples**: Include usage examples for new features

## License

Apache 2.0 - See LICENSE file for details.
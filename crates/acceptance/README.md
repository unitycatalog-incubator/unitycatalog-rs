# Unity Catalog Acceptance Testing Framework

A comprehensive testing framework for Unity Catalog that provides both simplified trait-based journeys and legacy JSON-based testing capabilities.

## Overview

The Unity Catalog Acceptance Testing Framework enables you to test complete user workflows through "journeys" - sequences of operations that exercise Unity Catalog functionality. The framework now offers two approaches:

1. **Simplified Journey Framework** (Recommended): Type-safe Rust traits using the actual Unity Catalog client
2. **Legacy JSON Framework**: JSON-based configurations with HTTP requests (deprecated)

## Key Benefits

- **Type Safety**: Use the actual `UnityCatalogClient` instead of raw HTTP requests
- **Simplicity**: Write journeys in Rust code instead of complex JSON configurations
- **Automatic Recording**: Responses are recorded to numbered files for easy comparison
- **Real Client Integration**: Test the actual API surface your applications use
- **Better Error Handling**: Leverage Rust's type system for robust error handling
- **IDE Support**: Full IntelliSense, refactoring, and debugging support

## Features

- **Simplified Journey Framework**: Write journeys as Rust traits with full type safety
- **Automatic Response Recording**: Capture real server responses to numbered files
- **Mock Server Support**: Fast testing with configurable mock responses (legacy)
- **Integration Testing**: Execute against live Unity Catalog instances
- **Multiple Journey Execution**: Run journeys in sequence or parallel
- **Comprehensive Examples**: Ready-to-use journeys for common workflows

## Quick Start

### Simplified Journey Framework (Recommended)

```rust
use unitycatalog_acceptance::{
    simple_journey::{JourneyConfig, SimpleJourneyExecutor},
    journeys::SimpleCatalogJourney,
};

#[tokio::test]
async fn test_catalog_lifecycle() {
    // Configuration from environment variables
    let config = JourneyConfig::default();
    let executor = config.create_executor().unwrap();

    // Execute a pre-built journey
    let journey = SimpleCatalogJourney::new();
    let result = executor.execute_journey(&journey).await.unwrap();
    
    assert!(result.is_success());
    println!("Completed {} steps in {:?}", result.steps_completed, result.duration);
}
```

### Creating Custom Journeys

```rust
use async_trait::async_trait;
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_acceptance::{
    AcceptanceResult,
    simple_journey::{SimpleJourney, JourneyRecorder},
};

struct MyCustomJourney {
    catalog_name: String,
}

#[async_trait]
impl SimpleJourney for MyCustomJourney {
    fn name(&self) -> &str { "my_custom_journey" }
    fn description(&self) -> &str { "Custom journey example" }
    fn tags(&self) -> Vec<&str> { vec!["custom", "example"] }

    async fn execute(
        &self,
        client: &UnityCatalogClient,
        recorder: &mut JourneyRecorder,
    ) -> AcceptanceResult<()> {
        // Create catalog using typed client
        let catalog = client
            .create_catalog(&self.catalog_name)
            .with_comment("My test catalog")
            .execute()
            .await?;

        // Record response automatically
        recorder
            .record_step("create_catalog", "Create test catalog", &catalog)
            .await?;

        Ok(())
    }

    async fn cleanup(
        &self,
        client: &UnityCatalogClient,
        _recorder: &mut JourneyRecorder,
    ) -> AcceptanceResult<()> {
        // Clean up resources
        let _ = client.catalog(&self.catalog_name).delete().await;
        Ok(())
    }
}
```

### Environment Configuration

```bash
# Enable response recording
export RECORD_JOURNEY_RESPONSES=true
export JOURNEY_RECORDING_DIR="./recordings"

# Unity Catalog server configuration
export UC_SERVER_URL="http://localhost:8080"
export UC_AUTH_TOKEN="your-auth-token"  # Optional

# Test configuration
export RUN_INTEGRATION_TESTS=true
export REQUEST_TIMEOUT_SECS=30
```

### Available Journeys

The framework includes several pre-built journeys:

```rust
use unitycatalog_acceptance::journeys::*;

// Basic CRUD operations
let catalog_journey = CatalogLifecycleJourney::new();
let schema_journey = SchemaOperationsJourney::new();
let table_journey = TableOperationsJourney::new();

// Advanced workflows
let sharing_journey = SharingWorkflowJourney::new();

// Execute multiple journeys
let journeys: Vec<Box<dyn SimpleJourney>> = vec![
    Box::new(catalog_journey),
    Box::new(schema_journey),
    Box::new(table_journey),
];

let journey_refs: Vec<&dyn SimpleJourney> = journeys.iter().map(|j| j.as_ref()).collect();
let results = executor.execute_journeys(journey_refs).await?;
```

### Response Recording

When `RECORD_JOURNEY_RESPONSES=true`, the framework automatically records all responses:

```
recordings/
├── catalog_lifecycle/
│   ├── 001_create_catalog.json      # First step response
│   ├── 002_get_catalog.json         # Second step response
│   ├── 003_update_catalog.json      # Third step response
│   └── journey_summary.json         # Complete journey summary
├── schema_operations/
│   ├── 001_setup_create_catalog.json
│   ├── 002_create_primary_schema.json
│   └── journey_summary.json
└── table_operations/
    ├── 001_setup_create_catalog.json
    ├── 002_setup_create_schema.json
    ├── 003_create_managed_table.json
    └── journey_summary.json
```

## Module Organization

### Simplified Journey Framework

- **`simple_journey`**: Core trait-based journey system
- **`journeys`**: Pre-built journey implementations
- **`models`**: Shared data models and builders
- **`assertions`**: Common assertion helpers

### Simplified Journey Module

The main module for the new framework:

```rust
use unitycatalog_acceptance::simple_journey::{
    SimpleJourney, JourneyConfig, JourneyRecorder
};

// Create executor from environment configuration
let config = JourneyConfig::default();
let executor = config.create_executor()?;

// Execute a journey
let journey = MyCustomJourney::new();
let result = executor.execute_journey(&journey).await?;
```

### Pre-built Journeys

Ready-to-use journey implementations:

```rust
use unitycatalog_acceptance::journeys::{
    CatalogLifecycleJourney,    // Basic catalog CRUD operations
    SchemaOperationsJourney,    // Schema management
    TableOperationsJourney,     // Table creation and management
    SharingWorkflowJourney,     // Data sharing scenarios
};

// Use with custom names
let journey = CatalogLifecycleJourney::with_catalog_name("my_test_catalog");
let result = executor.execute_journey(&journey).await?;
```

### Legacy Framework (Deprecated)

The original JSON-based framework is still available but deprecated:

- **`journey`**: Legacy JSON-based journey execution
- **`mock`**: Mock server utilities
- **`recorder`**: Legacy response recording

For new development, use the simplified framework. See [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md) for migration instructions.

## Testing Modes

### Integration Testing (Recommended)

Test against real Unity Catalog servers for maximum confidence:

```bash
# Enable integration testing
export RUN_INTEGRATION_TESTS=true
export UC_SERVER_URL="http://localhost:8080"
export UC_AUTH_TOKEN="your-auth-token"

# Enable response recording
export RECORD_JOURNEY_RESPONSES=true
export JOURNEY_RECORDING_DIR="./recordings"

cargo test
```

```rust
#[tokio::test]
async fn test_integration() {
    if std::env::var("RUN_INTEGRATION_TESTS").unwrap_or_default() != "true" {
        return; // Skip when not configured
    }
    
    let config = JourneyConfig::default();
    let executor = config.create_executor()?;
    
    let journey = CatalogLifecycleJourney::new();
    let result = executor.execute_journey(&journey).await?;
    
    assert!(result.is_success());
}
```

### Unit Testing

For fast development feedback, journeys can be unit tested:

```rust
#[test]
fn test_journey_properties() {
    let journey = CatalogLifecycleJourney::new();
    assert_eq!(journey.name(), "catalog_lifecycle");
    assert!(journey.tags().contains(&"catalog"));
}
```

### Response Recording and Comparison

Recorded responses can be used to:
- Compare behavior between different Unity Catalog implementations
- Create mock data for faster tests
- Validate API contract compliance
- Generate documentation from real examples

```bash
# Record against reference server
export UC_SERVER_URL="http://reference-server:8080"
cargo test -- catalog_lifecycle

# Compare against another implementation
export UC_SERVER_URL="http://other-server:8080"
cargo test -- catalog_lifecycle

# Diff the recorded responses
diff -r recordings/catalog_lifecycle/ other_recordings/catalog_lifecycle/
```

## Environment Variables

### Core Configuration
- `UC_SERVER_URL`: Unity Catalog server URL (default: "http://localhost:8080")
- `UC_AUTH_TOKEN`: Authentication token for server (optional)
- `REQUEST_TIMEOUT_SECS`: HTTP request timeout in seconds (default: 30)

### Testing Configuration
- `RUN_INTEGRATION_TESTS`: Enable integration tests (default: false)
- `RECORD_JOURNEY_RESPONSES`: Enable response recording (default: false)
- `JOURNEY_RECORDING_DIR`: Output directory for recordings (default: "test_data/recordings")

### Legacy Configuration (Deprecated)
- `RECORD_SUCCESS_ONLY`: Only record successful responses (default: true)
- `OVERWRITE_JOURNEY_RESPONSES`: Overwrite existing recordings (default: false)
- `TEST_CATALOG_PREFIX`: Prefix for test catalogs (default: test)
- `TEST_SUFFIX`: Suffix for test resources (default: random)

## Journey Implementation Reference

### SimpleJourney Trait

All journeys implement the `SimpleJourney` trait:

```rust
#[async_trait]
pub trait SimpleJourney: Send + Sync {
    /// Unique identifier for this journey
    fn name(&self) -> &str;
    
    /// Human-readable description
    fn description(&self) -> &str;
    
    /// Execute the main journey logic
    async fn execute(
        &self,
        client: &UnityCatalogClient,
        recorder: &mut JourneyRecorder,
    ) -> AcceptanceResult<()>;
    
    /// Optional setup (runs before execute)
    async fn setup(
        &self,
        client: &UnityCatalogClient,
        recorder: &mut JourneyRecorder,
    ) -> AcceptanceResult<()> { Ok(()) }
    
    /// Optional cleanup (runs after execute, even on failure)
    async fn cleanup(
        &self,
        client: &UnityCatalogClient,
        recorder: &mut JourneyRecorder,
    ) -> AcceptanceResult<()> { Ok(()) }
    
    /// Tags for organizing journeys
    fn tags(&self) -> Vec<&str> { vec![] }
}
```

### Recording Steps

Use the recorder to capture responses for later analysis:

```rust
async fn execute(&self, client: &UnityCatalogClient, recorder: &mut JourneyRecorder) -> AcceptanceResult<()> {
    // Perform operation
    let catalog = client.create_catalog("test").execute().await?;
    
    // Record the response
    recorder.record_step(
        "step_name",           // Becomes part of filename
        "Step description",    // Human-readable description
        &catalog,             // Response object (must be Serializable)
    ).await?;
    
    // Record errors if needed
    if let Err(e) = some_operation().await {
        recorder.record_error("error_step", "Operation failed", &e).await?;
    }
    
    Ok(())
}
```

### Error Handling

Journeys use standard Rust error handling:

```rust
async fn execute(&self, client: &UnityCatalogClient, recorder: &mut JourneyRecorder) -> AcceptanceResult<()> {
    // Use ? operator for error propagation
    let catalog = client.create_catalog("test").execute().await
        .map_err(|e| AcceptanceError::UnityCatalog(format!("Create failed: {}", e)))?;
    
    // Custom error handling
    match client.catalog("test").get().await {
        Ok(info) => recorder.record_step("get_catalog", "Retrieved catalog", &info).await?,
        Err(e) => {
            recorder.record_error("get_catalog_error", "Failed to get catalog", &e).await?;
            return Err(AcceptanceError::UnityCatalog(format!("Get failed: {}", e)));
        }
    }
    
    Ok(())
}
```

### Resource Management

Always clean up resources in the `cleanup` method:

```rust
async fn cleanup(&self, client: &UnityCatalogClient, recorder: &mut JourneyRecorder) -> AcceptanceResult<()> {
    // Clean up in reverse order of creation
    let _ = client.table("catalog.schema.table").delete().await;
    let _ = client.schema("catalog", "schema").delete().await;
    let _ = client.catalog("catalog").delete().await;
    
    // Record cleanup results
    recorder.record_step(
        "cleanup_complete",
        "Cleanup completed",
        &serde_json::json!({"status": "cleaned_up"}),
    ).await?;
    
    Ok(())
}
```

## Best Practices

### Journey Design

1. **Single Responsibility**: Each journey should test one workflow or feature
2. **Meaningful Names**: Use descriptive names for journeys, steps, and resources
3. **Proper Cleanup**: Always implement cleanup, even if operations fail
4. **Resource Isolation**: Use timestamps or UUIDs to avoid name conflicts
5. **Error Resilience**: Handle errors gracefully and continue cleanup

### Code Organization

1. **Reusable Journeys**: Create generic journeys that can be configured
2. **Helper Methods**: Extract common operations into helper methods
3. **Clear Documentation**: Document complex workflows and edge cases
4. **Type Safety**: Leverage Rust's type system for better reliability
5. **Consistent Structure**: Follow the setup -> execute -> cleanup pattern

### Testing Strategy

1. **Integration First**: Test against real servers for confidence
2. **Record Responses**: Enable recording to capture real behavior
3. **Compare Implementations**: Use recordings to validate different servers
4. **Parallel Safe**: Design journeys to run independently
5. **Environment Aware**: Make tests configurable for different environments

### Performance and Reliability

1. **Resource Cleanup**: Always clean up, even on test failure
2. **Timeout Handling**: Set appropriate timeouts for operations
3. **Retry Logic**: Consider retries for transient failures
4. **Concurrent Testing**: Design for parallel execution
5. **Monitoring**: Use recordings to monitor API changes over time

## Migration from Legacy Framework

The framework includes both the new simplified approach and the legacy JSON-based system for backward compatibility.

### Migrating to Simplified Journeys

See [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md) for detailed migration instructions.

**Quick Migration Steps:**

1. **Replace JSON with Rust**: Convert JSON journey definitions to `SimpleJourney` implementations
2. **Use Typed Client**: Replace raw HTTP calls with `UnityCatalogClient` methods
3. **Implement Trait**: Implement the `SimpleJourney` trait for your workflows
4. **Update Tests**: Use the simplified journey framework
5. **Enable Recording**: Set environment variables for automatic response recording

### Benefits of Migration

- **Type Safety**: Compile-time validation instead of runtime JSON parsing
- **Better Errors**: Clear Rust error messages instead of generic HTTP errors
- **IDE Support**: Full IntelliSense, go-to-definition, and refactoring
- **Maintainability**: Easier to refactor and modify journey logic
- **Real API**: Test the actual client API your applications use

## Examples and Documentation

### Examples

- `examples/simple_journey_example.rs`: Complete usage demonstration
- `src/journeys/`: Pre-built journey implementations
- `tests/simple_journey_tests.rs`: Comprehensive test examples

### Documentation

- [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md): Detailed migration instructions
- [Examples Directory](examples/): Complete working examples
- [Journey Implementations](src/journeys/): Reference implementations

### Running Examples

```bash
# Set up environment
export UC_SERVER_URL="http://localhost:8080"
export RECORD_JOURNEY_RESPONSES="true"

# Run the main example
cargo run --example simple_journey_example

# Run integration tests
export RUN_INTEGRATION_TESTS="true"
cargo test --test simple_journey_tests
```

## Contributing

When adding new journeys or functionality:

1. **Implement SimpleJourney**: Use the trait-based approach for new journeys
2. **Add Tests**: Include both unit and integration tests
3. **Document Examples**: Provide clear usage examples
4. **Follow Patterns**: Use existing journey implementations as templates
5. **Update Documentation**: Keep README and migration guide current

### Adding New Journeys

1. Create a new file in `src/journeys/`
2. Implement the `SimpleJourney` trait
3. Add comprehensive tests
4. Export from `src/journeys/mod.rs`
5. Add example usage to documentation

### Testing Contributions

```bash
# Run all tests
cargo test

# Run with integration tests
export RUN_INTEGRATION_TESTS=true
cargo test

# Test specific journey
cargo test --test simple_journey_tests -- catalog_lifecycle
```

## License

Apache 2.0 - See LICENSE file for details.
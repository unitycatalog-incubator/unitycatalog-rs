# Unity Catalog Journey Testing Framework

## Overview

The Journey Testing Framework is a powerful extension to the Unity Catalog client test suite that enables testing of multi-step, dependent API workflows. Unlike traditional unit tests that test individual operations in isolation, journey tests validate complete user workflows with proper dependency management and variable passing between steps.

## Key Features

### 🔄 Multi-Step Workflows
- Define complex workflows with dependent API calls
- Automatic dependency resolution and execution ordering
- Variable extraction and substitution between steps
- Support for conditional execution and error handling

### 📝 JSON-Based Configuration
- Human-readable journey definitions
- Easy to create, modify, and version control
- Template variables for reusable test scenarios
- Rich metadata and documentation support

### 🧪 Mock and Integration Testing
- Execute journeys against mock servers for fast testing
- Record real server responses for accuracy
- Switch between mock and integration modes seamlessly
- Capture actual API responses for test data generation

### 🏷️ Advanced Organization
- Tag-based step categorization (setup, main, cleanup)
- Dependency-aware execution with proper cleanup
- Continue-on-failure options for robust cleanup
- Comprehensive validation and error reporting

## Architecture

```
Journey Framework
├── journeys.rs              # Core journey execution engine
├── journey_tests.rs         # Journey-based test implementations
├── journey_integration_recorder.rs  # Real response recording
├── test_data/journeys/      # Journey definition files
│   ├── catalog_lifecycle.json
│   ├── hierarchical_data_structure.json
│   ├── error_handling.json
│   ├── simple_example.json
│   └── recorded/            # Captured real responses
└── simple_journey_test.rs   # Basic validation tests
```

## Journey Definition Format

A journey consists of:
- **Metadata**: Name, description, variables, and documentation
- **Steps**: Ordered list of API operations with dependencies
- **Variables**: Template variables for reusable scenarios
- **Configuration**: Execution options and behavioral settings

### Example Journey Structure

```json
{
  "name": "workflow_name",
  "description": "What this workflow tests",
  "variables": {
    "resource_name": "test_resource",
    "environment": "test"
  },
  "steps": [
    {
      "id": "create_resource",
      "description": "Create the resource",
      "method": "POST",
      "path": "/resources",
      "request_body": {
        "name": "{resource_name}",
        "env": "{environment}"
      },
      "expected_status": 201,
      "extract_variables": {
        "resource_id": "$.id"
      },
      "tags": ["setup", "create"]
    },
    {
      "id": "verify_resource",
      "description": "Verify resource was created",
      "method": "GET",
      "path": "/resources/{resource_id}",
      "expected_status": 200,
      "depends_on": ["create_resource"],
      "tags": ["verification", "read"]
    }
  ]
}
```

## Quick Start Guide

### 1. Running Existing Journey Tests

```bash
# Run all journey tests with mock servers
cargo test journey_tests

# Run specific journey test
cargo test test_catalog_lifecycle_journey

# Run with debug output
RUST_LOG=debug cargo test journey_tests -- --nocapture
```

### 2. Creating a New Journey

1. **Define the Journey**: Create a JSON file in `tests/test_data/journeys/`
2. **Add Test Case**: Create a test function in `journey_tests.rs`
3. **Validate**: Run validation to check dependencies and structure

Example test function:
```rust
#[rstest]
#[tokio::test]
async fn test_my_workflow(
    #[future] journey_test_setup: (UnityCatalogClient, TestServer),
    journey_variables: HashMap<String, serde_json::Value>,
) {
    let (client, server) = journey_test_setup.await;
    
    let journey = JourneyLoader::load_journey("my_workflow.json")
        .expect("Failed to load journey");
    
    let mut executor = JourneyExecutor::new(client, Some(server))
        .with_variables(journey_variables);
    
    let result = executor.execute_journey(journey).await;
    assert!(result.success);
}
```

### 3. Recording Real Server Responses

```bash
# Enable response recording
export RECORD_JOURNEY_RESPONSES=true
export UC_SERVER_URL="http://localhost:8080"
export UC_AUTH_TOKEN="your-auth-token"

# Run tests to record responses
cargo test journey_tests

# Recorded files will be saved to tests/test_data/journeys/recorded/
```

## Available Journey Templates

### 1. Catalog Lifecycle (`catalog_lifecycle.json`)
**Purpose**: Complete catalog CRUD operations
**Steps**: Create → Get → Update → Verify → Delete
**Use Cases**: Basic catalog management testing

### 2. Hierarchical Data Structure (`hierarchical_data_structure.json`)
**Purpose**: Full hierarchy creation with dependencies
**Steps**: Catalog → Schema → Table → Updates → Cleanup
**Use Cases**: Testing parent-child relationships and dependency management

### 3. Error Handling (`error_handling.json`)
**Purpose**: Various error scenarios and edge cases
**Steps**: 404 errors, validation errors, conflicts, rate limiting
**Use Cases**: Negative testing and error condition validation

### 4. Simple Example (`simple_example.json`)
**Purpose**: Documentation and learning template
**Steps**: Basic workflow with comprehensive comments
**Use Cases**: New developer onboarding, framework tutorials

## Best Practices

### Journey Design
- **Start Simple**: Begin with basic workflows, add complexity gradually
- **Use Dependencies**: Properly model real-world step dependencies
- **Extract Variables**: Pass data between steps using variable extraction
- **Tag Appropriately**: Use tags for setup, main logic, and cleanup
- **Plan Cleanup**: Always include cleanup steps with `continue_on_failure: true`

### Variable Management
- **Descriptive Names**: Use clear, descriptive variable names
- **Template Consistently**: Use `{variable_name}` format consistently
- **Extract Key Data**: Extract IDs, timestamps, and other important data
- **Validate Extraction**: Verify variable extraction with JSONPath

### Error Handling
- **Test Failures**: Include expected error scenarios
- **Continue on Cleanup**: Use `continue_on_failure` for cleanup steps
- **Validate Errors**: Verify error codes and messages match expectations
- **Handle Dependencies**: Plan for dependency failures

### Performance Considerations
- **Minimize Steps**: Keep journeys focused and concise
- **Parallel Opportunities**: Identify steps that could run in parallel
- **Timeout Management**: Set appropriate timeouts for operations
- **Resource Cleanup**: Always clean up created resources

## Advanced Features

### Variable Extraction with JSONPath
Extract data from responses using JSONPath expressions:

```json
{
  "extract_variables": {
    "catalog_id": "$.name",
    "created_timestamp": "$.created_at",
    "storage_location": "$.storage_root",
    "property_value": "$.properties.environment"
  }
}
```

### Conditional Execution
Control step execution based on conditions:

```json
{
  "id": "optional_step",
  "description": "Only runs if conditions are met",
  "continue_on_failure": true,
  "depends_on": ["required_step"]
}
```

### Response Recording Configuration
Fine-tune response recording behavior:

```bash
# Record only successful responses (2xx status codes)
export RECORD_SUCCESS_ONLY=true

# Overwrite existing recorded files
export OVERWRITE_JOURNEY_RESPONSES=true

# Custom output directory for recorded responses
export JOURNEY_RECORDING_DIR="custom/path/recorded"
```

## Integration with CI/CD

### Mock Testing (Fast Feedback)
```yaml
- name: Run Journey Tests (Mock)
  run: cargo test journey_tests
  env:
    RUST_LOG: info
```

### Integration Testing (Full Validation)
```yaml
- name: Run Journey Tests (Integration)
  run: cargo test journey_tests
  env:
    UC_SERVER_URL: ${{ secrets.UC_SERVER_URL }}
    UC_AUTH_TOKEN: ${{ secrets.UC_AUTH_TOKEN }}
    RUN_INTEGRATION_TESTS: true
```

### Response Recording (Maintenance)
```yaml
- name: Update Journey Responses
  run: cargo test journey_tests
  env:
    RECORD_JOURNEY_RESPONSES: true
    UC_SERVER_URL: ${{ secrets.UC_SERVER_URL }}
    UC_AUTH_TOKEN: ${{ secrets.UC_AUTH_TOKEN }}
    OVERWRITE_JOURNEY_RESPONSES: true
```

## Troubleshooting

### Common Issues

**Journey Validation Errors**
- Check step IDs are unique
- Verify all dependencies exist
- Validate JSON syntax

**Variable Substitution Problems**
- Ensure variables are defined in journey or extracted from previous steps
- Check JSONPath expressions for variable extraction
- Verify variable names match exactly (case-sensitive)

**Mock Server Issues**
- Ensure expected responses match actual response format
- Check endpoint paths and HTTP methods
- Verify mock server is properly configured

**Dependency Resolution Failures**
- Check that dependent steps complete successfully
- Verify dependency chains don't create cycles
- Ensure cleanup steps use `continue_on_failure: true`

### Debug Tips

1. **Enable Detailed Logging**:
   ```bash
   RUST_LOG=debug cargo test journey_tests -- --nocapture
   ```

2. **Validate Journey Files**:
   ```rust
   let errors = JourneyLoader::validate_journey(&journey);
   println!("Validation errors: {:?}", errors);
   ```

3. **Check Variable State**:
   ```rust
   println!("Final variables: {:?}", result.final_variables);
   ```

4. **Examine Step Results**:
   ```rust
   for step_result in &result.step_results {
       println!("Step {}: success={}, status={}", 
                step_result.step.id, 
                step_result.success, 
                step_result.status_code);
   }
   ```

## Contributing

### Adding New Journey Types

1. **Identify Workflow**: Determine the user workflow to test
2. **Design Steps**: Break down into logical, dependent steps
3. **Create JSON**: Write the journey definition file
4. **Add Test**: Create corresponding test function
5. **Document**: Add documentation and examples
6. **Validate**: Ensure journey works in both mock and integration modes

### Framework Improvements

Areas for contribution:
- Enhanced JSONPath support for complex extraction
- Parallel step execution for independent operations
- Journey composition and inheritance
- Advanced error handling and retry logic
- Performance optimization and profiling
- Visual journey execution reporting

## Conclusion

The Journey Testing Framework provides a robust, maintainable approach to testing complex Unity Catalog workflows. By modeling real-world usage patterns with proper dependency management, it ensures comprehensive test coverage while remaining easy to understand and modify.

Key benefits:
- **Realistic Testing**: Models actual user workflows
- **Maintainable**: JSON-based configuration is easy to update
- **Comprehensive**: Covers both happy path and error scenarios
- **Flexible**: Works with mock servers and real deployments
- **Scalable**: Easy to add new workflows and scenarios

Start with the simple examples, then gradually build more complex journeys as you become familiar with the framework. The combination of mock testing for fast feedback and integration testing for validation provides the best of both worlds for Unity Catalog client testing.
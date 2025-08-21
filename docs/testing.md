# Unity Catalog Client Testing Framework

This document describes the comprehensive testing framework for Unity Catalog clients, starting with the catalog client implementation. The framework features professional CLI output with progress bars, spinners, colored text, and formatted tables for an enhanced user experience.

## Overview

The testing framework is designed to validate Unity Catalog client implementations against a deployed Unity Catalog server. It provides comprehensive coverage of all client operations, error handling scenarios, and integration patterns with beautiful, professional CLI output.

## Running Tests

Tests are executed using the CLI test command:

```bash
# Run against default server (http://localhost:8080)
cargo run --bin uc test

# Run against a specific server
cargo run --bin uc --server http://my-uc-server:8080 test
```

The server URL can also be set via environment variable:

```bash
export UC_SERVER_URL=http://my-uc-server:8080
cargo run --bin uc test
```

### Demo Mode

To see the professional output formatting without requiring a server, run the demo:

```bash
cargo run --bin uc demo
```

This showcases all the visual elements including progress bars, spinners, colored validation results, and formatted tables.

## Professional CLI Output Features

The testing framework includes rich CLI output formatting using professional libraries:

- **Progress Indicators**: Animated spinners for ongoing operations
- **Color Coding**: Green for success, red for errors, blue for info, yellow for warnings
- **Formatted Tables**: Beautiful tables for displaying catalog and schema information
- **Validation Results**: Clear visual indicators for assertion results
- **Category Organization**: Structured test execution with clear section headers
- **Summary Reports**: Comprehensive test results with statistics and breakdowns

### Visual Elements

- 🚀 **Section Headers**: Major test categories
- ⚙️ **Subsection Headers**: Test subcategories  
- ✅ **Success Messages**: Completed operations
- ❌ **Error Messages**: Failed operations
- ℹ️ **Info Messages**: General information
- → **Step Indicators**: Individual test steps
- ✓ **Validation Results**: Assertion outcomes

## Test Structure

### Catalog Tests (`CatalogTests`)

The catalog testing framework is organized into focused test categories:

#### 1. Lifecycle Tests (`test_catalog_lifecycle`)
- **Purpose**: Validates basic CRUD operations
- **Coverage**:
  - Create catalog with storage root and comment
  - Retrieve catalog by name
  - Delete catalog
  - Verify deletion (get should fail)
- **Assertions**: Validates catalog properties, IDs, and proper error handling

#### 2. List Operations (`test_catalog_list_operations`)
- **Purpose**: Tests catalog discovery and pagination
- **Coverage**:
  - Create multiple test catalogs
  - List all catalogs
  - Verify test catalogs appear in results
  - Test pagination with `max_results` parameter
- **Cleanup**: Removes all test catalogs

#### 3. Update Operations (`test_catalog_update_operations`)
- **Purpose**: Validates catalog modification capabilities
- **Coverage**:
  - Update catalog comment
  - Update catalog properties (add/modify)
  - Update catalog owner
  - Verify changes persist
- **Patterns**: Tests both individual and combined updates

#### 4. Properties Management (`test_catalog_properties`)
- **Purpose**: Tests property handling edge cases
- **Coverage**:
  - Create catalog with initial properties
  - Add new properties
  - Update existing properties
  - Clear all properties (empty HashMap)
- **Validation**: Ensures property operations work correctly

#### 5. Sharing Catalogs (`test_catalog_sharing`)
- **Purpose**: Tests shared catalog creation
- **Coverage**:
  - Create sharing catalog with provider and share names
  - Verify sharing-specific fields are set
  - Retrieve sharing catalog
- **Assertions**: Validates provider_name and share_name fields

#### 6. Error Handling (`test_catalog_error_handling`)
- **Purpose**: Tests error scenarios and edge cases
- **Coverage**:
  - Duplicate catalog creation (should fail)
  - Get non-existent catalog (should fail)
  - Update non-existent catalog (should fail)
  - Delete catalog with schemas (with/without force)
- **Validation**: Ensures proper error responses

#### 7. Schema Integration (`test_catalog_with_schemas`)
- **Purpose**: Tests catalog-schema interactions
- **Coverage**:
  - Create catalog and multiple schemas
  - List schemas in catalog
  - Access schemas through catalog client
  - Update schema through catalog client
  - Proper cleanup order (schemas first, then catalog)

## Test Patterns and Best Practices

### Naming Conventions
- Test catalogs use descriptive prefixes: `test_lifecycle_catalog`, `list_test_catalog_1`, etc.
- Schema names are simple: `schema_1`, `test_schema`
- All test resources include "test" in the name for easy identification

### Resource Management
- **Create-Test-Cleanup**: Each test creates its own resources, tests operations, then cleans up
- **Proper Cleanup Order**: Schemas are deleted before catalogs
- **Force Deletion**: Uses `force=true` when necessary for cleanup

### Assertion Strategy
- **Property Validation**: Verifies all expected fields are set correctly
- **State Persistence**: Confirms changes persist across get operations
- **Error Validation**: Ensures operations fail appropriately in error scenarios

### Professional Output
- **Interactive Progress**: Animated spinners and progress bars for real-time feedback
- **Color-Coded Results**: Visual distinction between success, failure, and info states
- **Structured Tables**: Formatted display of catalog and schema data
- **Validation Feedback**: Clear indicators showing expected vs actual results
- **Category Tracking**: Visual organization of test execution phases
- **Summary Statistics**: Professional test result summaries with percentages

## Extensibility

The testing framework is designed to be extensible to other Unity Catalog clients:

### Adding New Client Tests

1. **Create Client Test Struct**:
```rust
struct SchemaTests {
    client: UnityCatalogClient,
}
```

2. **Implement Test Categories**:
```rust
impl SchemaTests {
    async fn run_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.test_schema_lifecycle().await?;
        self.test_schema_operations().await?;
        // ... other tests
        Ok(())
    }
}
```

3. **Add to Main Test Runner**:
```rust
pub(crate) async fn run(opts: &GlobalOpts) -> Result<(), Box<dyn std::error::Error>> {
    let catalog_tests = CatalogTests::new(client.clone());
    catalog_tests.run_all().await?;
    
    let schema_tests = SchemaTests::new(client.clone());
    schema_tests.run_all().await?;
    
    Ok(())
}
```

### Test Categories Template

Each client should implement these test categories:

1. **Lifecycle Tests**: Basic CRUD operations
2. **List Operations**: Discovery and pagination
3. **Update Operations**: Modification capabilities
4. **Property Management**: Property-specific operations
5. **Error Handling**: Error scenarios and edge cases
6. **Integration Tests**: Cross-client interactions

## Configuration

### Test Data
- Storage roots use S3 paths: `s3://test-bucket/path`
- Comments are descriptive and include test context
- Properties use test-specific keys: `created_by: integration_test`

### CLI Dependencies
The professional output relies on these crates:
- **indicatif**: Progress bars and spinners
- **console**: Colored terminal output and styling
- **comfy-table**: Professional table formatting

### Error Expectations
- Duplicate creation should fail
- Operations on non-existent resources should fail
- Delete operations may succeed or fail depending on implementation

### Server Requirements
- Tests assume a clean Unity Catalog server
- Server should support all catalog operations
- Network connectivity to configured server URL

## Troubleshooting

### Common Issues

1. **Server Connectivity**: Verify server URL and network access
2. **Permission Errors**: Ensure client has necessary permissions
3. **Resource Conflicts**: Check for existing test resources
4. **Cleanup Failures**: May need manual cleanup of test resources
5. **Terminal Compatibility**: Some output features require terminal color support

### Debug Output
Enable debug logging for more detailed output:
```bash
RUST_LOG=debug cargo run --bin uc test
```

### Output Customization
The professional output automatically adapts to terminal capabilities:
- Unicode symbols fall back to ASCII when needed
- Colors are disabled in non-interactive environments
- Tables adjust width based on terminal size

### Test Isolation
Tests create uniquely named resources but should be run against a clean server for best results.

## Future Enhancements

- **Parallel Test Execution**: Run independent tests concurrently with coordinated output
- **Test Data Generation**: Faker integration for realistic test data
- **Performance Testing**: Measure operation latencies with performance graphs
- **Stress Testing**: High-volume operations testing with live progress tracking
- **Integration Suites**: Cross-client workflow testing
- **Export Capabilities**: Generate HTML or JSON test reports
- **Watch Mode**: Continuous testing with file change detection
- **Interactive Mode**: Allow user selection of specific test categories
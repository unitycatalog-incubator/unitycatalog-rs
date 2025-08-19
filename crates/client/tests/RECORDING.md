# Journey Response Recording Infrastructure

This document describes how to use the journey recording infrastructure to capture real Unity Catalog server responses for testing and validation.

## Overview

The recording infrastructure allows you to:
- Execute journeys against real Unity Catalog servers
- Capture HTTP responses and save them as JSON files
- Use recorded responses for mock testing and validation
- Generate accurate test data from production environments

## Quick Start

### 1. Set Up Environment Variables

```bash
# Required: Unity Catalog server URL
export UC_SERVER_URL="http://your-unity-catalog-server:8080"

# Optional: Authentication token
export UC_AUTH_TOKEN="your-auth-token"

# Enable recording mode
export RECORD_JOURNEY_RESPONSES=true

# Optional: Overwrite existing recorded files
export OVERWRITE_JOURNEY_RESPONSES=true

# Optional: Record only successful responses (2xx status codes)
export RECORD_SUCCESS_ONLY=true
```

### 2. Run Recording Tests

```bash
# Record all journey tests
cargo test journey_tests -- --nocapture

# Record a specific journey
cargo test test_catalog_lifecycle_journey -- --nocapture

# Test the recording infrastructure
cargo test test_recording_infrastructure -- --nocapture
```

### 3. View Recorded Responses

Recorded files are saved to `tests/test_data/journeys/recorded/` with the naming pattern:
```
{journey_name}_recorded.json
```

## Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `UC_SERVER_URL` | Unity Catalog server base URL | - | ✅ |
| `UC_AUTH_TOKEN` | Authentication token | - | ❌ |
| `RECORD_JOURNEY_RESPONSES` | Enable recording mode | `false` | ✅ |
| `OVERWRITE_JOURNEY_RESPONSES` | Overwrite existing files | `false` | ❌ |
| `RECORD_SUCCESS_ONLY` | Record only 2xx responses | `true` | ❌ |
| `JOURNEY_RECORDING_DIR` | Custom output directory | `tests/test_data/journeys/recorded` | ❌ |
| `JOURNEY_REQUEST_TIMEOUT` | Request timeout in seconds | `30` | ❌ |

## Recording Modes

### Mock Mode (Default)
When recording is disabled, tests run against mock servers:
```bash
export RECORD_JOURNEY_RESPONSES=false
cargo test journey_tests
```

### Recording Mode
When recording is enabled, tests execute against real servers and save responses:
```bash
export RECORD_JOURNEY_RESPONSES=true
export UC_SERVER_URL="http://localhost:8080"
cargo test journey_tests
```

### Integration Mode
Run tests against real servers without recording:
```bash
export RUN_INTEGRATION_TESTS=true
export UC_SERVER_URL="http://localhost:8080"
export RECORD_JOURNEY_RESPONSES=false
cargo test journey_tests
```

## Recorded File Format

Recorded journey files contain:

```json
{
  "journey": {
    "name": "catalog_lifecycle",
    "description": "Complete catalog CRUD operations",
    "variables": { ... },
    "steps": [ ... ]
  },
  "recorded_steps": [
    {
      "step": {
        "id": "create_catalog",
        "method": "POST",
        "path": "/catalogs",
        ...
      },
      "response": {
        "status_code": 201,
        "body": { ... },
        "headers": { ... },
        "recorded_at": "2024-01-15T10:30:00Z",
        "method": "POST",
        "path": "/catalogs",
        "request_body": { ... }
      },
      "extracted_variables": {
        "catalog_id": "test_catalog_123"
      }
    }
  ],
  "final_variables": { ... },
  "metadata": {
    "recorded_at": "2024-01-15T10:30:00Z",
    "server_url": "http://localhost:8080",
    "total_steps": 6,
    "successful_steps": 6,
    "config_summary": "success_only=true, overwrite=false, timeout=30s"
  }
}
```

## Use Cases

### 1. Generate Test Data
Record responses from a real Unity Catalog deployment to create realistic test fixtures:

```bash
export UC_SERVER_URL="https://production-uc.company.com"
export UC_AUTH_TOKEN="prod-readonly-token"
export RECORD_JOURNEY_RESPONSES=true
export RECORD_SUCCESS_ONLY=true
cargo test test_catalog_lifecycle_journey
```

### 2. Validate API Changes
Record responses before and after API changes to verify compatibility:

```bash
# Before changes
export UC_SERVER_URL="http://staging-before:8080"
cargo test journey_tests

# After changes  
export UC_SERVER_URL="http://staging-after:8080"
export OVERWRITE_JOURNEY_RESPONSES=true
cargo test journey_tests

# Compare recorded files to validate changes
```

### 3. Debug Integration Issues
Record failing scenarios to analyze server responses:

```bash
export UC_SERVER_URL="http://problematic-server:8080"
export RECORD_SUCCESS_ONLY=false  # Record errors too
export JOURNEY_REQUEST_TIMEOUT=60  # Longer timeout
cargo test test_error_handling_journey -- --nocapture
```

### 4. Performance Testing
Record response times and analyze server performance:

```bash
export UC_SERVER_URL="http://performance-test-server:8080"
export RECORD_JOURNEY_RESPONSES=true
time cargo test journey_tests
```

## Programming Interface

### Recording a Journey Programmatically

```rust
use crate::journey_integration_recorder::{JourneyRecorder, RecordingConfig, record_journey_from_file};

// Record from environment configuration
let recorded = record_journey_from_file("catalog_lifecycle.json").await?;
println!("Recorded {} steps", recorded.recorded_steps.len());

// Custom recording configuration
let config = RecordingConfig {
    server_url: "http://localhost:8080".to_string(),
    auth_token: Some("token".to_string()),
    output_dir: PathBuf::from("custom/path"),
    record_success_only: false,
    overwrite_existing: true,
    request_timeout_secs: 60,
};

let mut recorder = JourneyRecorder::new(config)?;
let journey = JourneyLoader::load_journey("my_journey.json")?;
let recorded = recorder.record_journey(journey).await?;
```

### Using Recorded Responses in Tests

```rust
// Load recorded journey
let recorded_path = "tests/test_data/journeys/recorded/catalog_lifecycle_recorded.json";
let recorded: RecordedJourney = serde_json::from_str(&fs::read_to_string(recorded_path)?)?;

// Extract responses for mock setup
for recorded_step in &recorded.recorded_steps {
    server.mock(
        &recorded_step.response.method,
        &recorded_step.response.path
    )
    .with_status(recorded_step.response.status_code)
    .with_body(recorded_step.response.body.to_string())
    .create();
}
```

## Best Practices

### Recording Strategy
- **Start Simple**: Record basic journeys first, then move to complex scenarios
- **Use Staging**: Record against staging environments when possible
- **Version Control**: Commit recorded files to track API evolution
- **Clean Data**: Use test-specific prefixes to avoid polluting production data

### Security Considerations
- **Sanitize Tokens**: Never commit real authentication tokens to version control
- **Mock Secrets**: Replace sensitive data in recorded responses
- **Network Isolation**: Use isolated test environments when recording
- **Access Control**: Use read-only tokens when possible

### Performance Optimization
- **Parallel Recording**: Record independent journeys in parallel
- **Selective Recording**: Only record when responses change
- **Timeout Tuning**: Adjust timeouts based on server performance
- **Retry Logic**: Implement retries for transient failures

### Maintenance
- **Regular Updates**: Re-record periodically to catch API changes
- **Validation**: Verify recorded responses match expected schemas
- **Cleanup**: Remove obsolete recorded files
- **Documentation**: Keep this guide updated with new patterns

## Troubleshooting

### Common Issues

**Recording Not Enabled**
```
Error: Recording not enabled. Set RECORD_JOURNEY_RESPONSES=true
```
Solution: Set the required environment variable

**Server Connection Failed**
```
Error: Connection refused (os error 61)
```
Solution: Verify `UC_SERVER_URL` and server availability

**Authentication Failed**
```
Error: HTTP 401 Unauthorized
```
Solution: Check `UC_AUTH_TOKEN` or use correct authentication method

**Permission Denied**
```
Error: Permission denied (os error 13)
```
Solution: Ensure write permissions to output directory

**File Already Exists**
```
Skipping existing file: catalog_lifecycle_recorded.json
```
Solution: Set `OVERWRITE_JOURNEY_RESPONSES=true` to overwrite

### Debug Mode

Enable detailed logging for troubleshooting:

```bash
export RUST_LOG=debug
cargo test journey_tests -- --nocapture
```

### Validation

Validate recorded files:

```bash
# Check JSON syntax
jq . tests/test_data/journeys/recorded/catalog_lifecycle_recorded.json

# Verify recording structure
cargo test --test journey_integration_recorder
```

## Examples

### Complete Recording Session

```bash
#!/bin/bash
# setup-recording.sh

# Configure environment
export UC_SERVER_URL="http://localhost:8080"
export UC_AUTH_TOKEN="test-token"
export RECORD_JOURNEY_RESPONSES=true
export OVERWRITE_JOURNEY_RESPONSES=true
export RUST_LOG=info

# Clean previous recordings
rm -f tests/test_data/journeys/recorded/*.json

# Record all journeys
echo "🎬 Recording journeys..."
cargo test journey_tests -- --nocapture

# Verify recordings
echo "✅ Recorded files:"
ls -la tests/test_data/journeys/recorded/

# Test with recorded data
export RECORD_JOURNEY_RESPONSES=false
echo "🎭 Testing with recorded data..."
cargo test journey_tests

echo "🎉 Recording session complete!"
```

### Custom Journey Recording

```rust
// examples/custom_recording.rs
use std::collections::HashMap;
use serde_json::Value;
use unitycatalog_client_tests::journey_integration_recorder::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create custom recording configuration
    let config = RecordingConfig {
        server_url: "http://my-server:8080".to_string(),
        auth_token: None,
        output_dir: "custom_recordings".into(),
        record_success_only: true,
        overwrite_existing: true,
        request_timeout_secs: 45,
    };
    
    // Set up recorder with custom variables
    let mut variables = HashMap::new();
    variables.insert("environment".to_string(), Value::String("staging".to_string()));
    variables.insert("user_id".to_string(), Value::String("test-user-123".to_string()));
    
    let mut recorder = JourneyRecorder::new(config)?
        .with_variables(variables);
    
    // Load and record journey
    let journey = JourneyLoader::load_journey("my_custom_journey.json")?;
    let recorded = recorder.record_journey(journey).await?;
    
    println!("Successfully recorded {} steps", recorded.recorded_steps.len());
    println!("Success rate: {}/{}", 
             recorded.metadata.successful_steps, 
             recorded.metadata.total_steps);
    
    Ok(())
}
```

## Contributing

When adding new recording features:

1. **Update Tests**: Add tests for new functionality
2. **Document Changes**: Update this README
3. **Version Compatibility**: Ensure backward compatibility with existing recordings
4. **Security Review**: Review for security implications
5. **Performance Impact**: Measure impact on test execution time

## Future Enhancements

Planned improvements to the recording infrastructure:

- **Response Filtering**: Record only specific response fields
- **Data Masking**: Automatic sanitization of sensitive data
- **Parallel Execution**: Record multiple journeys simultaneously
- **Schema Validation**: Validate responses against OpenAPI specs
- **Metrics Collection**: Capture performance metrics during recording
- **Diff Generation**: Compare recorded responses across versions
- **Interactive Mode**: CLI tool for selective recording
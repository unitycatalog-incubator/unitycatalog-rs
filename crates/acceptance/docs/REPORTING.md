# Rich Reporting and Logging Utilities

This document describes the enhanced reporting and logging capabilities added to the Unity Catalog acceptance testing framework. These utilities provide rich, condensed output with progress indicators, tables, and structured logging to improve the development experience when writing integration tests.

## Overview

The reporting system consists of three main components:

1. **`reporting` module** - Core reporting infrastructure with rich terminal output
2. **`journey_helpers` module** - Simplified utilities for journey authors
3. **Enhanced macros** - Convenient macros for common operations

## Quick Start

### Basic Usage with JourneyLogger

The simplest way to add rich reporting to your journey is using `JourneyLogger`:

```rust
use unitycatalog_acceptance::{init_journey, journey_step, setup_journey_steps};
use unitycatalog_acceptance::journey_helpers::JourneyLogger;

#[async_trait]
impl UserJourney for MyJourney {
    async fn execute(&mut self, client: &UnityCatalogClient) -> AcceptanceResult<()> {
        // Initialize the logger
        let logger = init_journey!("my_journey", "Description of what this journey does");
        
        // Setup steps for progress tracking
        setup_journey_steps!(
            logger,
            "create_catalog" => "Create a new catalog",
            "verify_catalog" => "Verify catalog was created",
            "cleanup" => "Clean up resources"
        );
        
        // Execute steps with automatic progress tracking
        let catalog = journey_step!(
            logger,
            "create_catalog",
            "test catalog",
            create => {
                client.create_catalog("test_catalog")
                    .with_comment(Some("Test catalog".to_string()))
                    .await
            }
        )?;
        
        // Verify step
        journey_step!(
            logger,
            "verify_catalog",
            "catalog properties",
            verify => {
                async {
                    assert_eq!(catalog.name, "test_catalog");
                    logger.info(&format!("Catalog ID: {}", catalog.id.unwrap()))?;
                    Ok(())
                }
            }
        )?;
        
        // Cleanup step
        journey_step!(
            logger,
            "cleanup",
            "test catalog",
            delete => {
                client.catalog("test_catalog").delete(Some(true)).await
            }
        )?;
        
        logger.finish(true)?;
        Ok(())
    }
}
```

### Advanced Usage with Custom Configuration

For more control over the reporting behavior:

```rust
use unitycatalog_acceptance::reporting::ReportingConfig;
use unitycatalog_acceptance::journey_helpers::JourneyLogger;

let config = ReportingConfig {
    verbosity: 2,           // 0=minimal, 1=normal, 2=verbose
    show_progress: true,    // Show progress bars
    show_timing: true,      // Show timing information
    use_colors: true,       // Use colored output
    table_width: Some(120), // Custom table width
};

let logger = JourneyLogger::with_config("my_journey", config);
logger.start("Journey description")?;

// Add steps and execute...
```

## Available Macros

### `init_journey!`

Initializes a journey logger with a name and description:

```rust
let logger = init_journey!("catalog_operations", "Testing catalog CRUD operations");
```

### `setup_journey_steps!`

Registers multiple steps for progress tracking:

```rust
setup_journey_steps!(
    logger,
    "step1" => "Description of step 1",
    "step2" => "Description of step 2",
    "step3" => "Description of step 3",
);
```

### `journey_step!`

Executes a step with automatic status tracking. Supports different operation types:

```rust
// Create operation
journey_step!(logger, "create_catalog", "my catalog", create => operation)?;

// List operation  
journey_step!(logger, "list_catalogs", "all catalogs", list => operation)?;

// Get operation
journey_step!(logger, "get_catalog", "catalog details", get => operation)?;

// Delete operation
journey_step!(logger, "delete_catalog", "my catalog", delete => operation)?;

// Verify operation
journey_step!(logger, "verify_properties", "catalog metadata", verify => operation)?;

// Generic operation
journey_step!(logger, "custom_step", "custom description" => operation)?;
```

## Utility Functions

### Performance Metrics

Track and analyze operation performance:

```rust
use unitycatalog_acceptance::journey_helpers::PerformanceMetrics;

let mut metrics = PerformanceMetrics::new();

let result = metrics.measure("create_catalog", async {
    client.create_catalog("test").await
}).await;

let result2 = metrics.measure("list_catalogs", async {
    client.list_catalogs(None).collect::<Vec<_>>().await
}).await;

// Get performance summary
logger.info(&metrics.summary())?;
```

### Cleanup Operations

Handle cleanup with error suppression:

```rust
use unitycatalog_acceptance::journey_helpers::cleanup_step;

// Cleanup operations continue even if they fail
cleanup_step(
    &logger,
    "cleanup_catalog",
    client.catalog("test").delete(Some(true))
).await?;
```

### Progress Tracking

For long-running operations:

```rust
use unitycatalog_acceptance::journey_helpers::ProgressTracker;

let tracker = ProgressTracker::start("bulk_import");
// ... do work ...
logger.info(&tracker.finish())?;
```

## Output Examples

### Journey Execution with Progress

```
🚀 enhanced_catalog Enhanced catalog lifecycle for 'enhanced_catalog_1705123456'

  ⏳ Create new catalog
  ✅ Create new catalog (245ms)
    └─ Catalog created with ID: enhanced_catalog_1705123456

  ⏳ Verify catalog was created
  ✅ Verify catalog was created (89ms)

  ⏳ List all catalogs
  ✅ List all catalogs (156ms)
    └─ Found catalog 'enhanced_catalog_1705123456' in listing

  📊 Performance Summary (Total: 490ms):
    • create_catalog: 245ms (50.0%)
    • verify_creation: 89ms (18.2%)
    • list_catalogs: 156ms (31.8%)

🎉 Journey enhanced_catalog COMPLETED (490ms)
```

### Summary Table

```
┌─────────────────────┬────────────┬──────────┬───────┬─────────────────────────┐
│ Journey             │ Status     │ Duration │ Steps │ Error                   │
├─────────────────────┼────────────┼──────────┼───────┼─────────────────────────┤
│ simple_catalog      │ ✓ Success  │ 1250ms   │ 4     │ -                       │
│ enhanced_catalog    │ ✓ Success  │ 2100ms   │ 6     │ -                       │
│ complex_workflow    │ ✗ Failed   │ 890ms    │ 2     │ Catalog already exists  │
└─────────────────────┴────────────┴──────────┴───────┴─────────────────────────┘
```

## Configuration Options

### ReportingConfig

```rust
pub struct ReportingConfig {
    /// Whether to show progress bars (default: true)
    pub show_progress: bool,
    
    /// Whether to use colors in output (default: auto-detect)
    pub use_colors: bool,
    
    /// Verbosity level (default: 1)
    /// - 0: Minimal output (errors only)
    /// - 1: Normal output (progress and results)
    /// - 2: Verbose output (detailed info and timing)
    pub verbosity: u8,
    
    /// Whether to show timing information (default: true)
    pub show_timing: bool,
    
    /// Width for tables (default: auto-detect)
    pub table_width: Option<usize>,
}
```

### Environment Variables

You can also control reporting behavior via environment variables:

```bash
# Set verbosity level
export UC_ACCEPTANCE_VERBOSITY=2

# Disable colors
export NO_COLOR=1

# Disable progress bars
export UC_ACCEPTANCE_NO_PROGRESS=1
```

## Best Practices

### 1. Structure Your Steps

Break down your journey into logical steps for better progress tracking:

```rust
// Good: Clear, focused steps
setup_journey_steps!(
    logger,
    "create_catalog" => "Create test catalog",
    "create_schema" => "Create test schema", 
    "create_table" => "Create test table",
    "verify_structure" => "Verify resource hierarchy",
    "cleanup_all" => "Clean up all resources"
);

// Avoid: Vague or overly broad steps
setup_journey_steps!(
    logger,
    "setup" => "Do setup",
    "test" => "Run tests",
    "done" => "Finish"
);
```

### 2. Use Appropriate Operation Types

Choose the right macro variant for better visual organization:

```rust
// Use specific operation types for clarity
journey_step!(logger, "create_catalog", "my catalog", create => ...)?;
journey_step!(logger, "list_catalogs", "all catalogs", list => ...)?;
journey_step!(logger, "get_catalog", "catalog info", get => ...)?;
journey_step!(logger, "delete_catalog", "my catalog", delete => ...)?;
```

### 3. Add Contextual Information

Use the info and warn methods to provide context:

```rust
logger.info(&format!("Using storage root: {}", storage_root))?;
logger.warn("This operation might take a while due to network latency")?;
```

### 4. Measure Performance for Critical Operations

Track performance for operations that might be slow:

```rust
let result = metrics.measure("bulk_operation", async {
    // Potentially slow operation
    client.bulk_import(data).await
}).await?;
```

### 5. Handle Cleanup Gracefully

Use `cleanup_step` for cleanup operations that might fail:

```rust
async fn cleanup(&self, client: &UnityCatalogClient) -> AcceptanceResult<()> {
    if let Some(logger) = &self.logger {
        cleanup_step(logger, "cleanup_schema", 
            client.catalog(&self.catalog).schema(&self.schema).delete(Some(true))
        ).await?;
        
        cleanup_step(logger, "cleanup_catalog",
            client.catalog(&self.catalog).delete(Some(true))
        ).await?;
    }
    Ok(())
}
```

## Integration with Existing Code

The new reporting utilities are designed to be minimally invasive. You can:

1. **Gradually migrate**: Add reporting to new journeys while leaving existing ones unchanged
2. **Mix approaches**: Use both old `println!` and new reporting in the same journey during transition
3. **Disable reporting**: Set verbosity to 0 to get minimal output similar to the old system

## Troubleshooting

### Common Issues

1. **Colors not showing**: Check if your terminal supports colors and `NO_COLOR` env var is not set
2. **Progress bars not working**: Ensure your terminal supports cursor movement
3. **Table formatting issues**: Adjust `table_width` in ReportingConfig or let it auto-detect

### Debugging

Enable verbose output to see detailed information:

```rust
let config = ReportingConfig {
    verbosity: 2,
    ..ReportingConfig::default()
};
```

Or set environment variable:
```bash
export UC_ACCEPTANCE_VERBOSITY=2
```

## Examples

See the complete examples in:
- `examples/rich_reporting_journey.rs` - Comprehensive example showing all features
- `src/journeys/catalog_enhanced.rs` - Enhanced version of the simple catalog journey
- `src/journeys/catalog_simple.rs` - Original simple journey for comparison
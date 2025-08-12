# File Descriptors

This directory contains protobuf file descriptors used by the build script to generate code from gnostic annotations.

## Generating File Descriptors

To generate the file descriptors, run the following command from the project root:

```bash
buf build --output crates/common/descriptors/descriptors.bin
```

This will create a `descriptors.bin` file that contains the compiled protobuf file descriptors for all proto files in the project.

## What the Build Script Does

The build script (`build.rs`) in the common crate:

1. Loads the file descriptors from `descriptors.bin`
2. Parses all protobuf files and their definitions
3. Extracts gnostic annotations from:
   - File-level options
   - Message-level options
   - Field-level options  
   - Service-level options
   - Method-level options (where `operation_id` annotations are found)

## Current Status

The build script is currently a working skeleton that:
- ✅ Loads and parses file descriptors (37 files detected)
- ✅ Iterates through all protobuf definitions
- ✅ Detects services and methods (7 services with 35+ total methods found)
- ✅ Identifies that gnostic annotations are stored as compiled extension fields
- ✅ Provides detailed logging of processed structures
- ❌ Parses specific gnostic extension values (like `operation_id`) (TODO)
- ❌ Generates Rust code based on extracted annotations (TODO)

## Build Script Output Summary

When run, the build script successfully processes:
- **37 protobuf file descriptors** including Google, gnostic, and Unity Catalog definitions
- **7 Unity Catalog services**:
  - `CatalogsService` (5 methods)
  - `CredentialsService` (5 methods) 
  - `ExternalLocationsService` (5 methods)
  - `RecipientsService` (5 methods)
  - `SchemasService` (5 methods)
  - `SharingService` (multiple methods)
  - `TablesService` (multiple methods)

## Key Findings

1. **Gnostic annotations are compiled**: The annotations like `(gnostic.openapi.v3.operation) = {operation_id: "ListCatalogs"}` are stored as compiled protobuf extension fields, not as uninterpreted options.

2. **Extension field access needed**: To extract `operation_id` and other gnostic values, we need to decode the protobuf extension fields using the proper extension numbers from the gnostic schema.

3. **Generated code structure**: Each service method contains the information needed to generate REST handler traits and implementations.

## Next Steps

1. **Add gnostic extension parsing**: Import and use the gnostic protobuf definitions to properly decode extension fields containing `operation_id` and other OpenAPI metadata.

2. **Extract REST handler information**: For each service method, extract:
   - Operation ID (from gnostic.openapi.v3.operation)
   - HTTP method and path (from google.api.http)
   - Request/response types
   - Parameter sources (path, query, body)

3. **Generate handler traits**: Create Rust code to replace the current `rest_handlers!` macro with build-script generated trait definitions.

4. **Generate implementation scaffolding**: Auto-generate method signatures and basic implementations that can be customized.

5. **Integration**: Update the common crate to use generated code instead of macro-based generation.

## Testing the Build Script

To test the current build script:

```bash
# Generate descriptors
cd proto && buf build --output ../crates/common/descriptors/descriptors.bin

# Run build script (shows detailed processing info)
cd crates/common && cargo build
```
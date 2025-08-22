# Scripts Directory

This directory contains utility scripts for the Unity Catalog project.

## update_openapi_schemas.py

A Python script that updates the OpenAPI specification with rich type information from JSON Schema files.

### Purpose

The Unity Catalog project generates both an OpenAPI specification (`openapi/openapi.yaml`) and detailed JSON Schema files (`openapi/jsonschema/*.json`). The JSON Schema files contain much richer type information including:

- Detailed validation patterns (regex)
- String length constraints (minLength, maxLength)
- Numeric constraints (minimum, maximum, exclusiveMinimum)
- Enum values with proper descriptions
- More precise type definitions
- Better field descriptions

This script reads the JSON Schema files and updates the OpenAPI spec to include this richer type information, making the API documentation more complete and enabling better client code generation.

### Features

- ✅ Extracts rich schema definitions from JSON Schema bundle files
- ✅ Converts JSON Schema format to OpenAPI Schema format
- ✅ Preserves validation patterns, enums, constraints, and descriptions  
- ✅ Handles exclusiveMinimum/exclusiveMaximum conversion for OpenAPI compatibility
- ✅ Creates backup of original OpenAPI file before modification
- ✅ Provides detailed logging of the update process
- ✅ Handles duplicate schema names gracefully
- ✅ Error handling with automatic backup restoration

### Usage

Run the script from the project root directory:

```bash
python scripts/update_openapi_schemas.py
```

Or make it executable and run directly:

```bash
chmod +x scripts/update_openapi_schemas.py
./scripts/update_openapi_schemas.py
```

### Requirements

The script requires Python 3.10+ and the following dependencies (already included in the project):

- `pyyaml` - for YAML parsing and generation
- Standard library modules: `json`, `re`, `pathlib`, `shutil`

### How It Works

1. **Schema Discovery**: Scans `openapi/jsonschema/` for `*.json` files
2. **Name Parsing**: Extracts schema names from filenames using the pattern:
   ```
   unitycatalog.{service}.v1.{TypeName}.schema.strict.bundle.json
   ```
3. **Schema Extraction**: Reads JSON Schema definitions from the `$defs` section
4. **Format Conversion**: Converts JSON Schema to OpenAPI Schema format:
   - Removes JSON Schema-specific fields (`$schema`, `$id`)
   - Converts `exclusiveMinimum`/`exclusiveMaximum` from numeric to boolean format
   - Preserves all validation rules and constraints
5. **OpenAPI Update**: Updates the `components.schemas` section of the OpenAPI spec
6. **Backup Creation**: Creates a backup file before making changes

### Example Improvements

The script adds rich validation information like:

```yaml
# Before (basic OpenAPI schema)
CreateCatalogRequest:
  type: object
  properties:
    name:
      type: string
      description: Name of catalog.

# After (enriched with JSON Schema data)
CreateCatalogRequest:
  type: object
  additionalProperties: false
  description: Create a new catalog
  properties:
    name:
      type: string
      description: Name of catalog.
      minLength: 3
      pattern: ^[a-z][0-9a-z_]*[0-9a-z]$
  required:
    - name
```

### Files Modified

- **Input**: `openapi/jsonschema/*.json` (read-only)
- **Output**: `openapi/openapi.yaml` (modified)
- **Backup**: `openapi/openapi.yaml.bak` (created automatically)

### Error Handling

- Creates backup before making changes
- Validates input files exist
- Handles parsing errors gracefully
- Restores backup on failure
- Provides detailed error messages

### Integration

This script can be integrated into your build process or CI/CD pipeline to ensure the OpenAPI spec is always up-to-date with the latest schema definitions.

```bash
# Example: Run after generating schemas
make generate-schemas
python scripts/update_openapi_schemas.py
```

### Example Output

When you run the script, you'll see output like this:

```
================================================================================
Unity Catalog OpenAPI Schema Updater
================================================================================
Loading JSON schemas from: /path/to/openapi/jsonschema
Updating OpenAPI spec at: /path/to/openapi/openapi.yaml

Created backup at: /path/to/openapi/openapi.yaml.bak
Loaded schema for CatalogInfo
Loaded schema for CreateCatalogRequest
Loaded schema for CredentialInfo
... (loading more schemas)

Loaded 104 schemas from JSON schema files
Updating existing schema: CatalogInfo
Updating existing schema: CreateCatalogRequest
... (updating more schemas)

Updated OpenAPI spec:
  - Updated 104 existing schemas
  - Added 0 new schemas
  - Total schemas: 106

================================================================================
✅ Successfully updated OpenAPI specification!
📄 Backup of original file saved as: /path/to/openapi/openapi.yaml.bak
================================================================================
```

The updated OpenAPI schemas will now include rich validation patterns, constraints, and detailed descriptions from the JSON Schema files.

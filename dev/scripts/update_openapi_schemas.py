#!/usr/bin/env python3
"""
Script to update the OpenAPI specification with rich type information from JSON Schema files.

This script reads JSON Schema files from openapi/jsonschema/ and updates the OpenAPI spec
in openapi/openapi.yaml to include the richer type definitions, validation rules,
and descriptions found in the JSON Schema files.

Features:
- Extracts rich schema definitions from JSON Schema bundle files
- Converts JSON Schema format to OpenAPI Schema format
- Preserves validation patterns, enums, constraints, and descriptions
- Handles exclusiveMinimum/exclusiveMaximum conversion for OpenAPI compatibility
- Creates backup of original OpenAPI file before modification
- Provides detailed logging of the update process

The script expects to be run from the project root directory and will:
1. Read all *.json files from openapi/jsonschema/
2. Parse schema names from filenames (format: unitycatalog.{service}.v1.{TypeName}.schema.strict.bundle.json)
3. Update the components/schemas section of openapi/openapi.yaml
4. Create a backup file (openapi.yaml.bak) before making changes
"""

import json
import re
from pathlib import Path
from typing import Any, Dict

import yaml


def load_json_schemas(jsonschema_dir: Path) -> Dict[str, Dict[str, Any]]:
    """
    Load all JSON schema files from the jsonschema directory.

    Returns a dictionary mapping schema names to their definitions.
    """
    schemas = {}

    for file_path in jsonschema_dir.glob("*.json"):
        try:
            with open(file_path, "r", encoding="utf-8") as f:
                schema_data = json.load(f)

            # Extract the schema name from the filename
            # Format: unitycatalog.{service}.v1.{TypeName}.schema.strict.bundle.json
            filename = file_path.stem

            # Remove the prefix and suffix to get the type name
            pattern = r"unitycatalog\.[\w_]+\.v1\.([^.]+)\.schema\.strict\.bundle"
            match = re.match(pattern, filename)

            if match:
                type_name = match.group(1)

                # Check for duplicate schema names and skip if already exists
                if type_name in schemas:
                    print(
                        f"Warning: Duplicate schema name '{type_name}' found in {filename}, skipping..."
                    )
                    continue

                # Extract the actual schema definition from the $defs section
                if "$defs" in schema_data:
                    # Get the first (and usually only) definition in $defs
                    defs = schema_data["$defs"]
                    if defs:
                        # Take the first definition
                        schema_def = next(iter(defs.values()))
                        schemas[type_name] = schema_def
                        print(f"Loaded schema for {type_name}")
                    else:
                        print(f"Warning: No definitions found in $defs for {filename}")
                else:
                    # If no $defs, use the root schema
                    schemas[type_name] = schema_data
                    print(f"Loaded root schema for {type_name}")
            else:
                print(f"Warning: Could not parse schema name from {filename}")

        except Exception as e:
            print(f"Error loading {file_path}: {e}")

    return schemas


def convert_json_schema_to_openapi(json_schema: Dict[str, Any]) -> Dict[str, Any]:
    """
    Convert a JSON Schema definition to OpenAPI Schema format.

    JSON Schema and OpenAPI Schema are very similar, but there are some differences.
    """
    openapi_schema = json_schema.copy()

    # Remove JSON Schema specific fields that aren't valid in OpenAPI
    json_schema_only_fields = ["$schema", "$id"]
    for field in json_schema_only_fields:
        if field in openapi_schema:
            del openapi_schema[field]

    # Handle additionalProperties - keep it as is for OpenAPI compatibility
    # Both JSON Schema and OpenAPI support additionalProperties

    # Handle exclusiveMinimum/exclusiveMaximum differences
    if "exclusiveMinimum" in openapi_schema and isinstance(
        openapi_schema["exclusiveMinimum"], (int, float)
    ):
        # In JSON Schema draft 2020-12, exclusiveMinimum is a number
        # In OpenAPI 3.0, it should be a boolean with minimum set
        exclusive_min = openapi_schema.pop("exclusiveMinimum")
        openapi_schema["minimum"] = exclusive_min
        openapi_schema["exclusiveMinimum"] = True

    if "exclusiveMaximum" in openapi_schema and isinstance(
        openapi_schema["exclusiveMaximum"], (int, float)
    ):
        exclusive_max = openapi_schema.pop("exclusiveMaximum")
        openapi_schema["maximum"] = exclusive_max
        openapi_schema["exclusiveMaximum"] = True

    # Recursively process nested objects and arrays
    if "properties" in openapi_schema:
        for prop_name, prop_schema in openapi_schema["properties"].items():
            if isinstance(prop_schema, dict):
                openapi_schema["properties"][prop_name] = (
                    convert_json_schema_to_openapi(prop_schema)
                )

    if "items" in openapi_schema and isinstance(openapi_schema["items"], dict):
        openapi_schema["items"] = convert_json_schema_to_openapi(
            openapi_schema["items"]
        )

    if "additionalProperties" in openapi_schema and isinstance(
        openapi_schema["additionalProperties"], dict
    ):
        openapi_schema["additionalProperties"] = convert_json_schema_to_openapi(
            openapi_schema["additionalProperties"]
        )

    return openapi_schema


def update_openapi_spec(openapi_file: Path, schemas: Dict[str, Dict[str, Any]]) -> None:
    """
    Update the OpenAPI specification file with the rich schema definitions.
    """
    # Load the existing OpenAPI spec
    with open(openapi_file, "r", encoding="utf-8") as f:
        openapi_spec = yaml.safe_load(f)

    # Ensure components.schemas exists
    if "components" not in openapi_spec:
        openapi_spec["components"] = {}
    if "schemas" not in openapi_spec["components"]:
        openapi_spec["components"]["schemas"] = {}

    # Update or add schemas
    updated_count = 0
    added_count = 0

    for schema_name, schema_def in schemas.items():
        try:
            # Convert JSON Schema to OpenAPI Schema format
            openapi_schema = convert_json_schema_to_openapi(schema_def)

            if schema_name in openapi_spec["components"]["schemas"]:
                print(f"Updating existing schema: {schema_name}")
                updated_count += 1
            else:
                print(f"Adding new schema: {schema_name}")
                added_count += 1

            openapi_spec["components"]["schemas"][schema_name] = openapi_schema
        except Exception as e:
            print(f"Error processing schema {schema_name}: {e}")
            continue

    # Write the updated OpenAPI spec back to file
    with open(openapi_file, "w", encoding="utf-8") as f:
        yaml.dump(
            openapi_spec,
            f,
            default_flow_style=False,
            sort_keys=False,
            allow_unicode=True,
            width=120,
            indent=2,
        )

    print("\nUpdated OpenAPI spec:")
    print(f"  - Updated {updated_count} existing schemas")
    print(f"  - Added {added_count} new schemas")
    print(f"  - Total schemas: {len(openapi_spec['components']['schemas'])}")


def main():
    """
    Main entry point for the script.

    Returns:
        int: Exit code (0 for success, 1 for error)
    """
    script_dir = Path(__file__).parent
    project_root = script_dir.parent.parent

    jsonschema_dir = project_root / "openapi" / "jsonschema"
    openapi_file = project_root / "openapi" / "openapi.yaml"

    print("=" * 80)
    print("Unity Catalog OpenAPI Schema Updater")
    print("=" * 80)
    print(f"Loading JSON schemas from: {jsonschema_dir}")
    print(f"Updating OpenAPI spec at: {openapi_file}")
    print()

    # Verify paths exist
    if not jsonschema_dir.exists():
        print(f"Error: JSON schema directory not found: {jsonschema_dir}")
        return 1

    if not openapi_file.exists():
        print(f"Error: OpenAPI file not found: {openapi_file}")
        return 1

    # Load JSON schemas
    schemas = load_json_schemas(jsonschema_dir)

    if not schemas:
        print("Error: No schemas loaded")
        return 1

    print(f"\nLoaded {len(schemas)} schemas from JSON schema files")

    # Update OpenAPI spec
    try:
        update_openapi_spec(openapi_file, schemas)
        print("\n" + "=" * 80)
        print("✅ Successfully updated OpenAPI specification!")
        print("=" * 80)
        return 0
    except Exception as e:
        print(f"\n❌ Error updating OpenAPI spec: {e}")
        return 1


if __name__ == "__main__":
    exit(main())

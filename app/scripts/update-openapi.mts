import types from "../src/components/forms/editor/schemas";
import fs from "fs";
import path from "path";
import yaml from "yaml";

// Read and parse the OpenAPI YAML file
const openapiPath = path.resolve("../openapi/openapi.yaml");
const openapiContent = fs.readFileSync(openapiPath, "utf8");
const openapiSpec = yaml.parse(openapiContent);

for (const [key, value] of Object.entries(types)) {
  if (key in openapiSpec.components.schemas) {
    if (
      "properties" in value &&
      "properties" in openapiSpec.components.schemas[key]
    ) {
      openapiSpec.components.schemas[key].properties = value.properties;
    }
  }
}

let updatedOpenapiContent = yaml.stringify(openapiSpec, {
  indent: 2,
  indentSeq: true,
  singleQuote: true,
});

const namespaces = [
  "catalogs",
  "schemas",
  "tables",
  "credentials",
  "external_locations",
  "recipients",
  "shares",
];

for (const namespace of namespaces) {
  updatedOpenapiContent = updatedOpenapiContent.replaceAll(
    `unitycatalog.${namespace}.v1.`,
    "'#/components/schemas/",
  );
}
updatedOpenapiContent = updatedOpenapiContent.replaceAll(
  ".jsonschema.json",
  "'",
);
updatedOpenapiContent = updatedOpenapiContent.replaceAll(".schema.json", "'");
updatedOpenapiContent = updatedOpenapiContent.replaceAll(
  "$ref: google.protobuf.Struct",
  "type: object",
);

fs.writeFileSync(openapiPath, updatedOpenapiContent);

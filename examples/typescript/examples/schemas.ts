import { UnityCatalogClient } from "@unitycatalog/client";

// [snippet:list_schemas]
export async function listSchemasExample(): Promise<void> {
  const client = new UnityCatalogClient("http://localhost:8080");
  const schemas = await client.listSchemas("my_catalog");
  for (const schema of schemas) {
    console.log(schema.name);
  }
}
// [/snippet:list_schemas]

// [snippet:create_schema]
export async function createSchemaExample(): Promise<void> {
  const client = new UnityCatalogClient("http://localhost:8080");
  const schema = await client.createSchema("my_schema", "my_catalog", {
    comment: "My first schema",
  });
  console.log(`Created: ${schema.catalogName}.${schema.name}`);
}
// [/snippet:create_schema]

// [snippet:get_schema]
export async function getSchemaExample(): Promise<void> {
  const client = new UnityCatalogClient("http://localhost:8080");
  const schema = await client.schema("my_catalog", "my_schema").get();
  console.log(`Got: ${schema.catalogName}.${schema.name}`);
}
// [/snippet:get_schema]

// [snippet:update_schema]
export async function updateSchemaExample(): Promise<void> {
  const client = new UnityCatalogClient("http://localhost:8080");
  const schema = await client.schema("my_catalog", "my_schema").update({
    comment: "Updated comment",
  });
  console.log(`Updated: ${schema.catalogName}.${schema.name}`);
}
// [/snippet:update_schema]

// [snippet:delete_schema]
export async function deleteSchemaExample(): Promise<void> {
  const client = new UnityCatalogClient("http://localhost:8080");
  await client.schema("my_catalog", "my_schema").delete();
  console.log("Deleted schema");
}
// [/snippet:delete_schema]

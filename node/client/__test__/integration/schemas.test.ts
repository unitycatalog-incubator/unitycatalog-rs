import type { UnityCatalogClient } from "../../dist/index";
import { createClient, uniqueName } from "./helpers";

describe("Schemas", () => {
  let client: UnityCatalogClient;
  const catalogName = uniqueName("sch_cat");

  beforeAll(async () => {
    client = createClient();
    await client.createCatalog(catalogName);
  });

  afterAll(async () => {
    await client.catalog(catalogName).delete({ force: true });
  });

  it("should create a schema", async () => {
    const name = uniqueName("schema");
    const schema = await client.createSchema(name, catalogName, {
      comment: "test schema",
    });

    expect(schema.name).toBe(name);
    expect(schema.catalogName).toBe(catalogName);
    expect(schema.comment).toBe("test schema");
  });

  it("should list schemas in a catalog", async () => {
    const name = uniqueName("schema_list");
    await client.createSchema(name, catalogName);

    const schemas = await client.listSchemas(catalogName);
    const names = schemas.map((s) => s.name);
    expect(names).toContain(name);
  });

  it("should get a schema by name", async () => {
    const name = uniqueName("schema_get");
    await client.createSchema(name, catalogName, { comment: "fetch me" });

    const fetched = await client.schema(catalogName, name).get();
    expect(fetched.name).toBe(name);
    expect(fetched.catalogName).toBe(catalogName);
    expect(fetched.comment).toBe("fetch me");
  });

  it("should update a schema", async () => {
    const name = uniqueName("schema_upd");
    await client.createSchema(name, catalogName, { comment: "original" });

    const updated = await client.schema(catalogName, name).update({
      comment: "updated",
    });

    expect(updated.name).toBe(name);
    expect(updated.comment).toBe("updated");
  });

  it("should delete a schema", async () => {
    const name = uniqueName("schema_del");
    await client.createSchema(name, catalogName);

    await client.schema(catalogName, name).delete();

    const schemas = await client.listSchemas(catalogName);
    const names = schemas.map((s) => s.name);
    expect(names).not.toContain(name);
  });

  it("should create a schema with properties", async () => {
    const name = uniqueName("schema_props");
    const schema = await client.createSchema(name, catalogName, {
      properties: { tier: "gold" },
    });

    expect(schema.properties).toEqual({ tier: "gold" });
  });
});

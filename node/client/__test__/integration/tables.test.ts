import type { UnityCatalogClient } from "../../dist/index";
import { DataSourceFormat, TableType } from "../../dist/index";
import { createClient, uniqueName } from "./helpers";

describe("Tables", () => {
  let client: UnityCatalogClient;
  const catalogName = uniqueName("tbl_cat");
  const schemaName = uniqueName("tbl_sch");

  beforeAll(async () => {
    client = createClient();
    await client.createCatalog(catalogName);
    await client.createSchema(schemaName, catalogName);
  });

  afterAll(async () => {
    await client.catalog(catalogName).delete({ force: true });
  });

  it("should create a table", async () => {
    const name = uniqueName("table");
    const table = await client.createTable(
      name,
      schemaName,
      catalogName,
      TableType.MANAGED,
      DataSourceFormat.DELTA,
      { comment: "test table" },
    );

    expect(table.name).toBe(name);
    expect(table.schemaName).toBe(schemaName);
    expect(table.catalogName).toBe(catalogName);
    expect(table.comment).toBe("test table");
  });

  it("should list tables in a schema", async () => {
    const name = uniqueName("table_list");
    await client.createTable(
      name,
      schemaName,
      catalogName,
      TableType.MANAGED,
      DataSourceFormat.DELTA,
    );

    const tables = await client.listTables(catalogName, schemaName);
    const names = tables.map((t) => t.name);
    expect(names).toContain(name);
  });

  it("should get a table by full name", async () => {
    const name = uniqueName("table_get");
    await client.createTable(
      name,
      schemaName,
      catalogName,
      TableType.MANAGED,
      DataSourceFormat.DELTA,
      { comment: "get me" },
    );

    const fullName = `${catalogName}.${schemaName}.${name}`;
    const fetched = await client.table(fullName).get();
    expect(fetched.name).toBe(name);
    expect(fetched.comment).toBe("get me");
  });

  it("should delete a table", async () => {
    const name = uniqueName("table_del");
    await client.createTable(
      name,
      schemaName,
      catalogName,
      TableType.MANAGED,
      DataSourceFormat.DELTA,
    );

    const fullName = `${catalogName}.${schemaName}.${name}`;
    await client.table(fullName).delete();

    const tables = await client.listTables(catalogName, schemaName);
    const names = tables.map((t) => t.name);
    expect(names).not.toContain(name);
  });

  it("should create a table with properties", async () => {
    const name = uniqueName("table_props");
    const table = await client.createTable(
      name,
      schemaName,
      catalogName,
      TableType.EXTERNAL,
      DataSourceFormat.PARQUET,
      {
        comment: "external table",
        properties: { source: "s3" },
      },
    );

    expect(table.name).toBe(name);
    expect(table.properties).toEqual({ source: "s3" });
  });
});

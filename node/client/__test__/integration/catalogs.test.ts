import type { UnityCatalogClient } from "../../dist/index";
import { createClient, uniqueName } from "./helpers";

describe("Catalogs", () => {
  let client: UnityCatalogClient;

  beforeAll(() => {
    client = createClient();
  });

  it("should create a catalog", async () => {
    const name = uniqueName("cat");
    const catalog = await client.createCatalog(name, {
      comment: "test catalog",
    });

    expect(catalog.name).toBe(name);
    expect(catalog.comment).toBe("test catalog");
  });

  it("should list catalogs including a newly created one", async () => {
    const name = uniqueName("cat_list");
    await client.createCatalog(name);

    const catalogs = await client.listCatalogs();
    const names = catalogs.map((c) => c.name);
    expect(names).toContain(name);
  });

  it("should get a catalog by name", async () => {
    const name = uniqueName("cat_get");
    await client.createCatalog(name, { comment: "get-me" });

    const fetched = await client.catalog(name).get();
    expect(fetched.name).toBe(name);
    expect(fetched.comment).toBe("get-me");
  });

  it("should update a catalog", async () => {
    const name = uniqueName("cat_upd");
    await client.createCatalog(name, { comment: "original" });

    const updated = await client.catalog(name).update({
      comment: "updated comment",
    });

    expect(updated.name).toBe(name);
    expect(updated.comment).toBe("updated comment");
  });

  it("should delete a catalog", async () => {
    const name = uniqueName("cat_del");
    await client.createCatalog(name);

    await client.catalog(name).delete();

    const catalogs = await client.listCatalogs();
    const names = catalogs.map((c) => c.name);
    expect(names).not.toContain(name);
  });

  it("should create a catalog with properties", async () => {
    const name = uniqueName("cat_props");
    const catalog = await client.createCatalog(name, {
      properties: { env: "test", team: "platform" },
    });

    expect(catalog.name).toBe(name);
    expect(catalog.properties).toEqual({ env: "test", team: "platform" });
  });

  it("should fail to get a non-existent catalog", async () => {
    await expect(
      client.catalog("nonexistent_catalog_xyz").get(),
    ).rejects.toThrow();
  });
});

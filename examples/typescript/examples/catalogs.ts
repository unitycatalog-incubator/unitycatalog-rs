import { UnityCatalogClient } from "@unitycatalog/client";

// [snippet:list_catalogs]
export async function listCatalogsExample(): Promise<void> {
  const client = new UnityCatalogClient("http://localhost:8080");
  const catalogs = await client.listCatalogs();
  for (const catalog of catalogs) {
    console.log(catalog.name);
  }
}
// [/snippet:list_catalogs]

// [snippet:create_catalog]
export async function createCatalogExample(): Promise<void> {
  const client = new UnityCatalogClient("http://localhost:8080");
  const catalog = await client.createCatalog("my_catalog", {
    comment: "My first catalog",
  });
  console.log(`Created: ${catalog.name}`);
}
// [/snippet:create_catalog]

// [snippet:get_catalog]
export async function getCatalogExample(): Promise<void> {
  const client = new UnityCatalogClient("http://localhost:8080");
  const catalog = await client.catalog("my_catalog").get();
  console.log(`Got: ${catalog.name}`);
}
// [/snippet:get_catalog]

// [snippet:update_catalog]
export async function updateCatalogExample(): Promise<void> {
  const client = new UnityCatalogClient("http://localhost:8080");
  const catalog = await client.catalog("my_catalog").update({
    comment: "Updated comment",
  });
  console.log(`Updated: ${catalog.name}`);
}
// [/snippet:update_catalog]

// [snippet:delete_catalog]
export async function deleteCatalogExample(): Promise<void> {
  const client = new UnityCatalogClient("http://localhost:8080");
  await client.catalog("my_catalog").delete();
  console.log("Deleted catalog");
}
// [/snippet:delete_catalog]

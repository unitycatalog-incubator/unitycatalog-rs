import type { UnityCatalogClient } from "../../dist/index";
import { VolumeType } from "../../dist/index";
import { createClient, uniqueName } from "./helpers";

describe("Volumes", () => {
  let client: UnityCatalogClient;
  const catalogName = uniqueName("vol_cat");
  const schemaName = uniqueName("vol_sch");

  beforeAll(async () => {
    client = createClient();
    await client.createCatalog(catalogName);
    await client.createSchema(schemaName, catalogName);
  });

  afterAll(async () => {
    await client.catalog(catalogName).delete({ force: true });
  });

  it("should create a volume", async () => {
    const name = uniqueName("volume");
    const volume = await client.createVolume(
      catalogName,
      schemaName,
      name,
      VolumeType.MANAGED,
      { comment: "test volume" },
    );

    expect(volume.name).toBe(name);
    expect(volume.catalogName).toBe(catalogName);
    expect(volume.schemaName).toBe(schemaName);
    expect(volume.comment).toBe("test volume");
  });

  it("should list volumes in a schema", async () => {
    const name = uniqueName("vol_list");
    await client.createVolume(
      catalogName,
      schemaName,
      name,
      VolumeType.MANAGED,
    );

    const volumes = await client.listVolumes(catalogName, schemaName);
    const names = volumes.map((v) => v.name);
    expect(names).toContain(name);
  });

  it("should get a volume", async () => {
    const name = uniqueName("vol_get");
    await client.createVolume(
      catalogName,
      schemaName,
      name,
      VolumeType.MANAGED,
      { comment: "get me" },
    );

    const fetched = await client.volume(catalogName, schemaName, name).get();
    expect(fetched.name).toBe(name);
    expect(fetched.comment).toBe("get me");
  });

  it("should update a volume", async () => {
    const name = uniqueName("vol_upd");
    await client.createVolume(
      catalogName,
      schemaName,
      name,
      VolumeType.MANAGED,
      { comment: "original" },
    );

    const updated = await client
      .volume(catalogName, schemaName, name)
      .update({ comment: "updated" });
    expect(updated.comment).toBe("updated");
  });

  it("should delete a volume", async () => {
    const name = uniqueName("vol_del");
    await client.createVolume(
      catalogName,
      schemaName,
      name,
      VolumeType.MANAGED,
    );

    await client.volume(catalogName, schemaName, name).delete();

    const volumes = await client.listVolumes(catalogName, schemaName);
    const names = volumes.map((v) => v.name);
    expect(names).not.toContain(name);
  });
});

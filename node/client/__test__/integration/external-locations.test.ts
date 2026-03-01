import type { UnityCatalogClient } from "../../dist/index";
import { Purpose } from "../../dist/index";
import { createClient, uniqueName } from "./helpers";

describe("External Locations", () => {
  let client: UnityCatalogClient;
  let credentialName: string;

  beforeAll(async () => {
    client = createClient();
    credentialName = uniqueName("extloc_cred");
    await client.createCredential(credentialName, Purpose.STORAGE, {
      skipValidation: true,
    });
  });

  it("should create an external location", async () => {
    const name = uniqueName("extloc");
    const location = await client.createExternalLocation(
      name,
      "s3://test-bucket/path",
      credentialName,
      { comment: "test location", skipValidation: true },
    );

    expect(location.name).toBe(name);
    expect(location.url).toBe("s3://test-bucket/path");
    expect(location.comment).toBe("test location");
  });

  it("should list external locations including a newly created one", async () => {
    const name = uniqueName("extloc_list");
    await client.createExternalLocation(
      name,
      "s3://test-bucket/list-path",
      credentialName,
      { skipValidation: true },
    );

    const locations = await client.listExternalLocations();
    const names = locations.map((l) => l.name);
    expect(names).toContain(name);
  });

  it("should get an external location by name", async () => {
    const name = uniqueName("extloc_get");
    await client.createExternalLocation(
      name,
      "s3://test-bucket/get-path",
      credentialName,
      { comment: "get-me", skipValidation: true },
    );

    const fetched = await client.externalLocation(name).get();
    expect(fetched.name).toBe(name);
    expect(fetched.url).toBe("s3://test-bucket/get-path");
    expect(fetched.comment).toBe("get-me");
  });

  it("should update an external location", async () => {
    const name = uniqueName("extloc_upd");
    await client.createExternalLocation(
      name,
      "s3://test-bucket/upd-path",
      credentialName,
      { comment: "original", skipValidation: true },
    );

    const updated = await client.externalLocation(name).update({
      comment: "updated location",
      skipValidation: true,
    });

    expect(updated.name).toBe(name);
    expect(updated.comment).toBe("updated location");
  });

  it("should delete an external location", async () => {
    const name = uniqueName("extloc_del");
    await client.createExternalLocation(
      name,
      "s3://test-bucket/del-path",
      credentialName,
      { skipValidation: true },
    );

    await client.externalLocation(name).delete();

    const locations = await client.listExternalLocations();
    const names = locations.map((l) => l.name);
    expect(names).not.toContain(name);
  });
});

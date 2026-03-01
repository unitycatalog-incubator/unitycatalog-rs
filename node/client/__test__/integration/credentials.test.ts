import type { UnityCatalogClient } from "../../dist/index";
import { Purpose } from "../../dist/index";
import { createClient, uniqueName } from "./helpers";

describe("Credentials", () => {
  let client: UnityCatalogClient;

  beforeAll(() => {
    client = createClient();
  });

  it("should create a credential", async () => {
    const name = uniqueName("cred");
    const credential = await client.createCredential(name, Purpose.STORAGE, {
      comment: "test credential",
      skipValidation: true,
    });

    expect(credential.name).toBe(name);
    expect(credential.comment).toBe("test credential");
  });

  it("should list credentials including a newly created one", async () => {
    const name = uniqueName("cred_list");
    await client.createCredential(name, Purpose.STORAGE, {
      skipValidation: true,
    });

    const credentials = await client.listCredentials();
    const names = credentials.map((c) => c.name);
    expect(names).toContain(name);
  });

  it("should get a credential by name", async () => {
    const name = uniqueName("cred_get");
    await client.createCredential(name, Purpose.STORAGE, {
      comment: "get-me",
      skipValidation: true,
    });

    const fetched = await client.credential(name).get();
    expect(fetched.name).toBe(name);
    expect(fetched.comment).toBe("get-me");
  });

  it("should update a credential", async () => {
    const name = uniqueName("cred_upd");
    await client.createCredential(name, Purpose.STORAGE, {
      comment: "original",
      skipValidation: true,
    });

    const updated = await client.credential(name).update({
      comment: "updated credential",
      skipValidation: true,
    });

    expect(updated.name).toBe(name);
    expect(updated.comment).toBe("updated credential");
  });

  it("should delete a credential", async () => {
    const name = uniqueName("cred_del");
    await client.createCredential(name, Purpose.STORAGE, {
      skipValidation: true,
    });

    await client.credential(name).delete();

    const credentials = await client.listCredentials();
    const names = credentials.map((c) => c.name);
    expect(names).not.toContain(name);
  });
});

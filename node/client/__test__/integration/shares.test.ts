import type { UnityCatalogClient } from "../../dist/index";
import { createClient, uniqueName } from "./helpers";

describe("Shares", () => {
  let client: UnityCatalogClient;

  beforeAll(() => {
    client = createClient();
  });

  it("should create a share", async () => {
    const name = uniqueName("share");
    const share = await client.createShare(name, {
      comment: "test share",
    });

    expect(share.name).toBe(name);
    expect(share.comment).toBe("test share");
  });

  it("should list shares including a newly created one", async () => {
    const name = uniqueName("share_list");
    await client.createShare(name);

    const shares = await client.listShares();
    const names = shares.map((s) => s.name);
    expect(names).toContain(name);
  });

  it("should get a share by name", async () => {
    const name = uniqueName("share_get");
    await client.createShare(name, { comment: "get-me" });

    const fetched = await client.share(name).get();
    expect(fetched.name).toBe(name);
    expect(fetched.comment).toBe("get-me");
  });

  it("should update a share", async () => {
    const name = uniqueName("share_upd");
    await client.createShare(name, { comment: "original" });

    const updated = await client.share(name).update({
      comment: "updated share",
    });

    expect(updated.name).toBe(name);
    expect(updated.comment).toBe("updated share");
  });

  it("should delete a share", async () => {
    const name = uniqueName("share_del");
    await client.createShare(name);

    await client.share(name).delete();

    const shares = await client.listShares();
    const names = shares.map((s) => s.name);
    expect(names).not.toContain(name);
  });
});

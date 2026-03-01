import type { UnityCatalogClient } from "../../dist/index";
import { AuthenticationType } from "../../dist/index";
import { createClient, uniqueName } from "./helpers";

describe("Recipients", () => {
  let client: UnityCatalogClient;

  beforeAll(() => {
    client = createClient();
  });

  it("should create a recipient", async () => {
    const name = uniqueName("recip");
    const recipient = await client.createRecipient(
      name,
      AuthenticationType.TOKEN,
      "test-owner",
      { comment: "test recipient" },
    );

    expect(recipient.name).toBe(name);
    expect(recipient.comment).toBe("test recipient");
  });

  it("should list recipients including a newly created one", async () => {
    const name = uniqueName("recip_list");
    await client.createRecipient(name, AuthenticationType.TOKEN, "test-owner");

    const recipients = await client.listRecipients();
    const names = recipients.map((r) => r.name);
    expect(names).toContain(name);
  });

  it("should get a recipient by name", async () => {
    const name = uniqueName("recip_get");
    await client.createRecipient(name, AuthenticationType.TOKEN, "test-owner", {
      comment: "get-me",
    });

    const fetched = await client.recipient(name).get();
    expect(fetched.name).toBe(name);
    expect(fetched.comment).toBe("get-me");
  });

  it("should update a recipient", async () => {
    const name = uniqueName("recip_upd");
    await client.createRecipient(name, AuthenticationType.TOKEN, "test-owner", {
      comment: "original",
    });

    const updated = await client.recipient(name).update({
      comment: "updated recipient",
    });

    expect(updated.name).toBe(name);
    expect(updated.comment).toBe("updated recipient");
  });

  it("should delete a recipient", async () => {
    const name = uniqueName("recip_del");
    await client.createRecipient(name, AuthenticationType.TOKEN, "test-owner");

    await client.recipient(name).delete();

    const recipients = await client.listRecipients();
    const names = recipients.map((r) => r.name);
    expect(names).not.toContain(name);
  });
});

import { UnityCatalogClient } from "@unitycatalog/client";

// [snippet:list_tag_policies]
export async function listTagPoliciesExample(): Promise<void> {
  const client = new UnityCatalogClient("http://localhost:8080");
  const policies = await client.listTagPolicies();
  for (const policy of policies) {
    console.log(policy.tagKey);
  }
}
// [/snippet:list_tag_policies]

// [snippet:get_tag_policy]
export async function getTagPolicyExample(): Promise<void> {
  const client = new UnityCatalogClient("http://localhost:8080");
  const policy = await client.tagPolicy("classification").get();
  console.log(`Got: ${policy.tagKey}`);
}
// [/snippet:get_tag_policy]

// [snippet:delete_tag_policy]
export async function deleteTagPolicyExample(): Promise<void> {
  const client = new UnityCatalogClient("http://localhost:8080");
  await client.tagPolicy("classification").delete();
  console.log("Deleted tag policy");
}
// [/snippet:delete_tag_policy]

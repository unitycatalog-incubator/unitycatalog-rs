import { UnityCatalogClient } from "@unitycatalog/client";

// [snippet:list_volumes]
export async function listVolumesExample(): Promise<void> {
  const client = new UnityCatalogClient("http://localhost:8080");
  const volumes = await client.listVolumes("my_catalog", "my_schema");
  for (const volume of volumes) {
    console.log(volume.name);
  }
}
// [/snippet:list_volumes]

// [snippet:create_volume]
export async function createVolumeExample(): Promise<void> {
  const client = new UnityCatalogClient("http://localhost:8080");
  // volumeType: 1 = MANAGED
  const volume = await client.createVolume(
    "my_catalog",
    "my_schema",
    "my_volume",
    1,
    { comment: "My first volume" },
  );
  console.log(`Created: ${volume.name}`);
}
// [/snippet:create_volume]

// [snippet:get_volume]
export async function getVolumeExample(): Promise<void> {
  const client = new UnityCatalogClient("http://localhost:8080");
  const volume = await client
    .volume("my_catalog", "my_schema", "my_volume")
    .get();
  console.log(`Got: ${volume.name}`);
}
// [/snippet:get_volume]

// [snippet:update_volume]
export async function updateVolumeExample(): Promise<void> {
  const client = new UnityCatalogClient("http://localhost:8080");
  const volume = await client
    .volume("my_catalog", "my_schema", "my_volume")
    .update({ comment: "Updated comment" });
  console.log(`Updated: ${volume.name}`);
}
// [/snippet:update_volume]

// [snippet:delete_volume]
export async function deleteVolumeExample(): Promise<void> {
  const client = new UnityCatalogClient("http://localhost:8080");
  await client.volume("my_catalog", "my_schema", "my_volume").delete();
  console.log("Deleted volume");
}
// [/snippet:delete_volume]

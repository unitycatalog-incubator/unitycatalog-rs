# [snippet:list_volumes]
from unitycatalog_client import UnityCatalogClient

client = UnityCatalogClient(base_url="http://localhost:8080")
volumes = client.list_volumes("my_catalog", "my_schema")
for volume in volumes:
    print(volume.name)
# [/snippet:list_volumes]


# [snippet:create_volume]
def create_volume_example() -> None:
    from unitycatalog_client import UnityCatalogClient, VolumeType

    client = UnityCatalogClient(base_url="http://localhost:8080")
    volume = client.create_volume(
        "my_catalog",
        "my_schema",
        "my_volume",
        VolumeType.VOLUME_TYPE_MANAGED,
        comment="My first volume",
    )
    print(f"Created: {volume.name}")


# [/snippet:create_volume]


# [snippet:get_volume]
def get_volume_example() -> None:
    from unitycatalog_client import UnityCatalogClient

    client = UnityCatalogClient(base_url="http://localhost:8080")
    volume = client.volume("my_catalog", "my_schema", "my_volume").get()
    print(f"Got: {volume.name}")


# [/snippet:get_volume]


# [snippet:update_volume]
def update_volume_example() -> None:
    from unitycatalog_client import UnityCatalogClient

    client = UnityCatalogClient(base_url="http://localhost:8080")
    volume = client.volume("my_catalog", "my_schema", "my_volume").update(comment="Updated comment")
    print(f"Updated: {volume.name}")


# [/snippet:update_volume]


# [snippet:delete_volume]
def delete_volume_example() -> None:
    from unitycatalog_client import UnityCatalogClient

    client = UnityCatalogClient(base_url="http://localhost:8080")
    client.volume("my_catalog", "my_schema", "my_volume").delete()
    print("Deleted volume")


# [/snippet:delete_volume]

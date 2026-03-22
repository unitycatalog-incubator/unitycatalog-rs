# [snippet:list_catalogs]
from unitycatalog_client import UnityCatalogClient

client = UnityCatalogClient(base_url="http://localhost:8080")
catalogs = client.list_catalogs()
for catalog in catalogs:
    print(catalog.name)
# [/snippet:list_catalogs]


# [snippet:create_catalog]
def create_catalog_example() -> None:
    from unitycatalog_client import UnityCatalogClient

    client = UnityCatalogClient(base_url="http://localhost:8080")
    catalog = client.create_catalog("my_catalog", comment="My first catalog")
    print(f"Created: {catalog.name}")


# [/snippet:create_catalog]


# [snippet:get_catalog]
def get_catalog_example() -> None:
    from unitycatalog_client import UnityCatalogClient

    client = UnityCatalogClient(base_url="http://localhost:8080")
    catalog = client.catalog("my_catalog").get()
    print(f"Got: {catalog.name}")


# [/snippet:get_catalog]


# [snippet:update_catalog]
def update_catalog_example() -> None:
    from unitycatalog_client import UnityCatalogClient

    client = UnityCatalogClient(base_url="http://localhost:8080")
    catalog = client.catalog("my_catalog").update(comment="Updated comment")
    print(f"Updated: {catalog.name}")


# [/snippet:update_catalog]


# [snippet:delete_catalog]
def delete_catalog_example() -> None:
    from unitycatalog_client import UnityCatalogClient

    client = UnityCatalogClient(base_url="http://localhost:8080")
    client.catalog("my_catalog").delete()
    print("Deleted catalog")


# [/snippet:delete_catalog]

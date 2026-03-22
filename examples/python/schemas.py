# [snippet:list_schemas]
from unitycatalog_client import UnityCatalogClient

client = UnityCatalogClient(base_url="http://localhost:8080")
schemas = client.list_schemas("my_catalog")
for schema in schemas:
    print(schema.name)
# [/snippet:list_schemas]


# [snippet:create_schema]
def create_schema_example() -> None:
    from unitycatalog_client import UnityCatalogClient

    client = UnityCatalogClient(base_url="http://localhost:8080")
    schema = client.create_schema("my_schema", "my_catalog", comment="My first schema")
    print(f"Created: {schema.catalog_name}.{schema.name}")


# [/snippet:create_schema]


# [snippet:get_schema]
def get_schema_example() -> None:
    from unitycatalog_client import UnityCatalogClient

    client = UnityCatalogClient(base_url="http://localhost:8080")
    schema = client.schema("my_catalog", "my_schema").get()
    print(f"Got: {schema.catalog_name}.{schema.name}")


# [/snippet:get_schema]


# [snippet:update_schema]
def update_schema_example() -> None:
    from unitycatalog_client import UnityCatalogClient

    client = UnityCatalogClient(base_url="http://localhost:8080")
    schema = client.schema("my_catalog", "my_schema").update(comment="Updated comment")
    print(f"Updated: {schema.catalog_name}.{schema.name}")


# [/snippet:update_schema]


# [snippet:delete_schema]
def delete_schema_example() -> None:
    from unitycatalog_client import UnityCatalogClient

    client = UnityCatalogClient(base_url="http://localhost:8080")
    client.schema("my_catalog", "my_schema").delete()
    print("Deleted schema")


# [/snippet:delete_schema]

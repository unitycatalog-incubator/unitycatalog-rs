# [snippet:list_tables]
from unitycatalog_client import UnityCatalogClient

client = UnityCatalogClient(base_url="http://localhost:8080")
tables = client.list_tables("my_catalog", "my_schema")
for table in tables:
    print(table.name)
# [/snippet:list_tables]


# [snippet:create_table]
def create_table_example() -> None:
    from unitycatalog_client import DataSourceFormat, TableType, UnityCatalogClient

    client = UnityCatalogClient(base_url="http://localhost:8080")
    table = client.create_table(
        "my_table",
        "my_schema",
        "my_catalog",
        TableType.MANAGED,
        DataSourceFormat.DELTA,
        comment="My first table",
    )
    print(f"Created: {table.name}")


# [/snippet:create_table]


# [snippet:get_table]
def get_table_example() -> None:
    from unitycatalog_client import UnityCatalogClient

    client = UnityCatalogClient(base_url="http://localhost:8080")
    table = client.table("my_catalog.my_schema.my_table").get()
    print(f"Got: {table.name}")


# [/snippet:get_table]


# [snippet:delete_table]
def delete_table_example() -> None:
    from unitycatalog_client import UnityCatalogClient

    client = UnityCatalogClient(base_url="http://localhost:8080")
    client.table("my_catalog.my_schema.my_table").delete()
    print("Deleted table")


# [/snippet:delete_table]

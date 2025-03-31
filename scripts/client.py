from unitycatalog_client import (
    DataSourceFormat,
    TableType,
    UnityCatalogClient,
)

client = UnityCatalogClient("http://localhost:8080")

catalog_info = client.catalogs("dat").get()
print("catalog name:", catalog_info.name)

schema_info = client.catalogs("dat").schemas("dat").get()
print("schema name:", schema_info.full_name)

table_client = client.catalogs("dat").schemas("dat").tables("basic_append")
table_client.create(
    table_type=TableType.External,
    data_source_format=DataSourceFormat.Delta,
    storage_location="http://127.0.0.1:10000/devstoreaccount1/dat/basic_append/",
)

from unitycatalog_client import DataSourceFormat, TableType, UnityCatalogClient

client = UnityCatalogClient("http://localhost:8080")

catalog_info = client.catalogs("dat").get()
schema_info = client.catalogs("dat").schemas("dat").get()

external_location_info = client.external_locations("azurite-dat").get()
credential_info = client.credentials("azurite").get()

table_client = client.catalogs("dat").schemas("dat").tables("basic_append")
table_client.create(
    table_type=TableType.External,
    data_source_format=DataSourceFormat.Delta,
    storage_location="http://localhost:10000/devstoreaccount1/dat/basic_append/",
)

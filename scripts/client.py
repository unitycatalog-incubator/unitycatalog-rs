from unitycatalog_client import UnityCatalogClient


client = UnityCatalogClient("http://localhost:8080")
catalog_info = client.catalogs("dat").get()

print(catalog_info)

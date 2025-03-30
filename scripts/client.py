from unitycatalog_client import CreateCatalogRequest

catalog = CreateCatalogRequest(name="my_catalog", comment="This is my catalog")

print(catalog.name)

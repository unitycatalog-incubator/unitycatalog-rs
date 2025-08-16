import os

from unitycatalog_client import UnityCatalogClient

host = os.environ["DATABRICKS_HOST"]
client = UnityCatalogClient(
    base_url=f"{host}/api/2.1/unity-catalog/",
    token=os.environ["DATABRICKS_TOKEN"],
)

client.list_catalogs()

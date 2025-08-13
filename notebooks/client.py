import marimo

__generated_with = "0.14.17"
app = marimo.App()


@app.cell
def _():
    from unitycatalog_client import UnityCatalogClient
    import os
    return UnityCatalogClient, os


@app.cell
def _(UnityCatalogClient, os):
    host = os.environ["DATABRICKS_HOST"]
    # client = UnityCatalogClient(base_url=f"{host}/api/2.1/unity-catalog/", token=os.environ["DATABRICKS_TOKEN"])
    client = UnityCatalogClient(base_url="http://localhost:8080/api/2.1/unity-catalog/")
    return (client,)


@app.cell
def _(client):
    catalogs = client.list_catalogs()
    for catalog in catalogs:
        print(catalog.catalog_type)
    return


@app.cell
def _(client):
    share_client = client.shares("new_share")
    return (share_client,)


@app.cell
def _(share_client):
    share_client.get()
    return


@app.cell
def _(share_client):
    from unitycatalog_client import DataObjectUpdate, DataObject, DataObjectType, HistoryStatus, Action

    share = {
        # "name": "new_share",
        "updates": [
            DataObjectUpdate(**{
                "action": Action.Add,
                "data_object": DataObject(**{
                    "name": "dat.dat.all_primitive_types",
                    "data_object_type": DataObjectType.Table,
                    "shared_as": "dat.all_primitive_types",
                    "partitions": [],
                    "history_data_sharing_status": HistoryStatus.Disabled,
                    "enable_cdf": False
                })
            }),
            DataObjectUpdate(**{
                "action": Action.Add,
                "data_object": DataObject(**{
                    "name": "dat.dat.column_mapping",
                    "data_object_type": DataObjectType.Table,
                    "shared_as": "dat.column_mapping",
                    "partitions": [],
                    "history_data_sharing_status": HistoryStatus.Disabled,
                    "enable_cdf": False
                })
            })
        ]
    }

    share_client.update(**share)
    return


@app.cell(hide_code=True)
def _(mo):
    mo.md(r"""### Delta Sharing""")
    return


@app.cell
def _():
    from unitycatalog_client import SharingClient
    client_1 = SharingClient('http://localhost:8080')
    shares = client_1.list_shares()
    tables = client_1.list_share_tables(share=shares[0].name)
    client_1.get_table_metadata(share=tables[0].share, schema=tables[0].schema, name=tables[0].name)
    return client_1, tables


@app.cell
def _(client_1, tables):
    client_1.get_table_version(share=tables[0].share, schema=tables[0].schema, name=tables[0].name)
    return


@app.cell
def _():
    import marimo as mo
    return (mo,)


if __name__ == "__main__":
    app.run()

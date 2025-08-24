import marimo

__generated_with = "0.14.17"
app = marimo.App()


@app.cell
def _():
    from unitycatalog_client import SharingClient

    client = SharingClient("http://localhost:8080")

    shares = client.list_shares()
    tables = client.list_share_tables(share=shares[0].name)
    client.get_table_metadata(
        share=tables[0].share, schema=tables[0].schema, name=tables[0].name
    )
    return


if __name__ == "__main__":
    app.run()

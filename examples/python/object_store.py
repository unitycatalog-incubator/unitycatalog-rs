"""Walkthroughs for the `unitycatalog_client.obstore` integration.

These snippets are extracted into the docs site at build time.
"""

from __future__ import annotations

import os


# [snippet:object_store_client]
def build_client():
    """Construct a credential client that points at your UC endpoint.

    For production deployments, pull `base_url` and `token` from your
    secret store rather than environment variables.
    """
    from unitycatalog_client import TemporaryCredentialClient

    return TemporaryCredentialClient(
        base_url=os.environ.get("UC_BASE_URL", "http://localhost:8080/api/2.1/unity-catalog/"),
        token=os.environ.get("UC_TOKEN"),
    )


# [/snippet:object_store_client]


# [snippet:object_store_for_url]
def list_via_uc_url() -> None:
    """Address any Unity Catalog securable with a single `uc://` URL.

    The dispatcher routes:

    * ``uc:///Volumes/<c>/<s>/<v>[/<path>]`` -> volume credentials
    * ``uc:///Tables/<c>/<s>/<t>``           -> table credentials
    * ``s3://``, ``gs://``, ``abfss://``, ... -> path credentials
    """
    import obstore as obs
    from unitycatalog_client.obstore import store_for_url

    client = build_client()
    store = store_for_url(client, "uc:///Volumes/main/default/landing/raw/")

    for entry in obs.list(store).collect():
        print(entry["path"])


# [/snippet:object_store_for_url]


# [snippet:object_store_for_volume]
def read_volume_file() -> None:
    """Read a single file from a Unity Catalog volume."""
    import obstore as obs
    from unitycatalog_client.obstore import store_for_volume

    client = build_client()
    store = store_for_volume(client, "main.default.landing")
    payload = obs.get(store, "raw/2024/hello.txt").bytes()
    print(payload.to_bytes().decode())


# [/snippet:object_store_for_volume]


# [snippet:object_store_for_table]
def list_table_files() -> None:
    """List a Unity Catalog table's underlying data files."""
    import obstore as obs
    from unitycatalog_client.obstore import store_for_table

    client = build_client()
    store = store_for_table(client, "main.default.orders")
    for entry in obs.list(store).collect():
        print(entry["path"])


# [/snippet:object_store_for_table]


# [snippet:object_store_duckdb]
def query_with_duckdb() -> None:
    """Hand the vended credentials to DuckDB to query a UC volume.

    DuckDB's `httpfs` extension accepts standard S3-style configuration,
    so any credentials vended by UC for AWS or R2 work transparently.
    """
    from typing import Any

    import duckdb
    from unitycatalog_client import TemporaryCredentialClient

    client = TemporaryCredentialClient(
        base_url=os.environ.get("UC_BASE_URL", ""),
        token=os.environ.get("UC_TOKEN"),
    )
    credential, _ = client.temporary_volume_credential("main.default.landing", "read")
    # `TemporaryCredential` straddles a flat and a nested layout — use a
    # helper to pull the AWS credentials out regardless.
    aws: Any = getattr(credential, "aws_temp_credentials", None) or getattr(
        getattr(credential, "credentials", None), "aws_temp_credentials", None
    )
    assert aws is not None, "expected AWS credentials in the vended response"

    con = duckdb.connect()
    con.sql("INSTALL httpfs; LOAD httpfs;")
    con.sql(
        f"""
        CREATE OR REPLACE SECRET uc_landing (
            TYPE S3,
            KEY_ID '{aws.access_key_id}',
            SECRET '{aws.secret_access_key}',
            SESSION_TOKEN '{aws.session_token}'
        );
        """
    )
    print(con.sql(f"SELECT * FROM read_parquet('{credential.url}/*.parquet')").df())


# [/snippet:object_store_duckdb]

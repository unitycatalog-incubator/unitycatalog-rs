"""Build native obstore stores from Unity Catalog credential vending."""

from __future__ import annotations

from typing import Any, TYPE_CHECKING
from urllib.parse import urlparse

from obstore.store import AzureStore, GCSStore, S3Store

from .._client import parse_uc_url as _parse_uc_url
from ._provider import (
    Operation,
    UnityAwsCredentialProvider,
    UnityAzureCredentialProvider,
    UnityGcsCredentialProvider,
    _extract,
    _Securable,
)

if TYPE_CHECKING:
    from .. import TemporaryCredential, TemporaryCredentialClient


#: Union of the native obstore stores this module returns. Re-exported
#: so callers can annotate their own variables, e.g.::
#:
#:     store: UCStore = store_for_url(client, "uc:///Volumes/...")
UCStore = S3Store | GCSStore | AzureStore


def store_for_url(
    client: TemporaryCredentialClient,
    url: str,
    *,
    operation: Operation = "read",
    **store_kwargs: Any,
) -> UCStore:
    """Build an obstore store for any supported Unity Catalog URL.

    URL parsing is delegated to the Rust implementation of
    :class:`~unitycatalog_common.UCReference` so the URL grammar (kinds,
    schemes, case-insensitivity, percent-decoding, error messages) stays
    bit-for-bit identical to the one used by ``unitycatalog-object-store``
    in Rust.

    Supported URL shapes:

    * ``uc:///Volumes/<catalog>/<schema>/<volume>[/<path>]``
    * ``uc:///Tables/<catalog>/<schema>/<table>``
    * ``s3://``, ``gs://``, ``abfss://``, ``r2://``, ... — raw cloud URLs
    * ``vol+dbfs:/Volumes/<catalog>/<schema>/<volume>[/<path>]`` — alias

    Returns the matching native ``obstore`` store
    (:class:`~obstore.store.S3Store` / :class:`~obstore.store.GCSStore` /
    :class:`~obstore.store.AzureStore`) with a credential provider that
    refreshes through ``client``.

    Extra ``**store_kwargs`` are forwarded to the underlying obstore
    store constructor (e.g. ``region="us-west-2"`` for S3).
    """
    kind, payload = _parse_uc_url(url)
    if kind == "volume":
        path = payload["path"]
        return store_for_volume(
            client,
            f"{payload['catalog']}.{payload['schema']}.{payload['volume']}",
            sub_path=path or None,
            operation=operation,
            **store_kwargs,
        )
    if kind == "table":
        return store_for_table(
            client,
            f"{payload['catalog']}.{payload['schema']}.{payload['table']}",
            operation=operation,
            **store_kwargs,
        )
    if kind == "path":
        return store_for_path(client, payload["url"], operation=operation, **store_kwargs)
    raise AssertionError(f"unexpected UCReference kind {kind!r}")


def store_for_volume(
    client: TemporaryCredentialClient,
    volume: str,
    *,
    sub_path: str | None = None,
    operation: Operation = "read",
    **store_kwargs: Any,
) -> UCStore:
    """Build a refresh-aware obstore store rooted at ``volume``.

    The ``volume`` argument is the three-level ``catalog.schema.volume``
    name. When ``sub_path`` is provided, the returned store is further
    prefixed to that sub-path.
    """
    credential, _ = client.temporary_volume_credential(volume, operation)
    securable = _Securable("volume", volume, operation)
    return _build_store(credential, securable, client, sub_path, store_kwargs)


def store_for_table(
    client: TemporaryCredentialClient,
    table: str,
    *,
    operation: Operation = "read",
    **store_kwargs: Any,
) -> UCStore:
    """Build a refresh-aware obstore store rooted at ``table``'s storage location."""
    credential, _ = client.temporary_table_credential(table, operation)
    securable = _Securable("table", table, operation)
    return _build_store(credential, securable, client, None, store_kwargs)


def store_for_path(
    client: TemporaryCredentialClient,
    url: str,
    *,
    operation: Operation = "read",
    **store_kwargs: Any,
) -> UCStore:
    """Build a refresh-aware obstore store for a raw cloud URL."""
    credential, _ = client.temporary_path_credential(url, operation)
    securable = _Securable("path", url, operation)
    return _build_store(credential, securable, client, None, store_kwargs)


def _build_store(
    credential: TemporaryCredential,
    securable: _Securable,
    client: TemporaryCredentialClient,
    sub_path: str | None,
    store_kwargs: dict[str, Any],
) -> UCStore:
    """Dispatch to the cloud-specific obstore store based on the vended credential."""
    cloud_url = credential.url

    if _extract(credential, "aws_temp_credentials", "r2_temp_credentials") is not None:
        aws_provider = UnityAwsCredentialProvider(client, securable)
        bucket, prefix = _parse_s3_url(cloud_url)
        if sub_path:
            prefix = _join_prefix(prefix, sub_path)
        return S3Store(
            bucket, prefix=prefix or None, credential_provider=aws_provider, **store_kwargs
        )

    if _extract(credential, "azure_aad", "azure_user_delegation_sas") is not None:
        azure_provider = UnityAzureCredentialProvider(client, securable)
        container, account, prefix = _parse_azure_url(cloud_url)
        if sub_path:
            prefix = _join_prefix(prefix, sub_path)
        return AzureStore(
            container,
            account_name=account,
            prefix=prefix or None,
            credential_provider=azure_provider,
            **store_kwargs,
        )

    if _extract(credential, "gcp_oauth_token") is not None:
        gcs_provider = UnityGcsCredentialProvider(client, securable)
        bucket, prefix = _parse_gcs_url(cloud_url)
        if sub_path:
            prefix = _join_prefix(prefix, sub_path)
        return GCSStore(
            bucket, prefix=prefix or None, credential_provider=gcs_provider, **store_kwargs
        )

    raise ValueError(
        "Unity Catalog returned a credential whose cloud type is not supported by obstore"
    )


def _parse_s3_url(url: str) -> tuple[str, str]:
    """Split ``s3://bucket/key/prefix`` into ``(bucket, prefix)``."""
    parsed = urlparse(url)
    return parsed.netloc, parsed.path.lstrip("/")


def _parse_gcs_url(url: str) -> tuple[str, str]:
    """Split ``gs://bucket/key/prefix`` into ``(bucket, prefix)``."""
    parsed = urlparse(url)
    return parsed.netloc, parsed.path.lstrip("/")


def _parse_azure_url(url: str) -> tuple[str, str, str]:
    """Split an Azure URL into ``(container, account, prefix)``.

    Accepts both ``abfss://container@account.dfs.core.windows.net/path``
    (UC-native) and ``az://account/container/path`` forms.
    """
    parsed = urlparse(url)
    if parsed.scheme in {"abfs", "abfss"}:
        container, _, host = parsed.netloc.partition("@")
        account = host.split(".", 1)[0]
        return container, account, parsed.path.lstrip("/")
    if parsed.scheme in {"az", "azure"}:
        account = parsed.netloc
        parts = parsed.path.lstrip("/").split("/", 1)
        container = parts[0]
        prefix = parts[1] if len(parts) > 1 else ""
        return container, account, prefix
    raise ValueError(f"not an Azure URL: {url!r}")


def _join_prefix(*parts: str) -> str:
    """Joins one or more URL prefix fragments with a single ``/``."""
    cleaned = [p.strip("/") for p in parts if p]
    return "/".join(cleaned)

from __future__ import annotations

from typing import Any, Literal

from obstore.store import AzureStore, GCSStore, S3Store

from .. import (
    TemporaryCredential as TemporaryCredential,
    TemporaryCredentialClient as TemporaryCredentialClient,
)

Operation = Literal["read", "read_write"]

#: Union of the native obstore stores returned by ``store_for_*``.
UCStore = S3Store | GCSStore | AzureStore

class UnityAwsCredentialProvider:
    def __init__(self, client: TemporaryCredentialClient, securable: Any) -> None: ...
    def __call__(self) -> dict[str, Any]: ...

class UnityAzureCredentialProvider:
    def __init__(self, client: TemporaryCredentialClient, securable: Any) -> None: ...
    def __call__(self) -> dict[str, Any]: ...

class UnityGcsCredentialProvider:
    def __init__(self, client: TemporaryCredentialClient, securable: Any) -> None: ...
    def __call__(self) -> dict[str, Any]: ...

def store_for_url(
    client: TemporaryCredentialClient,
    url: str,
    *,
    operation: Operation = "read",
    **store_kwargs: Any,
) -> UCStore:
    """Build a native obstore store for any supported UC URL."""

def store_for_volume(
    client: TemporaryCredentialClient,
    volume: str,
    *,
    sub_path: str | None = None,
    operation: Operation = "read",
    **store_kwargs: Any,
) -> UCStore:
    """Build a native obstore store rooted at a UC volume."""

def store_for_table(
    client: TemporaryCredentialClient,
    table: str,
    *,
    operation: Operation = "read",
    **store_kwargs: Any,
) -> UCStore:
    """Build a native obstore store rooted at a UC table's storage location."""

def store_for_path(
    client: TemporaryCredentialClient,
    url: str,
    *,
    operation: Operation = "read",
    **store_kwargs: Any,
) -> UCStore:
    """Build a native obstore store for a raw cloud URL governed by UC."""

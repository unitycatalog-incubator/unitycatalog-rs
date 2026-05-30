"""Obstore integration for Unity Catalog.

Construct native `obstore <https://developmentseed.org/obstore/>`_ store
objects (``S3Store``, ``GCSStore``, ``AzureStore``) whose credentials are
vended by Unity Catalog and transparently refreshed.

Install the optional dependency::

    pip install "unitycatalog-client[obstore]"

Then::

    from unitycatalog_client import TemporaryCredentialClient
    from unitycatalog_client.obstore import store_for_url

    creds = TemporaryCredentialClient(base_url=..., token=...)
    store = store_for_url(creds, "uc:///Volumes/main/default/landing/raw/")

    # `store` is a native obstore S3Store / GCSStore / AzureStore.
    import obstore as obs
    files = obs.list(store).collect()

The returned store carries an ``obstore`` credential provider that calls
back into Unity Catalog whenever the existing token approaches expiry.
"""

from __future__ import annotations

from ._dispatch import (
    UCStore as UCStore,
    store_for_path as store_for_path,
    store_for_table as store_for_table,
    store_for_url as store_for_url,
    store_for_volume as store_for_volume,
)
from ._provider import (
    UnityAwsCredentialProvider as UnityAwsCredentialProvider,
    UnityAzureCredentialProvider as UnityAzureCredentialProvider,
    UnityGcsCredentialProvider as UnityGcsCredentialProvider,
)

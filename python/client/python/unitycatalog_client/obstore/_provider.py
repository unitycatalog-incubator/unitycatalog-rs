"""Refresh-aware ``obstore`` credential providers backed by Unity Catalog."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Any, Literal, TYPE_CHECKING

if TYPE_CHECKING:
    # `TemporaryCredential` lives in the package-level type stub
    # (the Rust extension doesn't re-export the class explicitly), so
    # we pull both names from the parent package for type checking.
    from .. import TemporaryCredential, TemporaryCredentialClient


Operation = Literal["read", "read_write"]


@dataclass(frozen=True)
class _Securable:
    """Describes the UC securable + operation a credential is bound to.

    Re-vends always hit the same endpoint with the same arguments so a
    refresh can never silently widen the credential's privileges.
    """

    kind: Literal["volume", "table", "path"]
    name: str
    operation: Operation


def _expiration_to_iso(expiration_time_ms: int) -> str:
    """Convert UC's ms-epoch expiration_time into an RFC 3339 string.

    obstore expects an ISO 8601 / RFC 3339 timestamp in the ``expires_at``
    field of a credential provider's return value.
    """
    return (
        datetime.fromtimestamp(expiration_time_ms / 1000, tz=timezone.utc)
        .isoformat()
        .replace("+00:00", "Z")
    )


class _BaseProvider:
    """Common refresh logic shared by all cloud-specific providers."""

    def __init__(
        self,
        client: TemporaryCredentialClient,
        securable: _Securable,
    ) -> None:
        self._client = client
        self._securable = securable

    def _vend(self) -> TemporaryCredential:
        sec = self._securable
        if sec.kind == "volume":
            credential, _ = self._client.temporary_volume_credential(sec.name, sec.operation)
        elif sec.kind == "table":
            credential, _ = self._client.temporary_table_credential(sec.name, sec.operation)
        elif sec.kind == "path":
            credential, _ = self._client.temporary_path_credential(sec.name, sec.operation)
        else:  # pragma: no cover - defensive
            raise ValueError(f"unknown securable kind: {sec.kind!r}")
        return credential


def _extract(credential: TemporaryCredential, *names: str) -> Any:
    """Return the first matching attribute on the credential or on a
    nested ``credentials`` field, whichever the runtime provides.

    The Python surface for ``TemporaryCredential`` straddles two shapes:
    the JSON-style flat one (used by older codegen output) and the
    proto-derived nested one (where credentials live under a ``credentials``
    oneof attribute). This helper papers over the difference so the
    obstore providers work either way.
    """
    for name in names:
        if hasattr(credential, name) and getattr(credential, name) is not None:
            return getattr(credential, name)
        inner = getattr(credential, "credentials", None)
        if inner is not None and hasattr(inner, name) and getattr(inner, name) is not None:
            return getattr(inner, name)
    return None


class UnityAwsCredentialProvider(_BaseProvider):
    """Vends AWS temporary credentials in the shape obstore expects.

    Compatible with both ``obstore.store.S3Store`` and the R2 path inside
    the same store builder. obstore calls this provider whenever the
    cached credential is within its refresh window.
    """

    def __call__(self) -> dict[str, Any]:
        credential = self._vend()
        aws = _extract(credential, "aws_temp_credentials", "r2_temp_credentials")
        if aws is None:
            raise ValueError("Unity Catalog did not return an AWS/R2 credential")
        return {
            "access_key_id": aws.access_key_id,
            "secret_access_key": aws.secret_access_key,
            "token": aws.session_token,
            "expires_at": _expiration_to_iso(credential.expiration_time),
        }


class UnityAzureCredentialProvider(_BaseProvider):
    """Vends Azure temporary credentials in the shape obstore expects.

    Returns either a bearer token (AAD) or a SAS token depending on what
    Unity Catalog issued.
    """

    def __call__(self) -> dict[str, Any]:
        credential = self._vend()
        aad = _extract(credential, "azure_aad")
        if aad is not None:
            return {
                "token": aad.aad_token,
                "expires_at": _expiration_to_iso(credential.expiration_time),
            }
        sas = _extract(credential, "azure_user_delegation_sas")
        if sas is not None:
            return {
                "sas_token": sas.sas_token,
                "expires_at": _expiration_to_iso(credential.expiration_time),
            }
        raise ValueError("Unity Catalog did not return an Azure credential")


class UnityGcsCredentialProvider(_BaseProvider):
    """Vends GCS temporary credentials in the shape obstore expects."""

    def __call__(self) -> dict[str, Any]:
        credential = self._vend()
        gcs = _extract(credential, "gcp_oauth_token")
        if gcs is None:
            raise ValueError("Unity Catalog did not return a GCS credential")
        return {
            "token": gcs.oauth_token,
            "expires_at": _expiration_to_iso(credential.expiration_time),
        }

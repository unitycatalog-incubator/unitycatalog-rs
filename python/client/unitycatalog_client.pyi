from __future__ import annotations

import enum

# Import Python class for runtime context
from typing import Literal

class CatalogInfo:
    id: str | None
    name: str
    """The name of the catalog."""

    owner: str | None
    """Username of current owner of catalog."""

    comment: str | None
    """User-provided free-form text description."""

    properties: dict | None
    """A map of key-value properties attached to the securable."""

    storage_root: str | None

    provider_name: str | None
    """The name of delta sharing provider."""

    share_name: str | None
    """The name of the share under the share provider."""

    catalog_type: Literal[
        "MANAGED_CATALOG", "DELTASHARING_CATALOG", "SYSTEM_CATALOG", "EXTERNAL_CATALOG"
    ]
    """The type of catalog."""

    created_at: int | None
    created_by: str | None
    updated_at: int | None
    updated_by: str | None

    browse_only: bool | None
    """Indicates whether the principal is limited to retrieving metadata for the associated object
    through the BROWSE privilege when include_browse is enabled in the request.
    """

class SchemaInfo:
    name: str
    catalog_name: str
    comment: str | None
    properties: dict | None
    full_name: str | None
    owner: str | None
    created_at: int | None
    created_by: str | None
    updated_at: int | None
    updated_by: str | None
    schema_id: str | None

class ColumnTypeName(enum.Enum):
    Unspecified = 0
    Boolean = 1
    Byte = 2
    Short = 3
    Int = 4
    Long = 5
    Float = 6
    Double = 7
    Date = 8
    Timestamp = 9
    String = 10
    Binary = 11
    Decimal = 12
    Interval = 13
    Array = 14
    Struct = 15
    Map = 16
    Char = 17
    Null = 18
    UserDefinedType = 19
    TimestampNtz = 20
    Variant = 21
    TableType = 22

class DataSourceFormat(enum.Enum):
    Unspecified = 0
    Delta = 1
    Iceberg = 2
    Hudi = 3
    Parquet = 4
    Csv = 5
    Json = 6
    Orc = 7
    Avro = 8
    Text = 9
    UnityCatalog = 10
    Deltasharing = 11

class TableType(enum.Enum):
    Unspecified = 0
    Managed = 1
    External = 2

class ColumnInfo:
    name: str
    type_text: str
    type_json: str
    type_name: ColumnTypeName
    type_precision: int | None
    type_scale: int | None
    type_interval_type: str | None
    position: int | None
    comment: str | None
    nullable: bool | None
    partition_index: int | None
    column_id: str | None

class TableSummary:
    full_name: str
    table_type: TableType

class TableInfo:
    name: str
    schema_name: str
    catalog_name: str
    table_type: TableType
    data_source_format: DataSourceFormat
    columns: list[ColumnInfo]
    storage_location: str | None
    owner: str | None
    comment: str | None
    properties: dict | None
    storage_credential_name: str | None
    full_name: str | None
    created_at: int | None
    created_by: str | None
    updated_at: int | None
    updated_by: str | None
    deleted_at: int | None
    table_id: str | None

class Purpose(enum.Enum):
    Unspecified = 0
    Storage = 1
    Service = 2

class AuthenticationType(enum.Enum):
    Unspecified = 0
    Token = 1
    OauthClientCredentials = 2

class AzureServicePrincipal:
    directory_id: str
    application_id: str
    client_secret: str | None
    federated_token_file: str | None

    def __init__(
        self,
        directory_id: str,
        application_id: str,
        client_secret: str | None = None,
        federated_token_file: str | None = None,
    ) -> None: ...

class AzureManagedIdentity:
    object_id: str | None
    application_id: str | None
    msi_resource_id: str | None

    def __init__(
        self,
        object_id: str | None = None,
        application_id: str | None = None,
        msi_resource_id: str | None = None,
    ) -> None: ...

class AzureStorageKey:
    account_name: str
    account_key: str

    def __init__(self, account_name: str, account_key: str) -> None: ...

class Credential:
    id: str
    name: str
    purpose: Purpose
    read_only: bool
    comment: str | None
    owner: str | None
    created_at: int | None
    created_by: str | None
    updated_at: int | None
    updated_by: str | None
    used_for_managed_storage: bool
    full_name: str | None
    azure_service_principal: AzureServicePrincipal | None
    azure_managed_identity: AzureManagedIdentity | None
    azure_storage_key: AzureStorageKey | None

class ExternalLocationInfo:
    name: str
    url: str
    credential_name: str
    read_only: bool
    comment: str | None
    owner: str | None
    credential_id: str
    created_at: int | None
    created_by: str | None
    updated_at: int | None
    updated_by: str | None
    browse_only: bool | None
    external_location_id: str | None

class DataObjectType(enum.Enum):
    Unspecified = 0
    Table = 1
    Schema = 2

class HistoryStatus(enum.Enum):
    Disabled = 0
    Enabled = 1

class Action(enum.Enum):
    Unspecified = 0
    Add = 1
    Remove = 2
    Update = 3

class DataObject:
    name: str
    data_object_type: DataObjectType
    added_at: int | None
    added_by: str | None
    comment: str | None
    shared_as: str | None
    partitions: list[str]
    enable_cdf: bool | None
    history_data_sharing_status: HistoryStatus | None
    start_version: int | None

    def __init__(
        self,
        name: str,
        data_object_type: DataObjectType,
        added_at: int | None = None,
        added_by: str | None = None,
        comment: str | None = None,
        shared_as: str | None = None,
        partitions: list[str] | None = None,
        enable_cdf: bool | None = None,
        history_data_sharing_status: HistoryStatus | None = None,
        start_version: int | None = None,
    ) -> None: ...

class DataObjectUpdate:
    action: str
    data_object: DataObject

    def __init__(self, action: str, data_object: DataObject) -> None: ...

class ShareInfo:
    id: str | None
    name: str
    owner: str | None
    comment: str | None
    data_objects: list[DataObject]
    created_at: int | None
    created_by: str | None
    updated_at: int | None
    updated_by: str | None

class RecipientInfo:
    id: str
    name: str
    comment: str | None
    owner: str | None
    created_at: int | None
    created_by: str | None
    updated_at: int | None
    updated_by: str | None

class Share:
    id: str | None
    name: str

class SharingSchema:
    id: str | None
    name: str
    share: str

class SharingTable:
    id: str | None
    name: str
    schema: str
    share: str
    share_id: str | None

class VolumeType:
    UNSPECIFIED = 0
    EXTERNAL = 1
    MANAGED = 2

class Volume:
    """Information about a Unity Catalog volume."""

    name: str
    catalog_name: str
    schema_name: str
    full_name: str
    storage_location: str | None
    volume_id: str
    volume_type: int
    owner: str | None
    comment: str | None
    created_at: int | None
    created_by: str | None
    updated_at: int | None
    updated_by: str | None

class TemporaryCredential:
    """Represents temporary credentials for accessing storage resources."""

    expiration_time: int
    url: str

class Protocol:
    def min_reader_version(self) -> int: ...
    def min_writer_version(self) -> int | None: ...

class Metadata:
    def partition_columns(self) -> list[str]: ...
    def configuration(self) -> dict[str, str]: ...

class SharingClient:
    def __init__(
        self,
        base_url: str,
        token: str | None = None,
        prefix: str | None = None,
    ) -> None:
        """
        Delta Sharing client.

        Args:
            base_url: The base URL of the server.
            token: Personal Access Token for the Databricks CLI.
            prefix: The path prefix where the Delta Sharing API is mounted.
        """

    def list_shares(self, max_results: int | None = None) -> list[Share]: ...
    def get_share(self, name: str) -> Share: ...
    def list_share_schemas(
        self, share: str, max_results: int | None = None
    ) -> list[SharingSchema]: ...
    def list_share_tables(
        self, share: str, max_results: int | None = None
    ) -> list[SharingTable]: ...
    def list_schema_tables(
        self, share: str, schema: str, max_results: int | None = None
    ) -> list[SharingTable]: ...
    def get_table_version(
        self, share: str, schema: str, table: str, starting_timestamp: str | None = None
    ) -> int: ...
    def get_table_metadata(
        self, share: str, schema: str, table: str
    ) -> tuple[Protocol, Metadata]: ...

class TableClient:
    def get(
        self,
        include_delta_metadata: bool | None = None,
        include_browse: bool | None = None,
        include_manifest_capabilities: bool | None = None,
    ) -> TableInfo: ...
    def delete(self) -> None: ...

class SchemaClient:
    def get(self) -> SchemaInfo: ...
    def table(self, name: str) -> TableClient:
        """Get a TableClient for the specified table in this schema.

        Args:
            name: The name of the table.

        Returns:
            A TableClient object.
        """
    def create_table(
        self,
        table_name: str,
        table_type: TableType,
        data_source_format: DataSourceFormat,
        columns: list[ColumnInfo],
        storage_location: str | None = None,
        comment: str | None = None,
        properties: dict[str, str] | None = None,
    ) -> TableInfo:
        """Create a new table in this schema.

        Args:
            table_name: The name of the table.
            table_type: The type of table (managed or external).
            data_source_format: The data source format.
            columns: List of column definitions.
            storage_location: Storage location for external tables.
            comment: User-provided free-form text description.
            properties: A map of key-value properties attached to the table.

        Returns:
            The created TableInfo object.
        """
    def update(
        self,
        new_name: str | None = None,
        comment: str | None = None,
        properties: dict[str, str] | None = None,
    ) -> SchemaInfo: ...
    def delete(self, force: bool | None = None) -> None: ...

class CatalogClient:
    def get(self) -> CatalogInfo: ...
    def create_schema(
        self,
        schema_name: str,
        comment: str | None = None,
    ) -> SchemaInfo:
        """Create a new schema in this catalog.

        Args:
            schema_name: The name of the schema.
            comment: User-provided free-form text description.

        Returns:
            The created SchemaInfo object.
        """
    def update(
        self,
        new_name: str | None = None,
        comment: str | None = None,
        owner: str | None = None,
        properties: dict[str, str] | None = None,
    ) -> CatalogInfo: ...
    def delete(self, force: bool | None = None) -> None: ...

class CredentialClient:
    def get(self) -> Credential: ...
    def update(
        self,
        new_name: str | None = None,
        comment: str | None = None,
        owner: str | None = None,
        read_only: bool | None = None,
        skip_validation: bool | None = None,
        force: bool | None = None,
        credential: object | None = None,
    ) -> Credential: ...
    def delete(self) -> None: ...

class ExternalLocationClient:
    def get(self) -> ExternalLocationInfo: ...
    def update(
        self,
        new_name: str | None = None,
        url: str | None = None,
        credential_name: str | None = None,
        comment: str | None = None,
        owner: str | None = None,
        read_only: bool | None = None,
        skip_validation: bool | None = None,
        force: bool | None = None,
    ) -> ExternalLocationInfo: ...
    def delete(self, force: bool | None = None) -> None: ...

class RecipientClient:
    def get(self) -> RecipientInfo: ...
    def update(
        self,
        new_name: str | None = None,
        comment: str | None = None,
        owner: str | None = None,
        properties: dict[str, str] | None = None,
        expiration_time: int | None = None,
    ) -> RecipientInfo: ...
    def delete(self) -> None: ...

class ShareClient:
    def get(self, include_shared_data: bool | None = None) -> ShareInfo: ...
    def update(
        self,
        new_name: str | None = None,
        updates: list[DataObjectUpdate] | None = None,
        comment: str | None = None,
        owner: str | None = None,
    ) -> ShareInfo: ...
    def delete(self) -> None: ...

class PyUnityCatalogClient:
    def __init__(self, base_url: str, token: str | None = None) -> None:
        """
        Unity Catalog client.

        Args:
            base_url: The base URL of the Unity Catalog API.
            token: Personal Access Token for the Databricks CLI.
        """

    # Catalog methods
    def list_catalogs(self, max_results: int | None = None) -> list[CatalogInfo]:
        """Gets an array of catalogs in the metastore.

        If the caller is the metastore admin, all catalogs will be retrieved.
        Otherwise, only catalogs owned by the caller (or for which the caller has the `USE_CATALOG`
        privilege) will be retrieved. There is no guarantee of a specific ordering of the elements
        in the array.

        Args:
            max_results: The maximum number of catalogs to return.

        Returns:
            A list of CatalogInfo objects.
        """

    def catalog(self, name: str) -> CatalogClient: ...

    # Schema methods
    def list_schemas(
        self, catalog_name: str, max_results: int | None = None
    ) -> list[SchemaInfo]: ...
    def schema(self, catalog_name: str, schema_name: str) -> SchemaClient: ...

    # Table methods
    def list_tables(
        self,
        catalog_name: str,
        schema_name: str,
        max_results: int | None = None,
        include_delta_metadata: bool | None = None,
        omit_columns: bool | None = None,
        omit_properties: bool | None = None,
        omit_username: bool | None = None,
    ) -> list[TableInfo]: ...
    def table(self, full_name: str) -> TableClient: ...

    # Share methods
    def list_shares(self, max_results: int | None = None) -> list[ShareInfo]: ...
    def share(self, name: str) -> ShareClient: ...

    # Recipient methods
    def list_recipients(
        self, max_results: int | None = None
    ) -> list[RecipientInfo]: ...
    def recipient(self, name: str) -> RecipientClient: ...

    # Credential methods
    def list_credentials(
        self, purpose: Purpose | None = None, max_results: int | None = None
    ) -> list[Credential]: ...
    def credential(self, name: str) -> CredentialClient: ...

    # External location methods
    def list_external_locations(
        self, max_results: int | None = None
    ) -> list[ExternalLocationInfo]: ...
    def external_location(self, name: str) -> ExternalLocationClient: ...

    # Create methods
    def create_catalog(
        self,
        name: str,
        storage_root: str | None = None,
        comment: str | None = None,
        properties: dict[str, str] | None = None,
    ) -> CatalogInfo:
        """Create a new managed catalog.

        Args:
            name: The name of the catalog.
            storage_root: Storage root for the catalog.
            comment: User-provided free-form text description.
            properties: A map of key-value properties attached to the catalog.

        Returns:
            The created CatalogInfo object.
        """

    def create_sharing_catalog(
        self,
        name: str,
        provider_name: str,
        share_name: str,
        comment: str | None = None,
        properties: dict[str, str] | None = None,
    ) -> CatalogInfo:
        """Create a new sharing catalog.

        Args:
            name: The name of the catalog.
            provider_name: The name of the delta sharing provider.
            share_name: The name of the share under the share provider.
            comment: User-provided free-form text description.
            properties: A map of key-value properties attached to the catalog.

        Returns:
            The created CatalogInfo object.
        """

    def create_schema(
        self,
        catalog_name: str,
        schema_name: str,
        comment: str | None = None,
    ) -> SchemaInfo:
        """Create a new schema.

        Args:
            catalog_name: The name of the catalog.
            schema_name: The name of the schema.
            comment: User-provided free-form text description.

        Returns:
            The created SchemaInfo object.
        """

    def create_share(
        self,
        name: str,
        comment: str | None = None,
    ) -> ShareInfo:
        """Create a new share.

        Args:
            name: The name of the share.
            comment: User-provided free-form text description.

        Returns:
            The created ShareInfo object.
        """

    def create_recipient(
        self,
        name: str,
        authentication_type: AuthenticationType,
        comment: str | None = None,
    ) -> RecipientInfo:
        """Create a new recipient.

        Args:
            name: The name of the recipient.
            authentication_type: The authentication type for the recipient.
            comment: User-provided free-form text description.

        Returns:
            The created RecipientInfo object.
        """

    def create_credential(
        self,
        name: str,
        purpose: Purpose,
        comment: str | None = None,
    ) -> Credential:
        """Create a new credential.

        Args:
            name: The name of the credential.
            purpose: The purpose of the credential.
            comment: User-provided free-form text description.

        Returns:
            The created Credential object.
        """

    def create_external_location(
        self,
        name: str,
        url: str,
        credential_name: str,
        comment: str | None = None,
    ) -> ExternalLocationInfo:
        """Create a new external location.

        Args:
            name: The name of the external location.
            url: The URL of the external location.
            credential_name: The name of the credential to use.
            comment: User-provided free-form text description.

        Returns:
            The created ExternalLocationInfo object.
        """
    def temporary_credentials(self) -> TemporaryCredentialClient:
        """Get a client for managing temporary credentials.

        Returns:
            A TemporaryCredentialClient instance.
        """

    # Volume methods
    def list_volumes(
        self,
        catalog_name: str,
        schema_name: str,
        max_results: int | None = None,
        include_browse: bool | None = None,
    ) -> list[Volume]: ...
    def volume(
        self, catalog_name: str, schema_name: str, volume_name: str
    ) -> VolumeClient: ...
    def volume_from_full_name(self, full_name: str) -> VolumeClient: ...
    def create_volume(
        self,
        catalog_name: str,
        schema_name: str,
        volume_name: str,
        volume_type: VolumeType,
        storage_location: str | None = None,
        comment: str | None = None,
    ) -> Volume: ...

class VolumeClient:
    def get(self, include_browse: bool | None = None) -> Volume: ...
    def update(
        self,
        new_name: str | None = None,
        comment: str | None = None,
        owner: str | None = None,
        include_browse: bool | None = None,
    ) -> Volume: ...
    def delete(self) -> None: ...

class TemporaryCredentialClient:
    """Client for managing temporary credentials for tables and paths."""

    def temporary_table_credential(
        self,
        table: str,
        operation: Literal["read", "read_write"],
    ) -> tuple[TemporaryCredential, str]:
        """Generate temporary credentials for accessing a table.

        Args:
            table: The full name of the table.
            operation: The operation type ('read' or 'read_write').

        Returns:
            A tuple containing the temporary credential and table UUID.
        """

    def temporary_path_credential(
        self,
        path: str,
        operation: Literal["read", "read_write", "create_table"],
        dry_run: bool | None = None,
    ) -> tuple[TemporaryCredential, str]:
        """Generate temporary credentials for accessing a storage path.

        Args:
            path: The storage path URL.
            operation: The operation type ('read', 'read_write', or 'create_table').
            dry_run: Whether this is a dry run operation.

        Returns:
            A tuple containing the temporary credential and resolved URL.
        """

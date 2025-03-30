import enum

class CatalogInfo:
    id: str | None
    name: str
    owner: str | None
    comment: str | None
    properties: dict | None
    storage_root: str | None
    provider_name: str | None
    share_name: str | None
    catalog_type: int | None
    created_at: int | None
    created_by: str | None
    updated_at: int | None
    updated_by: str | None
    browse_only: bool | None

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
    COLUMN_TYPE_NAME_UNSPECIFIED = 0
    BOOLEAN = 1
    BYTE = 2
    SHORT = 3
    INT = 4
    LONG = 5
    FLOAT = 6
    DOUBLE = 7
    DATE = 8
    TIMESTAMP = 9
    STRING = 10
    BINARY = 11
    DECIMAL = 12
    INTERVAL = 13
    ARRAY = 14
    STRUCT = 15
    MAP = 16
    CHAR = 17
    NULL = 18
    USER_DEFINED_TYPE = 19
    TIMESTAMP_NTZ = 20
    VARIANT = 21
    TABLE_TYPE = 22

class DataSourceFormat(enum.Enum):
    DATA_SOURCE_FORMAT_UNSPECIFIED = 0
    DELTA = 1
    ICEBERG = 2
    HUDI = 3
    PARQUET = 4
    CSV = 5
    JSON = 6
    ORC = 7
    AVRO = 8
    TEXT = 9
    UNITY_CATALOG = 10
    DELTASHARING = 11

class TableType(enum.Enum):
    TABLE_TYPE_UNSPECIFIED = 0
    MANAGED = 1
    EXTERNAL = 2

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
    PURPOSE_UNSPECIFIED = 0
    STORAGE = 1
    SERVICE = 2

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

class CredentialInfo:
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
    DATA_OBJECT_TYPE_UNSPECIFIED = 0
    TABLE = 1
    SCHEMA = 2

class HistoryStatus(enum.Enum):
    DISABLED = 0
    ENABLED = 1

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
        partitions: list[str] = [],
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

class TableClient:
    def get(self, include_delta_metadata: bool | None = None) -> TableInfo: ...
    def create(
        self,
        table_type: TableType,
        data_source_format: DataSourceFormat,
        columns: list[ColumnInfo],
        storage_location: str | None = None,
        comment: str | None = None,
        properties: dict | None = None,
    ) -> TableInfo: ...

class SchemaClient:
    def get(self) -> SchemaInfo: ...
    def create(self, name: str) -> SchemaInfo: ...
    def tables(self, name: str) -> TableClient: ...

class CatalogClient:
    def schemas(self, name: str) -> SchemaClient: ...
    def get(self) -> CatalogInfo: ...
    def create(
        self,
        comment: str | None = None,
        storage_root: str | None = None,
        provider_name: str | None = None,
        share_name: str | None = None,
    ) -> CatalogInfo: ...
    def update(
        self,
        new_name: str | None = None,
        comment: str | None = None,
        owner: str | None = None,
    ) -> CatalogInfo: ...
    def delete(self, force: bool = False) -> None: ...

class CredentialsClient:
    def get(self) -> CredentialInfo: ...
    def create(
        self,
        purpose: Purpose,
        comment: str | None = None,
        read_only: bool | None = None,
        skip_validation: bool = False,
        azure_service_principal: AzureServicePrincipal | None = None,
        azure_managed_identity: AzureManagedIdentity | None = None,
        azure_storage_key: AzureStorageKey | None = None,
    ) -> CredentialInfo: ...
    def update(
        self,
        new_name: str | None = None,
        comment: str | None = None,
        read_only: bool | None = None,
        owner: str | None = None,
        skip_validation: bool | None = None,
        force: bool | None = None,
        azure_service_principal: AzureServicePrincipal | None = None,
        azure_managed_identity: AzureManagedIdentity | None = None,
        azure_storage_key: AzureStorageKey | None = None,
    ) -> CredentialInfo: ...

class ExternalLocationsClient:
    def get(self) -> ExternalLocationInfo: ...
    def create(
        self,
        url: str,
        credential_name: str,
        comment: str | None = None,
        read_only: bool | None = None,
        skip_validation: bool = False,
    ) -> ExternalLocationInfo: ...

class RecipientsClient:
    def get(self) -> RecipientInfo: ...

class SharesClient:
    def get(self, include_shared_data: bool | None = None) -> ShareInfo: ...
    def create(
        self,
        name: str,
        comment: str | None = None,
    ) -> ShareInfo: ...
    def update(
        self,
        updates: list[DataObjectUpdate],
        new_name: str | None = None,
        comment: str | None = None,
        owner: str | None = None,
    ) -> ShareInfo: ...

class UnityCatalogClient:
    def __init__(self, base_url: str) -> None: ...
    def catalogs(self, name: str) -> CatalogClient: ...
    def credentials(self, name: str) -> CredentialsClient: ...
    def external_locations(self, name: str) -> ExternalLocationsClient: ...
    def recipients(self, name: str) -> RecipientsClient: ...
    def shares(self, name: str) -> SharesClient: ...

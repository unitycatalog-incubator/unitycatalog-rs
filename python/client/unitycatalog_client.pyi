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
    Unspecified = 0
    Table = 1
    Schema = 2

class HistoryStatus(enum.Enum):
    Disabled = 0
    Enabled = 1

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
        comment: str | None = None,
        storage_location: str | None = None,
        columns: list[ColumnInfo] | None = None,
        properties: dict[str, str] | None = None,
    ) -> TableInfo: ...

class SchemaClient:
    def tables(self, name: str) -> TableClient: ...
    def get(self) -> SchemaInfo: ...
    def create(
        self,
        name: str,
        properties: dict[str, str] | None = None,
    ) -> SchemaInfo: ...
    def update(
        self,
        new_name: str | None = None,
        comment: str | None = None,
        properties: dict[str, str] | None = None,
    ) -> SchemaInfo: ...
    def delete(self, force: bool = False) -> None: ...

class CatalogClient:
    def schemas(self, name: str) -> SchemaClient: ...
    def get(self) -> CatalogInfo: ...
    def create(
        self,
        comment: str | None = None,
        storage_root: str | None = None,
        provider_name: str | None = None,
        share_name: str | None = None,
        properties: dict[str, str] | None = None,
    ) -> CatalogInfo: ...
    def update(
        self,
        new_name: str | None = None,
        comment: str | None = None,
        owner: str | None = None,
        properties: dict[str, str] | None = None,
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

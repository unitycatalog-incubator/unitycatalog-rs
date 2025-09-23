from __future__ import annotations
import enum

class AzureManagedIdentity:
    application_id: str | None
    """The application ID of the application registration within the referenced AAD tenant."""
    msi_resource_id: str | None
    """Msi resource id for use with managed identity authentication"""
    object_id: str | None
    """Object id for use with managed identity authentication"""

    def __init__(
        self,
        application_id: str | None = None,
        msi_resource_id: str | None = None,
        object_id: str | None = None,
    ) -> None: ...

class AzureServicePrincipal:
    application_id: str
    """The application ID of the application registration within the referenced AAD tenant."""
    directory_id: str
    """
    The directory ID corresponding to the Azure Active Directory (AAD) tenant of the
    application.
    """
    client_secret: str | None
    """The client secret generated for the above app ID in AAD."""
    federated_token_file: str | None
    """Location of the file containing a federated token. Specifically useful for workload identity
federation."""

    def __init__(
        self,
        application_id: str,
        directory_id: str,
        client_secret: str | None = None,
        federated_token_file: str | None = None,
    ) -> None: ...

class AzureStorageKey:
    account_key: str
    """The account key of the storage account."""
    account_name: str
    """The name of the storage account."""

    def __init__(self, account_key: str, account_name: str) -> None: ...

class Catalog:
    """A catalog is a root-level namespace that contains schemas."""

    browse_only: bool | None
    """
    Indicates whether the principal is limited to retrieving metadata for the associated
    object through the BROWSE privilege when include_browse is enabled in the request.
    """
    catalog_type: CatalogType | None
    """The type of the catalog."""
    comment: str | None
    """User-provided free-form text description."""
    created_at: int | None
    """Time at which this catalog was created, in epoch milliseconds."""
    created_by: str | None
    """Username of catalog creator."""
    id: str | None
    """Unique identifier for the catalog."""
    name: str
    """Name of catalog."""
    owner: str | None
    """Username of current owner of catalog."""
    properties: list[dict[str, str]] | None
    """A map of key-value properties attached to the securable."""
    provider_name: str | None
    """
    The name of delta sharing provider. A Delta Sharing catalog is a catalog that is based on
    a Delta share on a remote sharing server.
    """
    share_name: str | None
    """The name of the share under the share provider."""
    storage_root: str | None
    """Storage root URL for managed tables within catalog."""
    updated_at: int | None
    """Time at which this catalog was last updated, in epoch milliseconds."""
    updated_by: str | None
    """Username of user who last modified catalog."""

    def __init__(
        self,
        name: str,
        browse_only: bool | None = None,
        catalog_type: CatalogType | None = None,
        comment: str | None = None,
        created_at: int | None = None,
        created_by: str | None = None,
        id: str | None = None,
        owner: str | None = None,
        properties: list[dict[str, str]] | None = None,
        provider_name: str | None = None,
        share_name: str | None = None,
        storage_root: str | None = None,
        updated_at: int | None = None,
        updated_by: str | None = None,
    ) -> None: ...

class Column:
    column_id: str | None
    """a unique id for the column"""
    comment: str | None
    """User-provided free-form text description."""
    name: str
    """Name of the column"""
    nullable: bool | None
    """Whether field may be Null."""
    partition_index: int | None
    """Partition index for column."""
    position: int | None
    """Ordinal position of column (starting at position 0)."""
    type_interval_type: str | None
    """Format of IntervalType."""
    type_json: str
    """Full data type specification, JSON-serialized."""
    type_name: ColumnTypeName
    """Data type name."""
    type_precision: int | None
    """Digits of precision; required for DecimalTypes."""
    type_scale: int | None
    """Digits to right of decimal; Required for DecimalTypes."""
    type_text: str
    """Full data type specification as SQL/catalogString text."""

    def __init__(
        self,
        name: str,
        type_json: str,
        type_name: ColumnTypeName,
        type_text: str,
        column_id: str | None = None,
        comment: str | None = None,
        nullable: bool | None = None,
        partition_index: int | None = None,
        position: int | None = None,
        type_interval_type: str | None = None,
        type_precision: int | None = None,
        type_scale: int | None = None,
    ) -> None: ...

class Credential:
    azure_managed_identity: AzureManagedIdentity | None
    azure_service_principal: AzureServicePrincipal | None
    azure_storage_key: AzureStorageKey | None
    comment: str | None
    """User-provided free-form text description."""
    created_at: int | None
    """Time at which this credential was created, in epoch milliseconds."""
    created_by: str | None
    """Username of credential creator."""
    full_name: str | None
    """The full name of the credential."""
    id: str | None
    """The unique identifier of the credential."""
    name: str
    """
    The credential name. The name must be unique among storage and service credentials within
    the metastore.
    """
    owner: str | None
    """Username of current owner of credential."""
    purpose: Purpose
    """Indicates the purpose of the credential."""
    read_only: bool
    """
    Whether the credential is usable only for read operations. Only applicable when purpose
    is STORAGE.
    """
    updated_at: int | None
    """Time at which this credential was last updated, in epoch milliseconds."""
    updated_by: str | None
    """Username of user who last modified credential."""
    used_for_managed_storage: bool
    """
    Whether this credential is the current metastore's root storage credential. Only
    applicable when purpose is STORAGE.
    """

    def __init__(
        self,
        name: str,
        purpose: Purpose,
        read_only: bool,
        used_for_managed_storage: bool,
        azure_managed_identity: AzureManagedIdentity | None = None,
        azure_service_principal: AzureServicePrincipal | None = None,
        azure_storage_key: AzureStorageKey | None = None,
        comment: str | None = None,
        created_at: int | None = None,
        created_by: str | None = None,
        full_name: str | None = None,
        id: str | None = None,
        owner: str | None = None,
        updated_at: int | None = None,
        updated_by: str | None = None,
    ) -> None: ...

class DataObject:
    added_at: int | None
    """The time when this data object is added to the share, in epoch milliseconds."""
    added_by: str | None
    """Username of the sharer."""
    comment: str | None
    """A user-provided comment when adding the data object to the share."""
    data_object_type: DataObjectType
    """Type of the data object."""
    enable_cdf: bool | None
    """Whether to enable cdf or indicate if cdf is enabled on the shared object."""
    history_data_sharing_status: HistoryStatus | None
    """
    Whether to enable or disable sharing of data history. If not specified, the default is
    DISABLED.
    """
    name: str
    """
    A fully qualified name that uniquely identifies a data object. For example, a table's
    fully qualified name is in the format of <catalog>.<schema>.<table>,
    """
    partitions: list[str] | None
    """Array of partitions for the shared data."""
    shared_as: str | None
    """
    A user-provided new name for the data object within the share. If this new name is not
    provided, the object's original name will be used as the shared_as name. The shared_as
    name must be unique within a share. For tables, the new name must follow the format of
    <schema>.<table>.
    """
    start_version: int | None
    """
    The start version associated with the object. This allows data providers to control the
    lowest object version that is accessible by clients. If specified, clients can query
    snapshots or changes for versions >= start_version. If not specified, clients can only
    query starting from the version of the object at the time it was added to the share. NOTE:
    The start_version should be <= the current version of the object.
    """

    def __init__(
        self,
        data_object_type: DataObjectType,
        name: str,
        added_at: int | None = None,
        added_by: str | None = None,
        comment: str | None = None,
        enable_cdf: bool | None = None,
        history_data_sharing_status: HistoryStatus | None = None,
        partitions: list[str] | None = None,
        shared_as: str | None = None,
        start_version: int | None = None,
    ) -> None: ...

class DataObjectUpdate:
    """Data object update."""

    action: Action
    """Name of the share."""
    data_object: DataObject
    """User-provided free-form text description."""

    def __init__(self, action: Action, data_object: DataObject) -> None: ...

class ExternalLocation:
    browse_only: bool | None
    """
    Indicates whether the principal is limited to retrieving metadata for the associated
    object through the BROWSE privilege when include_browse is enabled in the request.
    """
    comment: str | None
    """User-provided free-form text description."""
    created_at: int | None
    """Time at which this catalog was created, in epoch milliseconds."""
    created_by: str | None
    """Username of catalog creator."""
    credential_id: str
    """Unique ID of the location's storage credential."""
    credential_name: str
    """Name of the storage credential used with this location."""
    external_location_id: str | None
    name: str
    """Name of the external location."""
    owner: str | None
    """The owner of the external location."""
    read_only: bool
    """Indicates whether the external location is read-only."""
    updated_at: int | None
    """Time at which this catalog was last updated, in epoch milliseconds."""
    updated_by: str | None
    """Username of user who last modified catalog."""
    url: str
    """Path URL of the external location."""

    def __init__(
        self,
        credential_id: str,
        credential_name: str,
        name: str,
        read_only: bool,
        url: str,
        browse_only: bool | None = None,
        comment: str | None = None,
        created_at: int | None = None,
        created_by: str | None = None,
        external_location_id: str | None = None,
        owner: str | None = None,
        updated_at: int | None = None,
        updated_by: str | None = None,
    ) -> None: ...

class Recipient:
    authentication_type: AuthenticationType
    """The delta sharing authentication type."""
    comment: str | None
    """Description about the recipient."""
    created_at: int | None
    """Time at which this share was created, in epoch milliseconds."""
    created_by: str | None
    """Username of the creator of the share."""
    id: str | None
    """Unique ID of the recipient."""
    name: str
    """The name of the recipient."""
    owner: str
    """Username of the recipient owner."""
    properties: list[dict[str, str]] | None
    """A map of key-value properties attached to the securable."""
    tokens: list[RecipientToken] | None
    """This field is only present when the authentication_type is TOKEN."""
    updated_at: int | None
    """Time at which this share was updated, in epoch milliseconds."""
    updated_by: str | None
    """Username of share updater."""

    def __init__(
        self,
        authentication_type: AuthenticationType,
        name: str,
        owner: str,
        comment: str | None = None,
        created_at: int | None = None,
        created_by: str | None = None,
        id: str | None = None,
        properties: list[dict[str, str]] | None = None,
        tokens: list[RecipientToken] | None = None,
        updated_at: int | None = None,
        updated_by: str | None = None,
    ) -> None: ...

class RecipientToken:
    activation_url: str
    """
    Full activation URL to retrieve the access token. It will be empty if the token is already
    retrieved.
    """
    created_at: int
    """Time at which this recipient token was created, in epoch milliseconds."""
    created_by: str
    """Username of recipient token creator."""
    expiration_time: int
    """Expiration timestamp of the token in epoch milliseconds."""
    id: str
    """Unique ID of the recipient token."""
    updated_at: int
    """Time at which this recipient token was updated, in epoch milliseconds."""
    updated_by: str
    """Username of recipient token updater."""

    def __init__(
        self,
        activation_url: str,
        created_at: int,
        created_by: str,
        expiration_time: int,
        id: str,
        updated_at: int,
        updated_by: str,
    ) -> None: ...

class Schema:
    """A schema is a namespace within a catalog that contains tables."""

    catalog_name: str
    """Name of parent catalog."""
    comment: str | None
    """User-provided free-form text description."""
    created_at: int | None
    """Time at which this schema was created, in epoch milliseconds."""
    created_by: str | None
    """Username of schema creator."""
    full_name: str
    """Full name of schema, in form of catalog_name.schema_name."""
    name: str
    """Name of schema, relative to parent catalog."""
    owner: str | None
    """Username of current owner of schema."""
    properties: list[dict[str, str]] | None
    """A map of key-value properties attached to the securable."""
    schema_id: str | None
    """Unique identifier for the schema."""
    updated_at: int | None
    """Time at which this schema was last updated, in epoch milliseconds."""
    updated_by: str | None
    """Username of user who last modified schema."""

    def __init__(
        self,
        catalog_name: str,
        full_name: str,
        name: str,
        comment: str | None = None,
        created_at: int | None = None,
        created_by: str | None = None,
        owner: str | None = None,
        properties: list[dict[str, str]] | None = None,
        schema_id: str | None = None,
        updated_at: int | None = None,
        updated_by: str | None = None,
    ) -> None: ...

class Share:
    comment: str | None
    """User-provided free-form text description."""
    created_at: int | None
    """Time at which this share was created, in epoch milliseconds."""
    created_by: str | None
    """Username of the creator of the share."""
    id: str | None
    """Unique ID of the recipient."""
    name: str
    """Name of the share."""
    objects: list[DataObject] | None
    """A list of shared data objects within the share."""
    owner: str | None
    """Username of current owner of share."""
    storage_location: str | None
    """Storage Location URL (full path) for the share."""
    storage_root: str | None
    """Storage root URL for the share."""
    updated_at: int | None
    """Time at which this share was updated, in epoch milliseconds."""
    updated_by: str | None
    """Username of share updater."""

    def __init__(
        self,
        name: str,
        comment: str | None = None,
        created_at: int | None = None,
        created_by: str | None = None,
        id: str | None = None,
        objects: list[DataObject] | None = None,
        owner: str | None = None,
        storage_location: str | None = None,
        storage_root: str | None = None,
        updated_at: int | None = None,
        updated_by: str | None = None,
    ) -> None: ...

class Table:
    catalog_name: str
    """Name of parent catalog."""
    columns: list[Column] | None
    """The array of Column definitions of the table's columns."""
    comment: str | None
    """User-provided free-form text description."""
    created_at: int | None
    """Time at which this table was created, in epoch milliseconds."""
    created_by: str | None
    """Username of table creator."""
    data_source_format: DataSourceFormat
    """Data source format of the table."""
    deleted_at: int | None
    """
    Time at which this table was deleted, in epoch milliseconds. Field is omitted if table is
    not deleted.
    """
    full_name: str
    """Full name of table, in form of catalog_name.schema_name.table_name."""
    name: str
    """Name of table, relative to parent schema."""
    owner: str | None
    """Username of current owner of table."""
    properties: list[dict[str, str]] | None
    """A map of key-value properties attached to the securable."""
    schema_name: str
    """Name of parent schema."""
    storage_credential_name: str | None
    """
    Name of the storage credential, when a storage credential is configured for use with this
    table.
    """
    storage_location: str | None
    """Storage root URL for table (for MANAGED, EXTERNAL tables)"""
    table_id: str | None
    """Unique identifier for the table."""
    table_type: TableType
    updated_at: int | None
    """Time at which this table was last updated, in epoch milliseconds."""
    updated_by: str | None
    """Username of user who last modified table."""

    def __init__(
        self,
        catalog_name: str,
        data_source_format: DataSourceFormat,
        full_name: str,
        name: str,
        schema_name: str,
        table_type: TableType,
        columns: list[Column] | None = None,
        comment: str | None = None,
        created_at: int | None = None,
        created_by: str | None = None,
        deleted_at: int | None = None,
        owner: str | None = None,
        properties: list[dict[str, str]] | None = None,
        storage_credential_name: str | None = None,
        storage_location: str | None = None,
        table_id: str | None = None,
        updated_at: int | None = None,
        updated_by: str | None = None,
    ) -> None: ...

class Volume:
    browse_only: bool | None
    """
    Indicates whether the principal is limited to retrieving metadata for the associated
    object through the BROWSE privilege when include_browse is enabled in the request.
    """
    catalog_name: str
    """Name of parent catalog."""
    comment: str | None
    """User-provided free-form text description."""
    created_at: int | None
    """Time at which this catalog was created, in epoch milliseconds."""
    created_by: str | None
    """Username of catalog creator."""
    full_name: str
    """The three-level (fully qualified) name of the volume"""
    metastore_id: str | None
    """The unique identifier of the metastore"""
    name: str
    """Name of volume, relative to parent schema."""
    owner: str | None
    """Username of current owner of table."""
    schema_name: str
    """Name of parent schema."""
    storage_location: str
    """The storage location on the cloud"""
    updated_at: int | None
    """Time at which this catalog was last updated, in epoch milliseconds."""
    updated_by: str | None
    """Username of user who last modified catalog."""
    volume_id: str
    """The unique identifier of the volume"""
    volume_type: VolumeType
    """
    The type of the volume. An external volume is located in the specified external location.
    A managed volume is located in the default location which is specified by the parent
    schema, or the parent catalog, or the Metastore.
    """

    def __init__(
        self,
        catalog_name: str,
        full_name: str,
        name: str,
        schema_name: str,
        storage_location: str,
        volume_id: str,
        volume_type: VolumeType,
        browse_only: bool | None = None,
        comment: str | None = None,
        created_at: int | None = None,
        created_by: str | None = None,
        metastore_id: str | None = None,
        owner: str | None = None,
        updated_at: int | None = None,
        updated_by: str | None = None,
    ) -> None: ...

class Action(enum.Enum):
    ACTION_UNSPECIFIED = "ACTION_UNSPECIFIED"
    """Unspecified action."""
    ADD = "ADD"
    REMOVE = "REMOVE"
    UPDATE = "UPDATE"

class AuthenticationType(enum.Enum):
    AUTHENTICATION_TYPE_UNSPECIFIED = "AUTHENTICATION_TYPE_UNSPECIFIED"
    """No authentication is required."""
    OAUTH_CLIENT_CREDENTIALS = "OAUTH_CLIENT_CREDENTIALS"
    """OAuth2 authentication is required."""
    TOKEN = "TOKEN"
    """Basic authentication is required."""

class CatalogType(enum.Enum):
    """The type of the catalog."""

    CATALOG_TYPE_UNSPECIFIED = "CATALOG_TYPE_UNSPECIFIED"
    """Unknown catalog type."""
    DELTASHARING_CATALOG = "DELTASHARING_CATALOG"
    MANAGED_CATALOG = "MANAGED_CATALOG"
    SYSTEM_CATALOG = "SYSTEM_CATALOG"

class ColumnTypeName(enum.Enum):
    ARRAY = "ARRAY"
    BINARY = "BINARY"
    BOOLEAN = "BOOLEAN"
    BYTE = "BYTE"
    CHAR = "CHAR"
    COLUMN_TYPE_NAME_UNSPECIFIED = "COLUMN_TYPE_NAME_UNSPECIFIED"
    DATE = "DATE"
    DECIMAL = "DECIMAL"
    DOUBLE = "DOUBLE"
    FLOAT = "FLOAT"
    INT = "INT"
    INTERVAL = "INTERVAL"
    LONG = "LONG"
    MAP = "MAP"
    NULL = "NULL"
    SHORT = "SHORT"
    STRING = "STRING"
    STRUCT = "STRUCT"
    TABLE_TYPE = "TABLE_TYPE"
    TIMESTAMP = "TIMESTAMP"
    TIMESTAMP_NTZ = "TIMESTAMP_NTZ"
    USER_DEFINED_TYPE = "USER_DEFINED_TYPE"
    VARIANT = "VARIANT"

class DataObjectType(enum.Enum):
    DATA_OBJECT_TYPE_UNSPECIFIED = "DATA_OBJECT_TYPE_UNSPECIFIED"
    """Unknown data object type."""
    SCHEMA = "SCHEMA"
    TABLE = "TABLE"

class DataSourceFormat(enum.Enum):
    AVRO = "AVRO"
    CSV = "CSV"
    DATA_SOURCE_FORMAT_UNSPECIFIED = "DATA_SOURCE_FORMAT_UNSPECIFIED"
    DELTA = "DELTA"
    DELTASHARING = "DELTASHARING"
    HUDI = "HUDI"
    ICEBERG = "ICEBERG"
    JSON = "JSON"
    ORC = "ORC"
    PARQUET = "PARQUET"
    TEXT = "TEXT"
    UNITY_CATALOG = "UNITY_CATALOG"

class HistoryStatus(enum.Enum):
    DISABLED = "DISABLED"
    """Data history sharing is disabled."""
    ENABLED = "ENABLED"
    """Data history sharing is enabled."""

class Purpose(enum.Enum):
    PURPOSE_UNSPECIFIED = "PURPOSE_UNSPECIFIED"
    SERVICE = "SERVICE"
    STORAGE = "STORAGE"

class TableType(enum.Enum):
    """The type of the table."""

    EXTERNAL = "EXTERNAL"
    MANAGED = "MANAGED"
    TABLE_TYPE_UNSPECIFIED = "TABLE_TYPE_UNSPECIFIED"

class VolumeType(enum.Enum):
    VOLUME_TYPE_EXTERNAL = "VOLUME_TYPE_EXTERNAL"
    VOLUME_TYPE_MANAGED = "VOLUME_TYPE_MANAGED"
    VOLUME_TYPE_UNSPECIFIED = "VOLUME_TYPE_UNSPECIFIED"

class CatalogClient:
    def delete(self, force: bool | None = None) -> None:
        """
        Delete a catalog Deletes the catalog that matches the supplied name. The caller must be a metastore
        admin or the owner of the catalog.


        Args:
            force: Force deletion even if the catalog is not empty.


        Returns:
            None
        """
        ...
    def get(self, include_browse: bool | None = None) -> Catalog:
        """
        Get a catalog Gets the specified catalog in a metastore. The caller must be a metastore admin, the
        owner of the catalog, or a user that has the USE_CATALOG privilege set for their account.


        Args:
            include_browse: Whether to include catalogs in the response for which the principal can only
                            access selective metadata for


        Returns:
            A catalog is a root-level namespace that contains schemas.
        """
        ...
    def update(
        self,
        owner: str | None = None,
        comment: str | None = None,
        properties: dict[str, str] | None = None,
        new_name: str | None = None,
    ) -> Catalog:
        """
        Update a catalog Updates the catalog that matches the supplied name. The caller must be either the
        owner of the catalog, or a metastore admin (when changing the owner field of the catalog).


        Args:
            owner: Username of new owner of catalog.
            comment: User-provided free-form text description.
            properties: A map of key-value properties attached to the securable. When provided in update
                        request, the specified properties will override the existing properties. To add and
                        remove properties, one would need to perform a read-modify-write.
            new_name: Name of catalog.


        Returns:
            A catalog is a root-level namespace that contains schemas.
        """
        ...
    def schema(self, name: str) -> SchemaClient: ...

class CredentialClient:
    def delete(self) -> None:
        """
        Returns:
            None
        """
        ...
    def get(self) -> Credential:
        """
        Returns:
            The requested resource
        """
        ...
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
    ) -> Credential:
        """
        Args:
            new_name: Name of credential.
            comment: Comment associated with the credential.
            read_only: Whether the credential is usable only for read operations. Only applicable when
                       purpose is STORAGE.
            owner: Username of current owner of credential.
            skip_validation: Supply true to this argument to skip validation of the updated credential.
            force: Force an update even if there are dependent services (when purpose is SERVICE) or
                   dependent external locations and external tables (when purpose is STORAGE).


        Returns:
            The requested resource
        """
        ...

class ExternalLocationClient:
    def delete(self, force: bool | None = None) -> None:
        """
        Delete an external location


        Args:
            force: Force deletion even if the external location is not empty.


        Returns:
            None
        """
        ...
    def get(self) -> ExternalLocation:
        """
        Get an external location


        Returns:
            The requested resource
        """
        ...
    def update(
        self,
        url: str | None = None,
        credential_name: str | None = None,
        read_only: bool | None = None,
        owner: str | None = None,
        comment: str | None = None,
        new_name: str | None = None,
        force: bool | None = None,
        skip_validation: bool | None = None,
    ) -> ExternalLocation:
        """
        Update an external location


        Args:
            url: Path URL of the external location.
            credential_name: Name of the storage credential used with this location.
            read_only: Indicates whether the external location is read-only.
            owner: owner of the external location.
            comment: User-provided free-form text description.
            new_name: new name of the external location.
            force: force update of the external location.
            skip_validation: Skips validation of the storage credential associated with the external
                             location.


        Returns:
            The requested resource
        """
        ...

class RecipientClient:
    def delete(self) -> None:
        """
        Delete a recipient.


        Returns:
            None
        """
        ...
    def get(self) -> Recipient:
        """
        Get a recipient by name.


        Returns:
            The requested resource
        """
        ...
    def update(
        self,
        new_name: str | None = None,
        owner: str | None = None,
        comment: str | None = None,
        properties: dict[str, str] | None = None,
        expiration_time: int | None = None,
    ) -> Recipient:
        """
        Update a recipient.


        Args:
            new_name: New name for the recipient
            owner: Username of the recipient owner.
            comment: Description about the recipient.
            properties: Recipient properties as map of string key-value pairs. When provided in update
                        request, the specified properties will override the existing properties. To add and
                        remove properties, one would need to perform a read-modify-write.
            expiration_time: Expiration timestamp of the token, in epoch milliseconds.


        Returns:
            The requested resource
        """
        ...

class SchemaClient:
    def delete(self, force: bool | None = None) -> None:
        """
        Deletes the specified schema from the parent catalog. The caller must be the owner of the schema or
        an owner of the parent catalog.


        Args:
            force: Force deletion even if the schema is not empty.


        Returns:
            None
        """
        ...
    def get(self) -> Schema:
        """
        Gets the specified schema within the metastore. The caller must be a metastore admin, the owner of
        the schema, or a user that has the USE_SCHEMA privilege on the schema.


        Returns:
            A schema is a namespace within a catalog that contains tables.
        """
        ...
    def update(
        self,
        comment: str | None = None,
        properties: dict[str, str] | None = None,
        new_name: str | None = None,
    ) -> Schema:
        """
        Updates a schema for a catalog. The caller must be the owner of the schema or a metastore admin.
        If the caller is a metastore admin, only the owner field can be changed in the update. If the name
        field must be updated, the caller must be a metastore admin or have the CREATE_SCHEMA privilege on
        the parent catalog.


        Args:
            comment: User-provided free-form text description.
            properties: A map of key-value properties attached to the securable. When provided in update
                        request, the specified properties will override the existing properties. To add and
                        remove properties, one would need to perform a read-modify-write.
            new_name: Name of schema.


        Returns:
            A schema is a namespace within a catalog that contains tables.
        """
        ...
    def table(self, name: str) -> TableClient: ...

class ShareClient:
    def delete(self) -> None:
        """
        Deletes a share.


        Returns:
            None
        """
        ...
    def get(self, include_shared_data: bool | None = None) -> Share:
        """
        Get a share by name.


        Args:
            include_shared_data: Query for data to include in the share.


        Returns:
            The requested resource
        """
        ...
    def update(
        self,
        updates: list[DataObjectUpdate] | None = None,
        new_name: str | None = None,
        owner: str | None = None,
        comment: str | None = None,
    ) -> Share:
        """
        Update a share.


        Args:
            updates: Array of shared data object updates.
            new_name: A new name for the share.
            owner: Owner of the share.
            comment: User-provided free-form text description.


        Returns:
            The requested resource
        """
        ...

class TableClient:
    def delete(self) -> None:
        """
        Delete a table


        Returns:
            None
        """
        ...
    def get(
        self,
        include_delta_metadata: bool | None = None,
        include_browse: bool | None = None,
        include_manifest_capabilities: bool | None = None,
    ) -> Table:
        """
        Get a table


        Args:
            include_delta_metadata: Whether delta metadata should be included in the response.
            include_browse: Whether to include tables in the response for which the principal can only
                            access selective metadata for
            include_manifest_capabilities: Whether to include a manifest containing capabilities the table
                                           has.


        Returns:
            The requested resource
        """
        ...

class TemporaryCredentialClient: ...

class VolumeClient:
    def delete(self) -> None:
        """
        Returns:
            None
        """
        ...
    def get(self, include_browse: bool | None = None) -> Volume:
        """
        Args:
            include_browse: Whether to include schemas in the response for which the principal can only
                            access selective metadata for


        Returns:
            The requested resource
        """
        ...
    def update(
        self,
        new_name: str | None = None,
        comment: str | None = None,
        owner: str | None = None,
    ) -> Volume:
        """
        Args:
            new_name: New name for the volume.
            comment: The comment attached to the volume
            owner: The identifier of the user who owns the volume


        Returns:
            The requested resource
        """
        ...

class PyUnityCatalogClient:
    def __init__(self, base_url: str, token: str | None = None) -> None: ...
    def create_catalog(
        self,
        name: str,
        comment: str | None = None,
        properties: dict[str, str] | None = None,
        storage_root: str | None = None,
        provider_name: str | None = None,
        share_name: str | None = None,
    ) -> Catalog:
        """
        Create a new catalog Creates a new catalog instance in the parent metastore if the caller is a
        metastore admin or has the CREATE_CATALOG privilege.


        Args:
            name: Name of catalog.
            comment: User-provided free-form text description.
            properties: A map of key-value properties attached to the securable.
            storage_root: Storage root URL for managed tables within catalog.
            provider_name: The name of delta sharing provider. A Delta Sharing catalog is a catalog that is
                           based on a Delta share on a remote sharing server.
            share_name: The name of the share under the share provider.


        Returns:
            A catalog is a root-level namespace that contains schemas.
        """
        ...
    def create_credential(
        self,
        name: str,
        purpose: Purpose,
        comment: str | None = None,
        read_only: bool | None = None,
        skip_validation: bool | None = None,
        azure_service_principal: AzureServicePrincipal | None = None,
        azure_managed_identity: AzureManagedIdentity | None = None,
        azure_storage_key: AzureStorageKey | None = None,
    ) -> Credential:
        """
        Args:
            name: The credential name. The name must be unique among storage and service credentials within
                  the metastore.
            purpose: The credential purpose.
            comment: Comment associated with the credential.
            read_only: Whether the credential is usable only for read operations. Only applicable when
                       purpose is STORAGE.
            skip_validation: Supplying true to this argument skips validation of the created set of
                             credentials.


        Returns:
            The requested resource
        """
        ...
    def create_external_location(
        self,
        name: str,
        url: str,
        credential_name: str,
        read_only: bool | None = None,
        comment: str | None = None,
        skip_validation: bool | None = None,
    ) -> ExternalLocation:
        """
        Create a new external location


        Args:
            name: Name of external location.
            url: Path URL of the external location.
            credential_name: Name of the storage credential used with this location.
            read_only: Indicates whether the external location is read-only.
            comment: User-provided free-form text description.
            skip_validation: Skips validation of the storage credential associated with the external
                             location.


        Returns:
            The requested resource
        """
        ...
    def create_recipient(
        self,
        name: str,
        authentication_type: AuthenticationType,
        owner: str,
        comment: str | None = None,
        properties: dict[str, str] | None = None,
        expiration_time: int | None = None,
    ) -> Recipient:
        """
        Create a new recipient.


        Args:
            name: Name of the recipient.
            authentication_type: The delta sharing authentication type.
            owner: Username of the recipient owner.
            comment: Description about the recipient.
            properties: Recipient properties as map of string key-value pairs. When provided in update
                        request, the specified properties will override the existing properties. To add and
                        remove properties, one would need to perform a read-modify-write.
            expiration_time: Expiration timestamp of the token, in epoch milliseconds.


        Returns:
            The requested resource
        """
        ...
    def create_schema(
        self,
        name: str,
        catalog_name: str,
        comment: str | None = None,
        properties: dict[str, str] | None = None,
    ) -> Schema:
        """
        Creates a new schema for catalog in the Metatastore. The caller must be a metastore admin, or have
        the CREATE_SCHEMA privilege in the parent catalog.


        Args:
            name: Name of schema, relative to parent catalog.
            catalog_name: Name of parent catalog.
            comment: User-provided free-form text description.
            properties: A map of key-value properties attached to the securable.


        Returns:
            A schema is a namespace within a catalog that contains tables.
        """
        ...
    def create_share(self, name: str, comment: str | None = None) -> Share:
        """
        Create a new share.


        Args:
            name: Name of the share.
            comment: User-provided free-form text description.


        Returns:
            The requested resource
        """
        ...
    def create_table(
        self,
        name: str,
        schema_name: str,
        catalog_name: str,
        table_type: TableType,
        data_source_format: DataSourceFormat,
        columns: list[Column] | None = None,
        storage_location: str | None = None,
        comment: str | None = None,
        properties: dict[str, str] | None = None,
    ) -> Table:
        """
        Create a table


        Args:
            name: Name of table, relative to parent schema.
            schema_name: Name of parent schema relative to its parent catalog.
            catalog_name: Name of parent catalog.
            columns: The array of Column definitions of the table's columns.
            storage_location: Storage root URL for external table.
            comment: User-provided free-form text description.
            properties: A map of key-value properties attached to the securable.


        Returns:
            The requested resource
        """
        ...
    def create_volume(
        self,
        catalog_name: str,
        schema_name: str,
        name: str,
        volume_type: VolumeType,
        storage_location: str | None = None,
        comment: str | None = None,
    ) -> Volume:
        """
        Args:
            catalog_name: The identifier of the catalog
            schema_name: The identifier of the schema
            name: The identifier of the volume
            volume_type: The type of the volume. An external volume is located in the specified external
                         location. A managed volume is located in the default location which is specified by
                         the parent schema, or the parent catalog, or the Metastore.
            storage_location: The storage location on the cloud
            comment: The storage location on the cloud


        Returns:
            The requested resource
        """
        ...
    def list_catalogs(self, max_results: int | None = None) -> list[Catalog]:
        """
        List catalogs Gets an array of catalogs in the metastore. If the caller is the metastore admin, all
        catalogs will be retrieved. Otherwise, only catalogs owned by the caller (or for which the caller
        has the USE_CATALOG privilege) will be retrieved. There is no guarantee of a specific ordering of
        the elements in the array.


        Args:
            max_results: The maximum number of results per page that should be returned.


        Returns:
            List of The catalogs returned.
        """
        ...
    def list_credentials(
        self, purpose: Purpose | None = None, max_results: int | None = None
    ) -> list[Credential]:
        """
        Args:
            purpose: Return only credentials for the specified purpose.
            max_results: The maximum number of results per page that should be returned.


        Returns:
            List of The credentials returned.
        """
        ...
    def list_external_locations(
        self, max_results: int | None = None, include_browse: bool | None = None
    ) -> list[ExternalLocation]:
        """
        List external locations


        Args:
            max_results: The maximum number of results per page that should be returned.
            include_browse: Whether to include schemas in the response for which the principal can only
                            access selective metadata for


        Returns:
            List of The external locations returned.
        """
        ...
    def list_recipients(self, max_results: int | None = None) -> list[Recipient]:
        """
        List recipients.


        Args:
            max_results: The maximum number of results per page that should be returned.


        Returns:
            List of List of recipients.
        """
        ...
    def list_schemas(
        self,
        catalog_name: str,
        max_results: int | None = None,
        include_browse: bool | None = None,
    ) -> list[Schema]:
        """
        Gets an array of schemas for a catalog in the metastore. If the caller is the metastore admin or the
        owner of the parent catalog, all schemas for the catalog will be retrieved. Otherwise, only schemas
        owned by the caller (or for which the caller has the USE_SCHEMA privilege) will be retrieved. There
        is no guarantee of a specific ordering of the elements in the array.


        Args:
            catalog_name: Name of parent catalog.
            max_results: The maximum number of results per page that should be returned.
            include_browse: Whether to include schemas in the response for which the principal can only
                            access selective metadata for


        Returns:
            List of The schemas returned.
        """
        ...
    def list_shares(self, max_results: int | None = None) -> list[Share]:
        """
        List shares.


        Args:
            max_results: The maximum number of results per page that should be returned.


        Returns:
            List of List of shares.
        """
        ...
    def list_tables(
        self,
        catalog_name: str,
        schema_name: str,
        max_results: int | None = None,
        include_delta_metadata: bool | None = None,
        omit_columns: bool | None = None,
        omit_properties: bool | None = None,
        omit_username: bool | None = None,
        include_browse: bool | None = None,
        include_manifest_capabilities: bool | None = None,
    ) -> list[Table]:
        """
        Gets an array of all tables for the current metastore under the parent catalog and schema. The
        caller must be a metastore admin or an owner of (or have the SELECT privilege on) the table. For
        the latter case, the caller must also be the owner or have the USE_CATALOG privilege on the parent
        catalog and the USE_SCHEMA privilege on the parent schema. There is no guarantee of a specific
        ordering of the elements in the array.


        Args:
            catalog_name: Name of parent catalog for tables of interest.
            schema_name: Name of parent schema for tables of interest.
            max_results: The maximum number of results per page that should be returned.
            include_delta_metadata: Whether delta metadata should be included in the response.
            omit_columns: Whether to omit the columns of the table from the response or not.
            omit_properties: Whether to omit the properties of the table from the response or not.
            omit_username: Whether to omit the username of the table (e.g. owner, updated_by, created_by)
                           from the response or not.
            include_browse: Whether to include tables in the response for which the principal can only
                            access selective metadata for
            include_manifest_capabilities: Whether to include a manifest containing capabilities the table
                                           has.


        Returns:
            List of The tables returned.
        """
        ...
    def list_volumes(
        self,
        catalog_name: str,
        schema_name: str,
        max_results: int | None = None,
        include_browse: bool | None = None,
    ) -> list[Volume]:
        """
        Lists volumes.


        Args:
            catalog_name: The identifier of the catalog
            schema_name: The identifier of the schema
            max_results: The maximum number of results per page that should be returned.
            include_browse: Whether to include schemas in the response for which the principal can only
                            access selective metadata for


        Returns:
            List of The volumes returned.
        """
        ...
    def catalog(self, name: str) -> CatalogClient: ...
    def credential(self, name: str) -> CredentialClient: ...
    def external_location(self, name: str) -> ExternalLocationClient: ...
    def recipient(self, name: str) -> RecipientClient: ...
    def schema(self, catalog_name: str, schema_name: str) -> SchemaClient: ...
    def share(self, name: str) -> ShareClient: ...
    def table(self, full_name: str) -> TableClient: ...
    def volume(self, catalog_name: str, schema_name: str, volume_name: str) -> VolumeClient: ...

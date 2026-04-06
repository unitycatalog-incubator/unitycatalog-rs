# @generated — do not edit by hand.
from __future__ import annotations
from typing import Optional, List, Dict, Any, Literal
import enum

class AwsTemporaryCredentials:
    access_key_id: str
    """The access key ID that identifies the temporary credentials."""
    access_point: str
    """
    The Amazon Resource Name (ARN) of the S3 access point for temporary credentials related
    the external location.
    """
    secret_access_key: str
    """The secret access key that can be used to sign AWS API requests."""
    session_token: str
    """The token that users must pass to AWS API to use the temporary credentials."""

    def __init__(
        self, access_key_id: str, access_point: str, secret_access_key: str, session_token: str
    ) -> None: ...

class AzureAad:
    aad_token: str
    """
    Opaque token that contains claims that you can use in Azure Active Directory to access
    cloud services.
    """

    def __init__(self, aad_token: str) -> None: ...

class AzureManagedIdentity:
    application_id: Optional[str]
    """The application ID of the application registration within the referenced AAD tenant."""
    msi_resource_id: Optional[str]
    """Msi resource id for use with managed identity authentication"""
    object_id: Optional[str]
    """Object id for use with managed identity authentication"""

    def __init__(
        self,
        application_id: Optional[str] = None,
        msi_resource_id: Optional[str] = None,
        object_id: Optional[str] = None,
    ) -> None: ...

class AzureServicePrincipal:
    application_id: str
    """The application ID of the application registration within the referenced AAD tenant."""
    directory_id: str
    """
    The directory ID corresponding to the Azure Active Directory (AAD) tenant of the
    application.
    """
    client_secret: Optional[str]
    """The client secret generated for the above app ID in AAD."""
    federated_token_file: Optional[str]
    """Location of the file containing a federated token.

Specifically useful for workload identity federation."""

    def __init__(
        self,
        application_id: str,
        directory_id: str,
        client_secret: Optional[str] = None,
        federated_token_file: Optional[str] = None,
    ) -> None: ...

class AzureStorageKey:
    account_key: str
    """The account key of the storage account."""
    account_name: str
    """The name of the storage account."""

    def __init__(self, account_key: str, account_name: str) -> None: ...

class AzureUserDelegationSas:
    sas_token: str
    """The signed URI (SAS Token) used to access blob services for a given path"""

    def __init__(self, sas_token: str) -> None: ...

class Catalog:
    """A catalog is a root-level namespace that contains schemas."""

    browse_only: Optional[bool]
    """
    Indicates whether the principal is limited to retrieving metadata for the associated
    object through the BROWSE privilege when include_browse is enabled in the request.
    """
    catalog_type: Optional[CatalogType]
    """The type of the catalog."""
    comment: Optional[str]
    """User-provided free-form text description."""
    created_at: Optional[int]
    """Time at which this catalog was created, in epoch milliseconds."""
    created_by: Optional[str]
    """Username of catalog creator."""
    id: Optional[str]
    """Unique identifier for the catalog."""
    name: str
    """Name of catalog."""
    owner: Optional[str]
    """Username of current owner of catalog."""
    properties: Dict[str, str]
    """A map of key-value properties attached to the securable."""
    provider_name: Optional[str]
    """
    The name of delta sharing provider. A Delta Sharing catalog is a catalog that is based on
    a Delta share on a remote sharing server.
    """
    share_name: Optional[str]
    """The name of the share under the share provider."""
    storage_root: Optional[str]
    """Storage root URL for managed tables within catalog."""
    updated_at: Optional[int]
    """Time at which this catalog was last updated, in epoch milliseconds."""
    updated_by: Optional[str]
    """Username of user who last modified catalog."""

    def __init__(
        self,
        name: str,
        properties: Dict[str, str],
        browse_only: Optional[bool] = None,
        catalog_type: Optional[CatalogType] = None,
        comment: Optional[str] = None,
        created_at: Optional[int] = None,
        created_by: Optional[str] = None,
        id: Optional[str] = None,
        owner: Optional[str] = None,
        provider_name: Optional[str] = None,
        share_name: Optional[str] = None,
        storage_root: Optional[str] = None,
        updated_at: Optional[int] = None,
        updated_by: Optional[str] = None,
    ) -> None: ...

class Column:
    column_id: Optional[str]
    """a unique id for the column"""
    comment: Optional[str]
    """User-provided free-form text description."""
    name: str
    """Name of the column"""
    nullable: Optional[bool]
    """Whether field may be Null."""
    partition_index: Optional[int]
    """Partition index for column."""
    position: Optional[int]
    """Ordinal position of column (starting at position 0)."""
    type_interval_type: Optional[str]
    """Format of IntervalType."""
    type_json: str
    """Full data type specification, JSON-serialized."""
    type_name: ColumnTypeName
    """Data type name."""
    type_precision: Optional[int]
    """Digits of precision; required for DecimalTypes."""
    type_scale: Optional[int]
    """Digits to right of decimal; Required for DecimalTypes."""
    type_text: str
    """Full data type specification as SQL/catalogString text."""

    def __init__(
        self,
        name: str,
        type_json: str,
        type_name: ColumnTypeName,
        type_text: str,
        column_id: Optional[str] = None,
        comment: Optional[str] = None,
        nullable: Optional[bool] = None,
        partition_index: Optional[int] = None,
        position: Optional[int] = None,
        type_interval_type: Optional[str] = None,
        type_precision: Optional[int] = None,
        type_scale: Optional[int] = None,
    ) -> None: ...

class Credential:
    azure_managed_identity: Optional[AzureManagedIdentity]
    azure_service_principal: Optional[AzureServicePrincipal]
    azure_storage_key: Optional[AzureStorageKey]
    comment: Optional[str]
    """User-provided free-form text description."""
    created_at: Optional[int]
    """Time at which this credential was created, in epoch milliseconds."""
    created_by: Optional[str]
    """Username of credential creator."""
    full_name: Optional[str]
    """The full name of the credential."""
    id: Optional[str]
    """The unique identifier of the credential."""
    name: str
    """
    The credential name. The name must be unique among storage and service credentials within
    the metastore.
    """
    owner: Optional[str]
    """Username of current owner of credential."""
    purpose: Purpose
    """Indicates the purpose of the credential."""
    read_only: bool
    """
    Whether the credential is usable only for read operations. Only applicable when purpose
    is STORAGE.
    """
    updated_at: Optional[int]
    """Time at which this credential was last updated, in epoch milliseconds."""
    updated_by: Optional[str]
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
        azure_managed_identity: Optional[AzureManagedIdentity] = None,
        azure_service_principal: Optional[AzureServicePrincipal] = None,
        azure_storage_key: Optional[AzureStorageKey] = None,
        comment: Optional[str] = None,
        created_at: Optional[int] = None,
        created_by: Optional[str] = None,
        full_name: Optional[str] = None,
        id: Optional[str] = None,
        owner: Optional[str] = None,
        updated_at: Optional[int] = None,
        updated_by: Optional[str] = None,
    ) -> None: ...

class DataObject:
    added_at: Optional[int]
    """The time when this data object is added to the share, in epoch milliseconds."""
    added_by: Optional[str]
    """Username of the sharer."""
    comment: Optional[str]
    """A user-provided comment when adding the data object to the share."""
    data_object_type: DataObjectType
    """Type of the data object."""
    enable_cdf: Optional[bool]
    """Whether to enable cdf or indicate if cdf is enabled on the shared object."""
    history_data_sharing_status: Optional[HistoryStatus]
    """
    Whether to enable or disable sharing of data history. If not specified, the default is
    DISABLED.
    """
    name: str
    """
    A fully qualified name that uniquely identifies a data object. For example, a table's
    fully qualified name is in the format of <catalog>.<schema>.<table>,
    """
    partitions: List[str]
    """Array of partitions for the shared data."""
    shared_as: Optional[str]
    """
    A user-provided new name for the data object within the share. If this new name is not
    provided, the object's original name will be used as the shared_as name. The shared_as
    name must be unique within a share. For tables, the new name must follow the format of
    <schema>.<table>.
    """
    start_version: Optional[int]
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
        added_at: Optional[int] = None,
        added_by: Optional[str] = None,
        comment: Optional[str] = None,
        enable_cdf: Optional[bool] = None,
        history_data_sharing_status: Optional[HistoryStatus] = None,
        partitions: Optional[List[str]] = None,
        shared_as: Optional[str] = None,
        start_version: Optional[int] = None,
    ) -> None: ...

class DataObjectUpdate:
    """Data object update."""

    action: Action
    """Name of the share."""
    data_object: DataObject
    """User-provided free-form text description."""

    def __init__(self, action: Action, data_object: DataObject) -> None: ...

class ExternalLocation:
    browse_only: Optional[bool]
    """
    Indicates whether the principal is limited to retrieving metadata for the associated
    object through the BROWSE privilege when include_browse is enabled in the request.
    """
    comment: Optional[str]
    """User-provided free-form text description."""
    created_at: Optional[int]
    """Time at which this catalog was created, in epoch milliseconds."""
    created_by: Optional[str]
    """Username of catalog creator."""
    credential_id: str
    """Unique ID of the location's storage credential."""
    credential_name: str
    """Name of the storage credential used with this location."""
    external_location_id: Optional[str]
    name: str
    """Name of the external location."""
    owner: Optional[str]
    """The owner of the external location."""
    read_only: bool
    """Indicates whether the external location is read-only."""
    updated_at: Optional[int]
    """Time at which this catalog was last updated, in epoch milliseconds."""
    updated_by: Optional[str]
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
        browse_only: Optional[bool] = None,
        comment: Optional[str] = None,
        created_at: Optional[int] = None,
        created_by: Optional[str] = None,
        external_location_id: Optional[str] = None,
        owner: Optional[str] = None,
        updated_at: Optional[int] = None,
        updated_by: Optional[str] = None,
    ) -> None: ...

class Function:
    """A User-Defined Function (UDF) registered under a catalog + schema hierarchy."""

    catalog_name: str
    """Name of parent catalog."""
    comment: Optional[str]
    """User-provided free-form text description."""
    created_at: Optional[int]
    """Time at which this function was created, in epoch milliseconds."""
    created_by: Optional[str]
    """Username of function creator."""
    data_type: str
    """Full data type specification of the return type of the function."""
    full_data_type: str
    """Full data type specification as SQL/catalogString text."""
    full_name: str
    """
    The three-level (fully qualified) name of the function. Format:
    catalog_name.schema_name.function_name
    """
    function_id: Optional[str]
    """Unique identifier for the function."""
    input_params: Optional[FunctionParameterInfos]
    """The array of function parameter infos."""
    is_deterministic: bool
    """Indicates whether the function is deterministic."""
    is_null_call: bool
    """Indicates whether the function is null-calling."""
    name: str
    """Name of function, relative to parent schema."""
    owner: Optional[str]
    """Username of current owner of the function."""
    parameter_style: ParameterStyle
    """The parameter-passing style."""
    properties: Dict[str, str]
    """A map of key-value properties attached to the securable."""
    return_params: Optional[str]
    """The return type of the function in JSON format."""
    routine_body: RoutineBody
    """The routine body."""
    routine_body_language: Optional[str]
    """The language of the function routine body."""
    routine_definition: Optional[str]
    """Function body."""
    routine_dependencies: Optional[str]
    """Function dependencies (in JSON form)."""
    schema_name: str
    """Name of parent schema."""
    security_type: SecurityType
    """The security type of the function."""
    specific_name: Optional[str]
    """The type of the function (SCALAR or TABLE)."""
    sql_data_access: SqlDataAccess
    """SQL data access information."""
    updated_at: Optional[int]
    """Time at which this function was last updated, in epoch milliseconds."""
    updated_by: Optional[str]
    """Username of user who last modified the function."""

    def __init__(
        self,
        catalog_name: str,
        data_type: str,
        full_data_type: str,
        full_name: str,
        is_deterministic: bool,
        is_null_call: bool,
        name: str,
        parameter_style: ParameterStyle,
        properties: Dict[str, str],
        routine_body: RoutineBody,
        schema_name: str,
        security_type: SecurityType,
        sql_data_access: SqlDataAccess,
        comment: Optional[str] = None,
        created_at: Optional[int] = None,
        created_by: Optional[str] = None,
        function_id: Optional[str] = None,
        input_params: Optional[FunctionParameterInfos] = None,
        owner: Optional[str] = None,
        return_params: Optional[str] = None,
        routine_body_language: Optional[str] = None,
        routine_definition: Optional[str] = None,
        routine_dependencies: Optional[str] = None,
        specific_name: Optional[str] = None,
        updated_at: Optional[int] = None,
        updated_by: Optional[str] = None,
    ) -> None: ...

class FunctionParameterInfo:
    """Information about a single function parameter."""

    comment: Optional[str]
    """User-provided free-form text description."""
    name: str
    """Name of parameter."""
    parameter_default: Optional[str]
    """Default value of the parameter."""
    parameter_mode: ParameterMode
    """The mode of the function parameter."""
    parameter_type: FunctionParameterType
    """The type of function parameter."""
    position: Optional[int]
    """Ordinal position of column (starting at position 0)."""
    type_interval_type: Optional[str]
    """Format of IntervalType."""
    type_json: Optional[str]
    """Full data type specification, JSON-serialized."""
    type_name: ColumnTypeName
    """Data type name."""
    type_precision: Optional[int]
    """Digits of precision; required for DecimalTypes."""
    type_scale: Optional[int]
    """Digits to right of decimal; required for DecimalTypes."""
    type_text: str
    """Full data type specification as SQL/catalogString text."""

    def __init__(
        self,
        name: str,
        parameter_mode: ParameterMode,
        parameter_type: FunctionParameterType,
        type_name: ColumnTypeName,
        type_text: str,
        comment: Optional[str] = None,
        parameter_default: Optional[str] = None,
        position: Optional[int] = None,
        type_interval_type: Optional[str] = None,
        type_json: Optional[str] = None,
        type_precision: Optional[int] = None,
        type_scale: Optional[int] = None,
    ) -> None: ...

class FunctionParameterInfos:
    """A collection of function parameters."""

    parameters: List[FunctionParameterInfo]
    """The parameters of the function."""

    def __init__(self, parameters: Optional[List[FunctionParameterInfo]] = None) -> None: ...

class GcpOauthToken:
    oauth_token: str
    """The OAuth token used to access Google Cloud services."""

    def __init__(self, oauth_token: str) -> None: ...

class PermissionsChange:
    add: List[str]
    """The set of privileges to add."""
    principal: str
    """The principal (user email address or group name)."""
    remove: List[str]
    """The set of privileges to remove."""

    def __init__(
        self, principal: str, add: Optional[List[str]] = None, remove: Optional[List[str]] = None
    ) -> None: ...

class PrivilegeAssignment:
    principal: str
    """
    The principal (user email address or group name). For deleted principals, principal is
    empty while principal_id is populated.
    """
    privileges: List[str]
    """The privileges assigned to the principal."""

    def __init__(self, principal: str, privileges: Optional[List[str]] = None) -> None: ...

class R2TemporaryCredentials:
    access_key_id: str
    """The access key ID that identifies the temporary credentials."""
    secret_access_key: str
    """The secret access key associated with the access key."""
    session_token: str
    """The generated JWT that users must pass to use the temporary credentials."""

    def __init__(self, access_key_id: str, secret_access_key: str, session_token: str) -> None: ...

class Recipient:
    authentication_type: AuthenticationType
    """The delta sharing authentication type."""
    comment: Optional[str]
    """Description about the recipient."""
    created_at: Optional[int]
    """Time at which this share was created, in epoch milliseconds."""
    created_by: Optional[str]
    """Username of the creator of the share."""
    id: Optional[str]
    """Unique ID of the recipient."""
    name: str
    """The name of the recipient."""
    owner: str
    """Username of the recipient owner."""
    properties: Dict[str, str]
    """A map of key-value properties attached to the securable."""
    tokens: List[RecipientToken]
    """This field is only present when the authentication_type is TOKEN."""
    updated_at: Optional[int]
    """Time at which this share was updated, in epoch milliseconds."""
    updated_by: Optional[str]
    """Username of share updater."""

    def __init__(
        self,
        authentication_type: AuthenticationType,
        name: str,
        owner: str,
        properties: Dict[str, str],
        comment: Optional[str] = None,
        created_at: Optional[int] = None,
        created_by: Optional[str] = None,
        id: Optional[str] = None,
        tokens: Optional[List[RecipientToken]] = None,
        updated_at: Optional[int] = None,
        updated_by: Optional[str] = None,
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
    comment: Optional[str]
    """User-provided free-form text description."""
    created_at: Optional[int]
    """Time at which this schema was created, in epoch milliseconds."""
    created_by: Optional[str]
    """Username of schema creator."""
    full_name: str
    """Full name of schema, in form of catalog_name.schema_name."""
    name: str
    """Name of schema, relative to parent catalog."""
    owner: Optional[str]
    """Username of current owner of schema."""
    properties: Dict[str, str]
    """A map of key-value properties attached to the securable."""
    schema_id: Optional[str]
    """Unique identifier for the schema."""
    updated_at: Optional[int]
    """Time at which this schema was last updated, in epoch milliseconds."""
    updated_by: Optional[str]
    """Username of user who last modified schema."""

    def __init__(
        self,
        catalog_name: str,
        full_name: str,
        name: str,
        properties: Dict[str, str],
        comment: Optional[str] = None,
        created_at: Optional[int] = None,
        created_by: Optional[str] = None,
        owner: Optional[str] = None,
        schema_id: Optional[str] = None,
        updated_at: Optional[int] = None,
        updated_by: Optional[str] = None,
    ) -> None: ...

class Share:
    comment: Optional[str]
    """User-provided free-form text description."""
    created_at: Optional[int]
    """Time at which this share was created, in epoch milliseconds."""
    created_by: Optional[str]
    """Username of the creator of the share."""
    id: Optional[str]
    """Unique ID of the recipient."""
    name: str
    """Name of the share."""
    objects: List[DataObject]
    """A list of shared data objects within the share."""
    owner: Optional[str]
    """Username of current owner of share."""
    storage_location: Optional[str]
    """Storage Location URL (full path) for the share."""
    storage_root: Optional[str]
    """Storage root URL for the share."""
    updated_at: Optional[int]
    """Time at which this share was updated, in epoch milliseconds."""
    updated_by: Optional[str]
    """Username of share updater."""

    def __init__(
        self,
        name: str,
        comment: Optional[str] = None,
        created_at: Optional[int] = None,
        created_by: Optional[str] = None,
        id: Optional[str] = None,
        objects: Optional[List[DataObject]] = None,
        owner: Optional[str] = None,
        storage_location: Optional[str] = None,
        storage_root: Optional[str] = None,
        updated_at: Optional[int] = None,
        updated_by: Optional[str] = None,
    ) -> None: ...

class Table:
    catalog_name: str
    """Name of parent catalog."""
    columns: List[Column]
    """The array of Column definitions of the table's columns."""
    comment: Optional[str]
    """User-provided free-form text description."""
    created_at: Optional[int]
    """Time at which this table was created, in epoch milliseconds."""
    created_by: Optional[str]
    """Username of table creator."""
    data_source_format: DataSourceFormat
    """Data source format of the table."""
    deleted_at: Optional[int]
    """
    Time at which this table was deleted, in epoch milliseconds. Field is omitted if table is
    not deleted.
    """
    full_name: str
    """Full name of table, in form of catalog_name.schema_name.table_name."""
    name: str
    """Name of table, relative to parent schema."""
    owner: Optional[str]
    """Username of current owner of table."""
    properties: Dict[str, str]
    """A map of key-value properties attached to the securable."""
    schema_name: str
    """Name of parent schema."""
    storage_credential_name: Optional[str]
    """
    Name of the storage credential, when a storage credential is configured for use with this
    table.
    """
    storage_location: Optional[str]
    """Storage root URL for table (for MANAGED, EXTERNAL tables)"""
    table_id: Optional[str]
    """Unique identifier for the table."""
    table_type: TableType
    updated_at: Optional[int]
    """Time at which this table was last updated, in epoch milliseconds."""
    updated_by: Optional[str]
    """Username of user who last modified table."""

    def __init__(
        self,
        catalog_name: str,
        data_source_format: DataSourceFormat,
        full_name: str,
        name: str,
        properties: Dict[str, str],
        schema_name: str,
        table_type: TableType,
        columns: Optional[List[Column]] = None,
        comment: Optional[str] = None,
        created_at: Optional[int] = None,
        created_by: Optional[str] = None,
        deleted_at: Optional[int] = None,
        owner: Optional[str] = None,
        storage_credential_name: Optional[str] = None,
        storage_location: Optional[str] = None,
        table_id: Optional[str] = None,
        updated_at: Optional[int] = None,
        updated_by: Optional[str] = None,
    ) -> None: ...

class TableSummary:
    full_name: str
    """The full name of the table."""
    table_type: TableType

    def __init__(self, full_name: str, table_type: TableType) -> None: ...

class TemporaryCredential:
    """The response to the GenerateTemporaryTableCredentialsRequest."""

    expiration_time: int
    """
    Server time when the credential will expire, in epoch milliseconds. The API client is
    advised to cache the credential given this expiration time.
    """
    url: str
    """The URL of the storage path accessible by the temporary credential."""
    aws_temp_credentials: Optional[AwsTemporaryCredentials]
    """Credentials for AWS S3."""
    azure_aad: Optional[AzureAad]
    """Credentials for Azure Active Directory."""
    azure_user_delegation_sas: Optional[AzureUserDelegationSas]
    """Credentials for Azure Blob Storage."""
    gcp_oauth_token: Optional[GcpOauthToken]
    """Credentials for Google Cloud Storage."""
    r2_temp_credentials: Optional[R2TemporaryCredentials]
    """Credentials for R2."""

    def __init__(
        self,
        expiration_time: int,
        url: str,
        aws_temp_credentials: Optional[AwsTemporaryCredentials] = None,
        azure_aad: Optional[AzureAad] = None,
        azure_user_delegation_sas: Optional[AzureUserDelegationSas] = None,
        gcp_oauth_token: Optional[GcpOauthToken] = None,
        r2_temp_credentials: Optional[R2TemporaryCredentials] = None,
    ) -> None: ...

class Volume:
    browse_only: Optional[bool]
    """
    Indicates whether the principal is limited to retrieving metadata for the associated
    object through the BROWSE privilege when include_browse is enabled in the request.
    """
    catalog_name: str
    """Name of parent catalog."""
    comment: Optional[str]
    """User-provided free-form text description."""
    created_at: Optional[int]
    """Time at which this catalog was created, in epoch milliseconds."""
    created_by: Optional[str]
    """Username of catalog creator."""
    full_name: str
    """The three-level (fully qualified) name of the volume"""
    metastore_id: Optional[str]
    """The unique identifier of the metastore"""
    name: str
    """Name of volume, relative to parent schema."""
    owner: Optional[str]
    """Username of current owner of table."""
    schema_name: str
    """Name of parent schema."""
    storage_location: str
    """The storage location on the cloud"""
    updated_at: Optional[int]
    """Time at which this catalog was last updated, in epoch milliseconds."""
    updated_by: Optional[str]
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
        browse_only: Optional[bool] = None,
        comment: Optional[str] = None,
        created_at: Optional[int] = None,
        created_by: Optional[str] = None,
        metastore_id: Optional[str] = None,
        owner: Optional[str] = None,
        updated_at: Optional[int] = None,
        updated_by: Optional[str] = None,
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

class FunctionParameterType(enum.Enum):
    """The type of the function parameter."""

    COLUMN = "COLUMN"
    """A named column parameter."""
    FUNCTION_PARAMETER_TYPE_UNSPECIFIED = "FUNCTION_PARAMETER_TYPE_UNSPECIFIED"
    PARAM = "PARAM"
    """A named parameter (default)."""

class HistoryStatus(enum.Enum):
    DISABLED = "DISABLED"
    """Data history sharing is disabled."""
    ENABLED = "ENABLED"
    """Data history sharing is enabled."""

class ParameterMode(enum.Enum):
    """The mode of the function parameter."""

    IN = "IN"
    """Input parameter."""
    PARAMETER_MODE_UNSPECIFIED = "PARAMETER_MODE_UNSPECIFIED"

class ParameterStyle(enum.Enum):
    """The parameter-passing style."""

    PARAMETER_STYLE_UNSPECIFIED = "PARAMETER_STYLE_UNSPECIFIED"
    S = "S"
    """The parameters are passed positionally (S = SQL)."""

class Purpose(enum.Enum):
    PURPOSE_UNSPECIFIED = "PURPOSE_UNSPECIFIED"
    SERVICE = "SERVICE"
    STORAGE = "STORAGE"

class RoutineBody(enum.Enum):
    """Determines whether the function body is interpreted as SQL or as an external function."""

    EXTERNAL = "EXTERNAL"
    """The function is defined externally."""
    ROUTINE_BODY_UNSPECIFIED = "ROUTINE_BODY_UNSPECIFIED"
    SQL = "SQL"
    """The function is defined in SQL."""

class SecurityType(enum.Enum):
    """The security type of the function."""

    DEFINER = "DEFINER"
    """The function runs as the invoking user (DEFINER = standard SQL semantics)."""
    SECURITY_TYPE_UNSPECIFIED = "SECURITY_TYPE_UNSPECIFIED"

class SqlDataAccess(enum.Enum):
    """Information about the SQL data access capability of the function."""

    CONTAINS_SQL = "CONTAINS_SQL"
    """Function contains no SQL."""
    NO_SQL = "NO_SQL"
    """Function does not use SQL and does not access data."""
    READS_SQL_DATA = "READS_SQL_DATA"
    """Function reads from SQL tables or views."""
    SQL_DATA_ACCESS_UNSPECIFIED = "SQL_DATA_ACCESS_UNSPECIFIED"

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
    def delete(self, force: Optional[bool] = None) -> None:
        """
        Delete a catalog

        Deletes the catalog that matches the supplied name. The caller must be a metastore admin or the
        owner of the catalog.


        Args:
            force: Force deletion even if the catalog is not empty.


        Returns:
            None
        """
        ...
    def get(self, include_browse: Optional[bool] = None) -> Catalog:
        """
        Get a catalog

        Gets the specified catalog in a metastore. The caller must be a metastore admin, the owner of the
        catalog, or a user that has the USE_CATALOG privilege set for their account.


        Args:
            include_browse: Whether to include catalogs in the response for which the principal can only
                            access selective metadata for


        Returns:
            A catalog is a root-level namespace that contains schemas.
        """
        ...
    def update(
        self,
        owner: Optional[str] = None,
        comment: Optional[str] = None,
        properties: Optional[Dict[str, str]] = None,
        new_name: Optional[str] = None,
    ) -> Catalog:
        """
        Update a catalog

        Updates the catalog that matches the supplied name. The caller must be either the owner of the
        catalog, or a metastore admin (when changing the owner field of the catalog).


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
        new_name: Optional[str] = None,
        comment: Optional[str] = None,
        read_only: Optional[bool] = None,
        owner: Optional[str] = None,
        skip_validation: Optional[bool] = None,
        force: Optional[bool] = None,
        azure_service_principal: Optional[AzureServicePrincipal] = None,
        azure_managed_identity: Optional[AzureManagedIdentity] = None,
        azure_storage_key: Optional[AzureStorageKey] = None,
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
    def delete(self, force: Optional[bool] = None) -> None:
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
        url: Optional[str] = None,
        credential_name: Optional[str] = None,
        read_only: Optional[bool] = None,
        owner: Optional[str] = None,
        comment: Optional[str] = None,
        new_name: Optional[str] = None,
        force: Optional[bool] = None,
        skip_validation: Optional[bool] = None,
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

class FunctionClient:
    def delete(self, force: Optional[bool] = None) -> None:
        """
        Delete a function

        Deletes the function that matches the supplied name. For the deletion to succeed, the caller must be
        the owner of the function.


        Args:
            force: Force deletion even if the function is not empty.


        Returns:
            None
        """
        ...
    def get(self) -> Function:
        """
        Get a function

        Gets a function from within a parent catalog and schema. For the fetch to succeed, the caller must
        be a metastore admin, the owner of the function, or have SELECT on the function.


        Returns:
            A User-Defined Function (UDF) registered under a catalog + schema hierarchy.
        """
        ...
    def update(self, owner: Optional[str] = None) -> Function:
        """
        Update a function

        Updates the function that matches the supplied name. Only the owner of the function can be updated.


        Args:
            owner: Username of new owner of the function.


        Returns:
            A User-Defined Function (UDF) registered under a catalog + schema hierarchy.
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
        new_name: Optional[str] = None,
        owner: Optional[str] = None,
        comment: Optional[str] = None,
        properties: Optional[Dict[str, str]] = None,
        expiration_time: Optional[int] = None,
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
    def delete(self, force: Optional[bool] = None) -> None:
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
        comment: Optional[str] = None,
        properties: Optional[Dict[str, str]] = None,
        new_name: Optional[str] = None,
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

class ShareClient:
    def delete(self) -> None:
        """
        Deletes a share.


        Returns:
            None
        """
        ...
    def get(self, include_shared_data: Optional[bool] = None) -> Share:
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
        updates: Optional[List[DataObjectUpdate]] = None,
        new_name: Optional[str] = None,
        owner: Optional[str] = None,
        comment: Optional[str] = None,
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
        include_delta_metadata: Optional[bool] = None,
        include_browse: Optional[bool] = None,
        include_manifest_capabilities: Optional[bool] = None,
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
    def get(self, include_browse: Optional[bool] = None) -> Volume:
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
        new_name: Optional[str] = None,
        comment: Optional[str] = None,
        owner: Optional[str] = None,
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

class UnityCatalogClient:
    def __init__(self, base_url: str, token: Optional[str] = None) -> None: ...
    def create_catalog(
        self,
        name: str,
        comment: Optional[str] = None,
        properties: Optional[Dict[str, str]] = None,
        storage_root: Optional[str] = None,
        provider_name: Optional[str] = None,
        share_name: Optional[str] = None,
    ) -> Catalog:
        """
        Create a new catalog

        Creates a new catalog instance in the parent metastore if the caller is a metastore admin or has the
        CREATE_CATALOG privilege.


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
        comment: Optional[str] = None,
        read_only: Optional[bool] = None,
        skip_validation: Optional[bool] = None,
        azure_service_principal: Optional[AzureServicePrincipal] = None,
        azure_managed_identity: Optional[AzureManagedIdentity] = None,
        azure_storage_key: Optional[AzureStorageKey] = None,
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
        read_only: Optional[bool] = None,
        comment: Optional[str] = None,
        skip_validation: Optional[bool] = None,
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
    def create_function(
        self,
        name: str,
        catalog_name: str,
        schema_name: str,
        data_type: str,
        full_data_type: str,
        parameter_style: ParameterStyle,
        is_deterministic: bool,
        sql_data_access: SqlDataAccess,
        is_null_call: bool,
        security_type: SecurityType,
        routine_body: RoutineBody,
        input_params: Optional[FunctionParameterInfos] = None,
        routine_definition: Optional[str] = None,
        routine_body_language: Optional[str] = None,
        comment: Optional[str] = None,
        properties: Optional[Dict[str, str]] = None,
    ) -> Function:
        """
        Create a function

        Creates a new function. The caller must be a metastore admin or have the CREATE_FUNCTION privilege
        on the parent catalog and schema.


        Args:
            name: Name of function, relative to parent schema.
            catalog_name: Name of parent catalog.
            schema_name: Name of parent schema.
            data_type: Full data type specification of the return type of the function.
            full_data_type: Full data type specification as SQL/catalogString text.
            parameter_style: The parameter-passing style.
            is_deterministic: Indicates whether the function is deterministic.
            sql_data_access: SQL data access information.
            is_null_call: Indicates whether the function is null-calling.
            security_type: The security type of the function.
            routine_body: The routine body.
            input_params: The array of function parameter infos.
            routine_definition: Function body.
            routine_body_language: The language of the function routine body.
            comment: User-provided free-form text description.
            properties: A map of key-value properties attached to the securable.


        Returns:
            A User-Defined Function (UDF) registered under a catalog + schema hierarchy.
        """
        ...
    def create_recipient(
        self,
        name: str,
        authentication_type: AuthenticationType,
        owner: str,
        comment: Optional[str] = None,
        properties: Optional[Dict[str, str]] = None,
        expiration_time: Optional[int] = None,
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
        comment: Optional[str] = None,
        properties: Optional[Dict[str, str]] = None,
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
    def create_share(self, name: str, comment: Optional[str] = None) -> Share:
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
        columns: Optional[List[Column]] = None,
        storage_location: Optional[str] = None,
        comment: Optional[str] = None,
        properties: Optional[Dict[str, str]] = None,
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
        storage_location: Optional[str] = None,
        comment: Optional[str] = None,
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
    def list_catalogs(self, max_results: Optional[int] = None) -> List[Catalog]:
        """
        List catalogs

        Gets an array of catalogs in the metastore. If the caller is the metastore admin, all catalogs
        will be retrieved. Otherwise, only catalogs owned by the caller (or for which the caller has the
        USE_CATALOG privilege) will be retrieved. There is no guarantee of a specific ordering of the
        elements in the array.


        Args:
            max_results: The maximum number of results per page that should be returned.


        Returns:
            List of The catalogs returned.
        """
        ...
    def list_credentials(
        self, purpose: Optional[Purpose] = None, max_results: Optional[int] = None
    ) -> List[Credential]:
        """
        Args:
            purpose: Return only credentials for the specified purpose.
            max_results: The maximum number of results per page that should be returned.


        Returns:
            List of The credentials returned.
        """
        ...
    def list_external_locations(
        self, max_results: Optional[int] = None, include_browse: Optional[bool] = None
    ) -> List[ExternalLocation]:
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
    def list_functions(
        self,
        catalog_name: str,
        schema_name: str,
        max_results: Optional[int] = None,
        include_browse: Optional[bool] = None,
    ) -> List[Function]:
        """
        List functions

        List functions within the specified parent catalog and schema. If the caller is the metastore admin,
        all functions are returned in the response. Otherwise, the caller must have USE_CATALOG on the
        parent catalog and USE_SCHEMA on the parent schema, and the function must either be owned by the
        caller or have SELECT on the function.


        Args:
            catalog_name: Name of parent catalog for functions of interest.
            schema_name: Parent schema of functions.
            max_results: The maximum number of results per page that should be returned.
            include_browse: Whether to include functions in the response for which the principal can only
                            access selective metadata for.


        Returns:
            List of The functions returned.
        """
        ...
    def list_recipients(self, max_results: Optional[int] = None) -> List[Recipient]:
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
        max_results: Optional[int] = None,
        include_browse: Optional[bool] = None,
    ) -> List[Schema]:
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
    def list_shares(self, max_results: Optional[int] = None) -> List[Share]:
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
        max_results: Optional[int] = None,
        include_delta_metadata: Optional[bool] = None,
        omit_columns: Optional[bool] = None,
        omit_properties: Optional[bool] = None,
        omit_username: Optional[bool] = None,
        include_browse: Optional[bool] = None,
        include_manifest_capabilities: Optional[bool] = None,
    ) -> List[Table]:
        """
        Gets an array of all tables for the current metastore under the parent catalog and schema.

        The caller must be a metastore admin or an owner of (or have the SELECT privilege on) the table. For
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
        max_results: Optional[int] = None,
        include_browse: Optional[bool] = None,
    ) -> List[Volume]:
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
    def function(
        self, catalog_name: str, schema_name: str, function_name: str
    ) -> FunctionClient: ...
    def recipient(self, name: str) -> RecipientClient: ...
    def schema(self, catalog_name: str, schema_name: str) -> SchemaClient: ...
    def share(self, name: str) -> ShareClient: ...
    def table(self, name: str) -> TableClient: ...
    def volume(self, catalog_name: str, schema_name: str, volume_name: str) -> VolumeClient: ...

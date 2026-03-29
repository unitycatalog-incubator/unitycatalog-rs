// @generated — do not edit by hand.
import { fromBinary } from "@bufbuild/protobuf";
import {
  type Catalog,
  type Credential,
  type ExternalLocation,
  type Function,
  type Recipient,
  type Schema,
  type Share,
  type Table,
  type TemporaryCredential,
  type Volume,
  CatalogSchema,
  CredentialSchema,
  ExternalLocationSchema,
  FunctionSchema,
  RecipientSchema,
  SchemaSchema,
  ShareSchema,
  TableSchema,
  TemporaryCredentialSchema,
  VolumeSchema,
} from "./models";
import {
  NapiCatalogClient as NativeCatalogClient,
  NapiCredentialClient as NativeCredentialClient,
  NapiExternalLocationClient as NativeExternalLocationClient,
  NapiFunctionClient as NativeFunctionClient,
  NapiRecipientClient as NativeRecipientClient,
  NapiSchemaClient as NativeSchemaClient,
  NapiShareClient as NativeShareClient,
  NapiTableClient as NativeTableClient,
  NapiUnityCatalogClient as NativeClient,
  NapiVolumeClient as NativeVolumeClient,
} from "./native";

// ── UC error hierarchy ────────────────────────────────────────────────────────

/** Base class for all Unity Catalog errors. */
export class UnityCatalogError extends Error {
  readonly errorCode: string;
  constructor(message: string, errorCode: string) {
    super(message);
    this.name = "UnityCatalogError";
    this.errorCode = errorCode;
  }
}

export class NotFoundError extends UnityCatalogError {
  constructor(message: string) {
    super(message, "RESOURCE_NOT_FOUND");
    this.name = "NotFoundError";
  }
}

export class AlreadyExistsError extends UnityCatalogError {
  constructor(message: string) {
    super(message, "RESOURCE_ALREADY_EXISTS");
    this.name = "AlreadyExistsError";
  }
}

export class PermissionDeniedError extends UnityCatalogError {
  constructor(message: string) {
    super(message, "PERMISSION_DENIED");
    this.name = "PermissionDeniedError";
  }
}

export class UnauthenticatedError extends UnityCatalogError {
  constructor(message: string) {
    super(message, "UNAUTHENTICATED");
    this.name = "UnauthenticatedError";
  }
}

export class InvalidParameterError extends UnityCatalogError {
  constructor(message: string) {
    super(message, "INVALID_PARAMETER_VALUE");
    this.name = "InvalidParameterError";
  }
}

export class RequestLimitError extends UnityCatalogError {
  constructor(message: string) {
    super(message, "REQUEST_LIMIT_EXCEEDED");
    this.name = "RequestLimitError";
  }
}

export class InternalServerError extends UnityCatalogError {
  constructor(message: string) {
    super(message, "INTERNAL_ERROR");
    this.name = "InternalServerError";
  }
}

export class ServiceUnavailableError extends UnityCatalogError {
  constructor(message: string) {
    super(message, "TEMPORARILY_UNAVAILABLE");
    this.name = "ServiceUnavailableError";
  }
}

type UcErrorConstructor = new (message: string) => UnityCatalogError;

const UC_ERROR_MAP: Record<string, UcErrorConstructor> = {
  RESOURCE_NOT_FOUND: NotFoundError,
  RESOURCE_ALREADY_EXISTS: AlreadyExistsError,
  PERMISSION_DENIED: PermissionDeniedError,
  UNAUTHENTICATED: UnauthenticatedError,
  INVALID_PARAMETER_VALUE: InvalidParameterError,
  REQUEST_LIMIT_EXCEEDED: RequestLimitError,
  INTERNAL_ERROR: InternalServerError,
  TEMPORARILY_UNAVAILABLE: ServiceUnavailableError,
};

/**
 * Parse a native NAPI error that may carry a `UC:<CODE>:<message>` prefix
 * and re-throw as the appropriate typed subclass of `UnityCatalogError`.
 */
function parseNativeError(e: unknown): never {
  if (e instanceof Error) {
    const match = e.message.match(/^UC:([^:]+):([\s\S]*)$/);
    if (match) {
      const [, code, message] = match;
      const Ctor = UC_ERROR_MAP[code] ?? UnityCatalogError;
      throw new Ctor(message);
    }
  }
  throw e;
}

// ── end UC error hierarchy ─────────────────────────────────────────────────────

export interface ListSharesOptions {
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
}

export interface CreateShareOptions {
  /** User-provided free-form text description. */
  comment?: string;
}

export interface GetShareOptions {
  /** Query for data to include in the share. */
  includeSharedData?: boolean;
}

export interface UpdateShareOptions {
  /** A new name for the share. */
  newName?: string;
  /** Owner of the share. */
  owner?: string;
  /** User-provided free-form text description. */
  comment?: string;
}

export interface GetPermissionsOptions {
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
}

export interface UpdatePermissionsOptions {
  /** Whether to return the latest permissions list of the share in the response. */
  omitPermissionsList?: boolean;
}

export interface ListTableSummariesOptions {
  /** A sql LIKE pattern (% and _) for schema names. All schemas will be returned if not set or empty. */
  schemaNamePattern?: string;
  /** A sql LIKE pattern (% and _) for table names. All tables will be returned if not set or empty. */
  tableNamePattern?: string;
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
  /** Whether to include a manifest containing capabilities the table has. */
  includeManifestCapabilities?: boolean;
}

export interface ListTablesOptions {
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
  /** Whether delta metadata should be included in the response. */
  includeDeltaMetadata?: boolean;
  /** Whether to omit the columns of the table from the response or not. */
  omitColumns?: boolean;
  /** Whether to omit the properties of the table from the response or not. */
  omitProperties?: boolean;
  /** Whether to omit the username of the table (e.g. owner, updated_by, created_by) from the response or not. */
  omitUsername?: boolean;
  /** Whether to include tables in the response for which the principal can only access selective metadata for */
  includeBrowse?: boolean;
  /** Whether to include a manifest containing capabilities the table has. */
  includeManifestCapabilities?: boolean;
}

export interface CreateTableOptions {
  /** Storage root URL for external table. */
  storageLocation?: string;
  /** User-provided free-form text description. */
  comment?: string;
  /** A map of key-value properties attached to the securable. */
  properties?: Record<string, string>;
}

export interface GetTableOptions {
  /** Whether delta metadata should be included in the response. */
  includeDeltaMetadata?: boolean;
  /** Whether to include tables in the response for which the principal can only access selective metadata for */
  includeBrowse?: boolean;
  /** Whether to include a manifest containing capabilities the table has. */
  includeManifestCapabilities?: boolean;
}

export interface GenerateTemporaryPathCredentialsOptions {
  dryRun?: boolean;
}

export interface ListCatalogsOptions {
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
}

export interface CreateCatalogOptions {
  /** User-provided free-form text description. */
  comment?: string;
  /** A map of key-value properties attached to the securable. */
  properties?: Record<string, string>;
  /** Storage root URL for managed tables within catalog. */
  storageRoot?: string;
  /** The name of delta sharing provider.
   * 
   *  A Delta Sharing catalog is a catalog that is based on a Delta share on a remote sharing server. */
  providerName?: string;
  /** The name of the share under the share provider. */
  shareName?: string;
}

export interface GetCatalogOptions {
  /** Whether to include catalogs in the response for which the principal can only access selective metadata for */
  includeBrowse?: boolean;
}

export interface UpdateCatalogOptions {
  /** Username of new owner of catalog. */
  owner?: string;
  /** User-provided free-form text description. */
  comment?: string;
  /** A map of key-value properties attached to the securable.
   * 
   *  When provided in update request, the specified properties will override the existing properties.
   *  To add and remove properties, one would need to perform a read-modify-write. */
  properties?: Record<string, string>;
  /** Name of catalog. */
  newName?: string;
}

export interface DeleteCatalogOptions {
  /** Force deletion even if the catalog is not empty. */
  force?: boolean;
}

export interface ListCredentialsOptions {
  /** Return only credentials for the specified purpose. */
  purpose?: number;
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
}

export interface CreateCredentialOptions {
  /** Comment associated with the credential. */
  comment?: string;
  /** Whether the credential is usable only for read operations. Only applicable when purpose is STORAGE. */
  readOnly?: boolean;
  /** Supplying true to this argument skips validation of the created set of credentials. */
  skipValidation?: boolean;
}

export interface UpdateCredentialOptions {
  /** Name of credential. */
  newName?: string;
  /** Comment associated with the credential. */
  comment?: string;
  /** Whether the credential is usable only for read operations. Only applicable when purpose is STORAGE. */
  readOnly?: boolean;
  /** Username of current owner of credential. */
  owner?: string;
  /** Supply true to this argument to skip validation of the updated credential. */
  skipValidation?: boolean;
  /** Force an update even if there are dependent services (when purpose is SERVICE)
   *  or dependent external locations and external tables (when purpose is STORAGE). */
  force?: boolean;
}

export interface ListFunctionsOptions {
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
  /** Whether to include functions in the response for which the principal can only access selective metadata for. */
  includeBrowse?: boolean;
}

export interface CreateFunctionOptions {
  /** Function body. */
  routineDefinition?: string;
  /** The language of the function routine body. */
  routineBodyLanguage?: string;
  /** User-provided free-form text description. */
  comment?: string;
  /** A map of key-value properties attached to the securable. */
  properties?: Record<string, string>;
}

export interface UpdateFunctionOptions {
  /** Username of new owner of the function. */
  owner?: string;
}

export interface DeleteFunctionOptions {
  /** Force deletion even if the function is not empty. */
  force?: boolean;
}

export interface ListVolumesOptions {
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
  /** Whether to include schemas in the response for which the principal can only access selective metadata for */
  includeBrowse?: boolean;
}

export interface CreateVolumeOptions {
  /** The storage location on the cloud */
  storageLocation?: string;
  /** The storage location on the cloud */
  comment?: string;
}

export interface GetVolumeOptions {
  /** Whether to include schemas in the response for which the principal can only access selective metadata for */
  includeBrowse?: boolean;
}

export interface UpdateVolumeOptions {
  /** New name for the volume. */
  newName?: string;
  /** The comment attached to the volume */
  comment?: string;
  /** The identifier of the user who owns the volume */
  owner?: string;
}

export interface ListSchemasOptions {
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
  /** Whether to include schemas in the response for which the principal can only access selective metadata for */
  includeBrowse?: boolean;
}

export interface CreateSchemaOptions {
  /** User-provided free-form text description. */
  comment?: string;
  /** A map of key-value properties attached to the securable. */
  properties?: Record<string, string>;
}

export interface UpdateSchemaOptions {
  /** User-provided free-form text description. */
  comment?: string;
  /** A map of key-value properties attached to the securable.
   * 
   *  When provided in update request, the specified properties will override the existing properties.
   *  To add and remove properties, one would need to perform a read-modify-write. */
  properties?: Record<string, string>;
  /** Name of schema. */
  newName?: string;
}

export interface DeleteSchemaOptions {
  /** Force deletion even if the schema is not empty. */
  force?: boolean;
}

export interface ListRecipientsOptions {
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
}

export interface CreateRecipientOptions {
  /** Description about the recipient. */
  comment?: string;
  /** Recipient properties as map of string key-value pairs.
   * 
   *  When provided in update request, the specified properties will override the existing properties.
   *  To add and remove properties, one would need to perform a read-modify-write. */
  properties?: Record<string, string>;
  /** Expiration timestamp of the token, in epoch milliseconds. */
  expirationTime?: number;
}

export interface UpdateRecipientOptions {
  /** New name for the recipient */
  newName?: string;
  /** Username of the recipient owner. */
  owner?: string;
  /** Description about the recipient. */
  comment?: string;
  /** Recipient properties as map of string key-value pairs.
   * 
   *  When provided in update request, the specified properties will override the existing properties.
   *  To add and remove properties, one would need to perform a read-modify-write. */
  properties?: Record<string, string>;
  /** Expiration timestamp of the token, in epoch milliseconds. */
  expirationTime?: number;
}

export interface ListExternalLocationsOptions {
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
  /** Whether to include schemas in the response for which the principal can only access selective metadata for */
  includeBrowse?: boolean;
}

export interface CreateExternalLocationOptions {
  /** Indicates whether the external location is read-only. */
  readOnly?: boolean;
  /** User-provided free-form text description. */
  comment?: string;
  /** Skips validation of the storage credential associated with the external location. */
  skipValidation?: boolean;
}

export interface UpdateExternalLocationOptions {
  /** Path URL of the external location. */
  url?: string;
  /** Name of the storage credential used with this location. */
  credentialName?: string;
  /** Indicates whether the external location is read-only. */
  readOnly?: boolean;
  /** owner of the external location. */
  owner?: string;
  /** User-provided free-form text description. */
  comment?: string;
  /** new name of the external location. */
  newName?: string;
  /** force update of the external location. */
  force?: boolean;
  /** Skips validation of the storage credential associated with the external location. */
  skipValidation?: boolean;
}

export interface DeleteExternalLocationOptions {
  /** Force deletion even if the external location is not empty. */
  force?: boolean;
}

export class ShareClient {
  private readonly inner: NativeShareClient;

  /** @internal */
  constructor(inner: NativeShareClient) {
    this.inner = inner;
  }

  /**
     * Get a share by name.
     */
  async get(options?: GetShareOptions): Promise<Share> {
    const { includeSharedData } = options || {};
    try {
      return fromBinary(ShareSchema, await this.inner.get(includeSharedData));
    } catch (e) { parseNativeError(e); }
  }

  /**
     * Update a share.
     */
  async update(options?: UpdateShareOptions): Promise<Share> {
    const { newName, owner, comment } = options || {};
    try {
      return fromBinary(ShareSchema, await this.inner.update(newName, owner, comment));
    } catch (e) { parseNativeError(e); }
  }

  /**
     * Deletes a share.
     */
  async delete(): Promise<void> {
    try {
      await this.inner.delete();
    } catch (e) { parseNativeError(e); }
  }

}

export class TableClient {
  private readonly inner: NativeTableClient;

  /** @internal */
  constructor(inner: NativeTableClient) {
    this.inner = inner;
  }

  /**
     * Get a table
     */
  async get(options?: GetTableOptions): Promise<Table> {
    const { includeDeltaMetadata, includeBrowse, includeManifestCapabilities } = options || {};
    try {
      return fromBinary(TableSchema, await this.inner.get(includeDeltaMetadata, includeBrowse, includeManifestCapabilities));
    } catch (e) { parseNativeError(e); }
  }

  /**
     * Delete a table
     */
  async delete(): Promise<void> {
    try {
      await this.inner.delete();
    } catch (e) { parseNativeError(e); }
  }

}

export class CatalogClient {
  private readonly inner: NativeCatalogClient;

  /** @internal */
  constructor(inner: NativeCatalogClient) {
    this.inner = inner;
  }

  /**
     * Get a catalog
     * 
     * Gets the specified catalog in a metastore. The caller must be a metastore admin,
     * the owner of the catalog, or a user that has the USE_CATALOG privilege set for their account.
     */
  async get(options?: GetCatalogOptions): Promise<Catalog> {
    const { includeBrowse } = options || {};
    try {
      return fromBinary(CatalogSchema, await this.inner.get(includeBrowse));
    } catch (e) { parseNativeError(e); }
  }

  /**
     * Update a catalog
     * 
     * Updates the catalog that matches the supplied name. The caller must be either
     * the owner of the catalog, or a metastore admin (when changing the owner field of the catalog).
     */
  async update(options?: UpdateCatalogOptions): Promise<Catalog> {
    const { owner, comment, properties, newName } = options || {};
    try {
      return fromBinary(CatalogSchema, await this.inner.update(owner, comment, properties, newName));
    } catch (e) { parseNativeError(e); }
  }

  /**
     * Delete a catalog
     * 
     * Deletes the catalog that matches the supplied name. The caller must
     * be a metastore admin or the owner of the catalog.
     */
  async delete(options?: DeleteCatalogOptions): Promise<void> {
    const { force } = options || {};
    try {
      await this.inner.delete(force);
    } catch (e) { parseNativeError(e); }
  }

}

export class CredentialClient {
  private readonly inner: NativeCredentialClient;

  /** @internal */
  constructor(inner: NativeCredentialClient) {
    this.inner = inner;
  }

  async get(): Promise<Credential> {
    try {
      return fromBinary(CredentialSchema, await this.inner.get());
    } catch (e) { parseNativeError(e); }
  }

  async update(options?: UpdateCredentialOptions): Promise<Credential> {
    const { newName, comment, readOnly, owner, skipValidation, force } = options || {};
    try {
      return fromBinary(CredentialSchema, await this.inner.update(newName, comment, readOnly, owner, skipValidation, force));
    } catch (e) { parseNativeError(e); }
  }

  async delete(): Promise<void> {
    try {
      await this.inner.delete();
    } catch (e) { parseNativeError(e); }
  }

}

export class FunctionClient {
  private readonly inner: NativeFunctionClient;

  /** @internal */
  constructor(inner: NativeFunctionClient) {
    this.inner = inner;
  }

  /**
     * Get a function
     * 
     * Gets a function from within a parent catalog and schema. For the fetch to succeed,
     * the caller must be a metastore admin, the owner of the function, or have SELECT on
     * the function.
     */
  async get(): Promise<Function> {
    try {
      return fromBinary(FunctionSchema, await this.inner.get());
    } catch (e) { parseNativeError(e); }
  }

  /**
     * Update a function
     * 
     * Updates the function that matches the supplied name. Only the owner of the function
     * can be updated.
     */
  async update(options?: UpdateFunctionOptions): Promise<Function> {
    const { owner } = options || {};
    try {
      return fromBinary(FunctionSchema, await this.inner.update(owner));
    } catch (e) { parseNativeError(e); }
  }

  /**
     * Delete a function
     * 
     * Deletes the function that matches the supplied name. For the deletion to succeed,
     * the caller must be the owner of the function.
     */
  async delete(options?: DeleteFunctionOptions): Promise<void> {
    const { force } = options || {};
    try {
      await this.inner.delete(force);
    } catch (e) { parseNativeError(e); }
  }

}

export class VolumeClient {
  private readonly inner: NativeVolumeClient;

  /** @internal */
  constructor(inner: NativeVolumeClient) {
    this.inner = inner;
  }

  async get(options?: GetVolumeOptions): Promise<Volume> {
    const { includeBrowse } = options || {};
    try {
      return fromBinary(VolumeSchema, await this.inner.get(includeBrowse));
    } catch (e) { parseNativeError(e); }
  }

  async update(options?: UpdateVolumeOptions): Promise<Volume> {
    const { newName, comment, owner } = options || {};
    try {
      return fromBinary(VolumeSchema, await this.inner.update(newName, comment, owner));
    } catch (e) { parseNativeError(e); }
  }

  async delete(): Promise<void> {
    try {
      await this.inner.delete();
    } catch (e) { parseNativeError(e); }
  }

}

export class SchemaClient {
  private readonly inner: NativeSchemaClient;

  /** @internal */
  constructor(inner: NativeSchemaClient) {
    this.inner = inner;
  }

  /**
     * Gets the specified schema within the metastore.
     * The caller must be a metastore admin, the owner of the schema,
     * or a user that has the USE_SCHEMA privilege on the schema.
     */
  async get(): Promise<Schema> {
    try {
      return fromBinary(SchemaSchema, await this.inner.get());
    } catch (e) { parseNativeError(e); }
  }

  /**
     * Updates a schema for a catalog. The caller must be the owner of the schema or a metastore admin.
     * If the caller is a metastore admin, only the owner field can be changed in the update.
     * If the name field must be updated, the caller must be a metastore admin or have the CREATE_SCHEMA
     * privilege on the parent catalog.
     */
  async update(options?: UpdateSchemaOptions): Promise<Schema> {
    const { comment, properties, newName } = options || {};
    try {
      return fromBinary(SchemaSchema, await this.inner.update(comment, properties, newName));
    } catch (e) { parseNativeError(e); }
  }

  /**
     * Deletes the specified schema from the parent catalog. The caller must be the owner
     * of the schema or an owner of the parent catalog.
     */
  async delete(options?: DeleteSchemaOptions): Promise<void> {
    const { force } = options || {};
    try {
      await this.inner.delete(force);
    } catch (e) { parseNativeError(e); }
  }

}

export class RecipientClient {
  private readonly inner: NativeRecipientClient;

  /** @internal */
  constructor(inner: NativeRecipientClient) {
    this.inner = inner;
  }

  /**
     * Get a recipient by name.
     */
  async get(): Promise<Recipient> {
    try {
      return fromBinary(RecipientSchema, await this.inner.get());
    } catch (e) { parseNativeError(e); }
  }

  /**
     * Update a recipient.
     */
  async update(options?: UpdateRecipientOptions): Promise<Recipient> {
    const { newName, owner, comment, properties, expirationTime } = options || {};
    try {
      return fromBinary(RecipientSchema, await this.inner.update(newName, owner, comment, properties, expirationTime));
    } catch (e) { parseNativeError(e); }
  }

  /**
     * Delete a recipient.
     */
  async delete(): Promise<void> {
    try {
      await this.inner.delete();
    } catch (e) { parseNativeError(e); }
  }

}

export class ExternalLocationClient {
  private readonly inner: NativeExternalLocationClient;

  /** @internal */
  constructor(inner: NativeExternalLocationClient) {
    this.inner = inner;
  }

  /**
     * Get an external location
     */
  async get(): Promise<ExternalLocation> {
    try {
      return fromBinary(ExternalLocationSchema, await this.inner.get());
    } catch (e) { parseNativeError(e); }
  }

  /**
     * Update an external location
     */
  async update(options?: UpdateExternalLocationOptions): Promise<ExternalLocation> {
    const { url, credentialName, readOnly, owner, comment, newName, force, skipValidation } = options || {};
    try {
      return fromBinary(ExternalLocationSchema, await this.inner.update(url, credentialName, readOnly, owner, comment, newName, force, skipValidation));
    } catch (e) { parseNativeError(e); }
  }

  /**
     * Delete an external location
     */
  async delete(options?: DeleteExternalLocationOptions): Promise<void> {
    const { force } = options || {};
    try {
      await this.inner.delete(force);
    } catch (e) { parseNativeError(e); }
  }

}

export class UnityCatalogClient {
  private readonly inner: NativeClient;

  constructor(url: string, token?: string) {
    this.inner = NativeClient.fromUrl(url, token);
  }

  /**
     * List catalogs
     * 
     * Gets an array of catalogs in the metastore. If the caller is the metastore admin,
     * all catalogs will be retrieved. Otherwise, only catalogs owned by the caller
     * (or for which the caller has the USE_CATALOG privilege) will be retrieved.
     * There is no guarantee of a specific ordering of the elements in the array.
     */
  async listCatalogs(options?: ListCatalogsOptions): Promise<Catalog[]> {
    const { maxResults } = options || {};
    try {
      return (await this.inner.listCatalogs(maxResults)).map((data) =>
        fromBinary(CatalogSchema, data),
      );
    } catch (e) { parseNativeError(e); }
  }

  /**
     * Create a new catalog
     * 
     * Creates a new catalog instance in the parent metastore if the caller
     * is a metastore admin or has the CREATE_CATALOG privilege.
     */
  async createCatalog(name: string, options?: CreateCatalogOptions): Promise<Catalog> {
    const { comment, properties, storageRoot, providerName, shareName } = options || {};
    try {
      return fromBinary(CatalogSchema, await this.inner.createCatalog(name, comment, properties, storageRoot, providerName, shareName));
    } catch (e) { parseNativeError(e); }
  }

  catalog(name: string): CatalogClient {
    return new CatalogClient(this.inner.catalog(name));
  }

  async listCredentials(options?: ListCredentialsOptions): Promise<Credential[]> {
    const { purpose, maxResults } = options || {};
    try {
      return (await this.inner.listCredentials(purpose, maxResults)).map((data) =>
        fromBinary(CredentialSchema, data),
      );
    } catch (e) { parseNativeError(e); }
  }

  async createCredential(name: string, purpose: number, options?: CreateCredentialOptions): Promise<Credential> {
    const { comment, readOnly, skipValidation } = options || {};
    try {
      return fromBinary(CredentialSchema, await this.inner.createCredential(name, purpose, comment, readOnly, skipValidation));
    } catch (e) { parseNativeError(e); }
  }

  credential(name: string): CredentialClient {
    return new CredentialClient(this.inner.credential(name));
  }

  /**
     * List external locations
     */
  async listExternalLocations(options?: ListExternalLocationsOptions): Promise<ExternalLocation[]> {
    const { maxResults, includeBrowse } = options || {};
    try {
      return (await this.inner.listExternalLocations(maxResults, includeBrowse)).map((data) =>
        fromBinary(ExternalLocationSchema, data),
      );
    } catch (e) { parseNativeError(e); }
  }

  /**
     * Create a new external location
     */
  async createExternalLocation(name: string, url: string, credentialName: string, options?: CreateExternalLocationOptions): Promise<ExternalLocation> {
    const { readOnly, comment, skipValidation } = options || {};
    try {
      return fromBinary(ExternalLocationSchema, await this.inner.createExternalLocation(name, url, credentialName, readOnly, comment, skipValidation));
    } catch (e) { parseNativeError(e); }
  }

  externalLocation(name: string): ExternalLocationClient {
    return new ExternalLocationClient(this.inner.externalLocation(name));
  }

  /**
     * List functions
     * 
     * List functions within the specified parent catalog and schema. If the caller is the metastore
     * admin, all functions are returned in the response. Otherwise, the caller must have USE_CATALOG
     * on the parent catalog and USE_SCHEMA on the parent schema, and the function must either be
     * owned by the caller or have SELECT on the function.
     */
  async listFunctions(catalogName: string, schemaName: string, options?: ListFunctionsOptions): Promise<Function[]> {
    const { maxResults, includeBrowse } = options || {};
    try {
      return (await this.inner.listFunctions(catalogName, schemaName, maxResults, includeBrowse)).map((data) =>
        fromBinary(FunctionSchema, data),
      );
    } catch (e) { parseNativeError(e); }
  }

  /**
     * Create a function
     * 
     * Creates a new function. The caller must be a metastore admin or have the CREATE_FUNCTION
     * privilege on the parent catalog and schema.
     */
  async createFunction(name: string, catalogName: string, schemaName: string, dataType: string, fullDataType: string, parameterStyle: number, isDeterministic: boolean, sqlDataAccess: number, isNullCall: boolean, securityType: number, routineBody: number, options?: CreateFunctionOptions): Promise<Function> {
    const { routineDefinition, routineBodyLanguage, comment, properties } = options || {};
    try {
      return fromBinary(FunctionSchema, await this.inner.createFunction(name, catalogName, schemaName, dataType, fullDataType, parameterStyle, isDeterministic, sqlDataAccess, isNullCall, securityType, routineBody, routineDefinition, routineBodyLanguage, comment, properties));
    } catch (e) { parseNativeError(e); }
  }

  function(catalogName: string, schemaName: string, functionName: string): FunctionClient {
    return new FunctionClient(this.inner.function(catalogName, schemaName, functionName));
  }

  /**
     * List recipients.
     */
  async listRecipients(options?: ListRecipientsOptions): Promise<Recipient[]> {
    const { maxResults } = options || {};
    try {
      return (await this.inner.listRecipients(maxResults)).map((data) =>
        fromBinary(RecipientSchema, data),
      );
    } catch (e) { parseNativeError(e); }
  }

  /**
     * Create a new recipient.
     */
  async createRecipient(name: string, authenticationType: number, owner: string, options?: CreateRecipientOptions): Promise<Recipient> {
    const { comment, properties, expirationTime } = options || {};
    try {
      return fromBinary(RecipientSchema, await this.inner.createRecipient(name, authenticationType, owner, comment, properties, expirationTime));
    } catch (e) { parseNativeError(e); }
  }

  recipient(name: string): RecipientClient {
    return new RecipientClient(this.inner.recipient(name));
  }

  /**
     * Gets an array of schemas for a catalog in the metastore. If the caller is the metastore
     * admin or the owner of the parent catalog, all schemas for the catalog will be retrieved.
     * Otherwise, only schemas owned by the caller (or for which the caller has the USE_SCHEMA privilege)
     * will be retrieved. There is no guarantee of a specific ordering of the elements in the array.
     */
  async listSchemas(catalogName: string, options?: ListSchemasOptions): Promise<Schema[]> {
    const { maxResults, includeBrowse } = options || {};
    try {
      return (await this.inner.listSchemas(catalogName, maxResults, includeBrowse)).map((data) =>
        fromBinary(SchemaSchema, data),
      );
    } catch (e) { parseNativeError(e); }
  }

  /**
     * Creates a new schema for catalog in the Metatastore. The caller must be a metastore admin,
     * or have the CREATE_SCHEMA privilege in the parent catalog.
     */
  async createSchema(name: string, catalogName: string, options?: CreateSchemaOptions): Promise<Schema> {
    const { comment, properties } = options || {};
    try {
      return fromBinary(SchemaSchema, await this.inner.createSchema(name, catalogName, comment, properties));
    } catch (e) { parseNativeError(e); }
  }

  schema(catalogName: string, schemaName: string): SchemaClient {
    return new SchemaClient(this.inner.schema(catalogName, schemaName));
  }

  /**
     * List shares.
     */
  async listShares(options?: ListSharesOptions): Promise<Share[]> {
    const { maxResults } = options || {};
    try {
      return (await this.inner.listShares(maxResults)).map((data) =>
        fromBinary(ShareSchema, data),
      );
    } catch (e) { parseNativeError(e); }
  }

  /**
     * Create a new share.
     */
  async createShare(name: string, options?: CreateShareOptions): Promise<Share> {
    const { comment } = options || {};
    try {
      return fromBinary(ShareSchema, await this.inner.createShare(name, comment));
    } catch (e) { parseNativeError(e); }
  }

  share(name: string): ShareClient {
    return new ShareClient(this.inner.share(name));
  }

  /**
     * Gets an array of all tables for the current metastore under the parent catalog and schema.
     * 
     * The caller must be a metastore admin or an owner of (or have the SELECT privilege on) the table.
     * For the latter case, the caller must also be the owner or have the USE_CATALOG privilege on the
     * parent catalog and the USE_SCHEMA privilege on the parent schema. There is no guarantee of a
     * specific ordering of the elements in the array.
     */
  async listTables(catalogName: string, schemaName: string, options?: ListTablesOptions): Promise<Table[]> {
    const { maxResults, includeDeltaMetadata, omitColumns, omitProperties, omitUsername, includeBrowse, includeManifestCapabilities } = options || {};
    try {
      return (await this.inner.listTables(catalogName, schemaName, maxResults, includeDeltaMetadata, omitColumns, omitProperties, omitUsername, includeBrowse, includeManifestCapabilities)).map((data) =>
        fromBinary(TableSchema, data),
      );
    } catch (e) { parseNativeError(e); }
  }

  /**
     * Create a table
     */
  async createTable(name: string, schemaName: string, catalogName: string, tableType: number, dataSourceFormat: number, options?: CreateTableOptions): Promise<Table> {
    const { storageLocation, comment, properties } = options || {};
    try {
      return fromBinary(TableSchema, await this.inner.createTable(name, schemaName, catalogName, tableType, dataSourceFormat, storageLocation, comment, properties));
    } catch (e) { parseNativeError(e); }
  }

  table(name: string): TableClient {
    return new TableClient(this.inner.table(name));
  }

  /**
     * Lists volumes.
     */
  async listVolumes(catalogName: string, schemaName: string, options?: ListVolumesOptions): Promise<Volume[]> {
    const { maxResults, includeBrowse } = options || {};
    try {
      return (await this.inner.listVolumes(catalogName, schemaName, maxResults, includeBrowse)).map((data) =>
        fromBinary(VolumeSchema, data),
      );
    } catch (e) { parseNativeError(e); }
  }

  async createVolume(catalogName: string, schemaName: string, name: string, volumeType: number, options?: CreateVolumeOptions): Promise<Volume> {
    const { storageLocation, comment } = options || {};
    try {
      return fromBinary(VolumeSchema, await this.inner.createVolume(catalogName, schemaName, name, volumeType, storageLocation, comment));
    } catch (e) { parseNativeError(e); }
  }

  volume(catalogName: string, schemaName: string, volumeName: string): VolumeClient {
    return new VolumeClient(this.inner.volume(catalogName, schemaName, volumeName));
  }

}

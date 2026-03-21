// @generated — do not edit by hand.
import { fromBinary } from "@bufbuild/protobuf";
import {
  type Catalog,
  type Credential,
  type ExternalLocation,
  type Recipient,
  type Schema,
  type Share,
  type Table,
  type TemporaryCredential,
  type Volume,
  CatalogSchema,
  CredentialSchema,
  ExternalLocationSchema,
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
  NapiRecipientClient as NativeRecipientClient,
  NapiSchemaClient as NativeSchemaClient,
  NapiShareClient as NativeShareClient,
  NapiTableClient as NativeTableClient,
  NapiUnityCatalogClient as NativeClient,
  NapiVolumeClient as NativeVolumeClient,
} from "./native";

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

export interface GenerateTemporaryPathCredentialsOptions {
  dryRun?: boolean;
}

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

export class VolumeClient {
  private readonly inner: NativeVolumeClient;

  /** @internal */
  constructor(inner: NativeVolumeClient) {
    this.inner = inner;
  }

  async get(options?: GetVolumeOptions): Promise<Volume> {
    const { includeBrowse } = options || {};
    return fromBinary(VolumeSchema, await this.inner.get(includeBrowse));
  }

  async update(options?: UpdateVolumeOptions): Promise<Volume> {
    const { newName, comment, owner } = options || {};
    return fromBinary(VolumeSchema, await this.inner.update(newName, comment, owner));
  }

  async delete(): Promise<void> {
    await this.inner.delete();
  }

}

export class CredentialClient {
  private readonly inner: NativeCredentialClient;

  /** @internal */
  constructor(inner: NativeCredentialClient) {
    this.inner = inner;
  }

  async get(): Promise<Credential> {
    return fromBinary(CredentialSchema, await this.inner.get());
  }

  async update(options?: UpdateCredentialOptions): Promise<Credential> {
    const { newName, comment, readOnly, owner, skipValidation, force } = options || {};
    return fromBinary(CredentialSchema, await this.inner.update(newName, comment, readOnly, owner, skipValidation, force));
  }

  async delete(): Promise<void> {
    await this.inner.delete();
  }

}

export class ShareClient {
  private readonly inner: NativeShareClient;

  /** @internal */
  constructor(inner: NativeShareClient) {
    this.inner = inner;
  }

  async get(options?: GetShareOptions): Promise<Share> {
    const { includeSharedData } = options || {};
    return fromBinary(ShareSchema, await this.inner.get(includeSharedData));
  }

  async update(options?: UpdateShareOptions): Promise<Share> {
    const { newName, owner, comment } = options || {};
    return fromBinary(ShareSchema, await this.inner.update(newName, owner, comment));
  }

  async delete(): Promise<void> {
    await this.inner.delete();
  }

}

export class TableClient {
  private readonly inner: NativeTableClient;

  /** @internal */
  constructor(inner: NativeTableClient) {
    this.inner = inner;
  }

  async get(options?: GetTableOptions): Promise<Table> {
    const { includeDeltaMetadata, includeBrowse, includeManifestCapabilities } = options || {};
    return fromBinary(TableSchema, await this.inner.get(includeDeltaMetadata, includeBrowse, includeManifestCapabilities));
  }

  async delete(): Promise<void> {
    await this.inner.delete();
  }

}

export class SchemaClient {
  private readonly inner: NativeSchemaClient;

  /** @internal */
  constructor(inner: NativeSchemaClient) {
    this.inner = inner;
  }

  async get(): Promise<Schema> {
    return fromBinary(SchemaSchema, await this.inner.get());
  }

  async update(options?: UpdateSchemaOptions): Promise<Schema> {
    const { comment, properties, newName } = options || {};
    return fromBinary(SchemaSchema, await this.inner.update(comment, properties, newName));
  }

  async delete(options?: DeleteSchemaOptions): Promise<void> {
    const { force } = options || {};
    await this.inner.delete(force);
  }

}

export class ExternalLocationClient {
  private readonly inner: NativeExternalLocationClient;

  /** @internal */
  constructor(inner: NativeExternalLocationClient) {
    this.inner = inner;
  }

  async get(): Promise<ExternalLocation> {
    return fromBinary(ExternalLocationSchema, await this.inner.get());
  }

  async update(options?: UpdateExternalLocationOptions): Promise<ExternalLocation> {
    const { url, credentialName, readOnly, owner, comment, newName, force, skipValidation } = options || {};
    return fromBinary(ExternalLocationSchema, await this.inner.update(url, credentialName, readOnly, owner, comment, newName, force, skipValidation));
  }

  async delete(options?: DeleteExternalLocationOptions): Promise<void> {
    const { force } = options || {};
    await this.inner.delete(force);
  }

}

export class CatalogClient {
  private readonly inner: NativeCatalogClient;

  /** @internal */
  constructor(inner: NativeCatalogClient) {
    this.inner = inner;
  }

  async get(options?: GetCatalogOptions): Promise<Catalog> {
    const { includeBrowse } = options || {};
    return fromBinary(CatalogSchema, await this.inner.get(includeBrowse));
  }

  async update(options?: UpdateCatalogOptions): Promise<Catalog> {
    const { owner, comment, properties, newName } = options || {};
    return fromBinary(CatalogSchema, await this.inner.update(owner, comment, properties, newName));
  }

  async delete(options?: DeleteCatalogOptions): Promise<void> {
    const { force } = options || {};
    await this.inner.delete(force);
  }

}

export class RecipientClient {
  private readonly inner: NativeRecipientClient;

  /** @internal */
  constructor(inner: NativeRecipientClient) {
    this.inner = inner;
  }

  async get(): Promise<Recipient> {
    return fromBinary(RecipientSchema, await this.inner.get());
  }

  async update(options?: UpdateRecipientOptions): Promise<Recipient> {
    const { newName, owner, comment, properties, expirationTime } = options || {};
    return fromBinary(RecipientSchema, await this.inner.update(newName, owner, comment, properties, expirationTime));
  }

  async delete(): Promise<void> {
    await this.inner.delete();
  }

}

export class UnityCatalogClient {
  private readonly inner: NativeClient;

  constructor(url: string, token?: string) {
    this.inner = NativeClient.fromUrl(url, token);
  }

  async listCatalogs(options?: ListCatalogsOptions): Promise<Catalog[]> {
    const { maxResults } = options || {};
    return (await this.inner.listCatalogs(maxResults)).map((data) =>
      fromBinary(CatalogSchema, data),
    );
  }

  async createCatalog(name: string, options?: CreateCatalogOptions): Promise<Catalog> {
    const { comment, properties, storageRoot, providerName, shareName } = options || {};
    return fromBinary(CatalogSchema, await this.inner.createCatalog(name, comment, properties, storageRoot, providerName, shareName));
  }

  catalog(name: string): CatalogClient {
    return new CatalogClient(this.inner.catalog(name));
  }

  async listCredentials(options?: ListCredentialsOptions): Promise<Credential[]> {
    const { purpose, maxResults } = options || {};
    return (await this.inner.listCredentials(purpose, maxResults)).map((data) =>
      fromBinary(CredentialSchema, data),
    );
  }

  async createCredential(name: string, purpose: number, options?: CreateCredentialOptions): Promise<Credential> {
    const { comment, readOnly, skipValidation } = options || {};
    return fromBinary(CredentialSchema, await this.inner.createCredential(name, purpose, comment, readOnly, skipValidation));
  }

  credential(name: string): CredentialClient {
    return new CredentialClient(this.inner.credential(name));
  }

  async listExternalLocations(options?: ListExternalLocationsOptions): Promise<ExternalLocation[]> {
    const { maxResults, includeBrowse } = options || {};
    return (await this.inner.listExternalLocations(maxResults, includeBrowse)).map((data) =>
      fromBinary(ExternalLocationSchema, data),
    );
  }

  async createExternalLocation(name: string, url: string, credentialName: string, options?: CreateExternalLocationOptions): Promise<ExternalLocation> {
    const { readOnly, comment, skipValidation } = options || {};
    return fromBinary(ExternalLocationSchema, await this.inner.createExternalLocation(name, url, credentialName, readOnly, comment, skipValidation));
  }

  externalLocation(name: string): ExternalLocationClient {
    return new ExternalLocationClient(this.inner.externalLocation(name));
  }

  async listRecipients(options?: ListRecipientsOptions): Promise<Recipient[]> {
    const { maxResults } = options || {};
    return (await this.inner.listRecipients(maxResults)).map((data) =>
      fromBinary(RecipientSchema, data),
    );
  }

  async createRecipient(name: string, authenticationType: number, owner: string, options?: CreateRecipientOptions): Promise<Recipient> {
    const { comment, properties, expirationTime } = options || {};
    return fromBinary(RecipientSchema, await this.inner.createRecipient(name, authenticationType, owner, comment, properties, expirationTime));
  }

  recipient(name: string): RecipientClient {
    return new RecipientClient(this.inner.recipient(name));
  }

  async listSchemas(catalogName: string, options?: ListSchemasOptions): Promise<Schema[]> {
    const { maxResults, includeBrowse } = options || {};
    return (await this.inner.listSchemas(catalogName, maxResults, includeBrowse)).map((data) =>
      fromBinary(SchemaSchema, data),
    );
  }

  async createSchema(name: string, catalogName: string, options?: CreateSchemaOptions): Promise<Schema> {
    const { comment, properties } = options || {};
    return fromBinary(SchemaSchema, await this.inner.createSchema(name, catalogName, comment, properties));
  }

  schema(catalogName: string, schemaName: string): SchemaClient {
    return new SchemaClient(this.inner.schema(catalogName, schemaName));
  }

  async listShares(options?: ListSharesOptions): Promise<Share[]> {
    const { maxResults } = options || {};
    return (await this.inner.listShares(maxResults)).map((data) =>
      fromBinary(ShareSchema, data),
    );
  }

  async createShare(name: string, options?: CreateShareOptions): Promise<Share> {
    const { comment } = options || {};
    return fromBinary(ShareSchema, await this.inner.createShare(name, comment));
  }

  share(name: string): ShareClient {
    return new ShareClient(this.inner.share(name));
  }

  async listTables(catalogName: string, schemaName: string, options?: ListTablesOptions): Promise<Table[]> {
    const { maxResults, includeDeltaMetadata, omitColumns, omitProperties, omitUsername, includeBrowse, includeManifestCapabilities } = options || {};
    return (await this.inner.listTables(catalogName, schemaName, maxResults, includeDeltaMetadata, omitColumns, omitProperties, omitUsername, includeBrowse, includeManifestCapabilities)).map((data) =>
      fromBinary(TableSchema, data),
    );
  }

  async createTable(name: string, schemaName: string, catalogName: string, tableType: number, dataSourceFormat: number, options?: CreateTableOptions): Promise<Table> {
    const { storageLocation, comment, properties } = options || {};
    return fromBinary(TableSchema, await this.inner.createTable(name, schemaName, catalogName, tableType, dataSourceFormat, storageLocation, comment, properties));
  }

  table(name: string): TableClient {
    return new TableClient(this.inner.table(name));
  }

  async listVolumes(catalogName: string, schemaName: string, options?: ListVolumesOptions): Promise<Volume[]> {
    const { maxResults, includeBrowse } = options || {};
    return (await this.inner.listVolumes(catalogName, schemaName, maxResults, includeBrowse)).map((data) =>
      fromBinary(VolumeSchema, data),
    );
  }

  async createVolume(catalogName: string, schemaName: string, name: string, volumeType: number, options?: CreateVolumeOptions): Promise<Volume> {
    const { storageLocation, comment } = options || {};
    return fromBinary(VolumeSchema, await this.inner.createVolume(catalogName, schemaName, name, volumeType, storageLocation, comment));
  }

  volume(catalogName: string, schemaName: string, volumeName: string): VolumeClient {
    return new VolumeClient(this.inner.volume(catalogName, schemaName, volumeName));
  }

}

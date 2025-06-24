import { fromBinary } from "@bufbuild/protobuf";
import { CatalogInfo, CatalogInfoSchema, SchemaInfo, SchemaInfoSchema, TableInfo, TableInfoSchema } from './models';
import { CatalogClient as NativeCatalogClient, UnityCatalogClient as NativeClient, SchemaClient as NativeSchemaClient } from './native';

export class UnityCatalogClient {
  private readonly inner: NativeClient;

  constructor(url: string) {
    this.inner = NativeClient.fromUrl(url);
  }

  async listCatalogs(maxResults?: number): Promise<CatalogInfo[]> {
    return (await this.inner.listCatalogs(maxResults)).map((data) => fromBinary(CatalogInfoSchema, data));
  }

  catalog(name: string): CatalogClient {
    return new CatalogClient(this.inner.catalog(name));
  }
}

export type CreateCatalogOptions = {
  comment?: string | undefined | null,
  storageRoot?: string | undefined | null,
  providerName?: string | undefined | null,
  shareName?: string | undefined | null,
  properties?: Record<string, string> | undefined | null
}

export type UpdateCatalogOptions = {
  newName?: string | undefined | null,
  comment?: string | undefined | null,
  owner?: string | undefined | null,
  properties?: Record<string, string> | undefined | null
}

export class CatalogClient {
  private readonly inner: NativeCatalogClient;

  constructor(inner: NativeCatalogClient) {
    this.inner = inner;
  }

  get name(): string {
    return this.inner.name
  }

  set name(name: string) {
    this.inner.name = name
  }

  async listSchemas(maxResults?: number): Promise<SchemaInfo[]> {
    return (await this.inner.listSchemas(maxResults)).map((data) => fromBinary(SchemaInfoSchema, data));
  }

  schema(name: string): SchemaClient {
    return new SchemaClient(this.inner.schema(name));
  }

  async get(): Promise<CatalogInfo> {
    return fromBinary(CatalogInfoSchema, await this.inner.get())
  }

  async create(options?: CreateCatalogOptions): Promise<CatalogInfo> {
    let { comment, storageRoot, providerName, shareName, properties } = options || {};
    return fromBinary(CatalogInfoSchema, await this.inner.create(comment, storageRoot, providerName, shareName, properties))
  }

  async update(options?: UpdateCatalogOptions): Promise<CatalogInfo> {
    let { newName, comment, owner, properties } = options || {};
    return fromBinary(CatalogInfoSchema, await this.inner.update(newName, comment, owner, properties))
  }

  async delete(): Promise<void> {
    await this.inner.delete()
  }
}

export type CreateSchemaOptions = {
  comment?: string | undefined | null,
  properties?: Record<string, string> | undefined | null
}

export type UpdateSchemaOptions = {
  newName?: string | undefined | null,
  comment?: string | undefined | null,
  properties?: Record<string, string> | undefined | null
}

export type ListTablesOptions  = {
  maxResults?: number | undefined | null,
  includeDeltaMetadata?: boolean | undefined | null,
  omitColumns?: boolean | undefined | null,
  omitProperties?: boolean | undefined | null,
  omitUsername?: boolean | undefined | null
}

export class SchemaClient {
  private readonly inner: NativeSchemaClient;

  constructor(inner: NativeSchemaClient) {
    this.inner = inner;
  }

  get name(): string {
    return this.inner.name
  }

  set name(name: string) {
    this.inner.name = name
  }

  get catalogName(): string {
    return this.inner.catalogName
  }

  set catalogName(name: string) {
    this.inner.catalogName = name
  }

  async listTables(options: ListTablesOptions): Promise<TableInfo[]> {
    let { maxResults, includeDeltaMetadata, omitColumns, omitProperties, omitUsername } = options;
    return (await this.inner.listTables(maxResults, includeDeltaMetadata, omitColumns, omitProperties, omitUsername)).map((data) => fromBinary(TableInfoSchema, data));
  }

  async get(): Promise<SchemaInfo> {
    return fromBinary(SchemaInfoSchema, await this.inner.get())
  }

  async create(options?: CreateSchemaOptions): Promise<SchemaInfo> {
    let {  comment, properties } = options || {};
    return fromBinary(SchemaInfoSchema, await this.inner.create(comment, properties))
  }

  async update(options: UpdateSchemaOptions): Promise<SchemaInfo> {
    let { newName, comment, properties } = options;
    return fromBinary(SchemaInfoSchema, await this.inner.update(newName, comment, properties))
  }

  async delete(force?: boolean): Promise<void> {
    await this.inner.delete(force)
  }
}

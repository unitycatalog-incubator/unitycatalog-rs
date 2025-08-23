import { fromBinary } from "@bufbuild/protobuf";
import { type CatalogInfo, CatalogInfoSchema } from "./models";
import {
  type CatalogClient as NativeCatalogClient,
  UnityCatalogClient as NativeClient,
} from "./native";

export class UnityCatalogClient {
  private readonly inner: NativeClient;

  constructor(url: string) {
    this.inner = NativeClient.fromUrl(url);
  }

  async listCatalogs(maxResults?: number): Promise<CatalogInfo[]> {
    return (await this.inner.listCatalogs(maxResults)).map((data) =>
      fromBinary(CatalogInfoSchema, data),
    );
  }

  catalog(name: string): CatalogClient {
    return new CatalogClient(this.inner.catalog(name));
  }
}

export type CreateCatalogOptions = {
  comment?: string | undefined | null;
  storageRoot?: string | undefined | null;
  providerName?: string | undefined | null;
  shareName?: string | undefined | null;
  properties?: Record<string, string> | undefined | null;
};

export type UpdateCatalogOptions = {
  newName?: string | undefined | null;
  comment?: string | undefined | null;
  owner?: string | undefined | null;
  properties?: Record<string, string> | undefined | null;
};

export class CatalogClient {
  private readonly inner: NativeCatalogClient;

  constructor(inner: NativeCatalogClient) {
    this.inner = inner;
  }

  async get(): Promise<CatalogInfo> {
    return fromBinary(CatalogInfoSchema, await this.inner.get());
  }

  // async create(options?: CreateCatalogOptions): Promise<CatalogInfo> {
  //   let { comment, storageRoot, providerName, shareName, properties } =
  //     options || {};
  //   return fromBinary(
  //     CatalogInfoSchema,
  //     await this.inner.create(
  //       comment,
  //       storageRoot,
  //       providerName,
  //       shareName,
  //       properties,
  //     ),
  //   );
  // }

  async update(options?: UpdateCatalogOptions): Promise<CatalogInfo> {
    const { newName, comment, owner, properties } = options || {};
    return fromBinary(
      CatalogInfoSchema,
      await this.inner.update(newName, comment, owner, properties),
    );
  }

  async delete(): Promise<void> {
    await this.inner.delete();
  }
}

export type CreateSchemaOptions = {
  comment?: string | undefined | null;
  properties?: Record<string, string> | undefined | null;
};

export type UpdateSchemaOptions = {
  newName?: string | undefined | null;
  comment?: string | undefined | null;
  properties?: Record<string, string> | undefined | null;
};

export type ListTablesOptions = {
  maxResults?: number | undefined | null;
  includeDeltaMetadata?: boolean | undefined | null;
  omitColumns?: boolean | undefined | null;
  omitProperties?: boolean | undefined | null;
  omitUsername?: boolean | undefined | null;
};

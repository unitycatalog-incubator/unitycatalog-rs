import { UnityCatalogClient as NativeClient } from './native';
import { CatalogInfo, CatalogInfoSchema } from './models';
import { fromBinary } from "@bufbuild/protobuf";

export class UnityCatalogClient {
  private readonly inner: NativeClient;

  constructor(url: string) {
    this.inner = NativeClient.fromUrl(url);
  }

  async listCatalogs(): Promise<CatalogInfo[]> {
    return (await this.inner.listCatalogs()).map((data) => fromBinary(CatalogInfoSchema, data));
  }
}

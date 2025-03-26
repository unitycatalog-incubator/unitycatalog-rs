import { fromJson, toBinary } from "@bufbuild/protobuf";
import { invoke } from "@tauri-apps/api/core";
import { CatalogInfoJson } from "../gen/unitycatalog/catalogs/v1/models_pb";
import {
  CreateCatalogRequestJson,
  UpdateCatalogRequestJson,
  UpdateCatalogRequestSchema,
} from "../gen/unitycatalog/catalogs/v1/svc_pb";
import { CredentialInfoJson } from "../gen/unitycatalog/credentials/v1/models_pb";
import {
  CreateCredentialRequestJson,
  UpdateCredentialRequestJson,
  UpdateCredentialRequestSchema,
} from "../gen/unitycatalog/credentials/v1/svc_pb";
import { ExternalLocationInfoJson } from "../gen/unitycatalog/external_locations/v1/models_pb";
import {
  CreateExternalLocationRequestJson,
  UpdateExternalLocationRequestJson,
  UpdateExternalLocationRequestSchema,
} from "../gen/unitycatalog/external_locations/v1/svc_pb";
import { RecipientInfoJson } from "../gen/unitycatalog/recipients/v1/models_pb";
import {
  CreateRecipientRequestJson,
  UpdateRecipientRequestJson,
  UpdateRecipientRequestSchema,
} from "../gen/unitycatalog/recipients/v1/svc_pb";
import { SchemaInfoJson } from "../gen/unitycatalog/schemas/v1/models_pb";
import {
  CreateSchemaRequestJson,
  UpdateSchemaRequestJson,
  UpdateSchemaRequestSchema,
} from "../gen/unitycatalog/schemas/v1/svc_pb";
import { ShareInfoJson } from "../gen/unitycatalog/shares/v1/models_pb";
import {
  CreateShareRequestJson,
  UpdateShareRequestJson,
  UpdateShareRequestSchema,
} from "../gen/unitycatalog/shares/v1/svc_pb";
import {
  TableInfoJson,
  TableSummaryJson,
} from "../gen/unitycatalog/tables/v1/models_pb";
import { CreateTableRequestJson } from "../gen/unitycatalog/tables/v1/svc_pb";

export async function list_catalogs(maxResults?: number) {
  return await invoke<CatalogInfoJson[]>("list_catalogs", { maxResults });
}

export async function create_catalog(request: CreateCatalogRequestJson) {
  return await invoke<CatalogInfoJson>("create_catalog", { request });
}

export async function get_catalog(name: string) {
  return await invoke<CatalogInfoJson>("get_catalog", { name });
}

export async function update_catalog(data: UpdateCatalogRequestJson) {
  const jsonMsg = fromJson(UpdateCatalogRequestSchema, data);
  const request = toBinary(UpdateCatalogRequestSchema, jsonMsg);
  return await invoke<CatalogInfoJson>("update_catalog", { request });
}

export async function delete_catalog(name: string) {
  return await invoke<void>("delete_catalog", { name });
}

export async function list_schemas(catalog: string, maxResults?: number) {
  return await invoke<SchemaInfoJson[]>("list_schemas", { catalog });
}

export async function create_schema(request: CreateSchemaRequestJson) {
  console.log("create_schema", { request });
  return await invoke<SchemaInfoJson>("create_schema", { request });
}

export async function get_schema(catalog: string, name: string) {
  return await invoke<SchemaInfoJson>("get_schema", { catalog, name });
}

export async function update_schema(data: UpdateSchemaRequestJson) {
  const jsonMsg = fromJson(UpdateSchemaRequestSchema, data);
  const request = toBinary(UpdateSchemaRequestSchema, jsonMsg);
  return await invoke<SchemaInfoJson>("update_schema", { request });
}

export async function delete_schema(catalog: string, name: string) {
  return await invoke<void>("delete_schema", { catalog, name });
}

export async function list_credentials(maxResults?: number) {
  return await invoke<CredentialInfoJson[]>("list_credentials", {
    maxResults,
  });
}

export async function create_credential(request: CreateCredentialRequestJson) {
  return await invoke<CredentialInfoJson>("create_credential", { request });
}

export async function get_credential(name: string) {
  return await invoke<CredentialInfoJson>("get_credential", { name });
}

export async function update_credential(data: UpdateCredentialRequestJson) {
  const jsonMsg = fromJson(UpdateCredentialRequestSchema, data);
  const request = toBinary(UpdateCredentialRequestSchema, jsonMsg);
  return await invoke<CredentialInfoJson>("update_credential", { request });
}

export async function delete_credential(name: string) {
  return await invoke<void>("delete_credential", { name });
}

export async function list_external_locations(maxResults?: number) {
  return await invoke<ExternalLocationInfoJson[]>("list_external_locations", {
    maxResults,
  });
}

export async function create_external_location(
  request: CreateExternalLocationRequestJson,
) {
  return await invoke<ExternalLocationInfoJson>("create_external_location", {
    request,
  });
}

export async function get_external_location(name: string) {
  return await invoke<ExternalLocationInfoJson>("get_external_location", {
    name,
  });
}

export async function update_external_location(
  data: UpdateExternalLocationRequestJson,
) {
  const jsonMsg = fromJson(UpdateExternalLocationRequestSchema, data);
  const request = toBinary(UpdateExternalLocationRequestSchema, jsonMsg);
  return await invoke<ExternalLocationInfoJson>("update_external_location", {
    request,
  });
}

export async function delete_external_location(name: string) {
  return await invoke<void>("delete_external_location", { name });
}

export async function list_recipients(maxResults?: number) {
  return await invoke<RecipientInfoJson[]>("list_recipients", { maxResults });
}

export async function create_recipient(request: CreateRecipientRequestJson) {
  return await invoke<RecipientInfoJson>("create_recipient", { request });
}

export async function get_recipient(name: string) {
  return await invoke<RecipientInfoJson>("get_recipient", { name });
}

export async function update_recipient(data: UpdateRecipientRequestJson) {
  const jsonMsg = fromJson(UpdateRecipientRequestSchema, data);
  const request = toBinary(UpdateRecipientRequestSchema, jsonMsg);
  return await invoke<RecipientInfoJson>("update_recipient", { request });
}

export async function delete_recipient(name: string) {
  return await invoke<void>("delete_recipient", { name });
}

export async function list_shares(maxResults?: number) {
  return await invoke<ShareInfoJson[]>("list_shares", { maxResults });
}

export async function create_share(request: CreateShareRequestJson) {
  return await invoke<ShareInfoJson>("create_share", { request });
}

export async function get_share(name: string, includeSharedData?: boolean) {
  return await invoke<ShareInfoJson>("get_share", {
    name,
    includeSharedData,
  });
}

export async function update_share(data: UpdateShareRequestJson) {
  const jsonMsg = fromJson(UpdateShareRequestSchema, data);
  const request = toBinary(UpdateShareRequestSchema, jsonMsg);
  return await invoke<ShareInfoJson>("update_share", { request });
}

export async function delete_share(name: string) {
  return await invoke<void>("delete_share", { name });
}

export async function list_table_summaries(
  catalog: string,
  schemaPattern?: string,
  tablePattern?: string,
  maxResults?: number,
) {
  return await invoke<TableSummaryJson[]>("list_table_summaries", {
    catalog,
    schemaPattern,
    tablePattern,
    maxResults,
  });
}

export async function list_tables(
  catalog: string,
  schema: string,
  maxResults?: number,
) {
  return await invoke<TableInfoJson[]>("list_tables", {
    catalog,
    schema,
    maxResults,
  });
}

export async function create_table(request: CreateTableRequestJson) {
  return await invoke<TableInfoJson>("create_table", { request });
}

export async function get_table(catalog: string, schema: string, name: string) {
  return await invoke<TableInfoJson>("get_table", {
    fullName: `${catalog}.${schema}.${name}`,
  });
}

export async function delete_table(
  catalog: string,
  schema: string,
  name: string,
) {
  return await invoke<void>("delete_table", {
    fullName: `${catalog}.${schema}.${name}`,
  });
}

export default {
  list_catalogs,
  create_catalog,
  get_catalog,
  update_catalog,
  delete_catalog,
  list_schemas,
  create_schema,
  get_schema,
  update_schema,
  delete_schema,
  list_credentials,
  create_credential,
  get_credential,
  update_credential,
  delete_credential,
  list_external_locations,
  create_external_location,
  get_external_location,
  update_external_location,
  delete_external_location,
  list_recipients,
  create_recipient,
  get_recipient,
  update_recipient,
  delete_recipient,
  list_shares,
  create_share,
  get_share,
  update_share,
  delete_share,
  list_table_summaries,
  list_tables,
  create_table,
  get_table,
  delete_table,
};

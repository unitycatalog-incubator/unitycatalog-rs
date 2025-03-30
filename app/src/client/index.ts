import type { CatalogInfoJson } from "../gen/unitycatalog/catalogs/v1/models_pb";
import type {
  CreateCatalogRequestJson,
  UpdateCatalogRequestJson,
} from "../gen/unitycatalog/catalogs/v1/svc_pb";
import type {
  CredentialInfoJson,
  PurposeJson,
} from "../gen/unitycatalog/credentials/v1/models_pb";
import type {
  CreateCredentialRequestJson,
  UpdateCredentialRequestJson,
} from "../gen/unitycatalog/credentials/v1/svc_pb";
import type { ExternalLocationInfoJson } from "../gen/unitycatalog/external_locations/v1/models_pb";
import type {
  CreateExternalLocationRequestJson,
  UpdateExternalLocationRequestJson,
} from "../gen/unitycatalog/external_locations/v1/svc_pb";
import {
  AuthenticationTypeJson,
  RecipientInfoJson,
} from "../gen/unitycatalog/recipients/v1/models_pb";
import {
  CreateRecipientRequestJson,
  UpdateRecipientRequestJson,
} from "../gen/unitycatalog/recipients/v1/svc_pb";
import type { SchemaInfoJson } from "../gen/unitycatalog/schemas/v1/models_pb";
import type {
  CreateSchemaRequestJson,
  UpdateSchemaRequestJson,
} from "../gen/unitycatalog/schemas/v1/svc_pb";
import type {
  DataObjectJson,
  DataObjectTypeJson,
  ShareInfoJson,
} from "../gen/unitycatalog/shares/v1/models_pb";
import type {
  CreateShareRequestJson,
  DataObjectUpdateJson,
  UpdateShareRequestJson,
} from "../gen/unitycatalog/shares/v1/svc_pb";
import type {
  DataSourceFormatJson,
  TableInfoJson,
  TableSummaryJson,
  TableTypeJson,
} from "../gen/unitycatalog/tables/v1/models_pb";
import type { CreateTableRequestJson } from "../gen/unitycatalog/tables/v1/svc_pb";
import tauri from "./tauri";

export type {
  AuthenticationTypeJson as AuthenticationType,
  CatalogInfoJson as CatalogInfo,
  CreateCatalogRequestJson as CreateCatalogRequest,
  CreateCredentialRequestJson as CreateCredentialRequest,
  CreateExternalLocationRequestJson as CreateExternalLocationRequest,
  CreateRecipientRequestJson as CreateRecipientRequest,
  CreateSchemaRequestJson as CreateSchemaRequest,
  CreateShareRequestJson as CreateShareRequest,
  CreateTableRequestJson as CreateTableRequest,
  CredentialInfoJson as CredentialInfo,
  DataObjectJson as DataObject,
  DataObjectTypeJson as DataObjectType,
  DataObjectUpdateJson as DataObjectUpdate,
  DataSourceFormatJson as DataSourceFormat,
  ExternalLocationInfoJson as ExternalLocationInfo,
  PurposeJson as Purpose,
  RecipientInfoJson as RecipientInfo,
  SchemaInfoJson as SchemaInfo,
  ShareInfoJson as ShareInfo,
  TableInfoJson as TableInfo,
  TableSummaryJson as TableSummary,
  TableTypeJson as TableType,
  UpdateCatalogRequestJson as UpdateCatalogRequest,
  UpdateCredentialRequestJson as UpdateCredentialRequest,
  UpdateExternalLocationRequestJson as UpdateExternalLocationRequest,
  UpdateRecipientRequestJson as UpdateRecipientRequest,
  UpdateSchemaRequestJson as UpdateSchemaRequest,
  UpdateShareRequestJson as UpdateShareRequest,
};

export type ListOptions = {
  // The maximum number of items to return
  maxResults?: number;
};

export async function listCatalogs(maxResults?: number) {
  return await tauri.list_catalogs(maxResults);
}

export async function getCatalog(name: string) {
  return await tauri.get_catalog(name);
}

export async function createCatalog(request: CreateCatalogRequestJson) {
  return await tauri.create_catalog(request);
}

export async function updateCatalog(data: UpdateCatalogRequestJson) {
  return await tauri.update_catalog(data);
}

export async function deleteCatalog(name: string) {
  return await tauri.delete_catalog(name);
}

export type ListSchemasOptions = {
  // The parent catalog of the schemas to list
  catalog: string;
} & ListOptions;

export async function listSchemas({ catalog, maxResults }: ListSchemasOptions) {
  return await tauri.list_schemas(catalog, maxResults);
}

export async function createSchema(request: CreateSchemaRequestJson) {
  return await tauri.create_schema(request);
}

export async function getSchema(catalog: string, name: string) {
  return await tauri.get_schema(catalog, name);
}

export async function updateSchema(request: UpdateSchemaRequestJson) {
  return await tauri.update_schema(request);
}

export async function deleteSchema({
  catalog,
  name,
}: {
  catalog: string;
  name: string;
}) {
  return await tauri.delete_schema(catalog, name);
}

export async function listCredentials(maxResults?: number) {
  return await tauri.list_credentials(maxResults);
}

export async function getCredential(name: string) {
  return await tauri.get_credential(name);
}

export async function createCredential(request: CreateCredentialRequestJson) {
  return await tauri.create_credential(request);
}

export async function updateCredential(request: UpdateCredentialRequestJson) {
  return await tauri.update_credential(request);
}

export async function deleteCredential(name: string) {
  return await tauri.delete_credential(name);
}

export async function listExternalLocations(maxResults?: number) {
  return await tauri.list_external_locations(maxResults);
}

export async function getExternalLocation(name: string) {
  return await tauri.get_external_location(name);
}

export async function createExternalLocation(
  request: CreateExternalLocationRequestJson,
) {
  return await tauri.create_external_location(request);
}

export async function updateExternalLocation(
  request: UpdateExternalLocationRequestJson,
) {
  return await tauri.update_external_location(request);
}

export async function deleteExternalLocation(name: string) {
  return await tauri.delete_external_location(name);
}

export async function listRecipients(maxResults?: number) {
  return await tauri.list_recipients(maxResults);
}

export async function getRecipient(name: string) {
  return await tauri.get_recipient(name);
}

export async function createRecipient(request: CreateRecipientRequestJson) {
  return await tauri.create_recipient(request);
}

export async function updateRecipient(request: UpdateRecipientRequestJson) {
  return await tauri.update_recipient(request);
}

export async function deleteRecipient(name: string) {
  return await tauri.delete_recipient(name);
}

export async function listShares(maxResults?: number) {
  return await tauri.list_shares(maxResults);
}

export async function getShare({
  name,
  includeSharedData,
}: {
  name: string;
  includeSharedData?: boolean;
}) {
  return await tauri.get_share(name, includeSharedData);
}

export async function createShare(request: CreateShareRequestJson) {
  return await tauri.create_share(request);
}

export async function updateShare(request: UpdateShareRequestJson) {
  return await tauri.update_share(request);
}

export async function deleteShare(name: string) {
  return await tauri.delete_share(name);
}

export type ListTableSummariesOptions = {
  // The parent catalog of the tables to list
  catalog: string;
  // The parent schema of the tables to list
  schemaPattern?: string;
  // The table name pattern to match
  tablePattern?: string;
  // The maximum number of tables to return
  maxResults?: number;
};

export async function listTableSummaries({
  catalog,
  schemaPattern,
  tablePattern,
  maxResults,
}: ListTableSummariesOptions) {
  return await tauri.list_table_summaries(
    catalog,
    schemaPattern,
    tablePattern,
    maxResults,
  );
}

export type ListTablesOptions = {
  // The parent catalog of the tables to list
  catalog: string;
  // The parent schema of the tables to list
  schema: string;
  // The maximum number of tables to return
  maxResults?: number;
};

export async function listTables({
  catalog,
  schema,
  maxResults,
}: ListTablesOptions) {
  return await tauri.list_tables(catalog, schema, maxResults);
}

export async function createTable(request: CreateTableRequestJson) {
  return await tauri.create_table(request);
}

export async function getTable(catalog: string, schema: string, name: string) {
  return await tauri.get_table(catalog, schema, name);
}

export async function deleteTable({
  catalog,
  schema,
  name,
}: {
  catalog: string;
  schema: string;
  name: string;
}) {
  return await tauri.delete_table(catalog, schema, name);
}

export default {
  catalogs: {
    list: listCatalogs,
    get: getCatalog,
    create: createCatalog,
    update: updateCatalog,
    delete: deleteCatalog,
  },
  schemas: {
    list: listSchemas,
    get: getSchema,
    create: createSchema,
    update: updateSchema,
    delete: deleteSchema,
  },
  credentials: {
    list: listCredentials,
    get: getCredential,
    create: createCredential,
    update: updateCredential,
    delete: deleteCredential,
  },
  externalLocations: {
    list: listExternalLocations,
    get: getExternalLocation,
    create: createExternalLocation,
    update: updateExternalLocation,
    delete: deleteExternalLocation,
  },
  recipients: {
    list: listRecipients,
    get: getRecipient,
    create: createRecipient,
    update: updateRecipient,
    delete: deleteRecipient,
  },
  shares: {
    list: listShares,
    get: getShare,
    create: createShare,
    delete: deleteShare,
    update: updateShare,
  },
  tables: {
    listSummaries: listTableSummaries,
    list: listTables,
    create: createTable,
    get: getTable,
    delete: deleteTable,
  },
};

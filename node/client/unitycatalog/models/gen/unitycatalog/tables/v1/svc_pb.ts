// @generated by protoc-gen-es v2.5.2 with parameter "target=ts,json_types=false"
// @generated from file unitycatalog/tables/v1/svc.proto (package unitycatalog.tables.v1, syntax proto3)
/* eslint-disable */

import type { GenFile, GenMessage, GenService } from "@bufbuild/protobuf/codegenv2";
import { fileDesc, messageDesc, serviceDesc } from "@bufbuild/protobuf/codegenv2";
import { file_buf_validate_validate } from "../../../buf/validate/validate_pb";
import type { ColumnInfo, DataSourceFormat, TableInfo, TableInfoSchema, TableSummary, TableType } from "./models_pb";
import { file_unitycatalog_tables_v1_models } from "./models_pb";
import { file_gnostic_openapi_v3_annotations } from "../../../gnostic/openapi/v3/annotations_pb";
import { file_google_api_annotations } from "../../../google/api/annotations_pb";
import { file_google_api_field_behavior } from "../../../google/api/field_behavior_pb";
import type { EmptySchema } from "@bufbuild/protobuf/wkt";
import { file_google_protobuf_empty, file_google_protobuf_struct } from "@bufbuild/protobuf/wkt";
import type { JsonObject, Message } from "@bufbuild/protobuf";

/**
 * Describes the file unitycatalog/tables/v1/svc.proto.
 */
export const file_unitycatalog_tables_v1_svc: GenFile = /*@__PURE__*/
  fileDesc("CiB1bml0eWNhdGFsb2cvdGFibGVzL3YxL3N2Yy5wcm90bxIWdW5pdHljYXRhbG9nLnRhYmxlcy52MSKNAwoZTGlzdFRhYmxlU3VtbWFyaWVzUmVxdWVzdBI7CgxjYXRhbG9nX25hbWUYASABKAlCJeBBArpIH3IdEAMyGV5bYS16XVswLTlhLXpfXSpbMC05YS16XSQSJQoTc2NoZW1hX25hbWVfcGF0dGVybhgCIAEoCUID4EEBSACIAQESJAoSdGFibGVfbmFtZV9wYXR0ZXJuGAMgASgJQgPgQQFIAYgBARInCgttYXhfcmVzdWx0cxhkIAEoBUIN4EEBukgHGgUYkE4gAEgCiAEBEhwKCnBhZ2VfdG9rZW4YZSABKAlCA+BBAUgDiAEBEi8KHWluY2x1ZGVfbWFuaWZlc3RfY2FwYWJpbGl0aWVzGGYgASgIQgPgQQFIBIgBAUIWChRfc2NoZW1hX25hbWVfcGF0dGVybkIVChNfdGFibGVfbmFtZV9wYXR0ZXJuQg4KDF9tYXhfcmVzdWx0c0INCgtfcGFnZV90b2tlbkIgCh5faW5jbHVkZV9tYW5pZmVzdF9jYXBhYmlsaXRpZXMihAEKGkxpc3RUYWJsZVN1bW1hcmllc1Jlc3BvbnNlEjQKBnRhYmxlcxgBIAMoCzIkLnVuaXR5Y2F0YWxvZy50YWJsZXMudjEuVGFibGVTdW1tYXJ5EhwKD25leHRfcGFnZV90b2tlbhgCIAEoCUgAiAEBQhIKEF9uZXh0X3BhZ2VfdG9rZW4i2QQKEUxpc3RUYWJsZXNSZXF1ZXN0EjoKC3NjaGVtYV9uYW1lGAEgASgJQiXgQQK6SB9yHRADMhleW2Etel1bMC05YS16X10qWzAtOWEtel0kEjsKDGNhdGFsb2dfbmFtZRgCIAEoCUIl4EECukgfch0QAzIZXlthLXpdWzAtOWEtel9dKlswLTlhLXpdJBImCgttYXhfcmVzdWx0cxgDIAEoBUIM4EEBukgGGgQYMiAASACIAQESHAoKcGFnZV90b2tlbhgEIAEoCUID4EEBSAGIAQESKAoWaW5jbHVkZV9kZWx0YV9tZXRhZGF0YRgFIAEoCEID4EEBSAKIAQESHgoMb21pdF9jb2x1bW5zGAYgASgIQgPgQQFIA4gBARIhCg9vbWl0X3Byb3BlcnRpZXMYByABKAhCA+BBAUgEiAEBEh8KDW9taXRfdXNlcm5hbWUYCCABKAhCA+BBAUgFiAEBEiAKDmluY2x1ZGVfYnJvd3NlGAkgASgIQgPgQQFIBogBARIvCh1pbmNsdWRlX21hbmlmZXN0X2NhcGFiaWxpdGllcxgKIAEoCEID4EEBSAeIAQFCDgoMX21heF9yZXN1bHRzQg0KC19wYWdlX3Rva2VuQhkKF19pbmNsdWRlX2RlbHRhX21ldGFkYXRhQg8KDV9vbWl0X2NvbHVtbnNCEgoQX29taXRfcHJvcGVydGllc0IQCg5fb21pdF91c2VybmFtZUIRCg9faW5jbHVkZV9icm93c2VCIAoeX2luY2x1ZGVfbWFuaWZlc3RfY2FwYWJpbGl0aWVzInkKEkxpc3RUYWJsZXNSZXNwb25zZRIxCgZ0YWJsZXMYASADKAsyIS51bml0eWNhdGFsb2cudGFibGVzLnYxLlRhYmxlSW5mbxIcCg9uZXh0X3BhZ2VfdG9rZW4YAiABKAlIAIgBAUISChBfbmV4dF9wYWdlX3Rva2VuIosEChJDcmVhdGVUYWJsZVJlcXVlc3QSMwoEbmFtZRgBIAEoCUIl4EECukgfch0QAzIZXlthLXpdWzAtOWEtel9dKlswLTlhLXpdJBI6CgtzY2hlbWFfbmFtZRgCIAEoCUIl4EECukgfch0QAzIZXlthLXpdWzAtOWEtel9dKlswLTlhLXpdJBI7CgxjYXRhbG9nX25hbWUYAyABKAlCJeBBArpIH3IdEAMyGV5bYS16XVswLTlhLXpfXSpbMC05YS16XSQSNQoKdGFibGVfdHlwZRgEIAEoDjIhLnVuaXR5Y2F0YWxvZy50YWJsZXMudjEuVGFibGVUeXBlEkQKEmRhdGFfc291cmNlX2Zvcm1hdBgFIAEoDjIoLnVuaXR5Y2F0YWxvZy50YWJsZXMudjEuRGF0YVNvdXJjZUZvcm1hdBIzCgdjb2x1bW5zGAYgAygLMiIudW5pdHljYXRhbG9nLnRhYmxlcy52MS5Db2x1bW5JbmZvEh0KEHN0b3JhZ2VfbG9jYXRpb24YByABKAlIAIgBARIUCgdjb21tZW50GAggASgJSAGIAQESMAoKcHJvcGVydGllcxgJIAEoCzIXLmdvb2dsZS5wcm90b2J1Zi5TdHJ1Y3RIAogBAUITChFfc3RvcmFnZV9sb2NhdGlvbkIKCghfY29tbWVudEINCgtfcHJvcGVydGllcyLiAQoPR2V0VGFibGVSZXF1ZXN0EhEKCWZ1bGxfbmFtZRgBIAEoCRIjChZpbmNsdWRlX2RlbHRhX21ldGFkYXRhGAIgASgISACIAQESGwoOaW5jbHVkZV9icm93c2UYAyABKAhIAYgBARIqCh1pbmNsdWRlX21hbmlmZXN0X2NhcGFiaWxpdGllcxgEIAEoCEgCiAEBQhkKF19pbmNsdWRlX2RlbHRhX21ldGFkYXRhQhEKD19pbmNsdWRlX2Jyb3dzZUIgCh5faW5jbHVkZV9tYW5pZmVzdF9jYXBhYmlsaXRpZXMiKgoVR2V0VGFibGVFeGlzdHNSZXF1ZXN0EhEKCWZ1bGxfbmFtZRgBIAEoCSIuChZHZXRUYWJsZUV4aXN0c1Jlc3BvbnNlEhQKDHRhYmxlX2V4aXN0cxgBIAEoCCInChJEZWxldGVUYWJsZVJlcXVlc3QSEQoJZnVsbF9uYW1lGAEgASgJMvMGCg1UYWJsZXNTZXJ2aWNlEqwBChJMaXN0VGFibGVTdW1tYXJpZXMSMS51bml0eWNhdGFsb2cudGFibGVzLnYxLkxpc3RUYWJsZVN1bW1hcmllc1JlcXVlc3QaMi51bml0eWNhdGFsb2cudGFibGVzLnYxLkxpc3RUYWJsZVN1bW1hcmllc1Jlc3BvbnNlIi+6RxQqEkxpc3RUYWJsZVN1bW1hcmllc4LT5JMCEhIQL3RhYmxlLXN1bW1hcmllcxKDAQoKTGlzdFRhYmxlcxIpLnVuaXR5Y2F0YWxvZy50YWJsZXMudjEuTGlzdFRhYmxlc1JlcXVlc3QaKi51bml0eWNhdGFsb2cudGFibGVzLnYxLkxpc3RUYWJsZXNSZXNwb25zZSIeukcMKgpMaXN0VGFibGVzgtPkkwIJEgcvdGFibGVzEoABCgtDcmVhdGVUYWJsZRIqLnVuaXR5Y2F0YWxvZy50YWJsZXMudjEuQ3JlYXRlVGFibGVSZXF1ZXN0GiEudW5pdHljYXRhbG9nLnRhYmxlcy52MS5UYWJsZUluZm8iIrpHDSoLQ3JlYXRlVGFibGWC0+STAgw6ASoiBy90YWJsZXMSgAEKCEdldFRhYmxlEicudW5pdHljYXRhbG9nLnRhYmxlcy52MS5HZXRUYWJsZVJlcXVlc3QaIS51bml0eWNhdGFsb2cudGFibGVzLnYxLlRhYmxlSW5mbyIoukcKKghHZXRUYWJsZYLT5JMCFRITL3RhYmxlcy97ZnVsbF9uYW1lfRKmAQoOR2V0VGFibGVFeGlzdHMSLS51bml0eWNhdGFsb2cudGFibGVzLnYxLkdldFRhYmxlRXhpc3RzUmVxdWVzdBouLnVuaXR5Y2F0YWxvZy50YWJsZXMudjEuR2V0VGFibGVFeGlzdHNSZXNwb25zZSI1ukcQKg5HZXRUYWJsZUV4aXN0c4LT5JMCHBIaL3RhYmxlcy97ZnVsbF9uYW1lfS9leGlzdHMSfgoLRGVsZXRlVGFibGUSKi51bml0eWNhdGFsb2cudGFibGVzLnYxLkRlbGV0ZVRhYmxlUmVxdWVzdBoWLmdvb2dsZS5wcm90b2J1Zi5FbXB0eSIrukcNKgtEZWxldGVUYWJsZYLT5JMCFSoTL3RhYmxlcy97ZnVsbF9uYW1lfULwAQoaY29tLnVuaXR5Y2F0YWxvZy50YWJsZXMudjFCCFN2Y1Byb3RvUAFaTmdpdGh1Yi5jb20vZGVsdGEtaW5jdWJhdG9yL2RlbHRhLXNoYXJpbmctcnMvZ28vdW5pdHljYXRhbG9nL3RhYmxlcy92MTt0YWJsZXN2MaICA1VUWKoCFlVuaXR5Y2F0YWxvZy5UYWJsZXMuVjHKAhZVbml0eWNhdGFsb2dcVGFibGVzXFYx4gIiVW5pdHljYXRhbG9nXFRhYmxlc1xWMVxHUEJNZXRhZGF0YeoCGFVuaXR5Y2F0YWxvZzo6VGFibGVzOjpWMWIGcHJvdG8z", [file_buf_validate_validate, file_unitycatalog_tables_v1_models, file_gnostic_openapi_v3_annotations, file_google_api_annotations, file_google_api_field_behavior, file_google_protobuf_empty, file_google_protobuf_struct]);

/**
 * @generated from message unitycatalog.tables.v1.ListTableSummariesRequest
 */
export type ListTableSummariesRequest = Message<"unitycatalog.tables.v1.ListTableSummariesRequest"> & {
  /**
   * Name of parent catalog for tables of interest.
   *
   * @generated from field: string catalog_name = 1;
   */
  catalogName: string;

  /**
   * A sql LIKE pattern (% and _) for schema names. All schemas will be returned if not set or empty.
   *
   * @generated from field: optional string schema_name_pattern = 2;
   */
  schemaNamePattern?: string;

  /**
   * A sql LIKE pattern (% and _) for table names. All tables will be returned if not set or empty.
   *
   * @generated from field: optional string table_name_pattern = 3;
   */
  tableNamePattern?: string;

  /**
   * The maximum number of results per page that should be returned.
   *
   * @generated from field: optional int32 max_results = 100;
   */
  maxResults?: number;

  /**
   * Opaque pagination token to go to next page based on previous query.
   *
   * @generated from field: optional string page_token = 101;
   */
  pageToken?: string;

  /**
   * Whether to include a manifest containing capabilities the table has.
   *
   * @generated from field: optional bool include_manifest_capabilities = 102;
   */
  includeManifestCapabilities?: boolean;
};

/**
 * Describes the message unitycatalog.tables.v1.ListTableSummariesRequest.
 * Use `create(ListTableSummariesRequestSchema)` to create a new message.
 */
export const ListTableSummariesRequestSchema: GenMessage<ListTableSummariesRequest> = /*@__PURE__*/
  messageDesc(file_unitycatalog_tables_v1_svc, 0);

/**
 * @generated from message unitycatalog.tables.v1.ListTableSummariesResponse
 */
export type ListTableSummariesResponse = Message<"unitycatalog.tables.v1.ListTableSummariesResponse"> & {
  /**
   * The table summaries returned.
   *
   * @generated from field: repeated unitycatalog.tables.v1.TableSummary tables = 1;
   */
  tables: TableSummary[];

  /**
   * The next_page_token value to include in the next List request.
   *
   * @generated from field: optional string next_page_token = 2;
   */
  nextPageToken?: string;
};

/**
 * Describes the message unitycatalog.tables.v1.ListTableSummariesResponse.
 * Use `create(ListTableSummariesResponseSchema)` to create a new message.
 */
export const ListTableSummariesResponseSchema: GenMessage<ListTableSummariesResponse> = /*@__PURE__*/
  messageDesc(file_unitycatalog_tables_v1_svc, 1);

/**
 * @generated from message unitycatalog.tables.v1.ListTablesRequest
 */
export type ListTablesRequest = Message<"unitycatalog.tables.v1.ListTablesRequest"> & {
  /**
   * Name of parent schema for tables of interest.
   *
   * @generated from field: string schema_name = 1;
   */
  schemaName: string;

  /**
   * Name of parent catalog for tables of interest.
   *
   * @generated from field: string catalog_name = 2;
   */
  catalogName: string;

  /**
   * The maximum number of results per page that should be returned.
   *
   * @generated from field: optional int32 max_results = 3;
   */
  maxResults?: number;

  /**
   * Opaque pagination token to go to next page based on previous query.
   *
   * @generated from field: optional string page_token = 4;
   */
  pageToken?: string;

  /**
   * Whether delta metadata should be included in the response.
   *
   * @generated from field: optional bool include_delta_metadata = 5;
   */
  includeDeltaMetadata?: boolean;

  /**
   * Whether to omit the columns of the table from the response or not.
   *
   * @generated from field: optional bool omit_columns = 6;
   */
  omitColumns?: boolean;

  /**
   * Whether to omit the properties of the table from the response or not.
   *
   * @generated from field: optional bool omit_properties = 7;
   */
  omitProperties?: boolean;

  /**
   * Whether to omit the username of the table (e.g. owner, updated_by, created_by) from the response or not.
   *
   * @generated from field: optional bool omit_username = 8;
   */
  omitUsername?: boolean;

  /**
   * Whether to include tables in the response for which the principal can only access selective metadata for
   *
   * @generated from field: optional bool include_browse = 9;
   */
  includeBrowse?: boolean;

  /**
   * Whether to include a manifest containing capabilities the table has.
   *
   * @generated from field: optional bool include_manifest_capabilities = 10;
   */
  includeManifestCapabilities?: boolean;
};

/**
 * Describes the message unitycatalog.tables.v1.ListTablesRequest.
 * Use `create(ListTablesRequestSchema)` to create a new message.
 */
export const ListTablesRequestSchema: GenMessage<ListTablesRequest> = /*@__PURE__*/
  messageDesc(file_unitycatalog_tables_v1_svc, 2);

/**
 * @generated from message unitycatalog.tables.v1.ListTablesResponse
 */
export type ListTablesResponse = Message<"unitycatalog.tables.v1.ListTablesResponse"> & {
  /**
   * The tables returned.
   *
   * @generated from field: repeated unitycatalog.tables.v1.TableInfo tables = 1;
   */
  tables: TableInfo[];

  /**
   * The next_page_token value to include in the next List request.
   *
   * @generated from field: optional string next_page_token = 2;
   */
  nextPageToken?: string;
};

/**
 * Describes the message unitycatalog.tables.v1.ListTablesResponse.
 * Use `create(ListTablesResponseSchema)` to create a new message.
 */
export const ListTablesResponseSchema: GenMessage<ListTablesResponse> = /*@__PURE__*/
  messageDesc(file_unitycatalog_tables_v1_svc, 3);

/**
 * Create a table
 *
 * WARNING: this API is experimental and subject to change.
 *
 * @generated from message unitycatalog.tables.v1.CreateTableRequest
 */
export type CreateTableRequest = Message<"unitycatalog.tables.v1.CreateTableRequest"> & {
  /**
   * Name of table, relative to parent schema.
   *
   * @generated from field: string name = 1;
   */
  name: string;

  /**
   * Name of parent schema relative to its parent catalog.
   *
   * @generated from field: string schema_name = 2;
   */
  schemaName: string;

  /**
   * Name of parent catalog.
   *
   * @generated from field: string catalog_name = 3;
   */
  catalogName: string;

  /**
   * @generated from field: unitycatalog.tables.v1.TableType table_type = 4;
   */
  tableType: TableType;

  /**
   * @generated from field: unitycatalog.tables.v1.DataSourceFormat data_source_format = 5;
   */
  dataSourceFormat: DataSourceFormat;

  /**
   * The array of ColumnInfo definitions of the table's columns.
   *
   * @generated from field: repeated unitycatalog.tables.v1.ColumnInfo columns = 6;
   */
  columns: ColumnInfo[];

  /**
   * Storage root URL for external table.
   *
   * @generated from field: optional string storage_location = 7;
   */
  storageLocation?: string;

  /**
   * User-provided free-form text description.
   *
   * @generated from field: optional string comment = 8;
   */
  comment?: string;

  /**
   * A map of key-value properties attached to the securable.
   *
   * @generated from field: optional google.protobuf.Struct properties = 9;
   */
  properties?: JsonObject;
};

/**
 * Describes the message unitycatalog.tables.v1.CreateTableRequest.
 * Use `create(CreateTableRequestSchema)` to create a new message.
 */
export const CreateTableRequestSchema: GenMessage<CreateTableRequest> = /*@__PURE__*/
  messageDesc(file_unitycatalog_tables_v1_svc, 4);

/**
 * Get a table
 *
 * @generated from message unitycatalog.tables.v1.GetTableRequest
 */
export type GetTableRequest = Message<"unitycatalog.tables.v1.GetTableRequest"> & {
  /**
   * Full name of the table.
   *
   * @generated from field: string full_name = 1;
   */
  fullName: string;

  /**
   * Whether delta metadata should be included in the response.
   *
   * @generated from field: optional bool include_delta_metadata = 2;
   */
  includeDeltaMetadata?: boolean;

  /**
   * Whether to include tables in the response for which the principal can only access selective metadata for
   *
   * @generated from field: optional bool include_browse = 3;
   */
  includeBrowse?: boolean;

  /**
   * Whether to include a manifest containing capabilities the table has.
   *
   * @generated from field: optional bool include_manifest_capabilities = 4;
   */
  includeManifestCapabilities?: boolean;
};

/**
 * Describes the message unitycatalog.tables.v1.GetTableRequest.
 * Use `create(GetTableRequestSchema)` to create a new message.
 */
export const GetTableRequestSchema: GenMessage<GetTableRequest> = /*@__PURE__*/
  messageDesc(file_unitycatalog_tables_v1_svc, 5);

/**
 * Get boolean reflecting if table exists
 *
 * @generated from message unitycatalog.tables.v1.GetTableExistsRequest
 */
export type GetTableExistsRequest = Message<"unitycatalog.tables.v1.GetTableExistsRequest"> & {
  /**
   * Full name of the table.
   *
   * @generated from field: string full_name = 1;
   */
  fullName: string;
};

/**
 * Describes the message unitycatalog.tables.v1.GetTableExistsRequest.
 * Use `create(GetTableExistsRequestSchema)` to create a new message.
 */
export const GetTableExistsRequestSchema: GenMessage<GetTableExistsRequest> = /*@__PURE__*/
  messageDesc(file_unitycatalog_tables_v1_svc, 6);

/**
 * @generated from message unitycatalog.tables.v1.GetTableExistsResponse
 */
export type GetTableExistsResponse = Message<"unitycatalog.tables.v1.GetTableExistsResponse"> & {
  /**
   * Boolean reflecting if table exists.
   *
   * @generated from field: bool table_exists = 1;
   */
  tableExists: boolean;
};

/**
 * Describes the message unitycatalog.tables.v1.GetTableExistsResponse.
 * Use `create(GetTableExistsResponseSchema)` to create a new message.
 */
export const GetTableExistsResponseSchema: GenMessage<GetTableExistsResponse> = /*@__PURE__*/
  messageDesc(file_unitycatalog_tables_v1_svc, 7);

/**
 * Delete a table
 *
 * @generated from message unitycatalog.tables.v1.DeleteTableRequest
 */
export type DeleteTableRequest = Message<"unitycatalog.tables.v1.DeleteTableRequest"> & {
  /**
   * Full name of the table.
   *
   * @generated from field: string full_name = 1;
   */
  fullName: string;
};

/**
 * Describes the message unitycatalog.tables.v1.DeleteTableRequest.
 * Use `create(DeleteTableRequestSchema)` to create a new message.
 */
export const DeleteTableRequestSchema: GenMessage<DeleteTableRequest> = /*@__PURE__*/
  messageDesc(file_unitycatalog_tables_v1_svc, 8);

/**
 * @generated from service unitycatalog.tables.v1.TablesService
 */
export const TablesService: GenService<{
  /**
   * Gets an array of summaries for tables for a schema and catalog within the metastore. The table summaries returned are either:
   * - summaries for tables (within the current metastore and parent catalog and schema), when the user is a metastore admin, or:
   * - summaries for tables and schemas (within the current metastore and parent catalog) for which the user has ownership or the
   *   SELECT privilege on the table and ownership or USE_SCHEMA privilege on the schema, provided that the user also has ownership
   *   or the USE_CATALOG privilege on the parent catalog.
   *
   * There is no guarantee of a specific ordering of the elements in the array.
   *
   * @generated from rpc unitycatalog.tables.v1.TablesService.ListTableSummaries
   */
  listTableSummaries: {
    methodKind: "unary";
    input: typeof ListTableSummariesRequestSchema;
    output: typeof ListTableSummariesResponseSchema;
  },
  /**
   * Gets an array of all tables for the current metastore under the parent catalog and schema.
   *
   * The caller must be a metastore admin or an owner of (or have the SELECT privilege on) the table.
   * For the latter case, the caller must also be the owner or have the USE_CATALOG privilege on the
   * parent catalog and the USE_SCHEMA privilege on the parent schema. There is no guarantee of a
   * specific ordering of the elements in the array.
   *
   * @generated from rpc unitycatalog.tables.v1.TablesService.ListTables
   */
  listTables: {
    methodKind: "unary";
    input: typeof ListTablesRequestSchema;
    output: typeof ListTablesResponseSchema;
  },
  /**
   * Create a table
   *
   * @generated from rpc unitycatalog.tables.v1.TablesService.CreateTable
   */
  createTable: {
    methodKind: "unary";
    input: typeof CreateTableRequestSchema;
    output: typeof TableInfoSchema;
  },
  /**
   * Get a table
   *
   * @generated from rpc unitycatalog.tables.v1.TablesService.GetTable
   */
  getTable: {
    methodKind: "unary";
    input: typeof GetTableRequestSchema;
    output: typeof TableInfoSchema;
  },
  /**
   * Get boolean reflecting if table exists
   *
   * @generated from rpc unitycatalog.tables.v1.TablesService.GetTableExists
   */
  getTableExists: {
    methodKind: "unary";
    input: typeof GetTableExistsRequestSchema;
    output: typeof GetTableExistsResponseSchema;
  },
  /**
   * Delete a table
   *
   * @generated from rpc unitycatalog.tables.v1.TablesService.DeleteTable
   */
  deleteTable: {
    methodKind: "unary";
    input: typeof DeleteTableRequestSchema;
    output: typeof EmptySchema;
  },
}> = /*@__PURE__*/
  serviceDesc(file_unitycatalog_tables_v1_svc, 0);


// @generated by protoc-gen-es v2.2.3 with parameter "target=ts,json_types=true"
// @generated from file unitycatalog/sharing/v1/svc.proto (package unitycatalog.sharing.v1, syntax proto3)
/* eslint-disable */

import type { GenFile, GenMessage, GenService } from "@bufbuild/protobuf/codegenv1";
import { fileDesc, messageDesc, serviceDesc } from "@bufbuild/protobuf/codegenv1";
import { file_buf_validate_validate } from "../../../buf/validate/validate_pb";
import type { Share, ShareJson, ShareSchema, SharingSchema, SharingSchemaJson, SharingTable, SharingTableJson } from "./models_pb";
import { file_unitycatalog_sharing_v1_models } from "./models_pb";
import type { GetTableMetadataRequestSchema, GetTableVersionRequestSchema, GetTableVersionResponseSchema, QueryResponseSchema } from "./query_pb";
import { file_unitycatalog_sharing_v1_query } from "./query_pb";
import { file_gnostic_openapi_v3_annotations } from "../../../gnostic/openapi/v3/annotations_pb";
import { file_google_api_annotations } from "../../../google/api/annotations_pb";
import { file_google_api_field_behavior } from "../../../google/api/field_behavior_pb";
import { file_google_api_resource } from "../../../google/api/resource_pb";
import { file_google_protobuf_empty, file_google_protobuf_struct } from "@bufbuild/protobuf/wkt";
import type { Message } from "@bufbuild/protobuf";

/**
 * Describes the file unitycatalog/sharing/v1/svc.proto.
 */
export const file_unitycatalog_sharing_v1_svc: GenFile = /*@__PURE__*/
  fileDesc("CiF1bml0eWNhdGFsb2cvc2hhcmluZy92MS9zdmMucHJvdG8SF3VuaXR5Y2F0YWxvZy5zaGFyaW5nLnYxInkKEUxpc3RTaGFyZXNSZXF1ZXN0EicKC21heF9yZXN1bHRzGAEgASgFQg3gQQG6SAcaBRDoByAASACIAQESHAoKcGFnZV90b2tlbhgCIAEoCUID4EEBSAGIAQFCDgoMX21heF9yZXN1bHRzQg0KC19wYWdlX3Rva2VuInUKEkxpc3RTaGFyZXNSZXNwb25zZRItCgVpdGVtcxgBIAMoCzIeLnVuaXR5Y2F0YWxvZy5zaGFyaW5nLnYxLlNoYXJlEhwKD25leHRfcGFnZV90b2tlbhgCIAEoCUgAiAEBQhIKEF9uZXh0X3BhZ2VfdG9rZW4iRwoPR2V0U2hhcmVSZXF1ZXN0EjQKBG5hbWUYASABKAlCJuBBArpIIHIeEAEyGl5bYS16XVswLTlhLXouX10qWzAtOWEtel0kIrgBChlMaXN0U2hhcmluZ1NjaGVtYXNSZXF1ZXN0EjUKBXNoYXJlGAEgASgJQibgQQK6SCByHhABMhpeW2Etel1bMC05YS16Ll9dKlswLTlhLXpdJBInCgttYXhfcmVzdWx0cxgCIAEoBUIN4EEBukgHGgUQ6AcgAEgAiAEBEhwKCnBhZ2VfdG9rZW4YAyABKAlCA+BBAUgBiAEBQg4KDF9tYXhfcmVzdWx0c0INCgtfcGFnZV90b2tlbiKFAQoaTGlzdFNoYXJpbmdTY2hlbWFzUmVzcG9uc2USNQoFaXRlbXMYASADKAsyJi51bml0eWNhdGFsb2cuc2hhcmluZy52MS5TaGFyaW5nU2NoZW1hEhwKD25leHRfcGFnZV90b2tlbhgCIAEoCUgAiAEBQhIKEF9uZXh0X3BhZ2VfdG9rZW4i6QEKF0xpc3RTY2hlbWFUYWJsZXNSZXF1ZXN0EjQKBG5hbWUYASABKAlCJuBBArpIIHIeEAEyGl5bYS16XVswLTlhLXouX10qWzAtOWEtel0kEjUKBXNoYXJlGAIgASgJQibgQQK6SCByHhABMhpeW2Etel1bMC05YS16Ll9dKlswLTlhLXpdJBIkCgttYXhfcmVzdWx0cxgDIAEoBUIK4EEBukgEGgIgAEgAiAEBEhwKCnBhZ2VfdG9rZW4YBCABKAlCA+BBAUgBiAEBQg4KDF9tYXhfcmVzdWx0c0INCgtfcGFnZV90b2tlbiKCAQoYTGlzdFNjaGVtYVRhYmxlc1Jlc3BvbnNlEjQKBWl0ZW1zGAEgAygLMiUudW5pdHljYXRhbG9nLnNoYXJpbmcudjEuU2hhcmluZ1RhYmxlEhwKD25leHRfcGFnZV90b2tlbhgCIAEoCUgAiAEBQhIKEF9uZXh0X3BhZ2VfdG9rZW4itAEKFkxpc3RTaGFyZVRhYmxlc1JlcXVlc3QSNAoEbmFtZRgBIAEoCUIm4EECukggch4QATIaXlthLXpdWzAtOWEtei5fXSpbMC05YS16XSQSJwoLbWF4X3Jlc3VsdHMYAiABKAVCDeBBAbpIBxoFEOgHIABIAIgBARIcCgpwYWdlX3Rva2VuGAMgASgJQgPgQQFIAYgBAUIOCgxfbWF4X3Jlc3VsdHNCDQoLX3BhZ2VfdG9rZW4igQEKF0xpc3RTaGFyZVRhYmxlc1Jlc3BvbnNlEjQKBWl0ZW1zGAEgAygLMiUudW5pdHljYXRhbG9nLnNoYXJpbmcudjEuU2hhcmluZ1RhYmxlEhwKD25leHRfcGFnZV90b2tlbhgCIAEoCUgAiAEBQhIKEF9uZXh0X3BhZ2VfdG9rZW4y0QkKE0RlbHRhU2hhcmluZ1NlcnZpY2UShQEKCkxpc3RTaGFyZXMSKi51bml0eWNhdGFsb2cuc2hhcmluZy52MS5MaXN0U2hhcmVzUmVxdWVzdBorLnVuaXR5Y2F0YWxvZy5zaGFyaW5nLnYxLkxpc3RTaGFyZXNSZXNwb25zZSIeukcMKgpMaXN0U2hhcmVzgtPkkwIJEgcvc2hhcmVzEnsKCEdldFNoYXJlEigudW5pdHljYXRhbG9nLnNoYXJpbmcudjEuR2V0U2hhcmVSZXF1ZXN0Gh4udW5pdHljYXRhbG9nLnNoYXJpbmcudjEuU2hhcmUiJbpHCioIR2V0U2hhcmWC0+STAhISEC97bmFtZT1zaGFyZXMvKn0StgEKEkxpc3RTaGFyaW5nU2NoZW1hcxIyLnVuaXR5Y2F0YWxvZy5zaGFyaW5nLnYxLkxpc3RTaGFyaW5nU2NoZW1hc1JlcXVlc3QaMy51bml0eWNhdGFsb2cuc2hhcmluZy52MS5MaXN0U2hhcmluZ1NjaGVtYXNSZXNwb25zZSI3ukcUKhJMaXN0U2hhcmluZ1NjaGVtYXOC0+STAhoSGC9zaGFyZXMve3NoYXJlfX0vc2NoZW1hcxK7AQoQTGlzdFNjaGVtYVRhYmxlcxIwLnVuaXR5Y2F0YWxvZy5zaGFyaW5nLnYxLkxpc3RTY2hlbWFUYWJsZXNSZXF1ZXN0GjEudW5pdHljYXRhbG9nLnNoYXJpbmcudjEuTGlzdFNjaGVtYVRhYmxlc1Jlc3BvbnNlIkK6RxIqEExpc3RTY2hlbWFUYWJsZXOC0+STAicSJS9zaGFyZXMve3NoYXJlfS9zY2hlbWFzL3tuYW1lfS90YWJsZXMSrQEKD0xpc3RTaGFyZVRhYmxlcxIvLnVuaXR5Y2F0YWxvZy5zaGFyaW5nLnYxLkxpc3RTaGFyZVRhYmxlc1JlcXVlc3QaMC51bml0eWNhdGFsb2cuc2hhcmluZy52MS5MaXN0U2hhcmVUYWJsZXNSZXNwb25zZSI3ukcRKg9MaXN0U2hhcmVUYWJsZXOC0+STAh0SGy97bmFtZT1zaGFyZXMvKn0vYWxsLXRhYmxlcxLIAQoPR2V0VGFibGVWZXJzaW9uEi8udW5pdHljYXRhbG9nLnNoYXJpbmcudjEuR2V0VGFibGVWZXJzaW9uUmVxdWVzdBowLnVuaXR5Y2F0YWxvZy5zaGFyaW5nLnYxLkdldFRhYmxlVmVyc2lvblJlc3BvbnNlIlK6RxEqD0dldFRhYmxlVmVyc2lvboLT5JMCOBI2L3NoYXJlcy97c2hhcmV9L3NjaGVtYXMve3NjaGVtYX0vdGFibGVzL3tuYW1lfS92ZXJzaW9uEsIBChBHZXRUYWJsZU1ldGFkYXRhEjAudW5pdHljYXRhbG9nLnNoYXJpbmcudjEuR2V0VGFibGVNZXRhZGF0YVJlcXVlc3QaJi51bml0eWNhdGFsb2cuc2hhcmluZy52MS5RdWVyeVJlc3BvbnNlIlS6RxIqEEdldFRhYmxlTWV0YWRhdGGC0+STAjkSNy9zaGFyZXMve3NoYXJlfS9zY2hlbWFzL3tzY2hlbWF9L3RhYmxlcy97bmFtZX0vbWV0YWRhdGFC/gIKG2NvbS51bml0eWNhdGFsb2cuc2hhcmluZy52MUIIU3ZjUHJvdG9QAVpQZ2l0aHViLmNvbS9kZWx0YS1pbmN1YmF0b3IvZGVsdGEtc2hhcmluZy1ycy9nby91bml0eWNhdGFsb2cvc2hhcmluZy92MTtzaGFyaW5ndjGiAgNVU1iqAhdVbml0eWNhdGFsb2cuU2hhcmluZy5WMcoCF1VuaXR5Y2F0YWxvZ1xTaGFyaW5nXFYx4gIjVW5pdHljYXRhbG9nXFNoYXJpbmdcVjFcR1BCTWV0YWRhdGHqAhlVbml0eWNhdGFsb2c6OlNoYXJpbmc6OlYxukeDARKAAQoRRGVsdGEgU2hhcmluZyBBUEkSKEFuIE9wZW4gUHJvdG9jb2wgZm9yIFNlY3VyZSBEYXRhIFNoYXJpbmcqOgoJQUdQTCB2My4wEi1odHRwczovL3d3dy5nbnUub3JnL2xpY2Vuc2VzL2FncGwtMy4wLmVuLmh0bWwyBTAuMC4wYgZwcm90bzM", [file_buf_validate_validate, file_unitycatalog_sharing_v1_models, file_unitycatalog_sharing_v1_query, file_gnostic_openapi_v3_annotations, file_google_api_annotations, file_google_api_field_behavior, file_google_api_resource, file_google_protobuf_empty, file_google_protobuf_struct]);

/**
 * Request to list shares.
 *
 * @generated from message unitycatalog.sharing.v1.ListSharesRequest
 */
export type ListSharesRequest = Message<"unitycatalog.sharing.v1.ListSharesRequest"> & {
  /**
   * The maximum number of results per page that should be returned.
   *
   * @generated from field: optional int32 max_results = 1;
   */
  maxResults?: number;

  /**
   * Specifies a page token to use. Set pageToken to the nextPageToken returned
   * by a previous list request to get the next page of results.
   *
   * @generated from field: optional string page_token = 2;
   */
  pageToken?: string;
};

/**
 * Request to list shares.
 *
 * @generated from message unitycatalog.sharing.v1.ListSharesRequest
 */
export type ListSharesRequestJson = {
  /**
   * The maximum number of results per page that should be returned.
   *
   * @generated from field: optional int32 max_results = 1;
   */
  maxResults?: number;

  /**
   * Specifies a page token to use. Set pageToken to the nextPageToken returned
   * by a previous list request to get the next page of results.
   *
   * @generated from field: optional string page_token = 2;
   */
  pageToken?: string;
};

/**
 * Describes the message unitycatalog.sharing.v1.ListSharesRequest.
 * Use `create(ListSharesRequestSchema)` to create a new message.
 */
export const ListSharesRequestSchema: GenMessage<ListSharesRequest, ListSharesRequestJson> = /*@__PURE__*/
  messageDesc(file_unitycatalog_sharing_v1_svc, 0);

/**
 * Response for ListSharesRequest.
 *
 * @generated from message unitycatalog.sharing.v1.ListSharesResponse
 */
export type ListSharesResponse = Message<"unitycatalog.sharing.v1.ListSharesResponse"> & {
  /**
   * The shares that were requested.
   *
   * @generated from field: repeated unitycatalog.sharing.v1.Share items = 1;
   */
  items: Share[];

  /**
   * Token that can be used to retrieve the next page of shares.
   * An empty or missing token means that no more shares are available for retrieval.
   *
   * @generated from field: optional string next_page_token = 2;
   */
  nextPageToken?: string;
};

/**
 * Response for ListSharesRequest.
 *
 * @generated from message unitycatalog.sharing.v1.ListSharesResponse
 */
export type ListSharesResponseJson = {
  /**
   * The shares that were requested.
   *
   * @generated from field: repeated unitycatalog.sharing.v1.Share items = 1;
   */
  items?: ShareJson[];

  /**
   * Token that can be used to retrieve the next page of shares.
   * An empty or missing token means that no more shares are available for retrieval.
   *
   * @generated from field: optional string next_page_token = 2;
   */
  nextPageToken?: string;
};

/**
 * Describes the message unitycatalog.sharing.v1.ListSharesResponse.
 * Use `create(ListSharesResponseSchema)` to create a new message.
 */
export const ListSharesResponseSchema: GenMessage<ListSharesResponse, ListSharesResponseJson> = /*@__PURE__*/
  messageDesc(file_unitycatalog_sharing_v1_svc, 1);

/**
 * Get a share by name.
 *
 * @generated from message unitycatalog.sharing.v1.GetShareRequest
 */
export type GetShareRequest = Message<"unitycatalog.sharing.v1.GetShareRequest"> & {
  /**
   * The share name to query. It's case-insensitive.
   *
   * @generated from field: string name = 1;
   */
  name: string;
};

/**
 * Get a share by name.
 *
 * @generated from message unitycatalog.sharing.v1.GetShareRequest
 */
export type GetShareRequestJson = {
  /**
   * The share name to query. It's case-insensitive.
   *
   * @generated from field: string name = 1;
   */
  name?: string;
};

/**
 * Describes the message unitycatalog.sharing.v1.GetShareRequest.
 * Use `create(GetShareRequestSchema)` to create a new message.
 */
export const GetShareRequestSchema: GenMessage<GetShareRequest, GetShareRequestJson> = /*@__PURE__*/
  messageDesc(file_unitycatalog_sharing_v1_svc, 2);

/**
 * List schemas in a share.
 *
 * @generated from message unitycatalog.sharing.v1.ListSharingSchemasRequest
 */
export type ListSharingSchemasRequest = Message<"unitycatalog.sharing.v1.ListSharingSchemasRequest"> & {
  /**
   * The share name to query. It's case-insensitive.
   *
   * @generated from field: string share = 1;
   */
  share: string;

  /**
   * The maximum number of results per page that should be returned.
   *
   * @generated from field: optional int32 max_results = 2;
   */
  maxResults?: number;

  /**
   * Specifies a page token to use. Set pageToken to the nextPageToken returned
   * by a previous list request to get the next page of results.
   *
   * @generated from field: optional string page_token = 3;
   */
  pageToken?: string;
};

/**
 * List schemas in a share.
 *
 * @generated from message unitycatalog.sharing.v1.ListSharingSchemasRequest
 */
export type ListSharingSchemasRequestJson = {
  /**
   * The share name to query. It's case-insensitive.
   *
   * @generated from field: string share = 1;
   */
  share?: string;

  /**
   * The maximum number of results per page that should be returned.
   *
   * @generated from field: optional int32 max_results = 2;
   */
  maxResults?: number;

  /**
   * Specifies a page token to use. Set pageToken to the nextPageToken returned
   * by a previous list request to get the next page of results.
   *
   * @generated from field: optional string page_token = 3;
   */
  pageToken?: string;
};

/**
 * Describes the message unitycatalog.sharing.v1.ListSharingSchemasRequest.
 * Use `create(ListSharingSchemasRequestSchema)` to create a new message.
 */
export const ListSharingSchemasRequestSchema: GenMessage<ListSharingSchemasRequest, ListSharingSchemasRequestJson> = /*@__PURE__*/
  messageDesc(file_unitycatalog_sharing_v1_svc, 3);

/**
 * Response for ListSharingSchemasRequest.
 *
 * @generated from message unitycatalog.sharing.v1.ListSharingSchemasResponse
 */
export type ListSharingSchemasResponse = Message<"unitycatalog.sharing.v1.ListSharingSchemasResponse"> & {
  /**
   * The schemas that were requested.
   *
   * @generated from field: repeated unitycatalog.sharing.v1.SharingSchema items = 1;
   */
  items: SharingSchema[];

  /**
   * Token that can be used to retrieve the next page of schemas.
   * An empty or missing token means that no more schemas are available for retrieval.
   *
   * @generated from field: optional string next_page_token = 2;
   */
  nextPageToken?: string;
};

/**
 * Response for ListSharingSchemasRequest.
 *
 * @generated from message unitycatalog.sharing.v1.ListSharingSchemasResponse
 */
export type ListSharingSchemasResponseJson = {
  /**
   * The schemas that were requested.
   *
   * @generated from field: repeated unitycatalog.sharing.v1.SharingSchema items = 1;
   */
  items?: SharingSchemaJson[];

  /**
   * Token that can be used to retrieve the next page of schemas.
   * An empty or missing token means that no more schemas are available for retrieval.
   *
   * @generated from field: optional string next_page_token = 2;
   */
  nextPageToken?: string;
};

/**
 * Describes the message unitycatalog.sharing.v1.ListSharingSchemasResponse.
 * Use `create(ListSharingSchemasResponseSchema)` to create a new message.
 */
export const ListSharingSchemasResponseSchema: GenMessage<ListSharingSchemasResponse, ListSharingSchemasResponseJson> = /*@__PURE__*/
  messageDesc(file_unitycatalog_sharing_v1_svc, 4);

/**
 * List tables in a schema.
 *
 * @generated from message unitycatalog.sharing.v1.ListSchemaTablesRequest
 */
export type ListSchemaTablesRequest = Message<"unitycatalog.sharing.v1.ListSchemaTablesRequest"> & {
  /**
   * The schema name to query. It's case-insensitive.
   *
   * @generated from field: string name = 1;
   */
  name: string;

  /**
   * The share name to query. It's case-insensitive.
   *
   * @generated from field: string share = 2;
   */
  share: string;

  /**
   * The maximum number of results per page that should be returned.
   *
   * @generated from field: optional int32 max_results = 3;
   */
  maxResults?: number;

  /**
   * Specifies a page token to use. Set pageToken to the nextPageToken returned
   * by a previous list request to get the next page of results.
   *
   * @generated from field: optional string page_token = 4;
   */
  pageToken?: string;
};

/**
 * List tables in a schema.
 *
 * @generated from message unitycatalog.sharing.v1.ListSchemaTablesRequest
 */
export type ListSchemaTablesRequestJson = {
  /**
   * The schema name to query. It's case-insensitive.
   *
   * @generated from field: string name = 1;
   */
  name?: string;

  /**
   * The share name to query. It's case-insensitive.
   *
   * @generated from field: string share = 2;
   */
  share?: string;

  /**
   * The maximum number of results per page that should be returned.
   *
   * @generated from field: optional int32 max_results = 3;
   */
  maxResults?: number;

  /**
   * Specifies a page token to use. Set pageToken to the nextPageToken returned
   * by a previous list request to get the next page of results.
   *
   * @generated from field: optional string page_token = 4;
   */
  pageToken?: string;
};

/**
 * Describes the message unitycatalog.sharing.v1.ListSchemaTablesRequest.
 * Use `create(ListSchemaTablesRequestSchema)` to create a new message.
 */
export const ListSchemaTablesRequestSchema: GenMessage<ListSchemaTablesRequest, ListSchemaTablesRequestJson> = /*@__PURE__*/
  messageDesc(file_unitycatalog_sharing_v1_svc, 5);

/**
 * Response for ListSchemaTablesRequest.
 *
 * @generated from message unitycatalog.sharing.v1.ListSchemaTablesResponse
 */
export type ListSchemaTablesResponse = Message<"unitycatalog.sharing.v1.ListSchemaTablesResponse"> & {
  /**
   * The tables that were requested.
   *
   * @generated from field: repeated unitycatalog.sharing.v1.SharingTable items = 1;
   */
  items: SharingTable[];

  /**
   * Token that can be used to retrieve the next page of tables.
   * An empty or missing token means that no more tables are available for retrieval.
   *
   * @generated from field: optional string next_page_token = 2;
   */
  nextPageToken?: string;
};

/**
 * Response for ListSchemaTablesRequest.
 *
 * @generated from message unitycatalog.sharing.v1.ListSchemaTablesResponse
 */
export type ListSchemaTablesResponseJson = {
  /**
   * The tables that were requested.
   *
   * @generated from field: repeated unitycatalog.sharing.v1.SharingTable items = 1;
   */
  items?: SharingTableJson[];

  /**
   * Token that can be used to retrieve the next page of tables.
   * An empty or missing token means that no more tables are available for retrieval.
   *
   * @generated from field: optional string next_page_token = 2;
   */
  nextPageToken?: string;
};

/**
 * Describes the message unitycatalog.sharing.v1.ListSchemaTablesResponse.
 * Use `create(ListSchemaTablesResponseSchema)` to create a new message.
 */
export const ListSchemaTablesResponseSchema: GenMessage<ListSchemaTablesResponse, ListSchemaTablesResponseJson> = /*@__PURE__*/
  messageDesc(file_unitycatalog_sharing_v1_svc, 6);

/**
 * List tables in a share.
 *
 * @generated from message unitycatalog.sharing.v1.ListShareTablesRequest
 */
export type ListShareTablesRequest = Message<"unitycatalog.sharing.v1.ListShareTablesRequest"> & {
  /**
   * The share name to query. It's case-insensitive.
   *
   * @generated from field: string name = 1;
   */
  name: string;

  /**
   * The maximum number of results per page that should be returned.
   *
   * @generated from field: optional int32 max_results = 2;
   */
  maxResults?: number;

  /**
   * Specifies a page token to use. Set pageToken to the nextPageToken returned
   * by a previous list request to get the next page of results.
   *
   * @generated from field: optional string page_token = 3;
   */
  pageToken?: string;
};

/**
 * List tables in a share.
 *
 * @generated from message unitycatalog.sharing.v1.ListShareTablesRequest
 */
export type ListShareTablesRequestJson = {
  /**
   * The share name to query. It's case-insensitive.
   *
   * @generated from field: string name = 1;
   */
  name?: string;

  /**
   * The maximum number of results per page that should be returned.
   *
   * @generated from field: optional int32 max_results = 2;
   */
  maxResults?: number;

  /**
   * Specifies a page token to use. Set pageToken to the nextPageToken returned
   * by a previous list request to get the next page of results.
   *
   * @generated from field: optional string page_token = 3;
   */
  pageToken?: string;
};

/**
 * Describes the message unitycatalog.sharing.v1.ListShareTablesRequest.
 * Use `create(ListShareTablesRequestSchema)` to create a new message.
 */
export const ListShareTablesRequestSchema: GenMessage<ListShareTablesRequest, ListShareTablesRequestJson> = /*@__PURE__*/
  messageDesc(file_unitycatalog_sharing_v1_svc, 7);

/**
 * Response for ListShareTablesRequest.
 *
 * @generated from message unitycatalog.sharing.v1.ListShareTablesResponse
 */
export type ListShareTablesResponse = Message<"unitycatalog.sharing.v1.ListShareTablesResponse"> & {
  /**
   * The tables that were requested.
   *
   * @generated from field: repeated unitycatalog.sharing.v1.SharingTable items = 1;
   */
  items: SharingTable[];

  /**
   * Token that can be used to retrieve the next page of tables.
   * An empty or missing token means that no more tables are available for retrieval.
   *
   * @generated from field: optional string next_page_token = 2;
   */
  nextPageToken?: string;
};

/**
 * Response for ListShareTablesRequest.
 *
 * @generated from message unitycatalog.sharing.v1.ListShareTablesResponse
 */
export type ListShareTablesResponseJson = {
  /**
   * The tables that were requested.
   *
   * @generated from field: repeated unitycatalog.sharing.v1.SharingTable items = 1;
   */
  items?: SharingTableJson[];

  /**
   * Token that can be used to retrieve the next page of tables.
   * An empty or missing token means that no more tables are available for retrieval.
   *
   * @generated from field: optional string next_page_token = 2;
   */
  nextPageToken?: string;
};

/**
 * Describes the message unitycatalog.sharing.v1.ListShareTablesResponse.
 * Use `create(ListShareTablesResponseSchema)` to create a new message.
 */
export const ListShareTablesResponseSchema: GenMessage<ListShareTablesResponse, ListShareTablesResponseJson> = /*@__PURE__*/
  messageDesc(file_unitycatalog_sharing_v1_svc, 8);

/**
 * Service exposing the official APIs for Delta Sharing.
 *
 * @generated from service unitycatalog.sharing.v1.DeltaSharingService
 */
export const DeltaSharingService: GenService<{
  /**
   * List shares accessible to a recipient.
   *
   * @generated from rpc unitycatalog.sharing.v1.DeltaSharingService.ListShares
   */
  listShares: {
    methodKind: "unary";
    input: typeof ListSharesRequestSchema;
    output: typeof ListSharesResponseSchema;
  },
  /**
   * Get the metadata for a specific share.
   *
   * @generated from rpc unitycatalog.sharing.v1.DeltaSharingService.GetShare
   */
  getShare: {
    methodKind: "unary";
    input: typeof GetShareRequestSchema;
    output: typeof ShareSchema;
  },
  /**
   * List the schemas in a share.
   *
   * @generated from rpc unitycatalog.sharing.v1.DeltaSharingService.ListSharingSchemas
   */
  listSharingSchemas: {
    methodKind: "unary";
    input: typeof ListSharingSchemasRequestSchema;
    output: typeof ListSharingSchemasResponseSchema;
  },
  /**
   * List the tables in a given share's schema.
   *
   * @generated from rpc unitycatalog.sharing.v1.DeltaSharingService.ListSchemaTables
   */
  listSchemaTables: {
    methodKind: "unary";
    input: typeof ListSchemaTablesRequestSchema;
    output: typeof ListSchemaTablesResponseSchema;
  },
  /**
   * List all the tables under all schemas in a share.
   *
   * @generated from rpc unitycatalog.sharing.v1.DeltaSharingService.ListShareTables
   */
  listShareTables: {
    methodKind: "unary";
    input: typeof ListShareTablesRequestSchema;
    output: typeof ListShareTablesResponseSchema;
  },
  /**
   * Get the current version for a table within a schema.
   *
   * @generated from rpc unitycatalog.sharing.v1.DeltaSharingService.GetTableVersion
   */
  getTableVersion: {
    methodKind: "unary";
    input: typeof GetTableVersionRequestSchema;
    output: typeof GetTableVersionResponseSchema;
  },
  /**
   * @generated from rpc unitycatalog.sharing.v1.DeltaSharingService.GetTableMetadata
   */
  getTableMetadata: {
    methodKind: "unary";
    input: typeof GetTableMetadataRequestSchema;
    output: typeof QueryResponseSchema;
  },
}> = /*@__PURE__*/
  serviceDesc(file_unitycatalog_sharing_v1_svc, 0);


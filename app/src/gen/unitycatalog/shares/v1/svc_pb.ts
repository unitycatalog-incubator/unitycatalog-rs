// @generated by protoc-gen-es v2.2.3 with parameter "target=ts,json_types=true"
// @generated from file unitycatalog/shares/v1/svc.proto (package unitycatalog.shares.v1, syntax proto3)
/* eslint-disable */

import type { GenEnum, GenFile, GenMessage, GenService } from "@bufbuild/protobuf/codegenv1";
import { enumDesc, fileDesc, messageDesc, serviceDesc } from "@bufbuild/protobuf/codegenv1";
import { file_buf_validate_validate } from "../../../buf/validate/validate_pb";
import type { DataObject, DataObjectJson, ShareInfo, ShareInfoJson, ShareInfoSchema } from "./models_pb";
import { file_unitycatalog_shares_v1_models } from "./models_pb";
import { file_gnostic_openapi_v3_annotations } from "../../../gnostic/openapi/v3/annotations_pb";
import { file_google_api_annotations } from "../../../google/api/annotations_pb";
import { file_google_api_field_behavior } from "../../../google/api/field_behavior_pb";
import type { EmptySchema } from "@bufbuild/protobuf/wkt";
import { file_google_protobuf_empty } from "@bufbuild/protobuf/wkt";
import type { Message } from "@bufbuild/protobuf";

/**
 * Describes the file unitycatalog/shares/v1/svc.proto.
 */
export const file_unitycatalog_shares_v1_svc: GenFile = /*@__PURE__*/
  fileDesc("CiB1bml0eWNhdGFsb2cvc2hhcmVzL3YxL3N2Yy5wcm90bxIWdW5pdHljYXRhbG9nLnNoYXJlcy52MSJ5ChFMaXN0U2hhcmVzUmVxdWVzdBInCgttYXhfcmVzdWx0cxgBIAEoBUIN4EEBukgHGgUQ6AcgAEgAiAEBEhwKCnBhZ2VfdG9rZW4YAiABKAlCA+BBAUgBiAEBQg4KDF9tYXhfcmVzdWx0c0INCgtfcGFnZV90b2tlbiJ5ChJMaXN0U2hhcmVzUmVzcG9uc2USMQoGc2hhcmVzGAEgAygLMiEudW5pdHljYXRhbG9nLnNoYXJlcy52MS5TaGFyZUluZm8SHAoPbmV4dF9wYWdlX3Rva2VuGAIgASgJSACIAQFCEgoQX25leHRfcGFnZV90b2tlbiJuChJDcmVhdGVTaGFyZVJlcXVlc3QSMQoEbmFtZRgBIAEoCUIj4EECukgdchsyGV5bYS16XVswLTlhLXpfXSpbMC05YS16XSQSGQoHY29tbWVudBgCIAEoCUID4EEBSACIAQFCCgoIX2NvbW1lbnQigwEKD0dldFNoYXJlUmVxdWVzdBIxCgRuYW1lGAEgASgJQiPgQQK6SB1yGzIZXlthLXpdWzAtOWEtel9dKlswLTlhLXpdJBIlChNpbmNsdWRlX3NoYXJlZF9kYXRhGAIgASgIQgPgQQFIAIgBAUIWChRfaW5jbHVkZV9zaGFyZWRfZGF0YSKFAQoQRGF0YU9iamVjdFVwZGF0ZRIzCgZhY3Rpb24YASABKA4yHi51bml0eWNhdGFsb2cuc2hhcmVzLnYxLkFjdGlvbkID4EECEjwKC2RhdGFfb2JqZWN0GAIgASgLMiIudW5pdHljYXRhbG9nLnNoYXJlcy52MS5EYXRhT2JqZWN0QgPgQQIinAIKElVwZGF0ZVNoYXJlUmVxdWVzdBIyCgRuYW1lGAEgASgJQiTgQQK6SB5yHDIaXlthLXpdWzAtOWEtei5fXSpbMC05YS16XSQSPgoHdXBkYXRlcxgCIAMoCzIoLnVuaXR5Y2F0YWxvZy5zaGFyZXMudjEuRGF0YU9iamVjdFVwZGF0ZUID4EEBEjsKCG5ld19uYW1lGAMgASgJQiTgQQG6SB5yHDIaXlthLXpdWzAtOWEtei5fXSpbMC05YS16XSRIAIgBARIXCgVvd25lchgEIAEoCUID4EEBSAGIAQESGQoHY29tbWVudBgFIAEoCUID4EEBSAKIAQFCCwoJX25ld19uYW1lQggKBl9vd25lckIKCghfY29tbWVudCJIChJEZWxldGVTaGFyZVJlcXVlc3QSMgoEbmFtZRgBIAEoCUIk4EECukgechwyGl5bYS16XVswLTlhLXouX10qWzAtOWEtel0kKkEKBkFjdGlvbhIWChJBQ1RJT05fVU5TUEVDSUZJRUQQABIHCgNBREQQARIKCgZSRU1PVkUQAhIKCgZVUERBVEUQAzKaBQoNU2hhcmVzU2VydmljZRKDAQoKTGlzdFNoYXJlcxIpLnVuaXR5Y2F0YWxvZy5zaGFyZXMudjEuTGlzdFNoYXJlc1JlcXVlc3QaKi51bml0eWNhdGFsb2cuc2hhcmVzLnYxLkxpc3RTaGFyZXNSZXNwb25zZSIeukcMKgpMaXN0U2hhcmVzgtPkkwIJEgcvc2hhcmVzEoABCgtDcmVhdGVTaGFyZRIqLnVuaXR5Y2F0YWxvZy5zaGFyZXMudjEuQ3JlYXRlU2hhcmVSZXF1ZXN0GiEudW5pdHljYXRhbG9nLnNoYXJlcy52MS5TaGFyZUluZm8iIrpHDSoLQ3JlYXRlU2hhcmWC0+STAgw6ASoiBy9zaGFyZXMSewoIR2V0U2hhcmUSJy51bml0eWNhdGFsb2cuc2hhcmVzLnYxLkdldFNoYXJlUmVxdWVzdBohLnVuaXR5Y2F0YWxvZy5zaGFyZXMudjEuU2hhcmVJbmZvIiO6RwoqCEdldFNoYXJlgtPkkwIQEg4vc2hhcmVzL3tuYW1lfRKHAQoLVXBkYXRlU2hhcmUSKi51bml0eWNhdGFsb2cuc2hhcmVzLnYxLlVwZGF0ZVNoYXJlUmVxdWVzdBohLnVuaXR5Y2F0YWxvZy5zaGFyZXMudjEuU2hhcmVJbmZvIim6Rw0qC1VwZGF0ZVNoYXJlgtPkkwITOgEqMg4vc2hhcmVzL3tuYW1lfRJ5CgtEZWxldGVTaGFyZRIqLnVuaXR5Y2F0YWxvZy5zaGFyZXMudjEuRGVsZXRlU2hhcmVSZXF1ZXN0GhYuZ29vZ2xlLnByb3RvYnVmLkVtcHR5Iia6Rw0qC0RlbGV0ZVNoYXJlgtPkkwIQKg4vc2hhcmVzL3tuYW1lfULwAQoaY29tLnVuaXR5Y2F0YWxvZy5zaGFyZXMudjFCCFN2Y1Byb3RvUAFaTmdpdGh1Yi5jb20vZGVsdGEtaW5jdWJhdG9yL2RlbHRhLXNoYXJpbmctcnMvZ28vdW5pdHljYXRhbG9nL3NoYXJlcy92MTtzaGFyZXN2MaICA1VTWKoCFlVuaXR5Y2F0YWxvZy5TaGFyZXMuVjHKAhZVbml0eWNhdGFsb2dcU2hhcmVzXFYx4gIiVW5pdHljYXRhbG9nXFNoYXJlc1xWMVxHUEJNZXRhZGF0YeoCGFVuaXR5Y2F0YWxvZzo6U2hhcmVzOjpWMWIGcHJvdG8z", [file_buf_validate_validate, file_unitycatalog_shares_v1_models, file_gnostic_openapi_v3_annotations, file_google_api_annotations, file_google_api_field_behavior, file_google_protobuf_empty]);

/**
 * Request to list shares.
 *
 * @generated from message unitycatalog.shares.v1.ListSharesRequest
 */
export type ListSharesRequest = Message<"unitycatalog.shares.v1.ListSharesRequest"> & {
  /**
   * The maximum number of results per page that should be returned.
   *
   * @generated from field: optional int32 max_results = 1;
   */
  maxResults?: number;

  /**
   * Opaque pagination token to go to next page based on previous query.
   *
   * @generated from field: optional string page_token = 2;
   */
  pageToken?: string;
};

/**
 * Request to list shares.
 *
 * @generated from message unitycatalog.shares.v1.ListSharesRequest
 */
export type ListSharesRequestJson = {
  /**
   * The maximum number of results per page that should be returned.
   *
   * @generated from field: optional int32 max_results = 1;
   */
  maxResults?: number;

  /**
   * Opaque pagination token to go to next page based on previous query.
   *
   * @generated from field: optional string page_token = 2;
   */
  pageToken?: string;
};

/**
 * Describes the message unitycatalog.shares.v1.ListSharesRequest.
 * Use `create(ListSharesRequestSchema)` to create a new message.
 */
export const ListSharesRequestSchema: GenMessage<ListSharesRequest, ListSharesRequestJson> = /*@__PURE__*/
  messageDesc(file_unitycatalog_shares_v1_svc, 0);

/**
 * Response to list shares.
 *
 * @generated from message unitycatalog.shares.v1.ListSharesResponse
 */
export type ListSharesResponse = Message<"unitycatalog.shares.v1.ListSharesResponse"> & {
  /**
   * List of shares.
   *
   * @generated from field: repeated unitycatalog.shares.v1.ShareInfo shares = 1;
   */
  shares: ShareInfo[];

  /**
   * Opaque pagination token to go to next page based on previous query.
   *
   * @generated from field: optional string next_page_token = 2;
   */
  nextPageToken?: string;
};

/**
 * Response to list shares.
 *
 * @generated from message unitycatalog.shares.v1.ListSharesResponse
 */
export type ListSharesResponseJson = {
  /**
   * List of shares.
   *
   * @generated from field: repeated unitycatalog.shares.v1.ShareInfo shares = 1;
   */
  shares?: ShareInfoJson[];

  /**
   * Opaque pagination token to go to next page based on previous query.
   *
   * @generated from field: optional string next_page_token = 2;
   */
  nextPageToken?: string;
};

/**
 * Describes the message unitycatalog.shares.v1.ListSharesResponse.
 * Use `create(ListSharesResponseSchema)` to create a new message.
 */
export const ListSharesResponseSchema: GenMessage<ListSharesResponse, ListSharesResponseJson> = /*@__PURE__*/
  messageDesc(file_unitycatalog_shares_v1_svc, 1);

/**
 * Creates a new share for data objects.
 *
 * Data objects can be added after creation with update.
 * The caller must be a metastore admin or have the CREATE_SHARE privilege on the metastore.
 *
 * @generated from message unitycatalog.shares.v1.CreateShareRequest
 */
export type CreateShareRequest = Message<"unitycatalog.shares.v1.CreateShareRequest"> & {
  /**
   * Name of the share.
   *
   * @generated from field: string name = 1;
   */
  name: string;

  /**
   * User-provided free-form text description.
   *
   * @generated from field: optional string comment = 2;
   */
  comment?: string;
};

/**
 * Creates a new share for data objects.
 *
 * Data objects can be added after creation with update.
 * The caller must be a metastore admin or have the CREATE_SHARE privilege on the metastore.
 *
 * @generated from message unitycatalog.shares.v1.CreateShareRequest
 */
export type CreateShareRequestJson = {
  /**
   * Name of the share.
   *
   * @generated from field: string name = 1;
   */
  name?: string;

  /**
   * User-provided free-form text description.
   *
   * @generated from field: optional string comment = 2;
   */
  comment?: string;
};

/**
 * Describes the message unitycatalog.shares.v1.CreateShareRequest.
 * Use `create(CreateShareRequestSchema)` to create a new message.
 */
export const CreateShareRequestSchema: GenMessage<CreateShareRequest, CreateShareRequestJson> = /*@__PURE__*/
  messageDesc(file_unitycatalog_shares_v1_svc, 2);

/**
 * Get a share by name.
 *
 * @generated from message unitycatalog.shares.v1.GetShareRequest
 */
export type GetShareRequest = Message<"unitycatalog.shares.v1.GetShareRequest"> & {
  /**
   * Name of the share.
   *
   * @generated from field: string name = 1;
   */
  name: string;

  /**
   * Query for data to include in the share.
   *
   * @generated from field: optional bool include_shared_data = 2;
   */
  includeSharedData?: boolean;
};

/**
 * Get a share by name.
 *
 * @generated from message unitycatalog.shares.v1.GetShareRequest
 */
export type GetShareRequestJson = {
  /**
   * Name of the share.
   *
   * @generated from field: string name = 1;
   */
  name?: string;

  /**
   * Query for data to include in the share.
   *
   * @generated from field: optional bool include_shared_data = 2;
   */
  includeSharedData?: boolean;
};

/**
 * Describes the message unitycatalog.shares.v1.GetShareRequest.
 * Use `create(GetShareRequestSchema)` to create a new message.
 */
export const GetShareRequestSchema: GenMessage<GetShareRequest, GetShareRequestJson> = /*@__PURE__*/
  messageDesc(file_unitycatalog_shares_v1_svc, 3);

/**
 * Data object update.
 *
 * @generated from message unitycatalog.shares.v1.DataObjectUpdate
 */
export type DataObjectUpdate = Message<"unitycatalog.shares.v1.DataObjectUpdate"> & {
  /**
   * Name of the share.
   *
   * @generated from field: unitycatalog.shares.v1.Action action = 1;
   */
  action: Action;

  /**
   * User-provided free-form text description.
   *
   * @generated from field: unitycatalog.shares.v1.DataObject data_object = 2;
   */
  dataObject?: DataObject;
};

/**
 * Data object update.
 *
 * @generated from message unitycatalog.shares.v1.DataObjectUpdate
 */
export type DataObjectUpdateJson = {
  /**
   * Name of the share.
   *
   * @generated from field: unitycatalog.shares.v1.Action action = 1;
   */
  action?: ActionJson;

  /**
   * User-provided free-form text description.
   *
   * @generated from field: unitycatalog.shares.v1.DataObject data_object = 2;
   */
  dataObject?: DataObjectJson;
};

/**
 * Describes the message unitycatalog.shares.v1.DataObjectUpdate.
 * Use `create(DataObjectUpdateSchema)` to create a new message.
 */
export const DataObjectUpdateSchema: GenMessage<DataObjectUpdate, DataObjectUpdateJson> = /*@__PURE__*/
  messageDesc(file_unitycatalog_shares_v1_svc, 4);

/**
 * Update a share.
 *
 * The caller must be a metastore admin or have the UPDATE_SHARE privilege on the metastore.
 *
 * @generated from message unitycatalog.shares.v1.UpdateShareRequest
 */
export type UpdateShareRequest = Message<"unitycatalog.shares.v1.UpdateShareRequest"> & {
  /**
   * Name of the share.
   *
   * @generated from field: string name = 1;
   */
  name: string;

  /**
   * Array of shared data object updates.
   *
   * @generated from field: repeated unitycatalog.shares.v1.DataObjectUpdate updates = 2;
   */
  updates: DataObjectUpdate[];

  /**
   * A new name for the share.
   *
   * @generated from field: optional string new_name = 3;
   */
  newName?: string;

  /**
   * Owner of the share.
   *
   * @generated from field: optional string owner = 4;
   */
  owner?: string;

  /**
   * User-provided free-form text description.
   *
   * @generated from field: optional string comment = 5;
   */
  comment?: string;
};

/**
 * Update a share.
 *
 * The caller must be a metastore admin or have the UPDATE_SHARE privilege on the metastore.
 *
 * @generated from message unitycatalog.shares.v1.UpdateShareRequest
 */
export type UpdateShareRequestJson = {
  /**
   * Name of the share.
   *
   * @generated from field: string name = 1;
   */
  name?: string;

  /**
   * Array of shared data object updates.
   *
   * @generated from field: repeated unitycatalog.shares.v1.DataObjectUpdate updates = 2;
   */
  updates?: DataObjectUpdateJson[];

  /**
   * A new name for the share.
   *
   * @generated from field: optional string new_name = 3;
   */
  newName?: string;

  /**
   * Owner of the share.
   *
   * @generated from field: optional string owner = 4;
   */
  owner?: string;

  /**
   * User-provided free-form text description.
   *
   * @generated from field: optional string comment = 5;
   */
  comment?: string;
};

/**
 * Describes the message unitycatalog.shares.v1.UpdateShareRequest.
 * Use `create(UpdateShareRequestSchema)` to create a new message.
 */
export const UpdateShareRequestSchema: GenMessage<UpdateShareRequest, UpdateShareRequestJson> = /*@__PURE__*/
  messageDesc(file_unitycatalog_shares_v1_svc, 5);

/**
 * Delete a share.
 *
 * The caller must be a metastore admin or have the DELETE_SHARE privilege on the metastore.
 *
 * @generated from message unitycatalog.shares.v1.DeleteShareRequest
 */
export type DeleteShareRequest = Message<"unitycatalog.shares.v1.DeleteShareRequest"> & {
  /**
   * Name of the share.
   *
   * @generated from field: string name = 1;
   */
  name: string;
};

/**
 * Delete a share.
 *
 * The caller must be a metastore admin or have the DELETE_SHARE privilege on the metastore.
 *
 * @generated from message unitycatalog.shares.v1.DeleteShareRequest
 */
export type DeleteShareRequestJson = {
  /**
   * Name of the share.
   *
   * @generated from field: string name = 1;
   */
  name?: string;
};

/**
 * Describes the message unitycatalog.shares.v1.DeleteShareRequest.
 * Use `create(DeleteShareRequestSchema)` to create a new message.
 */
export const DeleteShareRequestSchema: GenMessage<DeleteShareRequest, DeleteShareRequestJson> = /*@__PURE__*/
  messageDesc(file_unitycatalog_shares_v1_svc, 6);

/**
 * @generated from enum unitycatalog.shares.v1.Action
 */
export enum Action {
  /**
   * Unspecified action.
   *
   * @generated from enum value: ACTION_UNSPECIFIED = 0;
   */
  ACTION_UNSPECIFIED = 0,

  /**
   * @generated from enum value: ADD = 1;
   */
  ADD = 1,

  /**
   * @generated from enum value: REMOVE = 2;
   */
  REMOVE = 2,

  /**
   * @generated from enum value: UPDATE = 3;
   */
  UPDATE = 3,
}

/**
 * @generated from enum unitycatalog.shares.v1.Action
 */
export type ActionJson = "ACTION_UNSPECIFIED" | "ADD" | "REMOVE" | "UPDATE";

/**
 * Describes the enum unitycatalog.shares.v1.Action.
 */
export const ActionSchema: GenEnum<Action, ActionJson> = /*@__PURE__*/
  enumDesc(file_unitycatalog_shares_v1_svc, 0);

/**
 * Service for managing shares
 *
 * @generated from service unitycatalog.shares.v1.SharesService
 */
export const SharesService: GenService<{
  /**
   * List shares.
   *
   * @generated from rpc unitycatalog.shares.v1.SharesService.ListShares
   */
  listShares: {
    methodKind: "unary";
    input: typeof ListSharesRequestSchema;
    output: typeof ListSharesResponseSchema;
  },
  /**
   * Create a new share.
   *
   * @generated from rpc unitycatalog.shares.v1.SharesService.CreateShare
   */
  createShare: {
    methodKind: "unary";
    input: typeof CreateShareRequestSchema;
    output: typeof ShareInfoSchema;
  },
  /**
   * Get a share by name.
   *
   * @generated from rpc unitycatalog.shares.v1.SharesService.GetShare
   */
  getShare: {
    methodKind: "unary";
    input: typeof GetShareRequestSchema;
    output: typeof ShareInfoSchema;
  },
  /**
   * Update a share.
   *
   * @generated from rpc unitycatalog.shares.v1.SharesService.UpdateShare
   */
  updateShare: {
    methodKind: "unary";
    input: typeof UpdateShareRequestSchema;
    output: typeof ShareInfoSchema;
  },
  /**
   * Deletes a share.
   *
   * @generated from rpc unitycatalog.shares.v1.SharesService.DeleteShare
   */
  deleteShare: {
    methodKind: "unary";
    input: typeof DeleteShareRequestSchema;
    output: typeof EmptySchema;
  },
}> = /*@__PURE__*/
  serviceDesc(file_unitycatalog_shares_v1_svc, 0);


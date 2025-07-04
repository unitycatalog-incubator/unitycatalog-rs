// @generated by protoc-gen-es v2.5.2 with parameter "target=ts,json_types=false"
// @generated from file unitycatalog/shares/v1/svc.proto (package unitycatalog.shares.v1, syntax proto3)
/* eslint-disable */

import type { GenEnum, GenFile, GenMessage, GenService } from "@bufbuild/protobuf/codegenv2";
import { enumDesc, fileDesc, messageDesc, serviceDesc } from "@bufbuild/protobuf/codegenv2";
import { file_buf_validate_validate } from "../../../buf/validate/validate_pb";
import { file_gnostic_openapi_v3_annotations } from "../../../gnostic/openapi/v3/annotations_pb";
import { file_google_api_annotations } from "../../../google/api/annotations_pb";
import { file_google_api_field_behavior } from "../../../google/api/field_behavior_pb";
import type { EmptySchema } from "@bufbuild/protobuf/wkt";
import { file_google_protobuf_empty } from "@bufbuild/protobuf/wkt";
import type { DataObject, ShareInfo, ShareInfoSchema } from "./models_pb";
import { file_unitycatalog_shares_v1_models } from "./models_pb";
import type { Message } from "@bufbuild/protobuf";

/**
 * Describes the file unitycatalog/shares/v1/svc.proto.
 */
export const file_unitycatalog_shares_v1_svc: GenFile = /*@__PURE__*/
  fileDesc("CiB1bml0eWNhdGFsb2cvc2hhcmVzL3YxL3N2Yy5wcm90bxIWdW5pdHljYXRhbG9nLnNoYXJlcy52MSJ5ChFMaXN0U2hhcmVzUmVxdWVzdBInCgttYXhfcmVzdWx0cxgBIAEoBUIN4EEBukgHGgUQ6AcgAEgAiAEBEhwKCnBhZ2VfdG9rZW4YAiABKAlCA+BBAUgBiAEBQg4KDF9tYXhfcmVzdWx0c0INCgtfcGFnZV90b2tlbiJ5ChJMaXN0U2hhcmVzUmVzcG9uc2USMQoGc2hhcmVzGAEgAygLMiEudW5pdHljYXRhbG9nLnNoYXJlcy52MS5TaGFyZUluZm8SHAoPbmV4dF9wYWdlX3Rva2VuGAIgASgJSACIAQFCEgoQX25leHRfcGFnZV90b2tlbiJwChJDcmVhdGVTaGFyZVJlcXVlc3QSMwoEbmFtZRgBIAEoCUIl4EECukgfch0QAzIZXlthLXpdWzAtOWEtel9dKlswLTlhLXpdJBIZCgdjb21tZW50GAIgASgJQgPgQQFIAIgBAUIKCghfY29tbWVudCKDAQoPR2V0U2hhcmVSZXF1ZXN0EjEKBG5hbWUYASABKAlCI+BBArpIHXIbMhleW2Etel1bMC05YS16X10qWzAtOWEtel0kEiUKE2luY2x1ZGVfc2hhcmVkX2RhdGEYAiABKAhCA+BBAUgAiAEBQhYKFF9pbmNsdWRlX3NoYXJlZF9kYXRhIoUBChBEYXRhT2JqZWN0VXBkYXRlEjMKBmFjdGlvbhgBIAEoDjIeLnVuaXR5Y2F0YWxvZy5zaGFyZXMudjEuQWN0aW9uQgPgQQISPAoLZGF0YV9vYmplY3QYAiABKAsyIi51bml0eWNhdGFsb2cuc2hhcmVzLnYxLkRhdGFPYmplY3RCA+BBAiKgAgoSVXBkYXRlU2hhcmVSZXF1ZXN0EjQKBG5hbWUYASABKAlCJuBBArpIIHIeEAMyGl5bYS16XVswLTlhLXouX10qWzAtOWEtel0kEj4KB3VwZGF0ZXMYAiADKAsyKC51bml0eWNhdGFsb2cuc2hhcmVzLnYxLkRhdGFPYmplY3RVcGRhdGVCA+BBARI9CghuZXdfbmFtZRgDIAEoCUIm4EEBukggch4QAzIaXlthLXpdWzAtOWEtei5fXSpbMC05YS16XSRIAIgBARIXCgVvd25lchgEIAEoCUID4EEBSAGIAQESGQoHY29tbWVudBgFIAEoCUID4EEBSAKIAQFCCwoJX25ld19uYW1lQggKBl9vd25lckIKCghfY29tbWVudCJKChJEZWxldGVTaGFyZVJlcXVlc3QSNAoEbmFtZRgBIAEoCUIm4EECukggch4QAzIaXlthLXpdWzAtOWEtei5fXSpbMC05YS16XSQqQQoGQWN0aW9uEhYKEkFDVElPTl9VTlNQRUNJRklFRBAAEgcKA0FERBABEgoKBlJFTU9WRRACEgoKBlVQREFURRADMpoFCg1TaGFyZXNTZXJ2aWNlEoMBCgpMaXN0U2hhcmVzEikudW5pdHljYXRhbG9nLnNoYXJlcy52MS5MaXN0U2hhcmVzUmVxdWVzdBoqLnVuaXR5Y2F0YWxvZy5zaGFyZXMudjEuTGlzdFNoYXJlc1Jlc3BvbnNlIh66RwwqCkxpc3RTaGFyZXOC0+STAgkSBy9zaGFyZXMSgAEKC0NyZWF0ZVNoYXJlEioudW5pdHljYXRhbG9nLnNoYXJlcy52MS5DcmVhdGVTaGFyZVJlcXVlc3QaIS51bml0eWNhdGFsb2cuc2hhcmVzLnYxLlNoYXJlSW5mbyIiukcNKgtDcmVhdGVTaGFyZYLT5JMCDDoBKiIHL3NoYXJlcxJ7CghHZXRTaGFyZRInLnVuaXR5Y2F0YWxvZy5zaGFyZXMudjEuR2V0U2hhcmVSZXF1ZXN0GiEudW5pdHljYXRhbG9nLnNoYXJlcy52MS5TaGFyZUluZm8iI7pHCioIR2V0U2hhcmWC0+STAhASDi9zaGFyZXMve25hbWV9EocBCgtVcGRhdGVTaGFyZRIqLnVuaXR5Y2F0YWxvZy5zaGFyZXMudjEuVXBkYXRlU2hhcmVSZXF1ZXN0GiEudW5pdHljYXRhbG9nLnNoYXJlcy52MS5TaGFyZUluZm8iKbpHDSoLVXBkYXRlU2hhcmWC0+STAhM6ASoyDi9zaGFyZXMve25hbWV9EnkKC0RlbGV0ZVNoYXJlEioudW5pdHljYXRhbG9nLnNoYXJlcy52MS5EZWxldGVTaGFyZVJlcXVlc3QaFi5nb29nbGUucHJvdG9idWYuRW1wdHkiJrpHDSoLRGVsZXRlU2hhcmWC0+STAhAqDi9zaGFyZXMve25hbWV9QvABChpjb20udW5pdHljYXRhbG9nLnNoYXJlcy52MUIIU3ZjUHJvdG9QAVpOZ2l0aHViLmNvbS9kZWx0YS1pbmN1YmF0b3IvZGVsdGEtc2hhcmluZy1ycy9nby91bml0eWNhdGFsb2cvc2hhcmVzL3YxO3NoYXJlc3YxogIDVVNYqgIWVW5pdHljYXRhbG9nLlNoYXJlcy5WMcoCFlVuaXR5Y2F0YWxvZ1xTaGFyZXNcVjHiAiJVbml0eWNhdGFsb2dcU2hhcmVzXFYxXEdQQk1ldGFkYXRh6gIYVW5pdHljYXRhbG9nOjpTaGFyZXM6OlYxYgZwcm90bzM", [file_buf_validate_validate, file_gnostic_openapi_v3_annotations, file_google_api_annotations, file_google_api_field_behavior, file_google_protobuf_empty, file_unitycatalog_shares_v1_models]);

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
 * Describes the message unitycatalog.shares.v1.ListSharesRequest.
 * Use `create(ListSharesRequestSchema)` to create a new message.
 */
export const ListSharesRequestSchema: GenMessage<ListSharesRequest> = /*@__PURE__*/
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
 * Describes the message unitycatalog.shares.v1.ListSharesResponse.
 * Use `create(ListSharesResponseSchema)` to create a new message.
 */
export const ListSharesResponseSchema: GenMessage<ListSharesResponse> = /*@__PURE__*/
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
 * Describes the message unitycatalog.shares.v1.CreateShareRequest.
 * Use `create(CreateShareRequestSchema)` to create a new message.
 */
export const CreateShareRequestSchema: GenMessage<CreateShareRequest> = /*@__PURE__*/
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
 * Describes the message unitycatalog.shares.v1.GetShareRequest.
 * Use `create(GetShareRequestSchema)` to create a new message.
 */
export const GetShareRequestSchema: GenMessage<GetShareRequest> = /*@__PURE__*/
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
 * Describes the message unitycatalog.shares.v1.DataObjectUpdate.
 * Use `create(DataObjectUpdateSchema)` to create a new message.
 */
export const DataObjectUpdateSchema: GenMessage<DataObjectUpdate> = /*@__PURE__*/
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
 * Describes the message unitycatalog.shares.v1.UpdateShareRequest.
 * Use `create(UpdateShareRequestSchema)` to create a new message.
 */
export const UpdateShareRequestSchema: GenMessage<UpdateShareRequest> = /*@__PURE__*/
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
 * Describes the message unitycatalog.shares.v1.DeleteShareRequest.
 * Use `create(DeleteShareRequestSchema)` to create a new message.
 */
export const DeleteShareRequestSchema: GenMessage<DeleteShareRequest> = /*@__PURE__*/
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
 * Describes the enum unitycatalog.shares.v1.Action.
 */
export const ActionSchema: GenEnum<Action> = /*@__PURE__*/
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


// @generated by protoc-gen-es v2.2.3 with parameter "target=ts,json_types=true"
// @generated from file unitycatalog/shares/v1/models.proto (package unitycatalog.shares.v1, syntax proto3)
/* eslint-disable */

import type { GenEnum, GenFile, GenMessage } from "@bufbuild/protobuf/codegenv1";
import { enumDesc, fileDesc, messageDesc } from "@bufbuild/protobuf/codegenv1";
import { file_google_api_field_behavior } from "../../../google/api/field_behavior_pb";
import type { Message } from "@bufbuild/protobuf";

/**
 * Describes the file unitycatalog/shares/v1/models.proto.
 */
export const file_unitycatalog_shares_v1_models: GenFile = /*@__PURE__*/
  fileDesc("CiN1bml0eWNhdGFsb2cvc2hhcmVzL3YxL21vZGVscy5wcm90bxIWdW5pdHljYXRhbG9nLnNoYXJlcy52MSLHAwoKRGF0YU9iamVjdBIMCgRuYW1lGAEgASgJEkAKEGRhdGFfb2JqZWN0X3R5cGUYAiABKA4yJi51bml0eWNhdGFsb2cuc2hhcmVzLnYxLkRhdGFPYmplY3RUeXBlEhUKCGFkZGVkX2F0GAMgASgDSACIAQESFQoIYWRkZWRfYnkYBCABKAlIAYgBARIUCgdjb21tZW50GAUgASgJSAKIAQESFgoJc2hhcmVkX2FzGAYgASgJSAOIAQESEgoKcGFydGl0aW9ucxgHIAMoCRIXCgplbmFibGVfY2RmGAggASgISASIAQESTwobaGlzdG9yeV9kYXRhX3NoYXJpbmdfc3RhdHVzGAkgASgOMiUudW5pdHljYXRhbG9nLnNoYXJlcy52MS5IaXN0b3J5U3RhdHVzSAWIAQESGgoNc3RhcnRfdmVyc2lvbhgKIAEoA0gGiAEBQgsKCV9hZGRlZF9hdEILCglfYWRkZWRfYnlCCgoIX2NvbW1lbnRCDAoKX3NoYXJlZF9hc0INCgtfZW5hYmxlX2NkZkIeChxfaGlzdG9yeV9kYXRhX3NoYXJpbmdfc3RhdHVzQhAKDl9zdGFydF92ZXJzaW9uIssCCglTaGFyZUluZm8SDwoCaWQYZCABKAlIAIgBARIMCgRuYW1lGAEgASgJEhIKBW93bmVyGAIgASgJSAGIAQESFAoHY29tbWVudBgDIAEoCUgCiAEBEjgKDGRhdGFfb2JqZWN0cxgFIAMoCzIiLnVuaXR5Y2F0YWxvZy5zaGFyZXMudjEuRGF0YU9iamVjdBIXCgpjcmVhdGVkX2F0GAYgASgDSAOIAQESFwoKY3JlYXRlZF9ieRgHIAEoCUgEiAEBEhcKCnVwZGF0ZWRfYXQYCCABKANIBYgBARIXCgp1cGRhdGVkX2J5GAkgASgJSAaIAQFCBQoDX2lkQggKBl9vd25lckIKCghfY29tbWVudEINCgtfY3JlYXRlZF9hdEINCgtfY3JlYXRlZF9ieUINCgtfdXBkYXRlZF9hdEINCgtfdXBkYXRlZF9ieSpJCg5EYXRhT2JqZWN0VHlwZRIgChxEQVRBX09CSkVDVF9UWVBFX1VOU1BFQ0lGSUVEEAASCQoFVEFCTEUQARIKCgZTQ0hFTUEQAioqCg1IaXN0b3J5U3RhdHVzEgwKCERJU0FCTEVEEAASCwoHRU5BQkxFRBABQvMBChpjb20udW5pdHljYXRhbG9nLnNoYXJlcy52MUILTW9kZWxzUHJvdG9QAVpOZ2l0aHViLmNvbS9kZWx0YS1pbmN1YmF0b3IvZGVsdGEtc2hhcmluZy1ycy9nby91bml0eWNhdGFsb2cvc2hhcmVzL3YxO3NoYXJlc3YxogIDVVNYqgIWVW5pdHljYXRhbG9nLlNoYXJlcy5WMcoCFlVuaXR5Y2F0YWxvZ1xTaGFyZXNcVjHiAiJVbml0eWNhdGFsb2dcU2hhcmVzXFYxXEdQQk1ldGFkYXRh6gIYVW5pdHljYXRhbG9nOjpTaGFyZXM6OlYxYgZwcm90bzM", [file_google_api_field_behavior]);

/**
 * @generated from message unitycatalog.shares.v1.DataObject
 */
export type DataObject = Message<"unitycatalog.shares.v1.DataObject"> & {
  /**
   * A fully qualified name that uniquely identifies a data object.
   *
   * For example, a table's fully qualified name is in the format of <catalog>.<schema>.<table>,
   *
   * @generated from field: string name = 1;
   */
  name: string;

  /**
   * Type of the data object.
   *
   * @generated from field: unitycatalog.shares.v1.DataObjectType data_object_type = 2;
   */
  dataObjectType: DataObjectType;

  /**
   * The time when this data object is added to the share, in epoch milliseconds.
   *
   * @generated from field: optional int64 added_at = 3;
   */
  addedAt?: bigint;

  /**
   * Username of the sharer.
   *
   * @generated from field: optional string added_by = 4;
   */
  addedBy?: string;

  /**
   * A user-provided comment when adding the data object to the share.
   *
   * @generated from field: optional string comment = 5;
   */
  comment?: string;

  /**
   * A user-provided new name for the data object within the share.
   *
   * If this new name is not provided, the object's original name will be used as the shared_as name.
   * The shared_as name must be unique within a share.
   * For tables, the new name must follow the format of <schema>.<table>.
   *
   * @generated from field: optional string shared_as = 6;
   */
  sharedAs?: string;

  /**
   * Array of partitions for the shared data.
   *
   * @generated from field: repeated string partitions = 7;
   */
  partitions: string[];

  /**
   * Whether to enable cdf or indicate if cdf is enabled on the shared object.
   *
   * @generated from field: optional bool enable_cdf = 8;
   */
  enableCdf?: boolean;

  /**
   * Whether to enable or disable sharing of data history. If not specified, the default is DISABLED.
   *
   * @generated from field: optional unitycatalog.shares.v1.HistoryStatus history_data_sharing_status = 9;
   */
  historyDataSharingStatus?: HistoryStatus;

  /**
   * The start version associated with the object.
   *
   * This allows data providers to control the lowest object version that is accessible by clients.
   * If specified, clients can query snapshots or changes for versions >= start_version.
   * If not specified, clients can only query starting from the version of the object at the time it was added to the share.
   *
   * NOTE: The start_version should be <= the current version of the object.
   *
   * @generated from field: optional int64 start_version = 10;
   */
  startVersion?: bigint;
};

/**
 * @generated from message unitycatalog.shares.v1.DataObject
 */
export type DataObjectJson = {
  /**
   * A fully qualified name that uniquely identifies a data object.
   *
   * For example, a table's fully qualified name is in the format of <catalog>.<schema>.<table>,
   *
   * @generated from field: string name = 1;
   */
  name?: string;

  /**
   * Type of the data object.
   *
   * @generated from field: unitycatalog.shares.v1.DataObjectType data_object_type = 2;
   */
  dataObjectType?: DataObjectTypeJson;

  /**
   * The time when this data object is added to the share, in epoch milliseconds.
   *
   * @generated from field: optional int64 added_at = 3;
   */
  addedAt?: string;

  /**
   * Username of the sharer.
   *
   * @generated from field: optional string added_by = 4;
   */
  addedBy?: string;

  /**
   * A user-provided comment when adding the data object to the share.
   *
   * @generated from field: optional string comment = 5;
   */
  comment?: string;

  /**
   * A user-provided new name for the data object within the share.
   *
   * If this new name is not provided, the object's original name will be used as the shared_as name.
   * The shared_as name must be unique within a share.
   * For tables, the new name must follow the format of <schema>.<table>.
   *
   * @generated from field: optional string shared_as = 6;
   */
  sharedAs?: string;

  /**
   * Array of partitions for the shared data.
   *
   * @generated from field: repeated string partitions = 7;
   */
  partitions?: string[];

  /**
   * Whether to enable cdf or indicate if cdf is enabled on the shared object.
   *
   * @generated from field: optional bool enable_cdf = 8;
   */
  enableCdf?: boolean;

  /**
   * Whether to enable or disable sharing of data history. If not specified, the default is DISABLED.
   *
   * @generated from field: optional unitycatalog.shares.v1.HistoryStatus history_data_sharing_status = 9;
   */
  historyDataSharingStatus?: HistoryStatusJson;

  /**
   * The start version associated with the object.
   *
   * This allows data providers to control the lowest object version that is accessible by clients.
   * If specified, clients can query snapshots or changes for versions >= start_version.
   * If not specified, clients can only query starting from the version of the object at the time it was added to the share.
   *
   * NOTE: The start_version should be <= the current version of the object.
   *
   * @generated from field: optional int64 start_version = 10;
   */
  startVersion?: string;
};

/**
 * Describes the message unitycatalog.shares.v1.DataObject.
 * Use `create(DataObjectSchema)` to create a new message.
 */
export const DataObjectSchema: GenMessage<DataObject, DataObjectJson> = /*@__PURE__*/
  messageDesc(file_unitycatalog_shares_v1_models, 0);

/**
 * @generated from message unitycatalog.shares.v1.ShareInfo
 */
export type ShareInfo = Message<"unitycatalog.shares.v1.ShareInfo"> & {
  /**
   * Unique ID of the recipient.
   *
   * @generated from field: optional string id = 100;
   */
  id?: string;

  /**
   * Name of the share.
   *
   * @generated from field: string name = 1;
   */
  name: string;

  /**
   * Username of current owner of share.
   *
   * @generated from field: optional string owner = 2;
   */
  owner?: string;

  /**
   * User-provided free-form text description.
   *
   * @generated from field: optional string comment = 3;
   */
  comment?: string;

  /**
   * A list of shared data objects within the share.
   *
   * @generated from field: repeated unitycatalog.shares.v1.DataObject data_objects = 5;
   */
  dataObjects: DataObject[];

  /**
   * Time at which this share was created, in epoch milliseconds.
   *
   * @generated from field: optional int64 created_at = 6;
   */
  createdAt?: bigint;

  /**
   * Username of the creator of the share.
   *
   * @generated from field: optional string created_by = 7;
   */
  createdBy?: string;

  /**
   * Time at which this share was updated, in epoch milliseconds.
   *
   * @generated from field: optional int64 updated_at = 8;
   */
  updatedAt?: bigint;

  /**
   * Username of share updater.
   *
   * @generated from field: optional string updated_by = 9;
   */
  updatedBy?: string;
};

/**
 * @generated from message unitycatalog.shares.v1.ShareInfo
 */
export type ShareInfoJson = {
  /**
   * Unique ID of the recipient.
   *
   * @generated from field: optional string id = 100;
   */
  id?: string;

  /**
   * Name of the share.
   *
   * @generated from field: string name = 1;
   */
  name?: string;

  /**
   * Username of current owner of share.
   *
   * @generated from field: optional string owner = 2;
   */
  owner?: string;

  /**
   * User-provided free-form text description.
   *
   * @generated from field: optional string comment = 3;
   */
  comment?: string;

  /**
   * A list of shared data objects within the share.
   *
   * @generated from field: repeated unitycatalog.shares.v1.DataObject data_objects = 5;
   */
  dataObjects?: DataObjectJson[];

  /**
   * Time at which this share was created, in epoch milliseconds.
   *
   * @generated from field: optional int64 created_at = 6;
   */
  createdAt?: string;

  /**
   * Username of the creator of the share.
   *
   * @generated from field: optional string created_by = 7;
   */
  createdBy?: string;

  /**
   * Time at which this share was updated, in epoch milliseconds.
   *
   * @generated from field: optional int64 updated_at = 8;
   */
  updatedAt?: string;

  /**
   * Username of share updater.
   *
   * @generated from field: optional string updated_by = 9;
   */
  updatedBy?: string;
};

/**
 * Describes the message unitycatalog.shares.v1.ShareInfo.
 * Use `create(ShareInfoSchema)` to create a new message.
 */
export const ShareInfoSchema: GenMessage<ShareInfo, ShareInfoJson> = /*@__PURE__*/
  messageDesc(file_unitycatalog_shares_v1_models, 1);

/**
 * @generated from enum unitycatalog.shares.v1.DataObjectType
 */
export enum DataObjectType {
  /**
   * Unknown data object type.
   *
   * @generated from enum value: DATA_OBJECT_TYPE_UNSPECIFIED = 0;
   */
  DATA_OBJECT_TYPE_UNSPECIFIED = 0,

  /**
   * @generated from enum value: TABLE = 1;
   */
  TABLE = 1,

  /**
   * @generated from enum value: SCHEMA = 2;
   */
  SCHEMA = 2,
}

/**
 * @generated from enum unitycatalog.shares.v1.DataObjectType
 */
export type DataObjectTypeJson = "DATA_OBJECT_TYPE_UNSPECIFIED" | "TABLE" | "SCHEMA";

/**
 * Describes the enum unitycatalog.shares.v1.DataObjectType.
 */
export const DataObjectTypeSchema: GenEnum<DataObjectType, DataObjectTypeJson> = /*@__PURE__*/
  enumDesc(file_unitycatalog_shares_v1_models, 0);

/**
 * @generated from enum unitycatalog.shares.v1.HistoryStatus
 */
export enum HistoryStatus {
  /**
   * Data history sharing is disabled.
   *
   * @generated from enum value: DISABLED = 0;
   */
  DISABLED = 0,

  /**
   * Data history sharing is enabled.
   *
   * @generated from enum value: ENABLED = 1;
   */
  ENABLED = 1,
}

/**
 * @generated from enum unitycatalog.shares.v1.HistoryStatus
 */
export type HistoryStatusJson = "DISABLED" | "ENABLED";

/**
 * Describes the enum unitycatalog.shares.v1.HistoryStatus.
 */
export const HistoryStatusSchema: GenEnum<HistoryStatus, HistoryStatusJson> = /*@__PURE__*/
  enumDesc(file_unitycatalog_shares_v1_models, 1);


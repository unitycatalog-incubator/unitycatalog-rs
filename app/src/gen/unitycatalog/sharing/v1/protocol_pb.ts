// @generated by protoc-gen-es v2.2.3 with parameter "target=ts,json_types=true"
// @generated from file unitycatalog/sharing/v1/protocol.proto (package unitycatalog.sharing.v1, syntax proto3)
/* eslint-disable */

import type { GenFile, GenMessage } from "@bufbuild/protobuf/codegenv1";
import { fileDesc, messageDesc } from "@bufbuild/protobuf/codegenv1";
import { file_buf_validate_validate } from "../../../buf/validate/validate_pb";
import type { Message } from "@bufbuild/protobuf";

/**
 * Describes the file unitycatalog/sharing/v1/protocol.proto.
 */
export const file_unitycatalog_sharing_v1_protocol: GenFile = /*@__PURE__*/
  fileDesc("CiZ1bml0eWNhdGFsb2cvc2hhcmluZy92MS9wcm90b2NvbC5wcm90bxIXdW5pdHljYXRhbG9nLnNoYXJpbmcudjEiiQEKBkZvcm1hdBIQCghwcm92aWRlchgBIAEoCRI9CgdvcHRpb25zGAIgAygLMiwudW5pdHljYXRhbG9nLnNoYXJpbmcudjEuRm9ybWF0Lk9wdGlvbnNFbnRyeRouCgxPcHRpb25zRW50cnkSCwoDa2V5GAEgASgJEg0KBXZhbHVlGAIgASgJOgI4ASLcAgoITWV0YWRhdGESCgoCaWQYASABKAkSEQoEbmFtZRgCIAEoCUgAiAEBEhgKC2Rlc2NyaXB0aW9uGAMgASgJSAGIAQESLwoGZm9ybWF0GAQgASgLMh8udW5pdHljYXRhbG9nLnNoYXJpbmcudjEuRm9ybWF0EhUKDXNjaGVtYV9zdHJpbmcYBSABKAkSGQoRcGFydGl0aW9uX2NvbHVtbnMYBiADKAkSGQoMY3JlYXRlZF90aW1lGAcgASgDSAKIAQESPwoHb3B0aW9ucxgIIAMoCzIuLnVuaXR5Y2F0YWxvZy5zaGFyaW5nLnYxLk1ldGFkYXRhLk9wdGlvbnNFbnRyeRouCgxPcHRpb25zRW50cnkSCwoDa2V5GAEgASgJEg0KBXZhbHVlGAIgASgJOgI4AUIHCgVfbmFtZUIOCgxfZGVzY3JpcHRpb25CDwoNX2NyZWF0ZWRfdGltZUL8AQobY29tLnVuaXR5Y2F0YWxvZy5zaGFyaW5nLnYxQg1Qcm90b2NvbFByb3RvUAFaUGdpdGh1Yi5jb20vZGVsdGEtaW5jdWJhdG9yL2RlbHRhLXNoYXJpbmctcnMvZ28vdW5pdHljYXRhbG9nL3NoYXJpbmcvdjE7c2hhcmluZ3YxogIDVVNYqgIXVW5pdHljYXRhbG9nLlNoYXJpbmcuVjHKAhdVbml0eWNhdGFsb2dcU2hhcmluZ1xWMeICI1VuaXR5Y2F0YWxvZ1xTaGFyaW5nXFYxXEdQQk1ldGFkYXRh6gIZVW5pdHljYXRhbG9nOjpTaGFyaW5nOjpWMWIGcHJvdG8z", [file_buf_validate_validate]);

/**
 * File format for data files in a table
 *
 * @generated from message unitycatalog.sharing.v1.Format
 */
export type Format = Message<"unitycatalog.sharing.v1.Format"> & {
  /**
   * Name of the encoding for files in this table
   *
   * @generated from field: string provider = 1;
   */
  provider: string;

  /**
   * A map containing configuration options for the format
   *
   * @generated from field: map<string, string> options = 2;
   */
  options: { [key: string]: string };
};

/**
 * File format for data files in a table
 *
 * @generated from message unitycatalog.sharing.v1.Format
 */
export type FormatJson = {
  /**
   * Name of the encoding for files in this table
   *
   * @generated from field: string provider = 1;
   */
  provider?: string;

  /**
   * A map containing configuration options for the format
   *
   * @generated from field: map<string, string> options = 2;
   */
  options?: { [key: string]: string };
};

/**
 * Describes the message unitycatalog.sharing.v1.Format.
 * Use `create(FormatSchema)` to create a new message.
 */
export const FormatSchema: GenMessage<Format, FormatJson> = /*@__PURE__*/
  messageDesc(file_unitycatalog_sharing_v1_protocol, 0);

/**
 * Metadata for a table
 *
 * @generated from message unitycatalog.sharing.v1.Metadata
 */
export type Metadata = Message<"unitycatalog.sharing.v1.Metadata"> & {
  /**
   * Unique identifier for this table
   * Validate GUID
   *
   * @generated from field: string id = 1;
   */
  id: string;

  /**
   * User-provided identifier for this table
   *
   * @generated from field: optional string name = 2;
   */
  name?: string;

  /**
   * User-provided description for this table
   *
   * @generated from field: optional string description = 3;
   */
  description?: string;

  /**
   * Specification of the encoding for the files stored in the table
   *
   * @generated from field: unitycatalog.sharing.v1.Format format = 4;
   */
  format?: Format;

  /**
   * Schema of the table
   *
   * @generated from field: string schema_string = 5;
   */
  schemaString: string;

  /**
   * An array containing the names of columns by which the data should be partitioned
   *
   * @generated from field: repeated string partition_columns = 6;
   */
  partitionColumns: string[];

  /**
   * The time when this metadata action is created, in milliseconds since the Unix epoch
   *
   * @generated from field: optional int64 created_time = 7;
   */
  createdTime?: bigint;

  /**
   * A map containing configuration options for the metadata action
   *
   * @generated from field: map<string, string> options = 8;
   */
  options: { [key: string]: string };
};

/**
 * Metadata for a table
 *
 * @generated from message unitycatalog.sharing.v1.Metadata
 */
export type MetadataJson = {
  /**
   * Unique identifier for this table
   * Validate GUID
   *
   * @generated from field: string id = 1;
   */
  id?: string;

  /**
   * User-provided identifier for this table
   *
   * @generated from field: optional string name = 2;
   */
  name?: string;

  /**
   * User-provided description for this table
   *
   * @generated from field: optional string description = 3;
   */
  description?: string;

  /**
   * Specification of the encoding for the files stored in the table
   *
   * @generated from field: unitycatalog.sharing.v1.Format format = 4;
   */
  format?: FormatJson;

  /**
   * Schema of the table
   *
   * @generated from field: string schema_string = 5;
   */
  schemaString?: string;

  /**
   * An array containing the names of columns by which the data should be partitioned
   *
   * @generated from field: repeated string partition_columns = 6;
   */
  partitionColumns?: string[];

  /**
   * The time when this metadata action is created, in milliseconds since the Unix epoch
   *
   * @generated from field: optional int64 created_time = 7;
   */
  createdTime?: string;

  /**
   * A map containing configuration options for the metadata action
   *
   * @generated from field: map<string, string> options = 8;
   */
  options?: { [key: string]: string };
};

/**
 * Describes the message unitycatalog.sharing.v1.Metadata.
 * Use `create(MetadataSchema)` to create a new message.
 */
export const MetadataSchema: GenMessage<Metadata, MetadataJson> = /*@__PURE__*/
  messageDesc(file_unitycatalog_sharing_v1_protocol, 1);


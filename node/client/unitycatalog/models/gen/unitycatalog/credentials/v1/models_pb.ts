// @generated by protoc-gen-es v2.5.2 with parameter "target=ts,json_types=false"
// @generated from file unitycatalog/credentials/v1/models.proto (package unitycatalog.credentials.v1, syntax proto3)
/* eslint-disable */

import type { GenEnum, GenFile, GenMessage } from "@bufbuild/protobuf/codegenv2";
import { enumDesc, fileDesc, messageDesc } from "@bufbuild/protobuf/codegenv2";
import { file_buf_validate_validate } from "../../../buf/validate/validate_pb";
import { file_google_api_annotations } from "../../../google/api/annotations_pb";
import { file_google_api_client } from "../../../google/api/client_pb";
import { file_google_api_field_behavior } from "../../../google/api/field_behavior_pb";
import { file_google_api_resource } from "../../../google/api/resource_pb";
import { file_google_protobuf_struct } from "@bufbuild/protobuf/wkt";
import type { Message } from "@bufbuild/protobuf";

/**
 * Describes the file unitycatalog/credentials/v1/models.proto.
 */
export const file_unitycatalog_credentials_v1_models: GenFile = /*@__PURE__*/
  fileDesc("Cih1bml0eWNhdGFsb2cvY3JlZGVudGlhbHMvdjEvbW9kZWxzLnByb3RvEht1bml0eWNhdGFsb2cuY3JlZGVudGlhbHMudjEinQEKFUF6dXJlU2VydmljZVByaW5jaXBhbBIZCgxkaXJlY3RvcnlfaWQYASABKAlCA+BBAhIbCg5hcHBsaWNhdGlvbl9pZBgCIAEoCUID4EECEhcKDWNsaWVudF9zZWNyZXQYAyABKAlIABIeChRmZWRlcmF0ZWRfdG9rZW5fZmlsZRgFIAEoCUgAQhMKCmNyZWRlbnRpYWwSBbpIAggBIn0KFEF6dXJlTWFuYWdlZElkZW50aXR5EhgKCW9iamVjdF9pZBgBIAEoCUID4EEBSAASHQoOYXBwbGljYXRpb25faWQYAiABKAlCA+BBAUgAEh4KD21zaV9yZXNvdXJjZV9pZBgDIAEoCUID4EEBSABCDAoKaWRlbnRpZmllciJGCg9BenVyZVN0b3JhZ2VLZXkSGQoMYWNjb3VudF9uYW1lGAEgASgJQgPgQQISGAoLYWNjb3VudF9rZXkYAiABKAlCA+BBAiKhBQoOQ3JlZGVudGlhbEluZm8SCgoCaWQYASABKAkSDAoEbmFtZRgCIAEoCRI1CgdwdXJwb3NlGAMgASgOMiQudW5pdHljYXRhbG9nLmNyZWRlbnRpYWxzLnYxLlB1cnBvc2USEQoJcmVhZF9vbmx5GAQgASgIEhQKB2NvbW1lbnQYBSABKAlIAYgBARISCgVvd25lchgGIAEoCUgCiAEBEhcKCmNyZWF0ZWRfYXQYByABKANIA4gBARIXCgpjcmVhdGVkX2J5GAggASgJSASIAQESFwoKdXBkYXRlZF9hdBgJIAEoA0gFiAEBEhcKCnVwZGF0ZWRfYnkYCiABKAlIBogBARIgChh1c2VkX2Zvcl9tYW5hZ2VkX3N0b3JhZ2UYCyABKAgSFgoJZnVsbF9uYW1lGAwgASgJSAeIAQESVQoXYXp1cmVfc2VydmljZV9wcmluY2lwYWwYZCABKAsyMi51bml0eWNhdGFsb2cuY3JlZGVudGlhbHMudjEuQXp1cmVTZXJ2aWNlUHJpbmNpcGFsSAASUwoWYXp1cmVfbWFuYWdlZF9pZGVudGl0eRhlIAEoCzIxLnVuaXR5Y2F0YWxvZy5jcmVkZW50aWFscy52MS5BenVyZU1hbmFnZWRJZGVudGl0eUgAEkkKEWF6dXJlX3N0b3JhZ2Vfa2V5GGYgASgLMiwudW5pdHljYXRhbG9nLmNyZWRlbnRpYWxzLnYxLkF6dXJlU3RvcmFnZUtleUgAQgwKCmNyZWRlbnRpYWxCCgoIX2NvbW1lbnRCCAoGX293bmVyQg0KC19jcmVhdGVkX2F0Qg0KC19jcmVhdGVkX2J5Qg0KC191cGRhdGVkX2F0Qg0KC191cGRhdGVkX2J5QgwKCl9mdWxsX25hbWUqPAoHUHVycG9zZRIXChNQVVJQT1NFX1VOU1BFQ0lGSUVEEAASCwoHU1RPUkFHRRABEgsKB1NFUlZJQ0UQAkKWAgofY29tLnVuaXR5Y2F0YWxvZy5jcmVkZW50aWFscy52MUILTW9kZWxzUHJvdG9QAVpYZ2l0aHViLmNvbS9kZWx0YS1pbmN1YmF0b3IvZGVsdGEtc2hhcmluZy1ycy9nby91bml0eWNhdGFsb2cvY3JlZGVudGlhbHMvdjE7Y3JlZGVudGlhbHN2MaICA1VDWKoCG1VuaXR5Y2F0YWxvZy5DcmVkZW50aWFscy5WMcoCG1VuaXR5Y2F0YWxvZ1xDcmVkZW50aWFsc1xWMeICJ1VuaXR5Y2F0YWxvZ1xDcmVkZW50aWFsc1xWMVxHUEJNZXRhZGF0YeoCHVVuaXR5Y2F0YWxvZzo6Q3JlZGVudGlhbHM6OlYxYgZwcm90bzM", [file_buf_validate_validate, file_google_api_annotations, file_google_api_client, file_google_api_field_behavior, file_google_api_resource, file_google_protobuf_struct]);

/**
 * @generated from message unitycatalog.credentials.v1.AzureServicePrincipal
 */
export type AzureServicePrincipal = Message<"unitycatalog.credentials.v1.AzureServicePrincipal"> & {
  /**
   * The directory ID corresponding to the Azure Active Directory (AAD) tenant of the application.
   *
   * @generated from field: string directory_id = 1;
   */
  directoryId: string;

  /**
   * The application ID of the application registration within the referenced AAD tenant.
   *
   * @generated from field: string application_id = 2;
   */
  applicationId: string;

  /**
   * @generated from oneof unitycatalog.credentials.v1.AzureServicePrincipal.credential
   */
  credential: {
    /**
     * The client secret generated for the above app ID in AAD.
     *
     * @generated from field: string client_secret = 3;
     */
    value: string;
    case: "clientSecret";
  } | {
    /**
     * Location of the file containing a federated token.
     *
     * Specifically useful for workload identity federation.
     *
     * @generated from field: string federated_token_file = 5;
     */
    value: string;
    case: "federatedTokenFile";
  } | { case: undefined; value?: undefined };
};

/**
 * Describes the message unitycatalog.credentials.v1.AzureServicePrincipal.
 * Use `create(AzureServicePrincipalSchema)` to create a new message.
 */
export const AzureServicePrincipalSchema: GenMessage<AzureServicePrincipal> = /*@__PURE__*/
  messageDesc(file_unitycatalog_credentials_v1_models, 0);

/**
 * @generated from message unitycatalog.credentials.v1.AzureManagedIdentity
 */
export type AzureManagedIdentity = Message<"unitycatalog.credentials.v1.AzureManagedIdentity"> & {
  /**
   * @generated from oneof unitycatalog.credentials.v1.AzureManagedIdentity.identifier
   */
  identifier: {
    /**
     * Object id for use with managed identity authentication
     *
     * @generated from field: string object_id = 1;
     */
    value: string;
    case: "objectId";
  } | {
    /**
     * The application ID of the application registration within the referenced AAD tenant.
     *
     * @generated from field: string application_id = 2;
     */
    value: string;
    case: "applicationId";
  } | {
    /**
     * Msi resource id for use with managed identity authentication
     *
     * @generated from field: string msi_resource_id = 3;
     */
    value: string;
    case: "msiResourceId";
  } | { case: undefined; value?: undefined };
};

/**
 * Describes the message unitycatalog.credentials.v1.AzureManagedIdentity.
 * Use `create(AzureManagedIdentitySchema)` to create a new message.
 */
export const AzureManagedIdentitySchema: GenMessage<AzureManagedIdentity> = /*@__PURE__*/
  messageDesc(file_unitycatalog_credentials_v1_models, 1);

/**
 * @generated from message unitycatalog.credentials.v1.AzureStorageKey
 */
export type AzureStorageKey = Message<"unitycatalog.credentials.v1.AzureStorageKey"> & {
  /**
   * The name of the storage account.
   *
   * @generated from field: string account_name = 1;
   */
  accountName: string;

  /**
   * The account key of the storage account.
   *
   * @generated from field: string account_key = 2;
   */
  accountKey: string;
};

/**
 * Describes the message unitycatalog.credentials.v1.AzureStorageKey.
 * Use `create(AzureStorageKeySchema)` to create a new message.
 */
export const AzureStorageKeySchema: GenMessage<AzureStorageKey> = /*@__PURE__*/
  messageDesc(file_unitycatalog_credentials_v1_models, 2);

/**
 * @generated from message unitycatalog.credentials.v1.CredentialInfo
 */
export type CredentialInfo = Message<"unitycatalog.credentials.v1.CredentialInfo"> & {
  /**
   * The unique identifier of the credential.
   *
   * @generated from field: string id = 1;
   */
  id: string;

  /**
   * The credential name.
   *
   * The name must be unique among storage and service credentials within the metastore.
   *
   * @generated from field: string name = 2;
   */
  name: string;

  /**
   * Indicates the purpose of the credential.
   *
   * @generated from field: unitycatalog.credentials.v1.Purpose purpose = 3;
   */
  purpose: Purpose;

  /**
   * Whether the credential is usable only for read operations.
   *
   * Only applicable when purpose is STORAGE.
   *
   * @generated from field: bool read_only = 4;
   */
  readOnly: boolean;

  /**
   * User-provided free-form text description.
   *
   * @generated from field: optional string comment = 5;
   */
  comment?: string;

  /**
   * Username of current owner of credential.
   *
   * @generated from field: optional string owner = 6;
   */
  owner?: string;

  /**
   * Time at which this credential was created, in epoch milliseconds.
   *
   * @generated from field: optional int64 created_at = 7;
   */
  createdAt?: bigint;

  /**
   * Username of credential creator.
   *
   * @generated from field: optional string created_by = 8;
   */
  createdBy?: string;

  /**
   * Time at which this credential was last updated, in epoch milliseconds.
   *
   * @generated from field: optional int64 updated_at = 9;
   */
  updatedAt?: bigint;

  /**
   * Username of user who last modified credential.
   *
   * @generated from field: optional string updated_by = 10;
   */
  updatedBy?: string;

  /**
   * Whether this credential is the current metastore's root storage credential.
   *
   * Only applicable when purpose is STORAGE.
   *
   * @generated from field: bool used_for_managed_storage = 11;
   */
  usedForManagedStorage: boolean;

  /**
   * The full name of the credential.
   *
   * @generated from field: optional string full_name = 12;
   */
  fullName?: string;

  /**
   * @generated from oneof unitycatalog.credentials.v1.CredentialInfo.credential
   */
  credential: {
    /**
     * @generated from field: unitycatalog.credentials.v1.AzureServicePrincipal azure_service_principal = 100;
     */
    value: AzureServicePrincipal;
    case: "azureServicePrincipal";
  } | {
    /**
     * @generated from field: unitycatalog.credentials.v1.AzureManagedIdentity azure_managed_identity = 101;
     */
    value: AzureManagedIdentity;
    case: "azureManagedIdentity";
  } | {
    /**
     * @generated from field: unitycatalog.credentials.v1.AzureStorageKey azure_storage_key = 102;
     */
    value: AzureStorageKey;
    case: "azureStorageKey";
  } | { case: undefined; value?: undefined };
};

/**
 * Describes the message unitycatalog.credentials.v1.CredentialInfo.
 * Use `create(CredentialInfoSchema)` to create a new message.
 */
export const CredentialInfoSchema: GenMessage<CredentialInfo> = /*@__PURE__*/
  messageDesc(file_unitycatalog_credentials_v1_models, 3);

/**
 * @generated from enum unitycatalog.credentials.v1.Purpose
 */
export enum Purpose {
  /**
   * @generated from enum value: PURPOSE_UNSPECIFIED = 0;
   */
  PURPOSE_UNSPECIFIED = 0,

  /**
   * @generated from enum value: STORAGE = 1;
   */
  STORAGE = 1,

  /**
   * @generated from enum value: SERVICE = 2;
   */
  SERVICE = 2,
}

/**
 * Describes the enum unitycatalog.credentials.v1.Purpose.
 */
export const PurposeSchema: GenEnum<Purpose> = /*@__PURE__*/
  enumDesc(file_unitycatalog_credentials_v1_models, 0);


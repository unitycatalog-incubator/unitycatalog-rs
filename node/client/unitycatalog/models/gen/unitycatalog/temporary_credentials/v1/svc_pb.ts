// @generated by protoc-gen-es v2.5.2 with parameter "target=ts,json_types=false"
// @generated from file unitycatalog/temporary_credentials/v1/svc.proto (package unitycatalog.temporary_credentials.v1, syntax proto3)
/* eslint-disable */

import type { GenEnum, GenFile, GenMessage, GenService } from "@bufbuild/protobuf/codegenv2";
import { enumDesc, fileDesc, messageDesc, serviceDesc } from "@bufbuild/protobuf/codegenv2";
import { file_buf_validate_validate } from "../../../buf/validate/validate_pb";
import type { TemporaryCredentialSchema } from "./models_pb";
import { file_unitycatalog_temporary_credentials_v1_models } from "./models_pb";
import { file_gnostic_openapi_v3_annotations } from "../../../gnostic/openapi/v3/annotations_pb";
import { file_google_api_annotations } from "../../../google/api/annotations_pb";
import { file_google_api_field_behavior } from "../../../google/api/field_behavior_pb";
import type { Message } from "@bufbuild/protobuf";

/**
 * Describes the file unitycatalog/temporary_credentials/v1/svc.proto.
 */
export const file_unitycatalog_temporary_credentials_v1_svc: GenFile = /*@__PURE__*/
  fileDesc("Ci91bml0eWNhdGFsb2cvdGVtcG9yYXJ5X2NyZWRlbnRpYWxzL3YxL3N2Yy5wcm90bxIldW5pdHljYXRhbG9nLnRlbXBvcmFyeV9jcmVkZW50aWFscy52MSKLAQooR2VuZXJhdGVUZW1wb3JhcnlUYWJsZUNyZWRlbnRpYWxzUmVxdWVzdBIVCgh0YWJsZV9pZBgBIAEoCUID4EECEkgKCW9wZXJhdGlvbhgCIAEoDjIwLnVuaXR5Y2F0YWxvZy50ZW1wb3JhcnlfY3JlZGVudGlhbHMudjEuT3BlcmF0aW9uQgPgQQIijQEKKUdlbmVyYXRlVGVtcG9yYXJ5Vm9sdW1lQ3JlZGVudGlhbHNSZXF1ZXN0EhYKCXZvbHVtZV9pZBgBIAEoCUID4EECEkgKCW9wZXJhdGlvbhgCIAEoDjIwLnVuaXR5Y2F0YWxvZy50ZW1wb3JhcnlfY3JlZGVudGlhbHMudjEuT3BlcmF0aW9uQgPgQQIqQAoJT3BlcmF0aW9uEhkKFU9QRVJBVElPTl9VTlNQRUNJRklFRBAAEggKBFJFQUQQARIOCgpSRUFEX1dSSVRFEAIypQQKG1RlbXBvcmFyeUNyZWRlbnRpYWxzU2VydmljZRL/AQohR2VuZXJhdGVUZW1wb3JhcnlUYWJsZUNyZWRlbnRpYWxzEk8udW5pdHljYXRhbG9nLnRlbXBvcmFyeV9jcmVkZW50aWFscy52MS5HZW5lcmF0ZVRlbXBvcmFyeVRhYmxlQ3JlZGVudGlhbHNSZXF1ZXN0GjoudW5pdHljYXRhbG9nLnRlbXBvcmFyeV9jcmVkZW50aWFscy52MS5UZW1wb3JhcnlDcmVkZW50aWFsIk26RyMqIUdlbmVyYXRlVGVtcG9yYXJ5VGFibGVDcmVkZW50aWFsc4LT5JMCIToBKiIcL3RlbXBvcmFyeS10YWJsZS1jcmVkZW50aWFscxKDAgoiR2VuZXJhdGVUZW1wb3JhcnlWb2x1bWVDcmVkZW50aWFscxJQLnVuaXR5Y2F0YWxvZy50ZW1wb3JhcnlfY3JlZGVudGlhbHMudjEuR2VuZXJhdGVUZW1wb3JhcnlWb2x1bWVDcmVkZW50aWFsc1JlcXVlc3QaOi51bml0eWNhdGFsb2cudGVtcG9yYXJ5X2NyZWRlbnRpYWxzLnYxLlRlbXBvcmFyeUNyZWRlbnRpYWwiT7pHJCoiR2VuZXJhdGVUZW1wb3JhcnlWb2x1bWVDcmVkZW50aWFsc4LT5JMCIjoBKiIdL3RlbXBvcmFyeS12b2x1bWUtY3JlZGVudGlhbHNC1QIKKWNvbS51bml0eWNhdGFsb2cudGVtcG9yYXJ5X2NyZWRlbnRpYWxzLnYxQghTdmNQcm90b1ABWmxnaXRodWIuY29tL2RlbHRhLWluY3ViYXRvci9kZWx0YS1zaGFyaW5nLXJzL2dvL3VuaXR5Y2F0YWxvZy90ZW1wb3JhcnlfY3JlZGVudGlhbHMvdjE7dGVtcG9yYXJ5X2NyZWRlbnRpYWxzdjGiAgNVVFiqAiRVbml0eWNhdGFsb2cuVGVtcG9yYXJ5Q3JlZGVudGlhbHMuVjHKAiRVbml0eWNhdGFsb2dcVGVtcG9yYXJ5Q3JlZGVudGlhbHNcVjHiAjBVbml0eWNhdGFsb2dcVGVtcG9yYXJ5Q3JlZGVudGlhbHNcVjFcR1BCTWV0YWRhdGHqAiZVbml0eWNhdGFsb2c6OlRlbXBvcmFyeUNyZWRlbnRpYWxzOjpWMWIGcHJvdG8z", [file_buf_validate_validate, file_unitycatalog_temporary_credentials_v1_models, file_gnostic_openapi_v3_annotations, file_google_api_annotations, file_google_api_field_behavior]);

/**
 * Gebnerate a new set of credentials for a table.
 *
 * @generated from message unitycatalog.temporary_credentials.v1.GenerateTemporaryTableCredentialsRequest
 */
export type GenerateTemporaryTableCredentialsRequest = Message<"unitycatalog.temporary_credentials.v1.GenerateTemporaryTableCredentialsRequest"> & {
  /**
   * The name of the table for which to generate credentials.
   *
   * @generated from field: string table_id = 1;
   */
  tableId: string;

  /**
   * The operation to perform with the credentials.
   *
   * @generated from field: unitycatalog.temporary_credentials.v1.Operation operation = 2;
   */
  operation: Operation;
};

/**
 * Describes the message unitycatalog.temporary_credentials.v1.GenerateTemporaryTableCredentialsRequest.
 * Use `create(GenerateTemporaryTableCredentialsRequestSchema)` to create a new message.
 */
export const GenerateTemporaryTableCredentialsRequestSchema: GenMessage<GenerateTemporaryTableCredentialsRequest> = /*@__PURE__*/
  messageDesc(file_unitycatalog_temporary_credentials_v1_svc, 0);

/**
 * Genearte a new set of credentials for a volume.
 *
 * @generated from message unitycatalog.temporary_credentials.v1.GenerateTemporaryVolumeCredentialsRequest
 */
export type GenerateTemporaryVolumeCredentialsRequest = Message<"unitycatalog.temporary_credentials.v1.GenerateTemporaryVolumeCredentialsRequest"> & {
  /**
   * The name of the volume for which to generate credentials.
   *
   * @generated from field: string volume_id = 1;
   */
  volumeId: string;

  /**
   * The operation to perform with the credentials.
   *
   * @generated from field: unitycatalog.temporary_credentials.v1.Operation operation = 2;
   */
  operation: Operation;
};

/**
 * Describes the message unitycatalog.temporary_credentials.v1.GenerateTemporaryVolumeCredentialsRequest.
 * Use `create(GenerateTemporaryVolumeCredentialsRequestSchema)` to create a new message.
 */
export const GenerateTemporaryVolumeCredentialsRequestSchema: GenMessage<GenerateTemporaryVolumeCredentialsRequest> = /*@__PURE__*/
  messageDesc(file_unitycatalog_temporary_credentials_v1_svc, 1);

/**
 * The operation performed against the table data, either READ or READ_WRITE.
 * If READ_WRITE is specified, the credentials returned will have write permissions,
 * otherwise, it will be read only.
 *
 * @generated from enum unitycatalog.temporary_credentials.v1.Operation
 */
export enum Operation {
  /**
   * The operation is not specified.
   *
   * @generated from enum value: OPERATION_UNSPECIFIED = 0;
   */
  OPERATION_UNSPECIFIED = 0,

  /**
   * The operation is read only.
   *
   * @generated from enum value: READ = 1;
   */
  READ = 1,

  /**
   * The operation is read and write.
   *
   * @generated from enum value: READ_WRITE = 2;
   */
  READ_WRITE = 2,
}

/**
 * Describes the enum unitycatalog.temporary_credentials.v1.Operation.
 */
export const OperationSchema: GenEnum<Operation> = /*@__PURE__*/
  enumDesc(file_unitycatalog_temporary_credentials_v1_svc, 0);

/**
 * @generated from service unitycatalog.temporary_credentials.v1.TemporaryCredentialsService
 */
export const TemporaryCredentialsService: GenService<{
  /**
   * Generate a new set of credentials for a table.
   *
   * @generated from rpc unitycatalog.temporary_credentials.v1.TemporaryCredentialsService.GenerateTemporaryTableCredentials
   */
  generateTemporaryTableCredentials: {
    methodKind: "unary";
    input: typeof GenerateTemporaryTableCredentialsRequestSchema;
    output: typeof TemporaryCredentialSchema;
  },
  /**
   * Generate a new set of credentials for a volume.
   *
   * @generated from rpc unitycatalog.temporary_credentials.v1.TemporaryCredentialsService.GenerateTemporaryVolumeCredentials
   */
  generateTemporaryVolumeCredentials: {
    methodKind: "unary";
    input: typeof GenerateTemporaryVolumeCredentialsRequestSchema;
    output: typeof TemporaryCredentialSchema;
  },
}> = /*@__PURE__*/
  serviceDesc(file_unitycatalog_temporary_credentials_v1_svc, 0);


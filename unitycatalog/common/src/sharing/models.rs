use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ProtocolResponseData {
    /// Protocol in [Delta format]
    ///
    /// [Delta format]: https://github.com/delta-io/delta-sharing/blob/main/PROTOCOL.md#protocol-in-delta-format
    DeltaProtocol(delta_kernel::actions::Protocol),

    /// Protocol in [Parquet format]
    ///
    /// [Parquet format]: https://github.com/delta-io/delta-sharing/blob/main/PROTOCOL.md#protocol
    #[serde(untagged)]
    ParquetProtocol {
        #[serde(rename = "camelCase")]
        min_reader_version: i32,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MetadataResponse {
    Protocol(delta_kernel::actions::Protocol),
    Metadata(delta_kernel::actions::Metadata),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeltaFileResponse {
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

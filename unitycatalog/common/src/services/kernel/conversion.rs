use delta_kernel::actions::{Metadata, Protocol};

use crate::models::sharing::v1::{
    MetadataParquet, ParquetLogMessage, ParquetResponse, ProtocolParquet,
    QueryResponse, parquet_log_message::Entry as ParquetEntry,
    query_response::Response as QueryResponseType,
};

impl From<&Metadata> for MetadataParquet {
    fn from(value: &Metadata) -> Self {
        // MetadataParquet {
        //     id: value.id.clone(),
        //     name: value.name.clone(),
        //     description: value.description.clone(),
        //     format: Some(FormatMessage {
        //         provider: value.format.provider.clone(),
        //         options: value.format.options.clone(),
        //     }),
        //     schema_string: value.schema_string.clone(),
        //     partition_columns: value.partition_columns.clone(),
        // }
        todo!()
    }
}

impl From<&Metadata> for ParquetLogMessage {
    fn from(value: &Metadata) -> Self {
        ParquetLogMessage {
            entry: Some(ParquetEntry::Metadata(value.into())),
        }
    }
}

impl From<&Protocol> for ProtocolParquet {
    fn from(value: &Protocol) -> Self {
        ProtocolParquet {
            min_reader_version: value.min_reader_version(),
        }
    }
}

impl From<&Protocol> for ParquetLogMessage {
    fn from(value: &Protocol) -> Self {
        ParquetLogMessage {
            entry: Some(ParquetEntry::Protocol(value.into())),
        }
    }
}

impl From<ProtocolParquet> for ParquetLogMessage {
    fn from(value: ProtocolParquet) -> Self {
        ParquetLogMessage {
            entry: Some(ParquetEntry::Protocol(value)),
        }
    }
}

impl From<MetadataParquet> for ParquetLogMessage {
    fn from(value: MetadataParquet) -> Self {
        ParquetLogMessage {
            entry: Some(ParquetEntry::Metadata(value)),
        }
    }
}

impl<T: IntoIterator<Item = ParquetLogMessage>> From<T> for QueryResponse {
    fn from(value: T) -> Self {
        QueryResponse {
            response: Some(QueryResponseType::Parquet(ParquetResponse {
                entries: value.into_iter().collect(),
            })),
        }
    }
}

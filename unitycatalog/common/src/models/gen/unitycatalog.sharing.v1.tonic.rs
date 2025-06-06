// @generated
/// Generated server implementations.
pub mod delta_sharing_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with DeltaSharingServiceServer.
    #[async_trait]
    pub trait DeltaSharingService: Send + Sync + 'static {
        /** List shares accessible to a recipient.
*/
        async fn list_shares(
            &self,
            request: tonic::Request<super::ListSharesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListSharesResponse>,
            tonic::Status,
        >;
        /** Get the metadata for a specific share.
*/
        async fn get_share(
            &self,
            request: tonic::Request<super::GetShareRequest>,
        ) -> std::result::Result<tonic::Response<super::Share>, tonic::Status>;
        /** List the schemas in a share.
*/
        async fn list_sharing_schemas(
            &self,
            request: tonic::Request<super::ListSharingSchemasRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListSharingSchemasResponse>,
            tonic::Status,
        >;
        /** List the tables in a given share's schema.
*/
        async fn list_schema_tables(
            &self,
            request: tonic::Request<super::ListSchemaTablesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListSchemaTablesResponse>,
            tonic::Status,
        >;
        /** List all the tables under all schemas in a share.
*/
        async fn list_share_tables(
            &self,
            request: tonic::Request<super::ListShareTablesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListShareTablesResponse>,
            tonic::Status,
        >;
        /** Get the current version for a table within a schema.
*/
        async fn get_table_version(
            &self,
            request: tonic::Request<super::GetTableVersionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetTableVersionResponse>,
            tonic::Status,
        >;
        ///
        async fn get_table_metadata(
            &self,
            request: tonic::Request<super::GetTableMetadataRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryResponse>, tonic::Status>;
        ///
        async fn query_table(
            &self,
            request: tonic::Request<super::QueryTableRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryResponse>, tonic::Status>;
    }
    /** Service exposing the official APIs for Delta Sharing.
*/
    #[derive(Debug)]
    pub struct DeltaSharingServiceServer<T: DeltaSharingService> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T: DeltaSharingService> DeltaSharingServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for DeltaSharingServiceServer<T>
    where
        T: DeltaSharingService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            match req.uri().path() {
                "/unitycatalog.sharing.v1.DeltaSharingService/ListShares" => {
                    #[allow(non_camel_case_types)]
                    struct ListSharesSvc<T: DeltaSharingService>(pub Arc<T>);
                    impl<
                        T: DeltaSharingService,
                    > tonic::server::UnaryService<super::ListSharesRequest>
                    for ListSharesSvc<T> {
                        type Response = super::ListSharesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListSharesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DeltaSharingService>::list_shares(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ListSharesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/unitycatalog.sharing.v1.DeltaSharingService/GetShare" => {
                    #[allow(non_camel_case_types)]
                    struct GetShareSvc<T: DeltaSharingService>(pub Arc<T>);
                    impl<
                        T: DeltaSharingService,
                    > tonic::server::UnaryService<super::GetShareRequest>
                    for GetShareSvc<T> {
                        type Response = super::Share;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetShareRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DeltaSharingService>::get_share(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetShareSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/unitycatalog.sharing.v1.DeltaSharingService/ListSharingSchemas" => {
                    #[allow(non_camel_case_types)]
                    struct ListSharingSchemasSvc<T: DeltaSharingService>(pub Arc<T>);
                    impl<
                        T: DeltaSharingService,
                    > tonic::server::UnaryService<super::ListSharingSchemasRequest>
                    for ListSharingSchemasSvc<T> {
                        type Response = super::ListSharingSchemasResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListSharingSchemasRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DeltaSharingService>::list_sharing_schemas(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ListSharingSchemasSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/unitycatalog.sharing.v1.DeltaSharingService/ListSchemaTables" => {
                    #[allow(non_camel_case_types)]
                    struct ListSchemaTablesSvc<T: DeltaSharingService>(pub Arc<T>);
                    impl<
                        T: DeltaSharingService,
                    > tonic::server::UnaryService<super::ListSchemaTablesRequest>
                    for ListSchemaTablesSvc<T> {
                        type Response = super::ListSchemaTablesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListSchemaTablesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DeltaSharingService>::list_schema_tables(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ListSchemaTablesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/unitycatalog.sharing.v1.DeltaSharingService/ListShareTables" => {
                    #[allow(non_camel_case_types)]
                    struct ListShareTablesSvc<T: DeltaSharingService>(pub Arc<T>);
                    impl<
                        T: DeltaSharingService,
                    > tonic::server::UnaryService<super::ListShareTablesRequest>
                    for ListShareTablesSvc<T> {
                        type Response = super::ListShareTablesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListShareTablesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DeltaSharingService>::list_share_tables(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ListShareTablesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/unitycatalog.sharing.v1.DeltaSharingService/GetTableVersion" => {
                    #[allow(non_camel_case_types)]
                    struct GetTableVersionSvc<T: DeltaSharingService>(pub Arc<T>);
                    impl<
                        T: DeltaSharingService,
                    > tonic::server::UnaryService<super::GetTableVersionRequest>
                    for GetTableVersionSvc<T> {
                        type Response = super::GetTableVersionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetTableVersionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DeltaSharingService>::get_table_version(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetTableVersionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/unitycatalog.sharing.v1.DeltaSharingService/GetTableMetadata" => {
                    #[allow(non_camel_case_types)]
                    struct GetTableMetadataSvc<T: DeltaSharingService>(pub Arc<T>);
                    impl<
                        T: DeltaSharingService,
                    > tonic::server::UnaryService<super::GetTableMetadataRequest>
                    for GetTableMetadataSvc<T> {
                        type Response = super::QueryResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetTableMetadataRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DeltaSharingService>::get_table_metadata(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetTableMetadataSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/unitycatalog.sharing.v1.DeltaSharingService/QueryTable" => {
                    #[allow(non_camel_case_types)]
                    struct QueryTableSvc<T: DeltaSharingService>(pub Arc<T>);
                    impl<
                        T: DeltaSharingService,
                    > tonic::server::UnaryService<super::QueryTableRequest>
                    for QueryTableSvc<T> {
                        type Response = super::QueryResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryTableRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DeltaSharingService>::query_table(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = QueryTableSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", tonic::Code::Unimplemented as i32)
                                .header(
                                    http::header::CONTENT_TYPE,
                                    tonic::metadata::GRPC_CONTENT_TYPE,
                                )
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: DeltaSharingService> Clone for DeltaSharingServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: DeltaSharingService> tonic::server::NamedService
    for DeltaSharingServiceServer<T> {
        const NAME: &'static str = "unitycatalog.sharing.v1.DeltaSharingService";
    }
}

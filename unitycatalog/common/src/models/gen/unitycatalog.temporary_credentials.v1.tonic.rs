// @generated
/// Generated server implementations.
pub mod temporary_credentials_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with TemporaryCredentialsServiceServer.
    #[async_trait]
    pub trait TemporaryCredentialsService: Send + Sync + 'static {
        /** Generate a new set of credentials for a table.
*/
        async fn generate_temporary_table_credentials(
            &self,
            request: tonic::Request<super::GenerateTemporaryTableCredentialsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TemporaryCredential>,
            tonic::Status,
        >;
        /** Generate a new set of credentials for a volume.
*/
        async fn generate_temporary_volume_credentials(
            &self,
            request: tonic::Request<super::GenerateTemporaryVolumeCredentialsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TemporaryCredential>,
            tonic::Status,
        >;
    }
    ///
    #[derive(Debug)]
    pub struct TemporaryCredentialsServiceServer<T: TemporaryCredentialsService> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T: TemporaryCredentialsService> TemporaryCredentialsServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>>
    for TemporaryCredentialsServiceServer<T>
    where
        T: TemporaryCredentialsService,
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
                "/unitycatalog.temporary_credentials.v1.TemporaryCredentialsService/GenerateTemporaryTableCredentials" => {
                    #[allow(non_camel_case_types)]
                    struct GenerateTemporaryTableCredentialsSvc<
                        T: TemporaryCredentialsService,
                    >(
                        pub Arc<T>,
                    );
                    impl<
                        T: TemporaryCredentialsService,
                    > tonic::server::UnaryService<
                        super::GenerateTemporaryTableCredentialsRequest,
                    > for GenerateTemporaryTableCredentialsSvc<T> {
                        type Response = super::TemporaryCredential;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::GenerateTemporaryTableCredentialsRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as TemporaryCredentialsService>::generate_temporary_table_credentials(
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
                        let method = GenerateTemporaryTableCredentialsSvc(inner);
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
                "/unitycatalog.temporary_credentials.v1.TemporaryCredentialsService/GenerateTemporaryVolumeCredentials" => {
                    #[allow(non_camel_case_types)]
                    struct GenerateTemporaryVolumeCredentialsSvc<
                        T: TemporaryCredentialsService,
                    >(
                        pub Arc<T>,
                    );
                    impl<
                        T: TemporaryCredentialsService,
                    > tonic::server::UnaryService<
                        super::GenerateTemporaryVolumeCredentialsRequest,
                    > for GenerateTemporaryVolumeCredentialsSvc<T> {
                        type Response = super::TemporaryCredential;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::GenerateTemporaryVolumeCredentialsRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as TemporaryCredentialsService>::generate_temporary_volume_credentials(
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
                        let method = GenerateTemporaryVolumeCredentialsSvc(inner);
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
    impl<T: TemporaryCredentialsService> Clone for TemporaryCredentialsServiceServer<T> {
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
    impl<T: TemporaryCredentialsService> tonic::server::NamedService
    for TemporaryCredentialsServiceServer<T> {
        const NAME: &'static str = "unitycatalog.temporary_credentials.v1.TemporaryCredentialsService";
    }
}

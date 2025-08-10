#![allow(
    unused_variables,
    dead_code,
    missing_docs,
    clippy::wildcard_imports,
    clippy::let_unit_value
)]
use tonic::codegen::*;
/// Generated trait containing gRPC methods that should be implemented for use with CacheServiceServer.
#[async_trait]
pub trait CacheService: std::marker::Send + std::marker::Sync + 'static {
    /// Lists CachedContents.
    async fn list_cached_contents(
        &self,
        request: tonic::Request<super::ListCachedContentsRequest>,
    ) -> std::result::Result<tonic::Response<super::ListCachedContentsResponse>, tonic::Status>;
    /// Creates CachedContent resource.
    async fn create_cached_content(
        &self,
        request: tonic::Request<super::CreateCachedContentRequest>,
    ) -> std::result::Result<tonic::Response<super::CachedContent>, tonic::Status>;
    /// Reads CachedContent resource.
    async fn get_cached_content(
        &self,
        request: tonic::Request<super::GetCachedContentRequest>,
    ) -> std::result::Result<tonic::Response<super::CachedContent>, tonic::Status>;
    /// Updates CachedContent resource (only expiration is updatable).
    async fn update_cached_content(
        &self,
        request: tonic::Request<super::UpdateCachedContentRequest>,
    ) -> std::result::Result<tonic::Response<super::CachedContent>, tonic::Status>;
    /// Deletes CachedContent resource.
    async fn delete_cached_content(
        &self,
        request: tonic::Request<super::DeleteCachedContentRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status>;
}
/// API for managing cache of content (CachedContent resources) that can be used
/// in GenerativeService requests. This way generate content requests can benefit
/// from preprocessing work being done earlier, possibly lowering their
/// computational cost. It is intended to be used with large contexts.
#[derive(Debug)]
pub struct CacheServiceServer<T> {
    inner: Arc<T>,
    accept_compression_encodings: EnabledCompressionEncodings,
    send_compression_encodings: EnabledCompressionEncodings,
    max_decoding_message_size: Option<usize>,
    max_encoding_message_size: Option<usize>,
}
impl<T> CacheServiceServer<T> {
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
    pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
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
impl<T, B> tonic::codegen::Service<http::Request<B>> for CacheServiceServer<T>
where
    T: CacheService,
    B: Body + std::marker::Send + 'static,
    B::Error: Into<StdError> + std::marker::Send + 'static,
{
    type Response = http::Response<tonic::body::Body>;
    type Error = std::convert::Infallible;
    type Future = BoxFuture<Self::Response, Self::Error>;
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
    fn call(&mut self, req: http::Request<B>) -> Self::Future {
        match req.uri().path() {
            "/google.ai.generativelanguage.v1beta.CacheService/ListCachedContents" => {
                #[allow(non_camel_case_types)]
                struct ListCachedContentsSvc<T: CacheService>(pub Arc<T>);
                impl<T: CacheService> tonic::server::UnaryService<super::ListCachedContentsRequest>
                    for ListCachedContentsSvc<T>
                {
                    type Response = super::ListCachedContentsResponse;
                    type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                    fn call(
                        &mut self,
                        request: tonic::Request<super::ListCachedContentsRequest>,
                    ) -> Self::Future {
                        let inner = Arc::clone(&self.0);
                        let fut = async move {
                            <T as CacheService>::list_cached_contents(&inner, request).await
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
                    let method = ListCachedContentsSvc(inner);
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
            "/google.ai.generativelanguage.v1beta.CacheService/CreateCachedContent" => {
                #[allow(non_camel_case_types)]
                struct CreateCachedContentSvc<T: CacheService>(pub Arc<T>);
                impl<T: CacheService> tonic::server::UnaryService<super::CreateCachedContentRequest>
                    for CreateCachedContentSvc<T>
                {
                    type Response = super::CachedContent;
                    type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                    fn call(
                        &mut self,
                        request: tonic::Request<super::CreateCachedContentRequest>,
                    ) -> Self::Future {
                        let inner = Arc::clone(&self.0);
                        let fut = async move {
                            <T as CacheService>::create_cached_content(&inner, request).await
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
                    let method = CreateCachedContentSvc(inner);
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
            "/google.ai.generativelanguage.v1beta.CacheService/GetCachedContent" => {
                #[allow(non_camel_case_types)]
                struct GetCachedContentSvc<T: CacheService>(pub Arc<T>);
                impl<T: CacheService> tonic::server::UnaryService<super::GetCachedContentRequest>
                    for GetCachedContentSvc<T>
                {
                    type Response = super::CachedContent;
                    type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                    fn call(
                        &mut self,
                        request: tonic::Request<super::GetCachedContentRequest>,
                    ) -> Self::Future {
                        let inner = Arc::clone(&self.0);
                        let fut = async move {
                            <T as CacheService>::get_cached_content(&inner, request).await
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
                    let method = GetCachedContentSvc(inner);
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
            "/google.ai.generativelanguage.v1beta.CacheService/UpdateCachedContent" => {
                #[allow(non_camel_case_types)]
                struct UpdateCachedContentSvc<T: CacheService>(pub Arc<T>);
                impl<T: CacheService> tonic::server::UnaryService<super::UpdateCachedContentRequest>
                    for UpdateCachedContentSvc<T>
                {
                    type Response = super::CachedContent;
                    type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                    fn call(
                        &mut self,
                        request: tonic::Request<super::UpdateCachedContentRequest>,
                    ) -> Self::Future {
                        let inner = Arc::clone(&self.0);
                        let fut = async move {
                            <T as CacheService>::update_cached_content(&inner, request).await
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
                    let method = UpdateCachedContentSvc(inner);
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
            "/google.ai.generativelanguage.v1beta.CacheService/DeleteCachedContent" => {
                #[allow(non_camel_case_types)]
                struct DeleteCachedContentSvc<T: CacheService>(pub Arc<T>);
                impl<T: CacheService> tonic::server::UnaryService<super::DeleteCachedContentRequest>
                    for DeleteCachedContentSvc<T>
                {
                    type Response = ();
                    type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                    fn call(
                        &mut self,
                        request: tonic::Request<super::DeleteCachedContentRequest>,
                    ) -> Self::Future {
                        let inner = Arc::clone(&self.0);
                        let fut = async move {
                            <T as CacheService>::delete_cached_content(&inner, request).await
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
                    let method = DeleteCachedContentSvc(inner);
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
            _ => Box::pin(async move {
                let mut response = http::Response::new(tonic::body::Body::default());
                let headers = response.headers_mut();
                headers.insert(
                    tonic::Status::GRPC_STATUS,
                    (tonic::Code::Unimplemented as i32).into(),
                );
                headers.insert(
                    http::header::CONTENT_TYPE,
                    tonic::metadata::GRPC_CONTENT_TYPE,
                );
                Ok(response)
            }),
        }
    }
}
impl<T> Clone for CacheServiceServer<T> {
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
/// Generated gRPC service name
pub const SERVICE_NAME: &str = "google.ai.generativelanguage.v1beta.CacheService";
impl<T> tonic::server::NamedService for CacheServiceServer<T> {
    const NAME: &'static str = SERVICE_NAME;
}

#![allow(
    unused_variables,
    dead_code,
    missing_docs,
    clippy::wildcard_imports,
    clippy::let_unit_value
)]
use tonic::codegen::*;
/// Generated trait containing gRPC methods that should be implemented for use with ModelServiceServer.
#[async_trait]
pub trait ModelService: std::marker::Send + std::marker::Sync + 'static {
    /// Gets information about a specific `Model` such as its version number, token
    /// limits,
    /// [parameters](https://ai.google.dev/gemini-api/docs/models/generative-models#model-parameters)
    /// and other metadata. Refer to the [Gemini models
    /// guide](https://ai.google.dev/gemini-api/docs/models/gemini) for detailed
    /// model information.
    async fn get_model(
        &self,
        request: tonic::Request<super::GetModelRequest>,
    ) -> std::result::Result<tonic::Response<super::Model>, tonic::Status>;
    /// Lists the [`Model`s](https://ai.google.dev/gemini-api/docs/models/gemini)
    /// available through the Gemini API.
    async fn list_models(
        &self,
        request: tonic::Request<super::ListModelsRequest>,
    ) -> std::result::Result<tonic::Response<super::ListModelsResponse>, tonic::Status>;
    /// Gets information about a specific TunedModel.
    async fn get_tuned_model(
        &self,
        request: tonic::Request<super::GetTunedModelRequest>,
    ) -> std::result::Result<tonic::Response<super::TunedModel>, tonic::Status>;
    /// Lists created tuned models.
    async fn list_tuned_models(
        &self,
        request: tonic::Request<super::ListTunedModelsRequest>,
    ) -> std::result::Result<tonic::Response<super::ListTunedModelsResponse>, tonic::Status>;
    /// Creates a tuned model.
    /// Check intermediate tuning progress (if any) through the
    /// [google.longrunning.Operations] service.
    ///
    /// Access status and results through the Operations service.
    /// Example:
    ///   GET /v1/tunedModels/az2mb0bpw6i/operations/000-111-222
    async fn create_tuned_model(
        &self,
        request: tonic::Request<super::CreateTunedModelRequest>,
    ) -> std::result::Result<tonic::Response<crate::proto::longrunning::Operation>, tonic::Status>;
    /// Updates a tuned model.
    async fn update_tuned_model(
        &self,
        request: tonic::Request<super::UpdateTunedModelRequest>,
    ) -> std::result::Result<tonic::Response<super::TunedModel>, tonic::Status>;
    /// Deletes a tuned model.
    async fn delete_tuned_model(
        &self,
        request: tonic::Request<super::DeleteTunedModelRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status>;
}
/// Provides methods for getting metadata information about Generative Models.
#[derive(Debug)]
pub struct ModelServiceServer<T> {
    inner: Arc<T>,
    accept_compression_encodings: EnabledCompressionEncodings,
    send_compression_encodings: EnabledCompressionEncodings,
    max_decoding_message_size: Option<usize>,
    max_encoding_message_size: Option<usize>,
}
impl<T> ModelServiceServer<T> {
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
impl<T, B> tonic::codegen::Service<http::Request<B>> for ModelServiceServer<T>
where
    T: ModelService,
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
            "/google.ai.generativelanguage.v1beta.ModelService/GetModel" => {
                #[allow(non_camel_case_types)]
                struct GetModelSvc<T: ModelService>(pub Arc<T>);
                impl<T: ModelService> tonic::server::UnaryService<super::GetModelRequest> for GetModelSvc<T> {
                    type Response = super::Model;
                    type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                    fn call(
                        &mut self,
                        request: tonic::Request<super::GetModelRequest>,
                    ) -> Self::Future {
                        let inner = Arc::clone(&self.0);
                        let fut =
                            async move { <T as ModelService>::get_model(&inner, request).await };
                        Box::pin(fut)
                    }
                }
                let accept_compression_encodings = self.accept_compression_encodings;
                let send_compression_encodings = self.send_compression_encodings;
                let max_decoding_message_size = self.max_decoding_message_size;
                let max_encoding_message_size = self.max_encoding_message_size;
                let inner = self.inner.clone();
                let fut = async move {
                    let method = GetModelSvc(inner);
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
            "/google.ai.generativelanguage.v1beta.ModelService/ListModels" => {
                #[allow(non_camel_case_types)]
                struct ListModelsSvc<T: ModelService>(pub Arc<T>);
                impl<T: ModelService> tonic::server::UnaryService<super::ListModelsRequest> for ListModelsSvc<T> {
                    type Response = super::ListModelsResponse;
                    type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                    fn call(
                        &mut self,
                        request: tonic::Request<super::ListModelsRequest>,
                    ) -> Self::Future {
                        let inner = Arc::clone(&self.0);
                        let fut =
                            async move { <T as ModelService>::list_models(&inner, request).await };
                        Box::pin(fut)
                    }
                }
                let accept_compression_encodings = self.accept_compression_encodings;
                let send_compression_encodings = self.send_compression_encodings;
                let max_decoding_message_size = self.max_decoding_message_size;
                let max_encoding_message_size = self.max_encoding_message_size;
                let inner = self.inner.clone();
                let fut = async move {
                    let method = ListModelsSvc(inner);
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
            "/google.ai.generativelanguage.v1beta.ModelService/GetTunedModel" => {
                #[allow(non_camel_case_types)]
                struct GetTunedModelSvc<T: ModelService>(pub Arc<T>);
                impl<T: ModelService> tonic::server::UnaryService<super::GetTunedModelRequest>
                    for GetTunedModelSvc<T>
                {
                    type Response = super::TunedModel;
                    type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                    fn call(
                        &mut self,
                        request: tonic::Request<super::GetTunedModelRequest>,
                    ) -> Self::Future {
                        let inner = Arc::clone(&self.0);
                        let fut = async move {
                            <T as ModelService>::get_tuned_model(&inner, request).await
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
                    let method = GetTunedModelSvc(inner);
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
            "/google.ai.generativelanguage.v1beta.ModelService/ListTunedModels" => {
                #[allow(non_camel_case_types)]
                struct ListTunedModelsSvc<T: ModelService>(pub Arc<T>);
                impl<T: ModelService> tonic::server::UnaryService<super::ListTunedModelsRequest>
                    for ListTunedModelsSvc<T>
                {
                    type Response = super::ListTunedModelsResponse;
                    type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                    fn call(
                        &mut self,
                        request: tonic::Request<super::ListTunedModelsRequest>,
                    ) -> Self::Future {
                        let inner = Arc::clone(&self.0);
                        let fut = async move {
                            <T as ModelService>::list_tuned_models(&inner, request).await
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
                    let method = ListTunedModelsSvc(inner);
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
            "/google.ai.generativelanguage.v1beta.ModelService/CreateTunedModel" => {
                #[allow(non_camel_case_types)]
                struct CreateTunedModelSvc<T: ModelService>(pub Arc<T>);
                impl<T: ModelService> tonic::server::UnaryService<super::CreateTunedModelRequest>
                    for CreateTunedModelSvc<T>
                {
                    type Response = crate::proto::longrunning::Operation;
                    type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                    fn call(
                        &mut self,
                        request: tonic::Request<super::CreateTunedModelRequest>,
                    ) -> Self::Future {
                        let inner = Arc::clone(&self.0);
                        let fut = async move {
                            <T as ModelService>::create_tuned_model(&inner, request).await
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
                    let method = CreateTunedModelSvc(inner);
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
            "/google.ai.generativelanguage.v1beta.ModelService/UpdateTunedModel" => {
                #[allow(non_camel_case_types)]
                struct UpdateTunedModelSvc<T: ModelService>(pub Arc<T>);
                impl<T: ModelService> tonic::server::UnaryService<super::UpdateTunedModelRequest>
                    for UpdateTunedModelSvc<T>
                {
                    type Response = super::TunedModel;
                    type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                    fn call(
                        &mut self,
                        request: tonic::Request<super::UpdateTunedModelRequest>,
                    ) -> Self::Future {
                        let inner = Arc::clone(&self.0);
                        let fut = async move {
                            <T as ModelService>::update_tuned_model(&inner, request).await
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
                    let method = UpdateTunedModelSvc(inner);
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
            "/google.ai.generativelanguage.v1beta.ModelService/DeleteTunedModel" => {
                #[allow(non_camel_case_types)]
                struct DeleteTunedModelSvc<T: ModelService>(pub Arc<T>);
                impl<T: ModelService> tonic::server::UnaryService<super::DeleteTunedModelRequest>
                    for DeleteTunedModelSvc<T>
                {
                    type Response = ();
                    type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                    fn call(
                        &mut self,
                        request: tonic::Request<super::DeleteTunedModelRequest>,
                    ) -> Self::Future {
                        let inner = Arc::clone(&self.0);
                        let fut = async move {
                            <T as ModelService>::delete_tuned_model(&inner, request).await
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
                    let method = DeleteTunedModelSvc(inner);
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
impl<T> Clone for ModelServiceServer<T> {
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
pub const SERVICE_NAME: &str = "google.ai.generativelanguage.v1beta.ModelService";
impl<T> tonic::server::NamedService for ModelServiceServer<T> {
    const NAME: &'static str = SERVICE_NAME;
}

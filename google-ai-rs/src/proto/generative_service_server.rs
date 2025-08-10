#![allow(
    unused_variables,
    dead_code,
    missing_docs,
    clippy::wildcard_imports,
    clippy::let_unit_value
)]
use tonic::codegen::*;
/// Generated trait containing gRPC methods that should be implemented for use with GenerativeServiceServer.
#[async_trait]
pub trait GenerativeService: std::marker::Send + std::marker::Sync + 'static {
    /// Generates a model response given an input `GenerateContentRequest`.
    /// Refer to the [text generation
    /// guide](https://ai.google.dev/gemini-api/docs/text-generation) for detailed
    /// usage information. Input capabilities differ between models, including
    /// tuned models. Refer to the [model
    /// guide](https://ai.google.dev/gemini-api/docs/models/gemini) and [tuning
    /// guide](https://ai.google.dev/gemini-api/docs/model-tuning) for details.
    async fn generate_content(
        &self,
        request: tonic::Request<super::GenerateContentRequest>,
    ) -> std::result::Result<tonic::Response<super::GenerateContentResponse>, tonic::Status>;
    /// Generates a grounded answer from the model given an input
    /// `GenerateAnswerRequest`.
    async fn generate_answer(
        &self,
        request: tonic::Request<super::GenerateAnswerRequest>,
    ) -> std::result::Result<tonic::Response<super::GenerateAnswerResponse>, tonic::Status>;
    /// Server streaming response type for the StreamGenerateContent method.
    type StreamGenerateContentStream: tonic::codegen::tokio_stream::Stream<
            Item = std::result::Result<super::GenerateContentResponse, tonic::Status>,
        > + std::marker::Send
        + 'static;
    /// Generates a [streamed
    /// response](https://ai.google.dev/gemini-api/docs/text-generation?lang=python#generate-a-text-stream)
    /// from the model given an input `GenerateContentRequest`.
    async fn stream_generate_content(
        &self,
        request: tonic::Request<super::GenerateContentRequest>,
    ) -> std::result::Result<tonic::Response<Self::StreamGenerateContentStream>, tonic::Status>;
    /// Generates a text embedding vector from the input `Content` using the
    /// specified [Gemini Embedding
    /// model](https://ai.google.dev/gemini-api/docs/models/gemini#text-embedding).
    async fn embed_content(
        &self,
        request: tonic::Request<super::EmbedContentRequest>,
    ) -> std::result::Result<tonic::Response<super::EmbedContentResponse>, tonic::Status>;
    /// Generates multiple embedding vectors from the input `Content` which
    /// consists of a batch of strings represented as `EmbedContentRequest`
    /// objects.
    async fn batch_embed_contents(
        &self,
        request: tonic::Request<super::BatchEmbedContentsRequest>,
    ) -> std::result::Result<tonic::Response<super::BatchEmbedContentsResponse>, tonic::Status>;
    /// Runs a model's tokenizer on input `Content` and returns the token count.
    /// Refer to the [tokens guide](https://ai.google.dev/gemini-api/docs/tokens)
    /// to learn more about tokens.
    async fn count_tokens(
        &self,
        request: tonic::Request<super::CountTokensRequest>,
    ) -> std::result::Result<tonic::Response<super::CountTokensResponse>, tonic::Status>;
}
/// API for using Large Models that generate multimodal content and have
/// additional capabilities beyond text generation.
#[derive(Debug)]
pub struct GenerativeServiceServer<T> {
    inner: Arc<T>,
    accept_compression_encodings: EnabledCompressionEncodings,
    send_compression_encodings: EnabledCompressionEncodings,
    max_decoding_message_size: Option<usize>,
    max_encoding_message_size: Option<usize>,
}
impl<T> GenerativeServiceServer<T> {
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
impl<T, B> tonic::codegen::Service<http::Request<B>> for GenerativeServiceServer<T>
where
    T: GenerativeService,
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
            "/google.ai.generativelanguage.v1beta.GenerativeService/GenerateContent" => {
                #[allow(non_camel_case_types)]
                struct GenerateContentSvc<T: GenerativeService>(pub Arc<T>);
                impl<T: GenerativeService>
                    tonic::server::UnaryService<super::GenerateContentRequest>
                    for GenerateContentSvc<T>
                {
                    type Response = super::GenerateContentResponse;
                    type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                    fn call(
                        &mut self,
                        request: tonic::Request<super::GenerateContentRequest>,
                    ) -> Self::Future {
                        let inner = Arc::clone(&self.0);
                        let fut = async move {
                            <T as GenerativeService>::generate_content(&inner, request).await
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
                    let method = GenerateContentSvc(inner);
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
            "/google.ai.generativelanguage.v1beta.GenerativeService/GenerateAnswer" => {
                #[allow(non_camel_case_types)]
                struct GenerateAnswerSvc<T: GenerativeService>(pub Arc<T>);
                impl<T: GenerativeService> tonic::server::UnaryService<super::GenerateAnswerRequest>
                    for GenerateAnswerSvc<T>
                {
                    type Response = super::GenerateAnswerResponse;
                    type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                    fn call(
                        &mut self,
                        request: tonic::Request<super::GenerateAnswerRequest>,
                    ) -> Self::Future {
                        let inner = Arc::clone(&self.0);
                        let fut = async move {
                            <T as GenerativeService>::generate_answer(&inner, request).await
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
                    let method = GenerateAnswerSvc(inner);
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
            "/google.ai.generativelanguage.v1beta.GenerativeService/StreamGenerateContent" => {
                #[allow(non_camel_case_types)]
                struct StreamGenerateContentSvc<T: GenerativeService>(pub Arc<T>);
                impl<T: GenerativeService>
                    tonic::server::ServerStreamingService<super::GenerateContentRequest>
                    for StreamGenerateContentSvc<T>
                {
                    type Response = super::GenerateContentResponse;
                    type ResponseStream = T::StreamGenerateContentStream;
                    type Future = BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
                    fn call(
                        &mut self,
                        request: tonic::Request<super::GenerateContentRequest>,
                    ) -> Self::Future {
                        let inner = Arc::clone(&self.0);
                        let fut = async move {
                            <T as GenerativeService>::stream_generate_content(&inner, request).await
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
                    let method = StreamGenerateContentSvc(inner);
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
                    let res = grpc.server_streaming(method, req).await;
                    Ok(res)
                };
                Box::pin(fut)
            }
            "/google.ai.generativelanguage.v1beta.GenerativeService/EmbedContent" => {
                #[allow(non_camel_case_types)]
                struct EmbedContentSvc<T: GenerativeService>(pub Arc<T>);
                impl<T: GenerativeService> tonic::server::UnaryService<super::EmbedContentRequest>
                    for EmbedContentSvc<T>
                {
                    type Response = super::EmbedContentResponse;
                    type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                    fn call(
                        &mut self,
                        request: tonic::Request<super::EmbedContentRequest>,
                    ) -> Self::Future {
                        let inner = Arc::clone(&self.0);
                        let fut = async move {
                            <T as GenerativeService>::embed_content(&inner, request).await
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
                    let method = EmbedContentSvc(inner);
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
            "/google.ai.generativelanguage.v1beta.GenerativeService/BatchEmbedContents" => {
                #[allow(non_camel_case_types)]
                struct BatchEmbedContentsSvc<T: GenerativeService>(pub Arc<T>);
                impl<T: GenerativeService>
                    tonic::server::UnaryService<super::BatchEmbedContentsRequest>
                    for BatchEmbedContentsSvc<T>
                {
                    type Response = super::BatchEmbedContentsResponse;
                    type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                    fn call(
                        &mut self,
                        request: tonic::Request<super::BatchEmbedContentsRequest>,
                    ) -> Self::Future {
                        let inner = Arc::clone(&self.0);
                        let fut = async move {
                            <T as GenerativeService>::batch_embed_contents(&inner, request).await
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
                    let method = BatchEmbedContentsSvc(inner);
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
            "/google.ai.generativelanguage.v1beta.GenerativeService/CountTokens" => {
                #[allow(non_camel_case_types)]
                struct CountTokensSvc<T: GenerativeService>(pub Arc<T>);
                impl<T: GenerativeService> tonic::server::UnaryService<super::CountTokensRequest>
                    for CountTokensSvc<T>
                {
                    type Response = super::CountTokensResponse;
                    type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                    fn call(
                        &mut self,
                        request: tonic::Request<super::CountTokensRequest>,
                    ) -> Self::Future {
                        let inner = Arc::clone(&self.0);
                        let fut = async move {
                            <T as GenerativeService>::count_tokens(&inner, request).await
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
                    let method = CountTokensSvc(inner);
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
impl<T> Clone for GenerativeServiceServer<T> {
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
pub const SERVICE_NAME: &str = "google.ai.generativelanguage.v1beta.GenerativeService";
impl<T> tonic::server::NamedService for GenerativeServiceServer<T> {
    const NAME: &'static str = SERVICE_NAME;
}

#![allow(
    unused_variables,
    dead_code,
    missing_docs,
    clippy::wildcard_imports,
    clippy::let_unit_value
)]
use tonic::codegen::http::Uri;
use tonic::codegen::*;
/// API for using Large Models that generate multimodal content and have
/// additional capabilities beyond text generation.
#[derive(Debug, Clone)]
pub struct GenerativeServiceClient<T> {
    inner: tonic::client::Grpc<T>,
}
impl GenerativeServiceClient<tonic::transport::Channel> {
    /// Attempt to create a new client by connecting to a given endpoint.
    pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
    where
        D: TryInto<tonic::transport::Endpoint>,
        D::Error: Into<StdError>,
    {
        let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
        Ok(Self::new(conn))
    }
}
impl<T> GenerativeServiceClient<T>
where
    T: tonic::client::GrpcService<tonic::body::BoxBody>,
    T::Error: Into<StdError>,
    T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
    <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
{
    pub fn new(inner: T) -> Self {
        let inner = tonic::client::Grpc::new(inner);
        Self { inner }
    }
    pub fn with_origin(inner: T, origin: Uri) -> Self {
        let inner = tonic::client::Grpc::with_origin(inner, origin);
        Self { inner }
    }
    pub fn with_interceptor<F>(
        inner: T,
        interceptor: F,
    ) -> GenerativeServiceClient<InterceptedService<T, F>>
    where
        F: tonic::service::Interceptor,
        T::ResponseBody: Default,
        T: tonic::codegen::Service<
            http::Request<tonic::body::BoxBody>,
            Response = http::Response<
                <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
            >,
        >,
        <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
            Into<StdError> + std::marker::Send + std::marker::Sync,
    {
        GenerativeServiceClient::new(InterceptedService::new(inner, interceptor))
    }
    /// Compress requests with the given encoding.
    ///
    /// This requires the server to support it otherwise it might respond with an
    /// error.
    #[must_use]
    pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
        self.inner = self.inner.send_compressed(encoding);
        self
    }
    /// Enable decompressing responses.
    #[must_use]
    pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
        self.inner = self.inner.accept_compressed(encoding);
        self
    }
    /// Limits the maximum size of a decoded message.
    ///
    /// Default: `4MB`
    #[must_use]
    pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
        self.inner = self.inner.max_decoding_message_size(limit);
        self
    }
    /// Limits the maximum size of an encoded message.
    ///
    /// Default: `usize::MAX`
    #[must_use]
    pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
        self.inner = self.inner.max_encoding_message_size(limit);
        self
    }
    /// Generates a model response given an input `GenerateContentRequest`.
    /// Refer to the [text generation
    /// guide](https://ai.google.dev/gemini-api/docs/text-generation) for detailed
    /// usage information. Input capabilities differ between models, including
    /// tuned models. Refer to the [model
    /// guide](https://ai.google.dev/gemini-api/docs/models/gemini) and [tuning
    /// guide](https://ai.google.dev/gemini-api/docs/model-tuning) for details.
    pub async fn generate_content(
        &mut self,
        request: impl tonic::IntoRequest<super::GenerateContentRequest>,
    ) -> std::result::Result<tonic::Response<super::GenerateContentResponse>, tonic::Status> {
        self.inner
            .ready()
            .await
            .map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
        let codec = tonic::codec::ProstCodec::default();
        let path = http::uri::PathAndQuery::from_static(
            "/google.ai.generativelanguage.v1beta.GenerativeService/GenerateContent",
        );
        let mut req = request.into_request();
        req.extensions_mut().insert(GrpcMethod::new(
            "google.ai.generativelanguage.v1beta.GenerativeService",
            "GenerateContent",
        ));
        self.inner.unary(req, path, codec).await
    }
    /// Generates a grounded answer from the model given an input
    /// `GenerateAnswerRequest`.
    pub async fn generate_answer(
        &mut self,
        request: impl tonic::IntoRequest<super::GenerateAnswerRequest>,
    ) -> std::result::Result<tonic::Response<super::GenerateAnswerResponse>, tonic::Status> {
        self.inner
            .ready()
            .await
            .map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
        let codec = tonic::codec::ProstCodec::default();
        let path = http::uri::PathAndQuery::from_static(
            "/google.ai.generativelanguage.v1beta.GenerativeService/GenerateAnswer",
        );
        let mut req = request.into_request();
        req.extensions_mut().insert(GrpcMethod::new(
            "google.ai.generativelanguage.v1beta.GenerativeService",
            "GenerateAnswer",
        ));
        self.inner.unary(req, path, codec).await
    }
    /// Generates a [streamed
    /// response](https://ai.google.dev/gemini-api/docs/text-generation?lang=python#generate-a-text-stream)
    /// from the model given an input `GenerateContentRequest`.
    pub async fn stream_generate_content(
        &mut self,
        request: impl tonic::IntoRequest<super::GenerateContentRequest>,
    ) -> std::result::Result<
        tonic::Response<tonic::codec::Streaming<super::GenerateContentResponse>>,
        tonic::Status,
    > {
        self.inner
            .ready()
            .await
            .map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
        let codec = tonic::codec::ProstCodec::default();
        let path = http::uri::PathAndQuery::from_static(
            "/google.ai.generativelanguage.v1beta.GenerativeService/StreamGenerateContent",
        );
        let mut req = request.into_request();
        req.extensions_mut().insert(GrpcMethod::new(
            "google.ai.generativelanguage.v1beta.GenerativeService",
            "StreamGenerateContent",
        ));
        self.inner.server_streaming(req, path, codec).await
    }
    /// Generates a text embedding vector from the input `Content` using the
    /// specified [Gemini Embedding
    /// model](https://ai.google.dev/gemini-api/docs/models/gemini#text-embedding).
    pub async fn embed_content(
        &mut self,
        request: impl tonic::IntoRequest<super::EmbedContentRequest>,
    ) -> std::result::Result<tonic::Response<super::EmbedContentResponse>, tonic::Status> {
        self.inner
            .ready()
            .await
            .map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
        let codec = tonic::codec::ProstCodec::default();
        let path = http::uri::PathAndQuery::from_static(
            "/google.ai.generativelanguage.v1beta.GenerativeService/EmbedContent",
        );
        let mut req = request.into_request();
        req.extensions_mut().insert(GrpcMethod::new(
            "google.ai.generativelanguage.v1beta.GenerativeService",
            "EmbedContent",
        ));
        self.inner.unary(req, path, codec).await
    }
    /// Generates multiple embedding vectors from the input `Content` which
    /// consists of a batch of strings represented as `EmbedContentRequest`
    /// objects.
    pub async fn batch_embed_contents(
        &mut self,
        request: impl tonic::IntoRequest<super::BatchEmbedContentsRequest>,
    ) -> std::result::Result<tonic::Response<super::BatchEmbedContentsResponse>, tonic::Status>
    {
        self.inner
            .ready()
            .await
            .map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
        let codec = tonic::codec::ProstCodec::default();
        let path = http::uri::PathAndQuery::from_static(
            "/google.ai.generativelanguage.v1beta.GenerativeService/BatchEmbedContents",
        );
        let mut req = request.into_request();
        req.extensions_mut().insert(GrpcMethod::new(
            "google.ai.generativelanguage.v1beta.GenerativeService",
            "BatchEmbedContents",
        ));
        self.inner.unary(req, path, codec).await
    }
    /// Runs a model's tokenizer on input `Content` and returns the token count.
    /// Refer to the [tokens guide](https://ai.google.dev/gemini-api/docs/tokens)
    /// to learn more about tokens.
    pub async fn count_tokens(
        &mut self,
        request: impl tonic::IntoRequest<super::CountTokensRequest>,
    ) -> std::result::Result<tonic::Response<super::CountTokensResponse>, tonic::Status> {
        self.inner
            .ready()
            .await
            .map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
        let codec = tonic::codec::ProstCodec::default();
        let path = http::uri::PathAndQuery::from_static(
            "/google.ai.generativelanguage.v1beta.GenerativeService/CountTokens",
        );
        let mut req = request.into_request();
        req.extensions_mut().insert(GrpcMethod::new(
            "google.ai.generativelanguage.v1beta.GenerativeService",
            "CountTokens",
        ));
        self.inner.unary(req, path, codec).await
    }
}

#![allow(
    unused_variables,
    dead_code,
    missing_docs,
    clippy::wildcard_imports,
    clippy::let_unit_value
)]
use tonic::codegen::http::Uri;
use tonic::codegen::*;
/// Provides methods for getting metadata information about Generative Models.
#[derive(Debug, Clone)]
pub struct ModelServiceClient<T> {
    inner: tonic::client::Grpc<T>,
}
impl ModelServiceClient<tonic::transport::Channel> {
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
impl<T> ModelServiceClient<T>
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
    ) -> ModelServiceClient<InterceptedService<T, F>>
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
        ModelServiceClient::new(InterceptedService::new(inner, interceptor))
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
    /// Gets information about a specific `Model` such as its version number, token
    /// limits,
    /// [parameters](https://ai.google.dev/gemini-api/docs/models/generative-models#model-parameters)
    /// and other metadata. Refer to the [Gemini models
    /// guide](https://ai.google.dev/gemini-api/docs/models/gemini) for detailed
    /// model information.
    pub async fn get_model(
        &mut self,
        request: impl tonic::IntoRequest<super::GetModelRequest>,
    ) -> std::result::Result<tonic::Response<super::Model>, tonic::Status> {
        self.inner
            .ready()
            .await
            .map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
        let codec = tonic::codec::ProstCodec::default();
        let path = http::uri::PathAndQuery::from_static(
            "/google.ai.generativelanguage.v1beta.ModelService/GetModel",
        );
        let mut req = request.into_request();
        req.extensions_mut().insert(GrpcMethod::new(
            "google.ai.generativelanguage.v1beta.ModelService",
            "GetModel",
        ));
        self.inner.unary(req, path, codec).await
    }
    /// Lists the [`Model`s](https://ai.google.dev/gemini-api/docs/models/gemini)
    /// available through the Gemini API.
    pub async fn list_models(
        &mut self,
        request: impl tonic::IntoRequest<super::ListModelsRequest>,
    ) -> std::result::Result<tonic::Response<super::ListModelsResponse>, tonic::Status> {
        self.inner
            .ready()
            .await
            .map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
        let codec = tonic::codec::ProstCodec::default();
        let path = http::uri::PathAndQuery::from_static(
            "/google.ai.generativelanguage.v1beta.ModelService/ListModels",
        );
        let mut req = request.into_request();
        req.extensions_mut().insert(GrpcMethod::new(
            "google.ai.generativelanguage.v1beta.ModelService",
            "ListModels",
        ));
        self.inner.unary(req, path, codec).await
    }
    /// Gets information about a specific TunedModel.
    pub async fn get_tuned_model(
        &mut self,
        request: impl tonic::IntoRequest<super::GetTunedModelRequest>,
    ) -> std::result::Result<tonic::Response<super::TunedModel>, tonic::Status> {
        self.inner
            .ready()
            .await
            .map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
        let codec = tonic::codec::ProstCodec::default();
        let path = http::uri::PathAndQuery::from_static(
            "/google.ai.generativelanguage.v1beta.ModelService/GetTunedModel",
        );
        let mut req = request.into_request();
        req.extensions_mut().insert(GrpcMethod::new(
            "google.ai.generativelanguage.v1beta.ModelService",
            "GetTunedModel",
        ));
        self.inner.unary(req, path, codec).await
    }
    /// Lists created tuned models.
    pub async fn list_tuned_models(
        &mut self,
        request: impl tonic::IntoRequest<super::ListTunedModelsRequest>,
    ) -> std::result::Result<tonic::Response<super::ListTunedModelsResponse>, tonic::Status> {
        self.inner
            .ready()
            .await
            .map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
        let codec = tonic::codec::ProstCodec::default();
        let path = http::uri::PathAndQuery::from_static(
            "/google.ai.generativelanguage.v1beta.ModelService/ListTunedModels",
        );
        let mut req = request.into_request();
        req.extensions_mut().insert(GrpcMethod::new(
            "google.ai.generativelanguage.v1beta.ModelService",
            "ListTunedModels",
        ));
        self.inner.unary(req, path, codec).await
    }
    /// Creates a tuned model.
    /// Check intermediate tuning progress (if any) through the
    /// [google.longrunning.Operations] service.
    ///
    /// Access status and results through the Operations service.
    /// Example:
    ///   GET /v1/tunedModels/az2mb0bpw6i/operations/000-111-222
    pub async fn create_tuned_model(
        &mut self,
        request: impl tonic::IntoRequest<super::CreateTunedModelRequest>,
    ) -> std::result::Result<tonic::Response<crate::proto::longrunning::Operation>, tonic::Status>
    {
        self.inner
            .ready()
            .await
            .map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
        let codec = tonic::codec::ProstCodec::default();
        let path = http::uri::PathAndQuery::from_static(
            "/google.ai.generativelanguage.v1beta.ModelService/CreateTunedModel",
        );
        let mut req = request.into_request();
        req.extensions_mut().insert(GrpcMethod::new(
            "google.ai.generativelanguage.v1beta.ModelService",
            "CreateTunedModel",
        ));
        self.inner.unary(req, path, codec).await
    }
    /// Updates a tuned model.
    pub async fn update_tuned_model(
        &mut self,
        request: impl tonic::IntoRequest<super::UpdateTunedModelRequest>,
    ) -> std::result::Result<tonic::Response<super::TunedModel>, tonic::Status> {
        self.inner
            .ready()
            .await
            .map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
        let codec = tonic::codec::ProstCodec::default();
        let path = http::uri::PathAndQuery::from_static(
            "/google.ai.generativelanguage.v1beta.ModelService/UpdateTunedModel",
        );
        let mut req = request.into_request();
        req.extensions_mut().insert(GrpcMethod::new(
            "google.ai.generativelanguage.v1beta.ModelService",
            "UpdateTunedModel",
        ));
        self.inner.unary(req, path, codec).await
    }
    /// Deletes a tuned model.
    pub async fn delete_tuned_model(
        &mut self,
        request: impl tonic::IntoRequest<super::DeleteTunedModelRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        self.inner
            .ready()
            .await
            .map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
        let codec = tonic::codec::ProstCodec::default();
        let path = http::uri::PathAndQuery::from_static(
            "/google.ai.generativelanguage.v1beta.ModelService/DeleteTunedModel",
        );
        let mut req = request.into_request();
        req.extensions_mut().insert(GrpcMethod::new(
            "google.ai.generativelanguage.v1beta.ModelService",
            "DeleteTunedModel",
        ));
        self.inner.unary(req, path, codec).await
    }
}

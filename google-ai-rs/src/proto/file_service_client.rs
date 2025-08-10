#![allow(
    unused_variables,
    dead_code,
    missing_docs,
    clippy::wildcard_imports,
    clippy::let_unit_value
)]
use tonic::codegen::http::Uri;
use tonic::codegen::*;
/// An API for uploading and managing files.
#[derive(Debug, Clone)]
pub struct FileServiceClient<T> {
    inner: tonic::client::Grpc<T>,
}
impl FileServiceClient<tonic::transport::Channel> {
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
impl<T> FileServiceClient<T>
where
    T: tonic::client::GrpcService<tonic::body::Body>,
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
    ) -> FileServiceClient<InterceptedService<T, F>>
    where
        F: tonic::service::Interceptor,
        T::ResponseBody: Default,
        T: tonic::codegen::Service<
            http::Request<tonic::body::Body>,
            Response = http::Response<
                <T as tonic::client::GrpcService<tonic::body::Body>>::ResponseBody,
            >,
        >,
        <T as tonic::codegen::Service<http::Request<tonic::body::Body>>>::Error:
            Into<StdError> + std::marker::Send + std::marker::Sync,
    {
        FileServiceClient::new(InterceptedService::new(inner, interceptor))
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
    /// Creates a `File`.
    pub async fn create_file(
        &mut self,
        request: impl tonic::IntoRequest<super::CreateFileRequest>,
    ) -> std::result::Result<tonic::Response<super::CreateFileResponse>, tonic::Status> {
        self.inner
            .ready()
            .await
            .map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
        let codec = tonic::codec::ProstCodec::default();
        let path = http::uri::PathAndQuery::from_static(
            "/google.ai.generativelanguage.v1beta.FileService/CreateFile",
        );
        let mut req = request.into_request();
        req.extensions_mut().insert(GrpcMethod::new(
            "google.ai.generativelanguage.v1beta.FileService",
            "CreateFile",
        ));
        self.inner.unary(req, path, codec).await
    }
    /// Lists the metadata for `File`s owned by the requesting project.
    pub async fn list_files(
        &mut self,
        request: impl tonic::IntoRequest<super::ListFilesRequest>,
    ) -> std::result::Result<tonic::Response<super::ListFilesResponse>, tonic::Status> {
        self.inner
            .ready()
            .await
            .map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
        let codec = tonic::codec::ProstCodec::default();
        let path = http::uri::PathAndQuery::from_static(
            "/google.ai.generativelanguage.v1beta.FileService/ListFiles",
        );
        let mut req = request.into_request();
        req.extensions_mut().insert(GrpcMethod::new(
            "google.ai.generativelanguage.v1beta.FileService",
            "ListFiles",
        ));
        self.inner.unary(req, path, codec).await
    }
    /// Gets the metadata for the given `File`.
    pub async fn get_file(
        &mut self,
        request: impl tonic::IntoRequest<super::GetFileRequest>,
    ) -> std::result::Result<tonic::Response<super::File>, tonic::Status> {
        self.inner
            .ready()
            .await
            .map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
        let codec = tonic::codec::ProstCodec::default();
        let path = http::uri::PathAndQuery::from_static(
            "/google.ai.generativelanguage.v1beta.FileService/GetFile",
        );
        let mut req = request.into_request();
        req.extensions_mut().insert(GrpcMethod::new(
            "google.ai.generativelanguage.v1beta.FileService",
            "GetFile",
        ));
        self.inner.unary(req, path, codec).await
    }
    /// Deletes the `File`.
    pub async fn delete_file(
        &mut self,
        request: impl tonic::IntoRequest<super::DeleteFileRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        self.inner
            .ready()
            .await
            .map_err(|e| tonic::Status::unknown(format!("Service was not ready: {}", e.into())))?;
        let codec = tonic::codec::ProstCodec::default();
        let path = http::uri::PathAndQuery::from_static(
            "/google.ai.generativelanguage.v1beta.FileService/DeleteFile",
        );
        let mut req = request.into_request();
        req.extensions_mut().insert(GrpcMethod::new(
            "google.ai.generativelanguage.v1beta.FileService",
            "DeleteFile",
        ));
        self.inner.unary(req, path, codec).await
    }
}

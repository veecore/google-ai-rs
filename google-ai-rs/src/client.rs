use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tonic::transport::{Channel, ClientTlsConfig, Endpoint};
use tonic::IntoRequest;

use crate::auth::{add_auth, Auth};
use crate::content::UpdateFieldMask as _;
use crate::error::{status_into_error, Error, NetError, SetupError, TonicTransportError};
use crate::full_model_name;
use crate::proto::model_service_client::ModelServiceClient;
use crate::proto::{
    cache_service_client::CacheServiceClient, generative_service_client::GenerativeServiceClient,
    CachedContent, CreateCachedContentRequest, DeleteCachedContentRequest, GetCachedContentRequest,
    ListCachedContentsRequest, UpdateCachedContentRequest,
};
use crate::proto::{
    DeleteTunedModelRequest, GetModelRequest, GetTunedModelRequest, ListModelsRequest,
    ListTunedModelsRequest, Model, TunedModel, UpdateTunedModelRequest,
};

/// Default timeout for client requests (2 minutes)
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(120);
/// Base URL for Google's Generative Language API
const BASE_API_URL: &str = "https://generativelanguage.googleapis.com";
/// Default page size for paginated requests (server determines actual size when 0)
const DEFAULT_PAGE_SIZE: i32 = 0;

/// A thread-safe client for interacting with Google's Generative Language API.
///
/// # Features
/// - Manages authentication tokens and TLS configuration
/// - Provides access to generative AI operations
/// - Implements content caching functionality
/// - Supports automatic pagination of list operations
///
/// # Example
/// ```
/// use google_ai_rs::{Client, Auth};
///
/// # async fn f() -> Result<(), Box<dyn std::error::Error>> {
/// let auth = Auth::new("your-api-key");
/// let client = Client::new(auth).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct Client {
    /// Generative service gRPC client
    pub(super) gc: GenerativeServiceClient<Channel>,
    /// Cache service gRPC client
    pub(super) cc: CacheServiceClient<Channel>,
    pub(super) mc: ModelServiceClient<Channel>,
    /// Authentication credentials with concurrent access support
    auth: Arc<RwLock<Auth>>,
}

impl Client {
    /// Constructs a new client with authentication and optional configuration.
    ///
    /// # Arguments
    /// * `auth` - API authentication credentials
    ///
    /// # Errors
    /// Returns [`Error::Setup`] for configuration issues or [`Error::Net`] for connection failures.
    pub async fn new(auth: Auth) -> Result<Self, Error> {
        ClientBuilder::new()
            .timeout(DEFAULT_TIMEOUT)
            .build(auth)
            .await
    }

    /// Create a builder for configuring client options
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Updates authentication credentials atomically
    ///
    /// Subsequent requests will use the new credentials immediately. This operation
    /// is thread-safe.
    pub async fn update_auth(&self, new_auth: Auth) {
        *self.auth.write().await = new_auth;
    }

    /// Creates a new cached content entry
    ///
    /// # Arguments
    /// * `content` - Content to cache without name (server-generated)
    ///
    /// # Errors
    /// Returns [`Error::InvalidArgument`] if content contains a name
    pub async fn create_cached_content(
        &self,
        content: CachedContent,
    ) -> Result<CachedContent, Error> {
        if content.name.is_some() {
            return Err(Error::InvalidArgument(
                "CachedContent name must be empty for creation".into(),
            ));
        }

        let mut request = CreateCachedContentRequest {
            cached_content: Some(content),
        }
        .into_request();

        self.add_auth(&mut request).await?;

        self.cc
            .clone()
            .create_cached_content(request)
            .await
            .map_err(status_into_error)
            .map(|r| r.into_inner())
    }

    /// Retrieves the `CachedContent` with the given name.
    pub async fn get_cached_content(&self, name: &str) -> Result<CachedContent, Error> {
        let mut request = GetCachedContentRequest {
            name: name.to_owned(),
        }
        .into_request();

        self.add_auth(&mut request).await?;

        self.cc
            .clone()
            .get_cached_content(request)
            .await
            .map_err(status_into_error)
            .map(|r| r.into_inner())
    }

    /// Deletes the `CachedContent` with the given name.
    pub async fn delete_cached_content(&self, name: &str) -> Result<(), Error> {
        let mut request = DeleteCachedContentRequest {
            name: name.to_owned(),
        }
        .into_request();

        self.add_auth(&mut request).await?;

        self.cc
            .clone()
            .delete_cached_content(request)
            .await
            .map_err(status_into_error)
            .map(|r| r.into_inner())
    }

    /// Modifies the `CachedContent`.
    ///
    /// It returns the modified CachedContent.
    ///
    /// The argument CachedContent must have its name field and fields to update populated.
    pub async fn update_cached_content(&self, cc: &CachedContent) -> Result<CachedContent, Error> {
        let mut request = UpdateCachedContentRequest {
            cached_content: Some(cc.to_owned()),
            update_mask: Some(cc.field_mask()),
        }
        .into_request();

        self.add_auth(&mut request).await?;

        self.cc
            .clone()
            .update_cached_content(request)
            .await
            .map_err(status_into_error)
            .map(|r| r.into_inner())
    }

    /// Returns an async iterator over cached content entries
    ///
    /// Automatically handles pagination through server-side results.
    pub fn list_cached_contents(&self) -> CachedContentIterator {
        PageIterator::<CachedContentPager>::new(self)
    }

    /// Gets information about a specific `Model` such as its version number, token
    /// limits, etc
    pub async fn get_model(&self, name: &str) -> Result<Model, Error> {
        let mut request = GetModelRequest {
            name: full_model_name(name),
        }
        .into_request();

        self.add_auth(&mut request).await?;
        self.mc
            .clone()
            .get_model(request)
            .await
            .map_err(status_into_error)
            .map(|r| r.into_inner())
    }

    /// Gets information about a specific `TunedModel`.
    pub async fn get_tuned_model(&self, resource_name: &str) -> Result<TunedModel, Error> {
        let mut request = GetTunedModelRequest {
            name: resource_name.to_owned(),
        }
        .into_request();

        self.add_auth(&mut request).await?;
        self.mc
            .clone()
            .get_tuned_model(request)
            .await
            .map_err(status_into_error)
            .map(|r| r.into_inner())
    }

    /// Returns an async iterator over models list results
    ///
    /// Automatically handles pagination through server-side results.
    pub async fn list_models(&self) -> ModelsListIterator {
        PageIterator::<ModelsListPager>::new(self)
    }

    /// Returns an async iterator over tuned models list results
    ///
    /// Automatically handles pagination through server-side results.
    pub async fn list_tuned_models(&self) -> TunedModelsListIterator {
        PageIterator::<TunedModelsListPager>::new(self)
    }

    /// Updates a tuned model.
    pub async fn update_tuned_model(&self, m: &TunedModel) -> Result<TunedModel, Error> {
        let mut request = UpdateTunedModelRequest {
            tuned_model: Some(m.to_owned()),
            update_mask: Some(m.field_mask()),
        }
        .into_request();

        self.add_auth(&mut request).await?;

        self.mc
            .clone()
            .update_tuned_model(request)
            .await
            .map_err(status_into_error)
            .map(|r| r.into_inner())
    }

    /// Deletes the `TunedModel` with the given name.
    pub async fn delete_tuned_model(&self, name: &str) -> Result<(), Error> {
        let mut request = DeleteTunedModelRequest {
            name: name.to_owned(),
        }
        .into_request();

        self.add_auth(&mut request).await?;

        self.mc
            .clone()
            .delete_tuned_model(request)
            .await
            .map_err(status_into_error)
            .map(|r| r.into_inner())
    }

    /// Adds authentication metadata to a request
    pub(super) async fn add_auth<T>(&self, request: &mut tonic::Request<T>) -> Result<(), Error> {
        Ok(add_auth(request, &*self.auth.read().await).await?)
    }
}

#[derive(Debug, Clone)]
pub struct ClientBuilder {
    endpoint: Endpoint,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientBuilder {
    /// Creates new builder with required authentication
    pub fn new() -> Self {
        Self {
            endpoint: Endpoint::from_static(BASE_API_URL),
        }
    }

    /// Sets overall request timeout (default: 120s)
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.endpoint = self.endpoint.timeout(duration);
        self
    }

    /// Set connection establishment timeout
    pub fn connect_timeout(mut self, duration: Duration) -> Self {
        self.endpoint = self.endpoint.connect_timeout(duration);
        self
    }

    /// Set custom user agent string
    pub fn user_agent(mut self, ua: impl Into<String>) -> Result<Self, Error> {
        self.endpoint = self
            .endpoint
            .user_agent(ua.into())
            .map_err(|e| SetupError::new("User-Agent configuration", e))?;
        Ok(self)
    }

    /// Set maximum concurrent requests per connection
    pub fn concurrency_limit(mut self, limit: usize) -> Self {
        self.endpoint = self.endpoint.concurrency_limit(limit);
        self
    }

    /// Finalizes configuration and constructs client
    ///
    /// # Arguments
    /// * `auth` - Authentication credentials (API key or service account)
    ///
    /// # Errors
    /// - Returns [`Error::Setup`] for invalid configurations
    /// - Returns [`Error::Net`] for connection failures  
    pub async fn build(self, auth: Auth) -> Result<Client, Error> {
        let channel = self
            .endpoint
            .tls_config(ClientTlsConfig::new().with_enabled_roots())
            .map_err(|e| SetupError::new("TLS configuration", e))?
            .connect()
            .await
            .map_err(|e| {
                Error::Net(NetError::TransportFailure(TonicTransportError(Box::new(e))))
            })?;

        Ok(Client {
            gc: GenerativeServiceClient::new(channel.clone()),
            cc: CacheServiceClient::new(channel.clone()),
            mc: ModelServiceClient::new(channel),
            auth: Arc::new(RwLock::new(auth)),
        })
    }
}

/// Async iterator for paginated cached content results
pub type CachedContentIterator<'a> = PageIterator<'a, CachedContentPager>;

/// Async iterator for paginated models results
pub type ModelsListIterator<'a> = PageIterator<'a, ModelsListPager>;

/// Async iterator for paginated tuned models results
pub type TunedModelsListIterator<'a> = PageIterator<'a, TunedModelsListPager>;

/// Async iterator for paginated contents
///
/// Buffers results from multiple pages and provides linear access.
/// Implements automatic page fetching when buffer is exhausted.
pub struct PageIterator<'a, P>
where
    P: Page + Send,
{
    client: &'a Client,
    next_page_token: Option<String>,
    buffer: VecDeque<P::Content>,
}

impl<'a, P> PageIterator<'a, P>
where
    P: Page + Send,
{
    fn new(client: &'a Client) -> Self {
        Self {
            client,
            next_page_token: Some(String::new()),
            buffer: VecDeque::new(),
        }
    }

    /// Returns the next content item
    ///
    /// Returns `Ok(None)` when all items have been exhausted.
    pub async fn next(&mut self) -> Result<Option<P::Content>, Error> {
        if self.buffer.is_empty() {
            if self.next_page_token.is_none() {
                // We've already fetched all pages
                return Ok(None);
            }

            let (items, next_token) =
                P::next(self.client, self.next_page_token.as_ref().unwrap()).await?;

            self.next_page_token = if next_token.is_empty() {
                None
            } else {
                Some(next_token)
            };
            self.buffer.extend(items);
        }

        Ok(self.buffer.pop_front())
    }
}

#[tonic::async_trait]
pub trait Page: sealed::Sealed {
    type Content;
    /// Fetches the next page of results
    async fn next(client: &Client, page_token: &str)
        -> Result<(Vec<Self::Content>, String), Error>;
}

impl<T> sealed::Sealed for T {}

mod sealed {
    pub trait Sealed {}
}

pub struct CachedContentPager;

#[tonic::async_trait]
impl Page for CachedContentPager {
    type Content = CachedContent;

    async fn next(
        client: &Client,
        page_token: &str,
    ) -> Result<(Vec<Self::Content>, String), Error> {
        let mut request = ListCachedContentsRequest {
            page_size: DEFAULT_PAGE_SIZE,
            page_token: page_token.to_owned(),
        }
        .into_request();

        client.add_auth(&mut request).await?;

        let response = client
            .cc
            .clone()
            .list_cached_contents(request)
            .await
            .map_err(status_into_error)?
            .into_inner();
        Ok((response.cached_contents, response.next_page_token))
    }
}

pub struct ModelsListPager;

#[tonic::async_trait]
impl Page for ModelsListPager {
    type Content = Model;

    async fn next(
        client: &Client,
        page_token: &str,
    ) -> Result<(Vec<Self::Content>, String), Error> {
        let mut request = ListModelsRequest {
            page_size: DEFAULT_PAGE_SIZE,
            page_token: page_token.to_owned(),
        }
        .into_request();

        client.add_auth(&mut request).await?;

        let response = client
            .mc
            .clone()
            .list_models(request)
            .await
            .map_err(status_into_error)?
            .into_inner();
        Ok((response.models, response.next_page_token))
    }
}

pub struct TunedModelsListPager;

#[tonic::async_trait]
impl Page for TunedModelsListPager {
    type Content = TunedModel;

    async fn next(
        client: &Client,
        page_token: &str,
    ) -> Result<(Vec<Self::Content>, String), Error> {
        let mut request = ListTunedModelsRequest {
            page_size: DEFAULT_PAGE_SIZE,
            page_token: page_token.to_owned(),
            filter: String::new(),
        }
        .into_request();

        client.add_auth(&mut request).await?;

        let response = client
            .mc
            .clone()
            .list_tuned_models(request)
            .await
            .map_err(status_into_error)?
            .into_inner();
        Ok((response.tuned_models, response.next_page_token))
    }
}

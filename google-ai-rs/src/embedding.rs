use std::borrow::Cow;

use tonic::IntoRequest;

use crate::{
    client::CClient,
    content::{IntoContent, TryIntoContent},
    error::status_into_error,
    full_model_name,
    proto::{BatchEmbedContentsResponse, Content, EmbedContentResponse, Model as Info, TaskType},
};

use super::{
    client::Client,
    error::{Error, ServiceError},
    proto::{BatchEmbedContentsRequest, EmbedContentRequest},
};

/// A client for generating embeddings using Google's embedding service
///
/// Provides both single and batch embedding capabilities with configurable task types.
///
/// # Example
/// ```
/// use google_ai_rs::{Client, GenerativeModel};
///
/// # async fn f() -> Result<(), Box<dyn std::error::Error>> {
/// # let auth = "YOUR-API-KEY";
/// let client = Client::new(auth).await?;
/// let embedding_model = client.embedding_model("embedding-001");
///
/// // Single embedding
/// let embedding = embedding_model.embed_content("Hello world").await?;
///
/// // Batch embeddings
/// let batch_response = embedding_model.new_batch()
///     .add_content("First text")
///     .add_content("Second text")
///     .embed()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Model<'c> {
    /// Client for making API requests
    client: CClient<'c>,
    /// Fully qualified model name (e.g., "models/embedding-001")
    name: Box<str>,
    /// Optional task type specification for embedding generation
    ///
    /// Affects how embeddings are optimized:
    /// - `None`: General purpose embeddings
    /// - `TaskType::RetrievalDocument`: Optimized for document storage
    /// - `TaskType::RetrievalQuery`: Optimized for query matching
    pub task_type: Option<TaskType>,
}

impl<'c> Model<'c> {
    /// Creates a new Model instance
    ///
    /// # Arguments
    /// * `client` - Configured API client
    /// * `name` - Model identifier (e.g., "embedding-001")
    pub fn new(client: &'c Client, name: &str) -> Self {
        Self::new_inner(client, name)
    }

    fn new_inner(client: impl Into<CClient<'c>>, name: &str) -> Self {
        Self {
            client: client.into(),
            name: full_model_name(name).into(),
            task_type: None,
        }
    }

    /// Optional task type specification for embedding generation
    ///
    /// Affects how embeddings are optimized:
    /// - `TaskType::RetrievalDocument`: Optimized for document storage
    /// - `TaskType::RetrievalQuery`: Optimized for query matching
    pub fn task_type(mut self, task_type: TaskType) -> Self {
        self.task_type = Some(task_type);
        self
    }

    /// Embeds content using the API's embedding service.
    ///
    /// Consider batch embedding for multiple contents
    ///
    /// # Example
    /// ```
    /// # use google_ai_rs::{Client, GenerativeModel};
    /// # use google_ai_rs::Part;
    /// # async fn f() -> Result<(), Box<dyn std::error::Error>> {
    /// # let auth = "YOUR-API-KEY";
    /// # let client = Client::new(auth).await?;
    /// # let model = client.embedding_model("embedding-001");
    /// // Single text embedding
    /// let embedding = model.embed_content("Hello world").await?;
    ///
    /// # let image_data = vec![];
    /// // Multi-modal embedding
    /// model.embed_content((
    ///     "Query about this image",
    ///     Part::blob("image/jpeg", image_data)
    /// )).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns `Error::Net` for transport-level errors or `Error::Service` for service errors
    #[inline]
    pub async fn embed_content<T: TryIntoContent>(
        &self,
        content: T,
    ) -> Result<EmbedContentResponse, Error> {
        self.embed_content_with_title("", content).await
    }

    /// Embeds content with optional title context
    ///
    /// # Arguments
    /// * `title` - Optional document title for retrieval tasks
    /// * `parts` - Content input that converts to parts
    pub async fn embed_content_with_title<T>(
        &self,
        title: &str,
        content: T,
    ) -> Result<EmbedContentResponse, Error>
    where
        T: TryIntoContent,
    {
        let request = self
            .build_request(title, content.try_into_content()?)
            .await?;
        self.client
            .gc
            .clone()
            .embed_content(request)
            .await
            .map_err(status_into_error)
            .map(|response| response.into_inner())
    }

    /// Creates a new batch embedding context
    pub fn new_batch(&self) -> Batch<'_> {
        Batch {
            m: self,
            req: BatchEmbedContentsRequest {
                model: self.name.to_string(),
                requests: Vec::new(),
            },
        }
    }

    /// Embeds multiple contents as separate content items
    ///
    /// # Example
    /// ```
    /// # use google_ai_rs::{Client, GenerativeModel};
    /// # use google_ai_rs::Part;
    /// # async fn f() -> Result<(), Box<dyn std::error::Error>> {
    /// # let auth = "YOUR-API-KEY";
    /// # let client = Client::new(auth).await?;
    /// # let model = client.embedding_model("embedding-001");
    /// let texts = vec!["First", "Second", "Third"];
    /// let batch = model.embed_batch(texts).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn embed_batch<I, T>(&self, contents: I) -> Result<BatchEmbedContentsResponse, Error>
    where
        I: IntoIterator<Item = T>,
        T: TryIntoContent,
    {
        let mut batch = self.new_batch();
        for content in contents.into_iter() {
            batch = batch.add_content(content.try_into_content()?);
        }
        batch.embed().await
    }

    /// returns information about the model.
    pub async fn info(&self) -> Result<Info, Error> {
        self.client.get_model(&self.name).await
    }

    #[inline(always)]
    async fn build_request(
        &self,
        title: &str,
        content: Content,
    ) -> Result<tonic::Request<EmbedContentRequest>, Error> {
        let request = self._build_request(title, content).into_request();
        Ok(request)
    }

    fn _build_request(&self, title: &str, content: Content) -> EmbedContentRequest {
        let title = if title.is_empty() {
            None
        } else {
            Some(title.to_owned())
        };

        // A non-empty title overrides the task type.
        let task_type = title
            .as_ref()
            .map(|_| TaskType::RetrievalDocument.into())
            .or(self.task_type.map(Into::into));

        EmbedContentRequest {
            model: self.name.to_string(),
            content: Some(content),
            task_type,
            title,
            output_dimensionality: None,
        }
    }
}

/// Builder for batch embedding requests
///
/// Collects multiple embedding requests for efficient batch processing.
///
/// # Example
/// ```
/// # use google_ai_rs::{Client, GenerativeModel};
/// # async fn f() -> Result<(), Box<dyn std::error::Error>> {
/// # let auth = "YOUR-API-KEY";
/// # let client = Client::new(auth).await?;
/// # let embedding_model = client.embedding_model("embedding-001");
/// let batch = embedding_model.new_batch()
///     .add_content_with_title("Document 1", "Full text content...")
///     .add_content_with_title("Document 2", "Another text...");
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Batch<'m> {
    m: &'m Model<'m>,
    req: BatchEmbedContentsRequest,
}

impl Batch<'_> {
    /// Adds content to the batch
    #[inline]
    pub fn add_content<T: IntoContent>(self, content: T) -> Self {
        self.add_content_with_title("", content)
    }

    /// Adds content with title to the batch
    ///
    /// # Argument
    /// * `title` - Document title for retrieval context
    pub fn add_content_with_title<T: IntoContent>(mut self, title: &str, content: T) -> Self {
        self.req
            .requests
            .push(self.m._build_request(title, content.into_content()));
        self
    }

    /// Executes the batch embedding request
    pub async fn embed(self) -> Result<BatchEmbedContentsResponse, Error> {
        let expected = self.req.requests.len();
        let request = self.req.into_request();

        let response = self
            .m
            .client
            .gc
            .clone()
            .batch_embed_contents(request)
            .await
            .map_err(status_into_error)
            .map(|response| response.into_inner())?;

        if response.embeddings.len() != expected {
            return Err(Error::Service(ServiceError::InvalidResponse(
                format!(
                    "Expected {} embeddings, got {}",
                    expected,
                    response.embeddings.len()
                )
                .into(),
            )));
        }

        Ok(response)
    }
}

impl Client {
    /// Creates a new embedding model interface
    ///
    /// Shorthand for `EmbeddingModel::new()`
    pub fn embedding_model<'c>(&'c self, name: &str) -> Model<'c> {
        Model::new(self, name)
    }
}

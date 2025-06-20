use std::{
    fmt::Debug,
    io::Write,
    ops::{Deref, DerefMut},
};
use tonic::{IntoRequest, Streaming};

use crate::{
    client::Client,
    content::{IntoContent, TryFromCandidates, TryIntoContents},
    error::{status_into_error, ActionError, Error},
    full_model_name,
    proto::{
        CachedContent, Content as ContentP, CountTokensRequest, CountTokensResponse,
        GenerateContentRequest, GenerateContentResponse, GenerationConfig as GenerationConfigP,
        Model, SafetySetting as SafetySettingP, Schema, Tool as ToolP, ToolConfig as ToolConfigP,
        TunedModel,
    },
    schema::AsSchema,
};

/// Type-safe wrapper for [`GenerativeModel`] guaranteeing response type `T`.
///
/// This type enforces schema contracts through Rust's type system while maintaining
/// compatibility with Google's Generative AI API. Use when:
/// - You need structured output from the model
/// - Response schema stability is critical
/// - You want compile-time validation of response handling
///
/// # Example
/// ```
/// use google_ai_rs::{Client, GenerativeModel, AsSchema};
/// # async fn f() -> Result<(), Box<dyn std::error::Error>> {
/// # let auth = "YOUR-API-KEY".into();
/// # use std::collections::HashMap;
///
/// #[derive(AsSchema)]
/// struct Recipe {
///     name: String,
///     ingredients: Vec<String>,
/// }
///
/// let client = Client::new(auth).await?;
/// let model = client.typed_model::<Recipe>("gemini-pro");
/// # Ok(())
/// # }
#[derive(Debug)]
pub struct TypedModel<'c, T> {
    inner: GenerativeModel<'c>,
    _marker: std::marker::PhantomData<T>,
}

impl<'c, T> TypedModel<'c, T>
where
    T: AsSchema,
{
    /// Creates a new typed model with schema validation.
    ///
    /// # Arguments
    /// - `client`: Authenticated API client
    /// - `name`: Model name (e.g., "gemini-pro")
    pub fn new(client: &'c Client, name: &str) -> Self {
        let inner = GenerativeModel::new(client, name).as_response_schema::<T>();
        Self {
            inner,
            _marker: std::marker::PhantomData,
        }
    }

    /// Generates content with full response metadata.
    ///
    /// Returns both parsed content and raw API response.
    ///
    /// # Example
    /// ```
    /// # use google_ai_rs::{AsSchema, Client, TypedModel, TypedResponse};
    /// # #[derive(AsSchema, serde::Deserialize, Debug)] struct StockAnalysis;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("api-key".into()).await?;
    /// let model = TypedModel::<StockAnalysis>::new(&client, "gemini-pro");
    /// let analysis: TypedResponse<StockAnalysis> = model.generate_typed_content((
    ///     "Analyze NVDA stock performance",
    ///     "Consider PE ratio and recent earnings"
    /// )).await?;
    /// println!("Analysis: {:?}", analysis);
    /// # Ok(()) }
    /// ```
    pub async fn generate_typed_content<I>(&self, contents: I) -> Result<TypedResponse<T>, Error>
    where
        I: TryIntoContents,
        T: TryFromCandidates,
    {
        let response = self.inner.generate_content(contents).await?;
        let t = T::try_from_candidates(&response.candidates)?;
        Ok(TypedResponse { t, raw: response })
    }

    /// Generates content and parses it directly into type `T`.
    ///
    /// This is the primary method for most users wanting type-safe responses without
    /// dealing with raw API structures. For 90% of use cases where you just want
    /// structured data from the AI, this is what you need.
    ///
    /// # Serde Integration
    /// When the `serde` feature is enabled, any type implementing `serde::Deserialize`
    /// automatically works with this method. Just define your response structure and
    /// let the library handle parsing.
    ///
    /// # Example: Simple JSON Response
    /// ```
    /// # use google_ai_rs::{AsSchema, Client, TypedModel};
    /// # use serde::Deserialize;
    /// #[derive(AsSchema, Deserialize)]
    /// struct StoryResponse {
    ///     title: String,
    ///     length: usize,
    ///     tags: Vec<String>,
    /// }
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("key".into()).await?;
    /// let model = TypedModel::<StoryResponse>::new(&client, "gemini-pro");
    /// let story = model.generate_content("Write a short story about a robot astronaut").await?;
    ///
    /// println!("{} ({} words)", story.title, story.length);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Example: Multi-part Input
    /// ```
    /// # use google_ai_rs::{AsSchema, Client, TypedModel, Part};
    /// # use serde::Deserialize;
    /// #[derive(AsSchema, Deserialize)]
    /// struct Analysis { safety_rating: u8 }
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("key".into()).await?;
    /// # let image_data = vec![];
    /// let model = TypedModel::<Analysis>::new(&client, "gemini-pro-vision");
    /// let analysis = model.generate_content((
    ///     "Analyze this scene safety:",
    ///     Part::blob("image/jpeg", image_data),
    ///     "Consider vehicles, pedestrians, and weather"
    /// )).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// - [`Error::InvalidArgument`] if input validation fails
    /// - [`Error::Service`] for model errors
    /// - [`Error::Net`] for network failures
    pub async fn generate_content<I>(&self, contents: I) -> Result<T, Error>
    where
        I: TryIntoContents,
        T: TryFromCandidates,
    {
        let response = self.inner.generate_content(contents).await?;
        let t = T::try_from_candidates(&response.candidates)?;
        Ok(t)
    }
}

impl<'c, T> Deref for TypedModel<'c, T> {
    type Target = GenerativeModel<'c>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'c, T> From<GenerativeModel<'c>> for TypedModel<'c, T>
where
    T: AsSchema,
{
    fn from(value: GenerativeModel<'c>) -> Self {
        let inner = value.as_response_schema::<T>();
        TypedModel {
            inner,
            _marker: std::marker::PhantomData,
        }
    }
}

/// Container for typed responses with raw API data.
///
/// Preserves full response details while providing parsed content.
pub struct TypedResponse<T> {
    /// Parsed content of type `T`    
    pub t: T,
    /// Raw API response structure    
    pub raw: GenerateContentResponse,
}

impl<T> Debug for TypedResponse<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.t.fmt(f)
    }
}

impl<T> Deref for TypedResponse<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.t
    }
}

impl<T> DerefMut for TypedResponse<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.t
    }
}

/// Type aliases for protocol buffer types to simplify API surface
type Content = ContentP;
type Tool = ToolP;
type ToolConfig = ToolConfigP;
type SafetySetting = SafetySettingP;
type GenerationConfig = GenerationConfigP;

/// Configured interface for a specific generative AI model
///
/// # Example
/// ```
/// use google_ai_rs::{Client, GenerativeModel};
///
/// # async fn f() -> Result<(), Box<dyn std::error::Error>> {
/// # let auth = "YOUR-API-KEY".into();
/// let client = Client::new(auth).await?;
/// let model = client.generative_model("gemini-pro")
///     .with_system_instruction("You are a helpful assistant")
///     .with_response_format("application/json");
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct GenerativeModel<'c> {
    /// Backing API client
    pub(super) client: &'c Client,
    /// Fully qualified model name (e.g., "models/gemini-1.0-pro")
    model_name: String,
    /// System prompt guiding model behavior
    pub system_instruction: Option<Content>,
    /// Available functions/tools the model can use
    pub tools: Option<Vec<Tool>>,
    /// Configuration for tool usage
    pub tool_config: Option<ToolConfig>,
    /// Content safety filters
    pub safety_settings: Option<Vec<SafetySetting>>,
    /// Generation parameters (temperature, top-k, etc.)
    pub generation_config: Option<GenerationConfig>,
    /// Fullname of the cached content to use as context
    /// (e.g., "cachedContents/NAME")
    pub cached_content: Option<String>,
}

impl<'c> GenerativeModel<'c> {
    /// Creates a new model interface with default configuration
    ///
    /// # Arguments
    /// * `client` - Authenticated API client
    /// * `name` - Model identifier (e.g., "gemini-pro")
    ///
    /// To access a tuned model named NAME, pass "tunedModels/NAME".
    pub fn new(client: &'c Client, name: &str) -> Self {
        Self {
            client,
            model_name: full_model_name(name),
            system_instruction: None,
            tools: None,
            tool_config: None,
            safety_settings: None,
            generation_config: None,
            cached_content: None,
        }
    }

    pub fn to_typed<T: AsSchema>(self) -> TypedModel<'c, T> {
        self.into()
    }

    /// Generates content from flexible input types
    ///
    /// # Example
    /// ```
    /// # use google_ai_rs::{Client, GenerativeModel};
    /// use google_ai_rs::Part;
    ///
    /// # async fn f() -> Result<(), Box<dyn std::error::Error>> {
    /// # let auth = "YOUR-API-KEY".into();
    /// # let client = Client::new(auth).await?;
    /// # let model = client.generative_model("gemini-pro");
    /// // Simple text generation
    /// let response = model.generate_content("Hello world!").await?;
    ///
    /// // Multi-part content
    /// # let image_data = vec![];
    /// model.generate_content((
    ///     "What's in this image?",
    ///     Part::blob("image/jpeg", image_data)
    /// )).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns [`Error::InvalidArgument`] for empty input or invalid combinations.
    /// [`Error::Service`] for model errors or [`Error::Net`] for transport failures
    pub async fn generate_content<T>(&self, contents: T) -> Result<GenerateContentResponse, Error>
    where
        T: TryIntoContents,
    {
        let contents = contents.try_into_contents()?;
        if contents.is_empty() {
            return Err(Error::InvalidArgument("Empty contents".into()));
        }

        let request = self.build_request(contents).await?;

        self.client
            .gc
            .clone()
            .generate_content(request)
            .await
            .map_err(status_into_error)
            .map(|r| r.into_inner())
    }

    pub async fn typed_generate_content<I, T>(&self, contents: I) -> Result<T, Error>
    where
        I: TryIntoContents,
        T: AsSchema + TryFromCandidates,
    {
        self.clone().to_typed().generate_content(contents).await
    }

    pub async fn generate_typed_content<I, T>(&self, contents: I) -> Result<TypedResponse<T>, Error>
    where
        I: TryIntoContents,
        T: AsSchema + TryFromCandidates,
    {
        self.clone()
            .to_typed()
            .generate_typed_content(contents)
            .await
    }

    /// Generates a streaming response from flexible input
    ///
    /// # Example
    /// ```
    /// # use google_ai_rs::{Client, GenerativeModel};
    /// # async fn f() -> Result<(), Box<dyn std::error::Error>> {
    /// # let auth = "YOUR-API-KEY".into();
    /// # let client = Client::new(auth).await?;
    /// # let model = client.generative_model("gemini-pro");
    /// let mut stream = model.stream_generate_content("Tell me a story.").await?;
    /// while let Some(chunk) = stream.next().await? {
    ///     // Process streaming response
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns [`Error::Service`] for model errors or [`Error::Net`] for transport failures
    pub async fn stream_generate_content<T>(&self, contents: T) -> Result<ResponseStream, Error>
    where
        T: TryIntoContents,
    {
        let contents = contents.try_into_contents()?;
        if contents.is_empty() {
            return Err(Error::InvalidArgument("Empty content".into()));
        }
        let request = self.build_request(contents).await?;

        self.client
            .gc
            .clone()
            .stream_generate_content(request)
            .await
            .map_err(status_into_error)
            .map(|s| ResponseStream(s.into_inner()))
    }

    /// Estimates token usage for given content
    ///
    /// Useful for cost estimation and validation before full generation
    ///
    /// # Arguments
    /// * `parts` - Content input that can be converted to parts
    ///
    /// # Example
    /// ```
    /// # use google_ai_rs::{Client, GenerativeModel};
    /// # async fn f() -> Result<(), Box<dyn std::error::Error>> {
    /// # let auth = "YOUR-API-KEY".into();
    /// # let client = Client::new(auth).await?;
    /// # let model = client.generative_model("gemini-pro");
    /// # let content = "";
    /// let token_count = model.count_tokens(content).await?;
    /// # const COST_PER_TOKEN: f64 = 1.0;
    /// println!("Estimated cost: ${}", token_count.total() * COST_PER_TOKEN);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns [`Error::InvalidArgument`] for empty input
    pub async fn count_tokens<T>(&self, contents: T) -> Result<CountTokensResponse, Error>
    where
        T: TryIntoContents,
    {
        let contents = contents.try_into_contents()?;
        if contents.is_empty() {
            return Err(Error::InvalidArgument("Empty content".into()));
        }

        let request = self.build_count_request(contents).await?;

        self.client
            .gc
            .clone()
            .count_tokens(request)
            .await
            .map_err(status_into_error)
            .map(|r| r.into_inner())
    }

    pub fn change_model(&mut self, to: &str) {
        self.model_name = full_model_name(to)
    }

    pub fn full_name(&self) -> &str {
        &self.model_name
    }
    /// info returns information about the model.
    ///
    /// `Info::Tuned` if the current model is a fine-tuned one,
    /// otherwise `Info::Model`.
    pub async fn info(&self) -> Result<Info, Error> {
        if self.model_name.starts_with("tunedModels") {
            Ok(Info::Tuned(
                self.client.get_tuned_model(&self.model_name).await?,
            ))
        } else {
            Ok(Info::Model(self.client.get_model(&self.model_name).await?))
        }
    }

    // Builder pattern methods
    // -----------------------------------------------------------------

    /// Sets system-level behavior instructions
    pub fn with_system_instruction<I: IntoContent>(mut self, instruction: I) -> Self {
        self.system_instruction = Some(instruction.into_content());
        self
    }

    pub fn to_model(mut self, to: &str) -> Self {
        self.change_model(to);
        self
    }

    /// Sets cached content for persisted context
    ///
    /// # Example
    /// ```
    /// # use google_ai_rs::{Client, GenerativeModel};
    /// use google_ai_rs::content::IntoContents as _;
    ///
    /// # async fn f() -> Result<(), Box<dyn std::error::Error>> {
    /// # let auth = "YOUR-API-KEY".into();
    /// # let client = Client::new(auth).await?;
    /// let content = "You are a helpful assistant".into_cached_content_for("gemini-1.0-pro");
    ///
    /// let cached_content = client.create_cached_content(content).await?;
    /// let model = client.generative_model("gemini-pro")
    ///             .with_cached_content(&cached_content);
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_cached_content(mut self, c: &CachedContent) -> Result<Self, Error> {
        self.cached_content = Some(
            c.name
                .as_ref()
                .ok_or(Error::InvalidArgument(
                    "cached content name is empty".into(),
                ))?
                .into(),
        );
        Ok(self)
    }

    /// Specifies expected response format (e.g., "application/json")
    pub fn with_response_format(mut self, mime_type: &str) -> Self {
        self.generation_config
            .get_or_insert_with(Default::default)
            .response_mime_type = mime_type.into();
        self
    }

    /// Set response schema with explicit Schema object using types implementing `AsSchema`
    ///
    /// Similar to `with_response_schema`.
    ///
    /// # Example
    ///
    ///
    /// ```rust
    /// use google_ai_rs::AsSchema;
    ///
    /// #[derive(Debug, AsSchema)]
    /// #[schema(description = "A primary colour")]
    /// struct PrimaryColor {
    ///     #[schema(description = "The name of the colour")]
    ///     name: String,
    ///     #[schema(description = "The RGB value of the color, in hex", rename = "RGB")]
    ///     rgb: String
    /// }
    ///
    /// # use google_ai_rs::{Client, GenerativeModel};
    /// # async fn f() -> Result<(), Box<dyn std::error::Error>> {
    /// # let auth = "YOUR-API-KEY".into();
    /// # let client = Client::new(auth).await?;
    /// let model = client.generative_model("gemini-pro")
    ///         .as_response_schema::<Vec<PrimaryColor>>();
    /// # Ok(())
    /// # }
    /// ```
    pub fn as_response_schema<T: AsSchema>(self) -> Self {
        self.with_response_schema(T::as_schema())
    }

    /// Set response schema with explicit Schema object
    ///
    /// Use when you need full control over schema details. Automatically
    /// sets response format to JSON if not specified.
    ///
    /// # Example
    ///
    /// ```rust
    /// use google_ai_rs::Schema;
    /// use google_ai_rs::SchemaType;
    ///
    /// # use google_ai_rs::{Client, GenerativeModel};
    /// # async fn f() -> Result<(), Box<dyn std::error::Error>> {
    /// # let auth = "YOUR-API-KEY".into();
    /// # let client = Client::new(auth).await?;
    /// let model = client.generative_model("gemini-pro")
    ///      .with_response_schema(Schema {
    ///      r#type: SchemaType::String as i32,
    ///      format: "enum".into(),
    ///      ..Default::default()
    /// });
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_response_schema(mut self, schema: Schema) -> Self {
        let c = self.generation_config.get_or_insert_with(Default::default);
        if c.response_mime_type.is_empty() {
            c.response_mime_type = "application/json".into();
        }
        c.response_schema = Some(schema);
        self
    }

    /// Creates a copy with new system instructions
    pub fn with_cloned_instruction<I: IntoContent>(&self, instruction: I) -> Self {
        let mut clone = self.clone();

        clone.system_instruction = Some(instruction.into_content());
        clone
    }

    pub fn set_candidate_count(&mut self, x: i32) {
        self.generation_config
            .get_or_insert_default()
            .candidate_count = Some(x)
    }

    pub fn set_max_output_tokens(&mut self, x: i32) {
        self.generation_config
            .get_or_insert_default()
            .max_output_tokens = Some(x)
    }

    pub fn set_temperature(&mut self, x: f32) {
        self.generation_config.get_or_insert_default().temperature = Some(x)
    }

    pub fn set_top_p(&mut self, x: f32) {
        self.generation_config.get_or_insert_default().top_p = Some(x)
    }

    pub fn set_top_k(&mut self, x: i32) {
        self.generation_config.get_or_insert_default().top_k = Some(x)
    }

    /// Builds authenticated generation request
    async fn build_request(
        &self,
        contents: Vec<Content>,
    ) -> Result<tonic::Request<GenerateContentRequest>, Error> {
        let mut request = self._build_request(contents).into_request();
        self.client.add_auth(&mut request).await?;
        Ok(request)
    }

    fn _build_request(&self, contents: Vec<Content>) -> GenerateContentRequest {
        GenerateContentRequest {
            model: self.model_name.clone(),
            contents,
            system_instruction: self.system_instruction.clone(),
            tools: self.tools.clone().unwrap_or_default(),
            tool_config: self.tool_config.clone(),
            safety_settings: self.safety_settings.clone().unwrap_or_default(),
            generation_config: self.generation_config.clone(),
            cached_content: self.cached_content.clone().map(|c| c.to_string()),
        }
    }

    /// Builds token counting request
    async fn build_count_request(
        &self,
        contents: Vec<Content>,
    ) -> Result<tonic::Request<CountTokensRequest>, Error> {
        let mut request = CountTokensRequest {
            model: self.model_name.clone(),
            contents: vec![],
            generate_content_request: Some(self._build_request(contents)),
        }
        .into_request();
        self.client.add_auth(&mut request).await?;
        Ok(request)
    }
}

/// Generation response containing model output and metadata
pub type Response = GenerateContentResponse;

impl Response {
    /// Total tokens used in request/response cycle
    pub fn total_tokens(&self) -> f64 {
        self.usage_metadata.as_ref().map_or(0.0, |meta| {
            meta.total_token_count as f64 + meta.cached_content_token_count as f64
        })
    }
}

/// Streaming response handler implementing async iteration
pub struct ResponseStream(Streaming<GenerateContentResponse>);

impl ResponseStream {
    /// Streams content chunks to any `Write` implementer
    ///
    /// # Arguments
    /// * `writer` - Target for streaming output
    ///
    /// # Returns
    /// Total bytes written
    pub async fn write_to<W: Write>(&mut self, writer: &mut W) -> Result<usize, Error> {
        let mut total = 0;

        while let Some(response) = self
            .next()
            .await
            .map_err(|e| Error::Stream(ActionError::Error(e.into())))?
        {
            let bytes = response.try_into_bytes()?;
            let written = writer
                .write(&bytes)
                .map_err(|e| Error::Stream(ActionError::Action(e)))?;
            total += written;
        }

        Ok(total)
    }

    /// Fetches next response chunk
    pub async fn next(&mut self) -> Result<Option<GenerateContentResponse>, Error> {
        self.0.message().await.map_err(status_into_error)
    }
}

impl Client {
    /// Creates a new generative model interface
    ///
    /// Shorthand for `GenerativeModel::new()`
    pub fn generative_model<'c>(&'c self, name: &str) -> GenerativeModel<'c> {
        GenerativeModel::new(self, name)
    }

    /// Creates a new typed generative model interface
    ///
    /// Shorthand for `TypedModel::new()`
    pub fn typed_model<'c, T: AsSchema>(&'c self, name: &str) -> TypedModel<'c, T> {
        TypedModel::<T>::new(self, name)
    }
}

impl CountTokensResponse {
    pub fn total(&self) -> f64 {
        self.total_tokens as f64 + self.cached_content_token_count as f64
    }
}

#[derive(Debug)]
pub enum Info {
    Tuned(TunedModel),
    Model(Model),
}

use std::{
    fmt::Debug,
    io::Write,
    ops::{Deref, DerefMut},
};
use tokio::io::AsyncWrite;
use tonic::{IntoRequest, Streaming};

use crate::{
    client::{CClient, Client, SharedClient},
    content::{IntoContent, TryFromCandidates, TryIntoContents},
    error::{status_into_error, ActionError, Error},
    full_model_name,
    schema::AsSchema,
};

pub use crate::proto::{
    safety_setting::HarmBlockThreshold, CachedContent, Content, CountTokensRequest,
    CountTokensResponse, GenerateContentRequest, GenerateContentResponse, GenerationConfig,
    HarmCategory, Model, SafetySetting, Schema, Tool, ToolConfig, TunedModel,
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
/// # let auth = "YOUR-API-KEY";
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
pub struct TypedModel<'c, T> {
    inner: GenerativeModel<'c>,
    _marker: PhantomInvariant<T>,
}

impl<T> Debug for TypedModel<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T> Clone for TypedModel<'_, T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _marker: PhantomInvariant(std::marker::PhantomData),
        }
    }
}

// std is unstable
struct PhantomInvariant<T>(std::marker::PhantomData<fn(T) -> T>);

impl<'c, T> TypedModel<'c, T>
where
    T: AsSchema,
{
    /// Creates a new typed model configured to return responses of type `T`.
    ///
    /// # Arguments
    /// - `client`: Authenticated API client.
    /// - `name`: Model name (e.g., "gemini-pro").
    pub fn new(client: &'c Client, name: &str) -> Self {
        let inner = GenerativeModel::new(client, name).as_response_schema::<T>();
        Self {
            inner,
            _marker: PhantomInvariant(std::marker::PhantomData),
        }
    }

    fn new_inner(client: impl Into<CClient<'c>>, name: &str) -> Self {
        let inner = GenerativeModel::new_inner(client, name).as_response_schema::<T>();
        Self {
            inner,
            _marker: PhantomInvariant(std::marker::PhantomData),
        }
    }

    /// Generates content with full response metadata.
    ///
    /// This method clones the model configuration and returns a `TypedResponse`,
    /// containing both the parsed `T` and the raw API response.
    ///
    /// # Example
    /// ```rust,ignore
    /// # use google_ai_rs::{AsSchema, Client, TypedModel, TypedResponse};
    /// # #[derive(AsSchema, serde::Deserialize, Debug)] struct StockAnalysis;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("api-key").await?;
    /// let model = TypedModel::<StockAnalysis>::new(&client, "gemini-pro");
    /// let analysis: TypedResponse<StockAnalysis> = model.generate_typed_content((
    ///     "Analyze NVDA stock performance",
    ///     "Consider PE ratio and recent earnings"
    /// )).await?;
    /// println!("Analysis: {:?}", analysis.t);
    /// println!("Token Usage: {:?}", analysis.raw.usage_metadata);
    /// # Ok(()) }
    /// ```
    #[inline]
    pub async fn generate_typed_content<I>(&self, contents: I) -> Result<TypedResponse<T>, Error>
    where
        I: TryIntoContents + Send,
        T: TryFromCandidates + Send,
    {
        self.cloned()
            .generate_typed_content_consuming(contents)
            .await
    }

    /// Generates content with metadata, consuming the model instance.
    ///
    /// An efficient alternative to `generate_typed_content` that avoids cloning
    /// the model configuration, useful for one-shot requests.
    #[inline]
    pub async fn generate_typed_content_consuming<I>(
        self,
        contents: I,
    ) -> Result<TypedResponse<T>, Error>
    where
        I: TryIntoContents + Send,
        T: TryFromCandidates + Send,
    {
        let response = self.inner.generate_content_consuming(contents).await?;
        let t = T::try_from_candidates(&response.candidates)?;
        Ok(TypedResponse { t, raw: response })
    }

    /// Generates content and parses it directly into type `T`.
    ///
    /// This is the primary method for most users wanting type-safe responses.
    /// It handles all the details of requesting structured JSON and deserializing
    /// it into your specified Rust type. It clones the model configuration to allow reuse.
    ///
    /// # Serde Integration
    /// When the `serde` feature is enabled, any type implementing `serde::Deserialize`
    /// automatically works with this method. Just define your response structure and
    /// let the library handle parsing.
    ///
    /// # Example: Simple JSON Response
    /// ```rust,ignore
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
    /// ```rust,ignore
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
    /// Returns an error if API communication fails or if the response cannot be
    /// parsed into type `T`.
    #[inline]
    pub async fn generate_content<I>(&self, contents: I) -> Result<T, Error>
    where
        I: TryIntoContents + Send,
        T: TryFromCandidates + Send,
    {
        self.cloned().generate_content_consuming(contents).await
    }

    #[inline]
    pub async fn generate_content_consuming<I>(self, contents: I) -> Result<T, Error>
    where
        I: TryIntoContents + Send,
        T: TryFromCandidates + Send,
    {
        let response = self.inner.generate_content_consuming(contents).await?;
        let t = T::try_from_candidates(&response.candidates)?;
        Ok(t)
    }

    /// Consumes the `TypedModel`, returning the underlying `GenerativeModel`.
    ///
    /// The returned `GenerativeModel` will retain the response schema configuration
    /// that was set for type `T`.
    pub fn into_inner(self) -> GenerativeModel<'c> {
        self.inner
    }

    /// Creates a `TypedModel` from a `GenerativeModel` without validation.
    ///
    /// This is an advanced-use method that assumes the provided `GenerativeModel`
    /// has already been configured with a response schema that is compatible with `T`.
    ///
    /// # Safety
    /// The caller must ensure that `inner.generation_config.response_schema` is `Some`
    /// and that its schema corresponds exactly to the schema of type `T`. Failure to
    /// uphold this invariant will likely result in API errors or deserialization failures.
    pub unsafe fn from_inner_unchecked(inner: GenerativeModel<'c>) -> Self {
        Self {
            inner,
            _marker: PhantomInvariant(std::marker::PhantomData),
        }
    }

    fn cloned(&self) -> TypedModel<'_, T> {
        TypedModel {
            inner: self.inner.cloned(),
            _marker: PhantomInvariant(std::marker::PhantomData),
        }
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
            _marker: PhantomInvariant(std::marker::PhantomData),
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

/// Configured interface for a specific generative AI model
///
/// # Example
/// ```
/// use google_ai_rs::{Client, GenerativeModel};
///
/// # async fn f() -> Result<(), Box<dyn std::error::Error>> {
/// # let auth = "YOUR-API-KEY";
/// let client = Client::new(auth).await?;
/// let model = client.generative_model("gemini-pro")
///     .with_system_instruction("You are a helpful assistant")
///     .with_response_format("application/json");
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct GenerativeModel<'c> {
    /// Backing API clienty
    pub(super) client: CClient<'c>,
    /// Fully qualified model name (e.g., "models/gemini-1.0-pro")
    model_name: Box<str>,
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
    pub cached_content: Option<Box<str>>,
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
        Self::new_inner(client, name)
    }

    fn new_inner(client: impl Into<CClient<'c>>, name: &str) -> Self {
        Self {
            client: client.into(),
            model_name: full_model_name(name).into(),
            system_instruction: None,
            tools: None,
            tool_config: None,
            safety_settings: None,
            generation_config: None,
            cached_content: None,
        }
    }

    /// Converts this `GenerativeModel` into a `TypedModel`.
    ///
    /// This prepares the model to return responses that are automatically
    /// parsed into the specified type `T`.
    pub fn to_typed<T: AsSchema>(self) -> TypedModel<'c, T> {
        self.into()
    }

    /// Generates content from flexible input types.
    ///
    /// This method clones the model's configuration for the request, allowing the original
    /// `GenerativeModel` instance to be reused.
    ///
    /// # Example
    /// ```
    /// # use google_ai_rs::{Client, GenerativeModel};
    /// use google_ai_rs::Part;
    ///
    /// # async fn f() -> Result<(), Box<dyn std::error::Error>> {
    /// # let auth = "YOUR-API-KEY";
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
    /// Returns [`Error::Service`] for model errors or [`Error::Net`] for transport failures.
    pub async fn generate_content<T>(&self, contents: T) -> Result<GenerateContentResponse, Error>
    where
        T: TryIntoContents,
    {
        self.cloned().generate_content_consuming(contents).await
    }

    /// Generates content by consuming the model instance.
    ///
    /// This is an efficient alternative to `generate_content` if you don't need to reuse the
    /// model instance, as it avoids cloning the model's configuration. This is useful
    /// for one-shot requests where the model is built, used, and then discarded.
    pub async fn generate_content_consuming<T>(
        self,
        contents: T,
    ) -> Result<GenerateContentResponse, Error>
    where
        T: TryIntoContents,
    {
        let mut gc = self.client.gc.clone();
        let request = self.build_request(contents)?;
        gc.generate_content(request)
            .await
            .map_err(status_into_error)
            .map(|r| r.into_inner())
    }

    /// A convenience method to generate a structured response of type `T`.
    ///
    /// This method internally converts the `GenerativeModel` to a `TypedModel<T>`,
    /// makes the request, and returns the parsed result directly. It clones the model
    /// configuration for the request.
    ///
    /// For repeated calls with the same target type, it may be more efficient to create a
    /// `TypedModel` instance directly.
    pub async fn typed_generate_content<I, T>(&self, contents: I) -> Result<T, Error>
    where
        I: TryIntoContents + Send,
        T: AsSchema + TryFromCandidates + Send,
    {
        // Cloning occurs just this once with owned_generate_content
        self.cloned()
            .to_typed()
            .generate_content_consuming(contents)
            .await
    }

    /// A convenience method to generate a structured response with metadata.
    ///
    /// Similar to `typed_generate_content`, but returns a `TypedResponse<T>` which includes
    /// both the parsed data and the raw API response metadata.
    pub async fn generate_typed_content<I, T>(&self, contents: I) -> Result<TypedResponse<T>, Error>
    where
        I: TryIntoContents + Send,
        T: AsSchema + TryFromCandidates + Send,
    {
        self.cloned()
            .to_typed()
            .generate_typed_content_consuming(contents)
            .await
    }

    /// Generates a streaming response from flexible input.
    ///
    /// This method clones the model's configuration for the request, allowing the original
    /// `GenerativeModel` instance to be reused.
    ///
    /// # Example
    /// ```
    /// # use google_ai_rs::{Client, GenerativeModel};
    /// # async fn f() -> Result<(), Box<dyn std::error::Error>> {
    /// # let auth = "YOUR-API-KEY";
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
    /// Returns [`Error::Service`] for model errors or [`Error::Net`] for transport failures.
    pub async fn stream_generate_content<T>(&self, contents: T) -> Result<ResponseStream, Error>
    where
        T: TryIntoContents,
    {
        self.cloned()
            .stream_generate_content_consuming(contents)
            .await
    }

    /// Generates a streaming response by consuming the model instance.
    ///
    /// This is an efficient alternative to `stream_generate_content` if you don't need to
    /// reuse the model instance, as it avoids cloning the model's configuration.
    pub async fn stream_generate_content_consuming<T>(
        self,
        contents: T,
    ) -> Result<ResponseStream, Error>
    where
        T: TryIntoContents,
    {
        let mut gc = self.client.gc.clone();
        let request = self.build_request(contents)?;
        gc.stream_generate_content(request)
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
    /// # let auth = "YOUR-API-KEY";
    /// # let client = Client::new(auth).await?;
    /// # let model = client.generative_model("gemini-pro");
    /// # let content = "";
    /// let token_count = model.count_tokens(content).await?;
    /// # const COST_PER_TOKEN: f64 = 1.0;
    /// println!("Estimated cost: ${}", token_count.total() * COST_PER_TOKEN);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn count_tokens<T>(&self, contents: T) -> Result<CountTokensResponse, Error>
    where
        T: TryIntoContents,
    {
        let mut gc = self.client.gc.clone();

        // Builds token counting request
        let request = CountTokensRequest {
            model: self.model_name.to_string(),
            contents: vec![],
            generate_content_request: Some(self.clone().build_request(contents)?),
        };

        gc.count_tokens(request)
            .await
            .map_err(status_into_error)
            .map(|r| r.into_inner())
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

    /// Changes the model identifier for this instance in place.
    pub fn change_model(&mut self, to: &str) {
        self.model_name = full_model_name(to).into()
    }

    /// Returns the full identifier of the model, including any `models/` prefix.
    pub fn full_name(&self) -> &str {
        &self.model_name
    }

    // Builder pattern methods
    // -----------------------------------------------------------------

    /// Sets system-level behavior instructions
    pub fn with_system_instruction<I: IntoContent>(mut self, instruction: I) -> Self {
        self.system_instruction = Some(instruction.into_content());
        self
    }

    /// Changes the model identifier, returning the modified instance.
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
    /// # let auth = "YOUR-API-KEY";
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
                .as_deref()
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

    /// Configures the model to respond with a schema matching the type `T`.
    ///
    /// This is a convenient way to get structured JSON output.
    ///
    /// # Example
    /// ```rust
    /// use google_ai_rs::AsSchema;
    ///
    /// #[derive(Debug, AsSchema)]
    /// #[schema(description = "A primary colour")]
    /// struct PrimaryColor {
    ///     #[schema(description = "The name of the colour")]
    ///     name: String,
    ///
    ///     #[schema(description = "The RGB value of the color, in hex")]
    ///     #[schema(rename = "RGB")]
    ///     rgb: String
    /// }
    ///
    /// # use google_ai_rs::{Client, GenerativeModel};
    /// # async fn f() -> Result<(), Box<dyn std::error::Error>> {
    /// # let auth = "YOUR-API-KEY";
    /// # let client = Client::new(auth).await?;
    /// let model = client.generative_model("gemini-pro")
    ///     .as_response_schema::<Vec<PrimaryColor>>();
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
    /// # let auth = "YOUR-API-KEY";
    /// # let client = Client::new(auth).await?;
    /// let model = client.generative_model("gemini-pro")
    ///      .with_response_schema(Schema {
    ///         r#type: SchemaType::String as i32,
    ///         format: "enum".into(),
    ///         ..Default::default()
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

    /// Adds a collection of tools to the model.
    ///
    /// Tools define external functions that the model can call.
    ///
    /// # Arguments
    /// * `tools` - An iterator of `Tool` instances.
    pub fn tools<I>(mut self, tools: I) -> Self
    where
        I: IntoIterator<Item = Tool>,
    {
        self.tools = Some(tools.into_iter().collect());
        self
    }

    /// Configures how the model uses tools.
    ///
    /// # Arguments
    /// * `tool_config` - The configuration for tool usage.
    pub fn tool_config(mut self, tool_config: impl Into<ToolConfig>) -> Self {
        self.tool_config = Some(tool_config.into());
        self
    }

    /// Applies content safety filters to the model.
    ///
    /// Safety settings control the probability thresholds for filtering
    /// potentially harmful content.
    ///
    /// # Arguments
    /// * `safety_settings` - An iterator of `SafetySetting` instances.
    pub fn safety_settings<I>(mut self, safety_settings: I) -> Self
    where
        I: IntoIterator<Item = SafetySetting>,
    {
        self.safety_settings = Some(safety_settings.into_iter().collect());
        self
    }

    /// Sets the generation parameters for the model.
    ///
    /// This includes settings like `temperature`, `top_k`, and `top_p`
    /// to control the creativity and randomness of the model's output.
    ///
    /// # Arguments
    /// * `generation_config` - The configuration for generation.
    pub fn generation_config(mut self, generation_config: impl Into<GenerationConfig>) -> Self {
        self.generation_config = Some(generation_config.into());
        self
    }

    /// Creates a copy with new system instructions
    pub fn with_cloned_instruction<I: IntoContent>(&self, instruction: I) -> Self {
        let mut clone = self.clone();

        clone.system_instruction = Some(instruction.into_content());
        clone
    }

    /// Sets the number of candidates to generate.
    ///
    /// This parameter specifies how many different response candidates the model should generate
    /// for a given prompt. The model will then select the best one based on its internal
    /// evaluation.
    pub fn candidate_count(mut self, x: i32) -> Self {
        self.set_candidate_count(x);
        self
    }

    /// Sets the maximum number of output tokens.
    ///
    /// This parameter caps the length of the generated response, measured in tokens.
    /// It's useful for controlling response size and preventing excessively long outputs.
    pub fn max_output_tokens(mut self, x: i32) -> Self {
        self.set_max_output_tokens(x);
        self
    }

    /// Sets the temperature for generation.
    ///
    /// Temperature controls the randomness of the output. Higher values, like 1.0,
    /// make the output more creative and unpredictable, while lower values, like 0.1,
    /// make it more deterministic and focused.
    pub fn temperature(mut self, x: f32) -> Self {
        self.set_temperature(x);
        self
    }

    /// Sets the top-p sampling parameter.
    ///
    /// Top-p (also known as nucleus sampling) chooses the smallest set of most likely
    /// tokens whose cumulative probability exceeds the value of `x`. This technique
    /// helps to prevent low-probability, nonsensical tokens from being chosen.
    pub fn top_p(mut self, x: f32) -> Self {
        self.set_top_p(x);
        self
    }

    /// Sets the top-k sampling parameter.
    ///
    /// Top-k restricts the model's token selection to the `k` most likely tokens at
    /// each step. It's a method for controlling the model's creativity and focus.
    pub fn top_k(mut self, x: i32) -> Self {
        self.set_top_k(x);
        self
    }

    /// Sets the number of candidates to generate.
    ///
    /// This parameter specifies how many different response candidates the model should generate
    /// for a given prompt. The model will then select the best one based on its internal
    /// evaluation.
    pub fn set_candidate_count(&mut self, x: i32) {
        self.generation_config
            .get_or_insert_default()
            .candidate_count = Some(x)
    }

    /// Sets the maximum number of output tokens.
    ///
    /// This parameter caps the length of the generated response, measured in tokens.
    /// It's useful for controlling response size and preventing excessively long outputs.
    pub fn set_max_output_tokens(&mut self, x: i32) {
        self.generation_config
            .get_or_insert_default()
            .max_output_tokens = Some(x)
    }

    /// Sets the temperature for generation.
    ///
    /// Temperature controls the randomness of the output. Higher values, like 1.0,
    /// make the output more creative and unpredictable, while lower values, like 0.1,
    /// make it more deterministic and focused.
    pub fn set_temperature(&mut self, x: f32) {
        self.generation_config.get_or_insert_default().temperature = Some(x)
    }

    /// Sets the top-p sampling parameter.
    ///
    /// Top-p (also known as nucleus sampling) chooses the smallest set of most likely
    /// tokens whose cumulative probability exceeds the value of `x`. This technique
    /// helps to prevent low-probability, nonsensical tokens from being chosen.
    pub fn set_top_p(&mut self, x: f32) {
        self.generation_config.get_or_insert_default().top_p = Some(x)
    }

    /// Sets the top-k sampling parameter.
    ///
    /// Top-k restricts the model's token selection to the `k` most likely tokens at
    /// each step. It's a method for controlling the model's creativity and focus.
    pub fn set_top_k(&mut self, x: i32) {
        self.generation_config.get_or_insert_default().top_k = Some(x)
    }

    #[inline(always)]
    fn build_request(
        self,
        contents: impl TryIntoContents,
    ) -> Result<GenerateContentRequest, Error> {
        let contents = contents.try_into_contents()?;
        Ok(GenerateContentRequest {
            model: self.model_name.into(),
            contents,
            system_instruction: self.system_instruction,
            tools: self.tools.unwrap_or_default(),
            tool_config: self.tool_config,
            safety_settings: self.safety_settings.unwrap_or_default(),
            generation_config: self.generation_config,
            cached_content: self.cached_content.map(|c| c.into()),
        })
    }

    // This is to avoid the performance overhead while cloning
    // SharedClient - Arc backed. Insignificant but unnecessary.
    fn cloned<'a>(&'a self) -> GenerativeModel<'a> {
        GenerativeModel {
            client: self.client.cloned(),
            ..Clone::clone(&self)
        }
    }
}

impl SafetySetting {
    /// Creates a new [`SafetySetting`] with default values
    pub fn new() -> Self {
        Self {
            category: 0,
            threshold: 0,
        }
    }

    /// Set the category for this setting
    pub fn harm_category(mut self, category: HarmCategory) -> Self {
        self.category = category.into();
        self
    }

    /// Control the probability threshold at which harm is blocked
    pub fn harm_threshold(mut self, threshold: HarmBlockThreshold) -> Self {
        self.threshold = threshold.into();
        self
    }
}

/// Generation response containing model output and metadata
pub type Response = GenerateContentResponse;

impl Response {
    /// Total tokens used in request/response cycle
    pub fn total_tokens(&self) -> f64 {
        // FIXME: I'm confused
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

    /// Streams content chunks to any `AsyncWrite` implementer
    ///
    /// # Returns
    /// Total bytes written
    pub async fn write_to_sync<W: AsyncWrite + std::marker::Unpin>(
        &mut self,
        dst: &mut W,
    ) -> Result<usize, Error> {
        use tokio::io::AsyncWriteExt;

        let mut total = 0;

        while let Some(response) = self
            .next()
            .await
            .map_err(|e| Error::Stream(ActionError::Error(e.into())))?
        {
            let bytes = response.try_into_bytes()?;
            let written = dst
                .write(&bytes)
                .await
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
        GenerativeModel::new_inner(self, name)
    }

    /// Creates a new typed generative model interface
    ///
    /// Shorthand for `TypedModel::new()`
    pub fn typed_model<'c, T: AsSchema>(&'c self, name: &str) -> TypedModel<'c, T> {
        TypedModel::<T>::new_inner(self, name)
    }
}

impl SharedClient {
    /// Creates a new generative model interface
    pub fn generative_model(&self, name: &str) -> GenerativeModel<'static> {
        GenerativeModel::new_inner(self.clone(), name)
    }

    /// Creates a new typed generative model interface
    pub fn typed_model<T: AsSchema>(&self, name: &str) -> TypedModel<'static, T> {
        TypedModel::<T>::new_inner(self.clone(), name)
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

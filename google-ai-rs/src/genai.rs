use core::fmt;
use std::io::Write;
use tonic::{IntoRequest, Streaming};

use crate::{
    client::Client,
    content::{IntoContent as _, IntoContents},
    error::{status_into_error, ActionError, Error},
    full_model_name,
    proto::{
        part::Data, CachedContent, Content as ContentP, CountTokensRequest, CountTokensResponse,
        GenerateContentRequest, GenerateContentResponse, GenerationConfig as GenerationConfigP,
        Model, Part, SafetySetting as SafetySettingP, Schema, Tool as ToolP,
        ToolConfig as ToolConfigP, TunedModel,
    },
    schema::AsSchema,
};

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
        T: IntoContents,
    {
        let contents = contents.into_contents();
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
        T: IntoContents,
    {
        let contents = contents.into_contents();
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
    /// # const COST_PER_TOKEN: i64 = 1;
    /// println!("Estimated cost: ${}", token_count.total() * COST_PER_TOKEN);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns [`Error::InvalidArgument`] for empty input
    pub async fn count_tokens<T>(&self, contents: T) -> Result<CountTokensResponse, Error>
    where
        T: IntoContents,
    {
        let contents = contents.into_contents();
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
    pub fn with_system_instruction(mut self, instruction: &str) -> Self {
        self.system_instruction = Some(instruction.into_content());
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
                    "cached content name is empty".to_owned(),
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

        c.response_mime_type = "application/json".into();
        c.response_schema = Some(schema.clone());
        self
    }

    /// Creates a copy with new system instructions
    pub fn with_cloned_instruction(&self, instruction: &str) -> Self {
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
    pub fn total_tokens(&self) -> i32 {
        self.usage_metadata
            .as_ref()
            .map_or(0, |meta| meta.total_token_count)
    }

    /// Serializes successful content text parts to String
    pub fn text(self) -> String {
        String::from_utf8(
            self.try_into_bytes_with(|d| match d {
                Some(Data::Text(text)) => Ok(text.into_bytes()),
                _ => Ok(Vec::new()),
            })
            .unwrap(),
        )
        .unwrap()
    }

    /// Serializes successful content text and inline data parts to bytes
    pub fn into_bytes(self) -> Vec<u8> {
        self.try_into_bytes_with(|d| match d {
            Some(Data::Text(text)) => Ok(text.into_bytes()),
            Some(Data::InlineData(blob)) => Ok(blob.data),
            _ => Ok(Vec::new()),
        })
        .unwrap()
    }

    /// Serializes successful content text parts to String.
    ///
    /// returns InvalidContent if it encounters data apart from
    /// text and inline data
    pub fn try_into_bytes(self) -> Result<Vec<u8>, Error> {
        self.try_into_bytes_with(|d| match d {
            Some(Data::Text(text)) => Ok(text.into_bytes()),
            _ => Err(Error::InvalidContent(
                "InvalidContent encountered".to_owned(),
            )),
        })
    }

    pub fn try_into_bytes_with(
        self,
        m: impl Fn(Option<Data>) -> Result<Vec<u8>, Error>,
    ) -> Result<Vec<u8>, Error> {
        let mut output = Vec::new();

        for candidate in self.candidates {
            if let Some(content) = candidate.content {
                for part in content.parts {
                    output.extend(m(part.data)?)
                }
            }
        }

        Ok(output)
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for candidate in &self.candidates {
            if let Some(content) = &candidate.content {
                write!(f, "{}", content)?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for ContentP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for part in &self.parts {
            write!(f, "{}", part)?;
        }
        Ok(())
    }
}

impl fmt::Display for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.data {
            Some(Data::Text(text)) => write!(f, "{}", text),
            None => Ok(()),
            // Handle other data types raw
            other => write!(f, "{:?}", other),
        }
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
}

impl CountTokensResponse {
    pub fn total(&self) -> i64 {
        (self.total_tokens + self.cached_content_token_count).into()
    }
}

#[derive(Debug)]
pub enum Info {
    Tuned(TunedModel),
    Model(Model),
}

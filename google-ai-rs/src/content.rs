/// Fallible conversion to multiple [`Content`] items
///
/// Use this for types that need to perform validation or conditional
/// construction of content. Prefer [`IntoContents`] for infallible conversions.
///
/// # Examples
///
/// Custom serialization with media attachment:
/// ```
/// use google_ai_rs::{TryIntoContents, Content, Error, IntoParts as _, Part};
///
/// # enum MediaAttachment { Image(Vec<u8>), VoiceNote(Vec<u8>) }
/// struct UserInput {
///     message: String,
///     media: Option<MediaAttachment>
/// }
///
/// impl TryIntoContents for UserInput {
///     fn try_into_contents(self) -> Result<Vec<Content>, Error> {
///         let mut parts = self.message.into_parts();
///
///         if let Some(media) = self.media {
///             let part = match media {
///                 MediaAttachment::Image(data) => Part::blob("image/png", data),
///                 MediaAttachment::VoiceNote(data) => Part::blob("audio/wave", data),
///             };
///             parts.push(part);
///         }
///        
///         Ok(vec![Content::from(parts)])
///     }
/// }
/// ```
///
/// # Implementations
/// - `IntoContents`
pub trait TryIntoContents {
    /// Convert to content items, validating input if needed
    fn try_into_contents(self) -> Result<Vec<Content>, Error>;

    /// Create cached content targeting a specific AI model
    #[inline]
    fn try_into_cached_content_for(self, model_name: &str) -> Result<CachedContent, Error>
    where
        Self: Sized,
    {
        let contents = self.try_into_contents()?;
        Ok(contents.into_cached_content_for(model_name))
    }
}

impl<T: IntoContents> TryIntoContents for T {
    #[inline]
    fn try_into_contents(self) -> Result<Vec<Content>, Error> {
        Ok(self.into_contents())
    }
}

/// Fallible conversion to a single [`Content`] item
///
/// See [`TryIntoContents`] for usage patterns. Prefer [`IntoContent`]
/// for infallible conversions.
///
/// # Example
/// ```
/// use google_ai_rs::{TryIntoContent, Content, Part, Error};
///
/// struct ValidatedInput(String);
///
/// impl TryIntoContent for ValidatedInput {
///     fn try_into_content(self) -> Result<Content, Error> {
///         if self.0.is_empty() {
///             Err(Error::InvalidContent("Input cannot be empty".into()))
///         } else {
///             Ok(Content::new(self.0))
///         }
///     }
/// }
/// ```
pub trait TryIntoContent {
    /// Convert to a content item with validation    
    fn try_into_content(self) -> Result<Content, Error>;
}

impl<T: IntoContent> TryIntoContent for T {
    #[inline]
    fn try_into_content(self) -> Result<Content, Error> {
        Ok(self.into_content())
    }
}

/// Infallible conversion to multiple [`Content`] items
///
/// Automatically implemented for:
/// - Any type implementing [`IntoContent`] (converted to single-item vector)
/// - `Vec<Content>` (direct passthrough)
///
/// # Implementations
/// - `IntoContent`
/// - `Vec<Content>`
///
/// See [`IntoParts`] for full doc
pub trait IntoContents: sealed::Sealed {
    /// Convert to content items without error possibility    
    fn into_contents(self) -> Vec<Content>;

    /// Create cached content targeting a specific model
    #[inline]
    fn into_cached_content_for(self, model_name: &str) -> CachedContent
    where
        Self: Sized,
    {
        CachedContent {
            model: Some(full_model_name(model_name).into()),
            contents: self.into_contents(),
            ..Default::default()
        }
    }
}

impl IntoContents for Vec<Content> {
    #[inline(always)]
    fn into_contents(self) -> Vec<Content> {
        self
    }
}

impl<T: IntoContent> IntoContents for T {
    #[inline]
    fn into_contents(self) -> Vec<Content> {
        vec![self.into_content()]
    }
}

/// Infallible conversion to a single [`Content`] item
///
/// Automatically implemented for:
/// - [`Content`] (identity)
/// - Any type implementing [`IntoParts`]
///
/// # Example
/// ```
/// use google_ai_rs::{IntoContent, Part, Content};
///
/// let content = ("text", Part::blob("image/png", vec![0u8; 8])).into_content();
/// assert_eq!(content.parts.len(), 2);
/// ```
///
/// See [`IntoParts`] for full doc
pub trait IntoContent: sealed::Sealed {
    fn into_content(self) -> Content;
}

impl IntoContent for Content {
    #[inline(always)]
    fn into_content(self) -> Content {
        self
    }
}

impl<T: IntoParts> IntoContent for T {
    #[inline]
    fn into_content(self) -> Content {
        self.into()
    }
}

/// Conversion trait for types that can be represented as single/multiple content parts
///
/// # Implementations
/// - `&str` → Text part
/// - `String` → Text part
/// - `Part` → Direct passthrough
/// - `Vec<T: IntoParts>` → Flattened parts
/// - Arrays/slices of `T: IntoParts`
/// - Tuples of `any implementations of IntoParts` up-to 16 elements
///
/// # Examples
/// ```
/// use google_ai_rs::Part;
/// use google_ai_rs::content::IntoParts as _;
///
/// // Single item
/// let parts = "hello".into_parts();
///
/// # let bytes = vec![];
/// // Mixed collection
/// let parts = (
///     "text",
///     Part::blob("image/png", bytes),
///     String::from("another")
/// ).into_parts();
/// ```
pub trait IntoParts: sealed::Sealed {
    fn into_parts(self) -> Vec<Part>;
}

impl IntoParts for &str {
    #[inline]
    fn into_parts(self) -> Vec<Part> {
        vec![self.into()]
    }
}

impl IntoParts for String {
    #[inline]
    fn into_parts(self) -> Vec<Part> {
        vec![self.into()]
    }
}

impl IntoParts for Part {
    #[inline]
    fn into_parts(self) -> Vec<Part> {
        vec![self]
    }
}

impl IntoParts for FunctionCall {
    #[inline]
    fn into_parts(self) -> Vec<Part> {
        Part {
            data: Some(Data::FunctionCall(self)),
        }
        .into_parts()
    }
}

impl IntoParts for Blob {
    #[inline]
    fn into_parts(self) -> Vec<Part> {
        Part {
            data: Some(Data::InlineData(self)),
        }
        .into_parts()
    }
}

impl IntoParts for FileData {
    #[inline]
    fn into_parts(self) -> Vec<Part> {
        Part {
            data: Some(Data::FileData(self)),
        }
        .into_parts()
    }
}

impl<T: IntoParts> IntoParts for Vec<T> {
    #[inline]
    fn into_parts(self) -> Vec<Part> {
        let mut out = Vec::new();
        for part in self {
            out.extend(part.into_parts());
        }
        out
    }
}

impl<T: IntoParts, const N: usize> IntoParts for [T; N] {
    #[inline]
    fn into_parts(self) -> Vec<Part> {
        let mut out = Vec::new();
        for part in self {
            out.extend(part.into_parts());
        }
        out
    }
}

// Tuple implementations (up to 16 elements)
macro_rules! into_parts_for_tuple {
    (
        $(($($T:ident)*))*
    ) => {
        $(impl<$($T, )*> IntoParts for ($($T, )*)
        where
            $($T: IntoParts),*
        {
            #[doc = "This trait is implemented for tuples up to sixteen items long."]
            #[allow(non_snake_case)]
            #[inline]
            fn into_parts(self) -> Vec<Part> {
                #[allow(unused_mut)] // for the unit
                let mut parts = Vec::new();
                let ($($T, )*) = self;
                $(
                    parts.extend($T.into_parts());
                )*
                parts
            }
        })*
    };
}

into_parts_for_tuple! {
    ()
    (T0)
    (T0 T1)
    (T0 T1 T2)
    (T0 T1 T2 T3)
    (T0 T1 T2 T3 T4)
    (T0 T1 T2 T3 T4 T5)
    (T0 T1 T2 T3 T4 T5 T6)
    (T0 T1 T2 T3 T4 T5 T6 T7)
    (T0 T1 T2 T3 T4 T5 T6 T7 T8)
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9)
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10)
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11)
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12)
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13)
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14)
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15)
}

/// Conversion trait for response candidates with serde support.
///
/// Implemented automatically for types implementing `serde::Deserialize`
/// when the "serde" feature is enabled.
///
/// # Manual Implementation
/// Required for non-JSON response formats or custom parsing logic:
/// ```
/// # use google_ai_rs::{TryFromCandidates, Candidate, Error};
/// struct CustomType;
///
/// impl TryFromCandidates for CustomType {
///     fn try_from_candidates(candidates: &[Candidate]) -> Result<Self, Error> {
///         // Custom parsing logic
/// #       Ok(Self)
///     }
/// }
/// ```
///
/// # Implementations
/// - [`TryFromContents`]
#[cfg_attr(
    not(no_diagnostic_namespace),
    diagnostic::on_unimplemented(
        note = "enable the `serde` feature to get a free implementation for types that implement serde::DeserializeOwned"
    )
)]
pub trait TryFromCandidates: Sized {
    /// Attempt to parse from multiple response candidates
    ///
    /// # Example
    /// ```no_run
    /// # use google_ai_rs::{Candidate, TryFromCandidates, Error};
    /// # struct AnalysisResult;
    /// # impl TryFromCandidates for AnalysisResult {
    /// #     fn try_from_candidates(_: &[Candidate]) -> Result<Self, Error> { todo!() }
    /// # }
    /// # let response = google_ai_rs::genai::Response::default();
    /// let analysis = AnalysisResult::try_from_candidates(&response.candidates)?;
    /// # Ok::<(), Error>(())
    /// ```
    fn try_from_candidates(candidates: &[Candidate]) -> Result<Self, Error>;
}

/// Conversion trait for parsing structured data from response contents.
///
/// This is the backbone of type-safe responses. While you can implement it directly,
/// most users should prefer the automatic implementation provided by `serde`.
///
/// # Serde Default
/// With the `serde` feature enabled, any `serde::Deserialize` type automatically
/// implements this trait by expecting JSON-formatted responses:
/// ```
/// # use serde::Deserialize;
/// #[derive(Deserialize)]
/// struct MyResponse {
///     answer: String,
///     confidence: f32,
/// }
///
/// // Automatically implements TryFromContents!
/// ```
///
/// # Manual Implementation
/// Implement this directly for non-JSON formats or custom parsing logic:
/// ```
/// # use google_ai_rs::TryFromContents;
/// use google_ai_rs::{Data, Part, Content, Error, error::ServiceError};
///
/// struct TextLength(usize);
///
/// impl TryFromContents for TextLength {
///     fn try_from_contents<'a, I: Iterator<Item = &'a Content>>(contents: I) -> Result<Self, Error> {
///         // Extract text from first content part
///         let text = contents.into_iter()
///              .flat_map(|c| c.parts.iter())
///              .find_map(|p| match p {
///                    Part { data: Some(Data::Text(text)) } => {
///                        Some(text)
///                    }
///                    _ => None
///              })
///             .ok_or(Error::Service(ServiceError::InvalidResponse("Empty response".into())))?;
///         
///         Ok(TextLength(text.len()))
///     }
/// }
/// ```
#[cfg_attr(
    not(no_diagnostic_namespace),
    diagnostic::on_unimplemented(
        note = "enable the `serde` feature to get a free implementation for types that implement serde::DeserializeOwned"
    )
)]
pub trait TryFromContents: Sized {
    /// Parses the response contents into a concrete type.
    ///
    /// Implementations should:
    /// 1. Extract relevant data from the content parts
    /// 2. Handle error cases (missing data, invalid formats)
    /// 3. Return the parsed value or appropriate error
    ///
    /// The default serde implementation expects exactly one JSON-formatted text part.
    fn try_from_contents<'a, I: Iterator<Item = &'a Content>>(contents: I) -> Result<Self, Error>;
}

impl<T: TryFromContents> TryFromCandidates for T {
    #[inline]
    fn try_from_candidates(candidates: &[Candidate]) -> Result<Self, Error> {
        let contents = candidates.iter().filter_map(|c| c.content.as_ref());

        T::try_from_contents(contents)
    }
}

#[cfg(feature = "serde")]
mod serde_support {
    use super::*;
    use serde::de::DeserializeOwned;

    /// JSON deserialization support
    ///
    /// Enabled with `serde` feature. Deserializes concatenated JSON content.
    ///
    /// The implementation for DeserializeOwned assumes the content is json
    ///
    /// returns ['ServiceError::InvalidResponse`] on invalid json
    impl<T: DeserializeOwned> TryFromContents for T {
        #[inline]
        fn try_from_contents<'a, I>(contents: I) -> Result<Self, Error>
        where
            I: Iterator<Item = &'a Content>,
        {
            // FIXME: Check for anything to enforce format...
            let mut buf = Vec::new();
            for content in contents {
                content._try_to_bytes_with(&mut buf, try_to_bytes)?;
            }

            serde_json::from_slice(&buf).map_err(|err| {
                Error::Service(crate::error::ServiceError::InvalidResponse(err.into()))
            })
        }
    }
}

// Content construction utilities
impl Part {
    /// Creates a text content part
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            data: Some(Data::Text(text.into())),
        }
    }

    /// Extracts the text in a part
    pub fn to_text(&self) -> &str {
        // use display?
        match &self.data {
            Some(Data::Text(text)) => text,
            _ => "",
        }
    }

    /// Extracts the text in a part and consumes it.
    pub fn into_text(self) -> String {
        match self.data {
            Some(Data::Text(text)) => text,
            _ => "".to_owned(),
        }
    }

    /// Create a binary blob part
    ///
    /// # Example
    /// ```
    /// # use google_ai_rs::Part;
    /// let image = Part::blob("image/png", vec![0u8; 1024]);
    /// ```
    pub fn blob(mime_type: &str, data: Vec<u8>) -> Self {
        Self {
            data: Some(Data::InlineData(Blob {
                mime_type: mime_type.to_owned(),
                data,
            })),
        }
    }

    /// Create a file reference part
    pub fn file_data(mime_type: &str, uri: &str) -> Self {
        Self {
            data: Some(Data::FileData(FileData {
                mime_type: mime_type.to_owned(),
                file_uri: uri.to_owned(),
            })),
        }
    }
}

impl From<&str> for Part {
    fn from(text: &str) -> Self {
        Part::text(text)
    }
}

impl From<String> for Part {
    fn from(text: String) -> Self {
        Part {
            data: Some(Data::Text(text)),
        }
    }
}

impl fmt::Display for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.data {
            Some(Data::Text(text)) => write!(f, "{text}"),
            _ => Ok(()),
            // This should be done in debug
            // Handle other data types raw
            // other => write!(f, "{:?}", other),
        }
    }
}

impl Content {
    /// Create new content from parts
    ///
    /// # Example
    /// ```
    /// # use google_ai_rs::{Content, Part};
    /// let content = Content::new((
    ///     "Describe this image:",
    ///     Part::blob("image/png", vec![0u8; 1024])
    /// ));
    /// ```
    #[inline]
    pub fn new<I: IntoParts>(parts: I) -> Self {
        Self {
            role: "user".into(),
            parts: parts.into_parts(),
        }
    }

    fn try_to_bytes_with(
        &self,
        m: impl Fn(Option<&Data>) -> Result<&[u8], Error>,
    ) -> Result<Vec<u8>, Error> {
        let mut output = Vec::new();
        self._try_to_bytes_with(&mut output, m)?;
        Ok(output)
    }

    fn _try_to_bytes_with(
        &self,
        buf: &mut Vec<u8>,
        m: impl Fn(Option<&Data>) -> Result<&[u8], Error>,
    ) -> Result<(), Error> {
        for part in &self.parts {
            buf.extend(m(part.data.as_ref())?)
        }
        Ok(())
    }
}

impl<T: IntoParts> From<T> for Content {
    fn from(parts: T) -> Self {
        Self::new(parts)
    }
}

impl TryInto<Vec<u8>> for &Content {
    type Error = Error;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        self.try_to_bytes_with(try_to_bytes)
    }
}

impl fmt::Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for part in &self.parts {
            write!(f, "{part}")?;
        }
        Ok(())
    }
}

impl Candidate {
    /// Returns all the `FunctionCall` parts in the candidate.
    pub fn function_calls(&self) -> Option<Vec<FunctionCall>> {
        if let Some(content) = &self.content {
            let mut out = Vec::new();
            for p in &content.parts {
                if let Part {
                    data: Some(Data::FunctionCall(ref fc)),
                } = p
                {
                    out.push(fc.clone());
                }
            }
            return Some(out);
        }
        None
    }
}

// Response processing implementation
impl Response {
    /// Serializes successful content text parts to String without consuming
    /// the response
    #[inline]
    pub fn to_text(&self) -> String {
        String::from_utf8(
            self.try_to_bytes_with(|d| match d {
                Some(Data::Text(text)) => Ok(text.as_bytes()),
                _ => Ok(b""),
            })
            .unwrap(),
        )
        .unwrap()
    }

    /// Serializes successful content text parts to String
    ///
    /// Prefer `to_text`.
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
    /// without consuming the response
    pub fn to_bytes(&self) -> Vec<u8> {
        self.try_to_bytes().unwrap_or_default()
    }

    /// Serializes successful content text and inline data parts to bytes
    ///
    /// Prefer `to_bytes`
    pub fn into_bytes(self) -> Vec<u8> {
        self.try_into_bytes().unwrap_or_default()
    }

    /// Serializes successful content text and inline data
    /// parts to bytes without consuming the response.
    ///
    /// returns InvalidContent if it encounters data apart from
    /// text and inline data
    pub fn try_to_bytes(&self) -> Result<Vec<u8>, Error> {
        self.try_to_bytes_with(try_to_bytes)
    }

    /// Serializes successful content text and inline data
    /// parts to bytes.
    ///
    /// returns InvalidContent if it encounters data apart from
    /// text and inline data
    ///
    /// Prefer `try_to_bytes`
    pub fn try_into_bytes(self) -> Result<Vec<u8>, Error> {
        self.try_into_bytes_with(try_into_bytes)
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

    fn try_to_bytes_with(
        &self,
        m: impl Fn(Option<&Data>) -> Result<&[u8], Error>,
    ) -> Result<Vec<u8>, Error> {
        let mut output = Vec::new();

        for candidate in &self.candidates {
            if let Some(content) = &candidate.content {
                for part in &content.parts {
                    output.extend(m(part.data.as_ref())?)
                }
            }
        }

        Ok(output)
    }
}

impl TryInto<Vec<u8>> for Response {
    type Error = Error;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        self.try_into_bytes()
    }
}

fn try_to_bytes(d: Option<&Data>) -> Result<&[u8], Error> {
    match d {
        Some(Data::Text(text)) => Ok(text.as_bytes()),
        Some(Data::InlineData(blob)) => Ok(&blob.data),
        d => Err(Error::InvalidContent(
            format!("InvalidContent encountered  {d:#?}").into(),
        )),
    }
}

fn try_into_bytes(d: Option<Data>) -> Result<Vec<u8>, Error> {
    match d {
        Some(Data::Text(text)) => Ok(text.into_bytes()),
        Some(Data::InlineData(blob)) => Ok(blob.data),
        d => Err(Error::InvalidContent(
            format!("InvalidContent encountered  {d:#?}").into(),
        )),
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for candidate in &self.candidates {
            if let Some(content) = &candidate.content {
                write!(f, "{content}")?;
            }
        }
        Ok(())
    }
}

use std::fmt;

use prost_types::FieldMask;

use crate::{
    full_model_name,
    genai::Response,
    proto::{
        cached_content, part::Data, tuned_model::SourceModel, Blob, CachedContent, Candidate,
        Content, FileData, FunctionCall, Part, TunedModel,
    },
    Error,
};

#[derive(Debug)]
#[doc(hidden)]
/// Specifies fields to update in a cached content.
pub enum CachedContentFieldToUpdate {
    /// Update the absolute expiration timestamp
    ExpireTime,
    /// Update the time-to-live duration (relative to current time)
    Ttl,
}

pub(crate) trait UpdateFieldMask {
    fn field_mask(&self) -> FieldMask;
}

impl UpdateFieldMask for CachedContent {
    fn field_mask(&self) -> FieldMask {
        FieldMask {
            paths: vec![if let Some(expiration) = self.expiration {
                match expiration {
                    cached_content::Expiration::ExpireTime(_) => "expire_time".to_owned(),
                    cached_content::Expiration::Ttl(_) => "ttl".to_owned(),
                }
            } else {
                "".to_owned()
            }],
        }
    }
}

impl UpdateFieldMask for TunedModel {
    fn field_mask(&self) -> FieldMask {
        let mut paths = Vec::new();

        macro_rules! fill {
            ($not:ident => $($property:ident)*) => {
                $(
                    {if !self.$property.$not() {
                        paths.push(stringify!($property).to_owned());
                    }}
                )*
            }
        }
        fill!(is_empty => display_name description reader_project_numbers);
        fill!(is_none => temperature top_p tuning_task);

        if let Some(SourceModel::TunedModelSource(_)) = self.source_model {
            paths.push("tuned_model_source".to_owned());
        }

        FieldMask { paths }
    }
}

mod sealed {
    pub trait Sealed {}
}

impl<T> sealed::Sealed for T {}

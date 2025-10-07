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

    /// # Implementation detail
    /// Default just calls `into_parts()`. Override for fewer allocations.
    #[doc(hidden)]
    fn into_parts_in_place(self, parts: &mut Vec<Part>)
    where
        Self: Sized,
    {
        parts.extend(self.into_parts());
    }

    /// # Implementation detail
    /// Default is `(1, None)`. Override if you know the size ahead of time.
    #[doc(hidden)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (1, None)
    }
}

macro_rules! into_parts_single {
    // For direct Part producers
    ($ty:ty, |$self:ident| $expr:expr) => {
        impl IntoParts for $ty {
            #[inline]
            fn into_parts(self) -> Vec<Part> {
                let $self = self;
                vec![$expr]
            }

            #[inline]
            fn into_parts_in_place(self, parts: &mut Vec<Part>) {
                let $self = self;
                parts.push($expr);
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                (1, Some(1))
            }
        }
    };
}

into_parts_single!(&str, |s| s.into());
into_parts_single!(String, |s| s.into());
into_parts_single!(Part, |p| p);
into_parts_single!(Blob, |b| Part {
    data: Some(Data::InlineData(b))
});
// TODO: Remove
into_parts_single!(FunctionCall, |f| Part {
    data: Some(Data::FunctionCall(f))
});
into_parts_single!(FileData, |f| Part {
    data: Some(Data::FileData(f))
});

macro_rules! into_parts_iter {
    ($ty:ty [$($b:tt)*]) => {
        impl<T: IntoParts, $($b)*> IntoParts for $ty {
            #[inline]
            fn into_parts(self) -> Vec<Part> {
                let mut out = Vec::with_capacity(self.size_hint().0);
                self.into_parts_in_place(&mut out);
                out
            }

            #[inline]
            fn into_parts_in_place(self, parts: &mut Vec<Part>) {
                for item in self.into_iter() {
                    item.into_parts_in_place(parts);
                }
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                let mut lower = 0;
                let mut upper: Option<usize> = Some(0);
                for item in self.into_iter() {
                    let (l, u) = item.size_hint();
                    lower += l;
                    upper = match (upper, u) {
                        (Some(a), Some(b)) => Some(a + b),
                        _ => None,
                    };
                }
                (lower, upper)
            }
        }
    };
}

into_parts_iter!(Vec<T> []);
into_parts_iter!([T; N] [const N: usize]);

impl<T: IntoParts + Clone> IntoParts for std::borrow::Cow<'_, T> {
    #[inline]
    fn into_parts(self) -> Vec<Part> {
        let mut parts = Vec::with_capacity(self.size_hint().0);
        self.into_parts_in_place(&mut parts);
        parts
    }

    #[inline]
    fn into_parts_in_place(self, parts: &mut Vec<Part>) {
        self.into_owned().into_parts_in_place(parts);
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.as_ref().size_hint()
    }
}

// Tuple implementations (up to 16 elements)
macro_rules! into_parts_for_tuple {
    (
        $(($($T:ident)*))*
    ) => {
        $(
            impl<$($T, )*> IntoParts for ($($T, )*)
            where
                $($T: IntoParts),*
            {
                #[doc = "This trait is implemented for tuples up to sixteen items long."]
                #[inline]
                fn into_parts(self) -> Vec<Part> {
                    #[allow(unused_mut)]
                    // Default: allocate once with correct capacity
                    let mut parts = Vec::with_capacity(self.size_hint().0);
                    self.into_parts_in_place(&mut parts);
                    parts
                }

                #[inline]
                fn into_parts_in_place(self, _parts: &mut Vec<Part>) {
                    #[allow(non_snake_case)]
                    let ($($T, )*) = self;
                    $(
                        $T.into_parts_in_place(_parts);
                    )*
                }

                #[inline]
                fn size_hint(&self) -> (usize, Option<usize>) {
                    #[allow(unused_mut)]
                    let mut lower = 0;
                    #[allow(unused_mut)]
                    let mut upper: Option<usize> = Some(0);
                    #[allow(non_snake_case)]
                    let ($($T, )*) = self;
                    $(
                        let (l, u) = $T.size_hint();
                        lower += l;
                        upper = match (upper, u) {
                            (Some(a), Some(b)) => Some(a + b),
                            _ => None,
                        };
                    )*
                    (lower, upper)
                }
            }
        )*
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
            // TODO: from_reader or some from_iter would be better here
            // for better memory, but the current from_reader is not that
            // performant.
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
    /// Creates new `Content` with the role set to "user".
    ///
    /// This is the standard way to represent a user's prompt to the model.
    /// This method is an alias for [`Content::user`].
    ///
    /// # Arguments
    /// * `parts` - Any type that can be converted into a collection of `Part`s,
    ///   such as a string, a `Part`, or a tuple of parts.
    ///
    /// # Example
    /// ```
    /// # use google_ai_rs::{Content, Part};
    /// // Create content from a simple string
    /// let text_content = Content::new("Describe this image:");
    ///
    /// // Create multi-part content from a tuple
    /// let mixed_content = Content::new((
    ///     "A photo of the beach.",
    ///     Part::blob("image/png", vec![0u8; 1024])
    /// ));
    /// ```
    #[inline]
    pub fn new<I: IntoParts>(parts: I) -> Self {
        Self::user(parts)
    }

    /// Creates new `Content` explicitly assigning it the "user" role.
    ///
    /// User content represents the prompts and inputs you provide to the model.
    /// It's the most common type of content you'll create.
    #[inline]
    pub fn user<I: IntoParts>(parts: I) -> Self {
        Self {
            role: "user".into(),
            parts: parts.into_parts(),
        }
    }

    /// Creates new `Content` explicitly assigning it the "model" role.
    ///
    /// Model content represents the responses generated by the AI. It is primarily
    /// used to build and maintain a multi-turn conversation history.
    #[inline]
    pub fn model<I: IntoParts>(parts: I) -> Self {
        Self {
            role: "model".into(),
            parts: parts.into_parts(),
        }
    }

    #[inline]
    fn try_to_bytes_with(
        &self,
        m: impl Fn(Option<&Data>) -> Result<&[u8], Error>,
    ) -> Result<Vec<u8>, Error> {
        let mut output = Vec::new();
        self._try_to_bytes_with(&mut output, m)?;
        Ok(output)
    }

    #[inline]
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

use base64::engine::general_purpose::NO_PAD;
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

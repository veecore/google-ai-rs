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

/// Conversion trait for types that can be represented as multiple contents
///
/// # Implementations
/// - [`IntoParts`]
/// - `IntoContent`
/// - `Vec<Content>`
///
/// See [`IntoParts`] for full doc
pub trait IntoContents: sealed::Sealed {
    fn into_contents(self) -> Vec<Content>;

    fn into_cached_content_for(self, model_name: &str) -> CachedContent
    where
        Self: Sized,
    {
        CachedContent {
            model: Some(full_model_name(model_name).to_owned()),
            contents: self.into_contents(),
            ..Default::default()
        }
    }
}

impl IntoContents for Vec<Content> {
    fn into_contents(self) -> Vec<Content> {
        self
    }
}

impl<T: IntoContent> IntoContents for T {
    fn into_contents(self) -> Vec<Content> {
        vec![self.into_content()]
    }
}

/// Conversion trait for types that can be represented as a content
///
/// # Implementations
/// - [`IntoParts`]
/// - `Content`
///
/// See [`IntoParts`] for full doc
pub trait IntoContent: sealed::Sealed {
    fn into_content(self) -> Content;
}

impl IntoContent for Content {
    fn into_content(self) -> Content {
        self
    }
}

impl<T: IntoParts> IntoContent for T {
    fn into_content(self) -> Content {
        self.into()
    }
}

impl IntoParts for &str {
    fn into_parts(self) -> Vec<Part> {
        vec![self.into()]
    }
}

impl IntoParts for String {
    fn into_parts(self) -> Vec<Part> {
        vec![self.into()]
    }
}

impl IntoParts for Part {
    fn into_parts(self) -> Vec<Part> {
        vec![self]
    }
}

impl<T: IntoParts> IntoParts for Vec<T> {
    fn into_parts(self) -> Vec<Part> {
        let mut out = Vec::new();
        for part in self {
            out.extend(part.into_parts());
        }
        out
    }
}

impl<T: IntoParts, const N: usize> IntoParts for [T; N] {
    fn into_parts(self) -> Vec<Part> {
        let mut out = Vec::new();
        for part in self {
            out.extend(part.into_parts());
        }
        out
    }
}

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

impl<T: IntoParts> From<T> for Content {
    fn from(parts: T) -> Self {
        Self {
            role: "user".into(),
            parts: parts.into_parts(),
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

use prost_types::FieldMask;

use crate::{
    full_model_name,
    proto::{
        cached_content, part::Data, tuned_model::SourceModel, Blob, CachedContent, Candidate,
        Content, FileData, FunctionCall, Part, TunedModel,
    },
};

impl Part {
    /// Creates a text content part
    pub fn text(text: &str) -> Self {
        Self {
            data: Some(Data::Text(text.into())),
        }
    }

    pub fn blob(mime_type: &str, data: Vec<u8>) -> Self {
        Self {
            data: Some(Data::InlineData(Blob {
                mime_type: mime_type.to_owned(),
                data,
            })),
        }
    }

    pub fn file_data(mime_type: &str, uri: &str) -> Self {
        Self {
            data: Some(Data::FileData(FileData {
                mime_type: mime_type.to_owned(),
                file_uri: uri.to_owned(),
            })),
        }
    }
}

impl Candidate {
    /// function_calls return all the `FunctionCall` parts in the candidate.
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

/// Specifies fields to update in a cached content.
#[derive(Debug)]
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

        if !self.display_name.is_empty() {
            paths.push("display_name".to_owned());
        }

        if !self.description.is_empty() {
            paths.push("description".to_owned());
        }

        if self.temperature.is_some() {
            paths.push("temperature".to_owned());
        }

        if self.top_p.is_some() {
            paths.push("top_p".to_owned());
        }

        if self.tuning_task.is_some() {
            paths.push("tuning_task".to_owned());
        }

        if !self.reader_project_numbers.is_empty() {
            paths.push("reader_project_numbers".to_owned());
        }

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

/// A set of the feedback metadata the prompt specified in
/// `GenerateContentRequest.content`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PromptFeedback {
    /// Optional. If set, the prompt was blocked and no candidates are returned.
    /// Rephrase the prompt.
    #[prost(enumeration = "prompt_feedback::BlockReason", tag = "1")]
    pub block_reason: i32,
    /// Ratings for safety of the prompt.
    /// There is at most one rating per category.
    #[prost(message, repeated, tag = "2")]
    pub safety_ratings: ::prost::alloc::vec::Vec<super::SafetyRating>,
}
/// Nested message and enum types in `PromptFeedback`.
pub mod prompt_feedback {
    /// Specifies the reason why the prompt was blocked.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum BlockReason {
        /// Default value. This value is unused.
        Unspecified = 0,
        /// Prompt was blocked due to safety reasons. Inspect `safety_ratings`
        /// to understand which safety category blocked it.
        Safety = 1,
        /// Prompt was blocked due to unknown reasons.
        Other = 2,
        /// Prompt was blocked due to the terms which are included from the
        /// terminology blocklist.
        Blocklist = 3,
        /// Prompt was blocked due to prohibited content.
        ProhibitedContent = 4,
        /// Candidates blocked due to unsafe image generation content.
        ImageSafety = 5,
    }
    impl BlockReason {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Self::Unspecified => "BLOCK_REASON_UNSPECIFIED",
                Self::Safety => "SAFETY",
                Self::Other => "OTHER",
                Self::Blocklist => "BLOCKLIST",
                Self::ProhibitedContent => "PROHIBITED_CONTENT",
                Self::ImageSafety => "IMAGE_SAFETY",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "BLOCK_REASON_UNSPECIFIED" => Some(Self::Unspecified),
                "SAFETY" => Some(Self::Safety),
                "OTHER" => Some(Self::Other),
                "BLOCKLIST" => Some(Self::Blocklist),
                "PROHIBITED_CONTENT" => Some(Self::ProhibitedContent),
                "IMAGE_SAFETY" => Some(Self::ImageSafety),
                _ => None,
            }
        }
    }
}
/// Metadata on the generation request's token usage.
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct UsageMetadata {
    /// Number of tokens in the prompt. When `cached_content` is set, this is
    /// still the total effective prompt size meaning this includes the number of
    /// tokens in the cached content.
    #[prost(int32, tag = "1")]
    pub prompt_token_count: i32,
    /// Number of tokens in the cached part of the prompt (the cached content)
    #[prost(int32, tag = "4")]
    pub cached_content_token_count: i32,
    /// Total number of tokens across all the generated response candidates.
    #[prost(int32, tag = "2")]
    pub candidates_token_count: i32,
    /// Total token count for the generation request (prompt + response
    /// candidates).
    #[prost(int32, tag = "3")]
    pub total_token_count: i32,
}

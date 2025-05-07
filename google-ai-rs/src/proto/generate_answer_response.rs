/// Feedback related to the input data used to answer the question, as opposed
/// to the model-generated response to the question.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InputFeedback {
    /// Optional. If set, the input was blocked and no candidates are returned.
    /// Rephrase the input.
    #[prost(enumeration = "input_feedback::BlockReason", optional, tag = "1")]
    pub block_reason: ::core::option::Option<i32>,
    /// Ratings for safety of the input.
    /// There is at most one rating per category.
    #[prost(message, repeated, tag = "2")]
    pub safety_ratings: ::prost::alloc::vec::Vec<super::SafetyRating>,
}
/// Nested message and enum types in `InputFeedback`.
pub mod input_feedback {
    /// Specifies what was the reason why input was blocked.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum BlockReason {
        /// Default value. This value is unused.
        Unspecified = 0,
        /// Input was blocked due to safety reasons. Inspect
        /// `safety_ratings` to understand which safety category blocked it.
        Safety = 1,
        /// Input was blocked due to other reasons.
        Other = 2,
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
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "BLOCK_REASON_UNSPECIFIED" => Some(Self::Unspecified),
                "SAFETY" => Some(Self::Safety),
                "OTHER" => Some(Self::Other),
                _ => None,
            }
        }
    }
}

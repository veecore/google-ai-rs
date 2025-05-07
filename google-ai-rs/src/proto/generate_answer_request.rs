/// Style for grounded answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum AnswerStyle {
    /// Unspecified answer style.
    Unspecified = 0,
    /// Succint but abstract style.
    Abstractive = 1,
    /// Very brief and extractive style.
    Extractive = 2,
    /// Verbose style including extra details. The response may be formatted as a
    /// sentence, paragraph, multiple paragraphs, or bullet points, etc.
    Verbose = 3,
}
impl AnswerStyle {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "ANSWER_STYLE_UNSPECIFIED",
            Self::Abstractive => "ABSTRACTIVE",
            Self::Extractive => "EXTRACTIVE",
            Self::Verbose => "VERBOSE",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "ANSWER_STYLE_UNSPECIFIED" => Some(Self::Unspecified),
            "ABSTRACTIVE" => Some(Self::Abstractive),
            "EXTRACTIVE" => Some(Self::Extractive),
            "VERBOSE" => Some(Self::Verbose),
            _ => None,
        }
    }
}
/// The sources in which to ground the answer.
#[derive(Clone, PartialEq, ::prost::Oneof)]
pub enum GroundingSource {
    /// Passages provided inline with the request.
    #[prost(message, tag = "6")]
    InlinePassages(super::GroundingPassages),
    /// Content retrieved from resources created via the Semantic Retriever
    /// API.
    #[prost(message, tag = "7")]
    SemanticRetriever(super::SemanticRetrieverConfig),
}

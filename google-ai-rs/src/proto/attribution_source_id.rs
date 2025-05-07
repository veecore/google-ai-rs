/// Identifier for a part within a `GroundingPassage`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GroundingPassageId {
    /// Output only. ID of the passage matching the `GenerateAnswerRequest`'s
    /// `GroundingPassage.id`.
    #[prost(string, tag = "1")]
    pub passage_id: ::prost::alloc::string::String,
    /// Output only. Index of the part within the `GenerateAnswerRequest`'s
    /// `GroundingPassage.content`.
    #[prost(int32, tag = "2")]
    pub part_index: i32,
}
/// Identifier for a `Chunk` retrieved via Semantic Retriever specified in the
/// `GenerateAnswerRequest` using `SemanticRetrieverConfig`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SemanticRetrieverChunk {
    /// Output only. Name of the source matching the request's
    /// `SemanticRetrieverConfig.source`. Example: `corpora/123` or
    /// `corpora/123/documents/abc`
    #[prost(string, tag = "1")]
    pub source: ::prost::alloc::string::String,
    /// Output only. Name of the `Chunk` containing the attributed text.
    /// Example: `corpora/123/documents/abc/chunks/xyz`
    #[prost(string, tag = "2")]
    pub chunk: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Oneof)]
pub enum Source {
    /// Identifier for an inline passage.
    #[prost(message, tag = "1")]
    GroundingPassage(GroundingPassageId),
    /// Identifier for a `Chunk` fetched via Semantic Retriever.
    #[prost(message, tag = "2")]
    SemanticRetrieverChunk(SemanticRetrieverChunk),
}

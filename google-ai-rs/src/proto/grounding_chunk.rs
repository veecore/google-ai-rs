/// Chunk from the web.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Web {
    /// URI reference of the chunk.
    #[prost(string, optional, tag = "1")]
    pub uri: ::core::option::Option<::prost::alloc::string::String>,
    /// Title of the chunk.
    #[prost(string, optional, tag = "2")]
    pub title: ::core::option::Option<::prost::alloc::string::String>,
}
/// Chunk type.
#[derive(Clone, PartialEq, ::prost::Oneof)]
pub enum ChunkType {
    /// Grounding chunk from the web.
    #[prost(message, tag = "1")]
    Web(Web),
}

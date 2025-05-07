#[derive(Clone, PartialEq, ::prost::Oneof)]
pub enum Data {
    /// The `Chunk` content as a string.
    /// The maximum number of tokens per chunk is 2043.
    #[prost(string, tag = "1")]
    StringValue(::prost::alloc::string::String),
}

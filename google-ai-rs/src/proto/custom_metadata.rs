#[derive(Clone, PartialEq, ::prost::Oneof)]
pub enum Value {
    /// The string value of the metadata to store.
    #[prost(string, tag = "2")]
    StringValue(::prost::alloc::string::String),
    /// The StringList value of the metadata to store.
    #[prost(message, tag = "6")]
    StringListValue(super::StringList),
    /// The numeric value of the metadata to store.
    #[prost(float, tag = "7")]
    NumericValue(f32),
}

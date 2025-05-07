/// Metadata on the usage of the cached content.
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct UsageMetadata {
    /// Total number of tokens that the cached content consumes.
    #[prost(int32, tag = "1")]
    pub total_token_count: i32,
}
/// Specifies when this resource will expire.
#[derive(Clone, Copy, PartialEq, ::prost::Oneof)]
pub enum Expiration {
    /// Timestamp in UTC of when this resource is considered expired.
    /// This is *always* provided on output, regardless of what was sent
    /// on input.
    #[prost(message, tag = "9")]
    ExpireTime(::prost_types::Timestamp),
    /// Input only. New TTL for this resource, input only.
    #[prost(message, tag = "10")]
    Ttl(::prost_types::Duration),
}

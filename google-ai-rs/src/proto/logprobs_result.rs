/// Candidate for the logprobs token and score.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Candidate {
    /// The candidate’s token string value.
    #[prost(string, optional, tag = "1")]
    pub token: ::core::option::Option<::prost::alloc::string::String>,
    /// The candidate’s token id value.
    #[prost(int32, optional, tag = "3")]
    pub token_id: ::core::option::Option<i32>,
    /// The candidate's log probability.
    #[prost(float, optional, tag = "2")]
    pub log_probability: ::core::option::Option<f32>,
}
/// Candidates with top log probabilities at each decoding step.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TopCandidates {
    /// Sorted by log probability in descending order.
    #[prost(message, repeated, tag = "1")]
    pub candidates: ::prost::alloc::vec::Vec<Candidate>,
}

/// Inline data or a reference to the data.
#[derive(Clone, PartialEq, ::prost::Oneof)]
pub enum Dataset {
    /// Optional. Inline examples with simple input/output text.
    #[prost(message, tag = "1")]
    Examples(super::TuningExamples),
}

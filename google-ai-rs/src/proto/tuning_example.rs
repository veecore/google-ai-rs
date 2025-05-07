/// The input to the model for this example.
#[derive(Clone, PartialEq, ::prost::Oneof)]
pub enum ModelInput {
    /// Optional. Text model input.
    #[prost(string, tag = "1")]
    TextInput(::prost::alloc::string::String),
}

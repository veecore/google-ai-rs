/// The configuration for the speaker to use.
#[derive(Clone, PartialEq, ::prost::Oneof)]
pub enum VoiceConfig {
    /// The configuration for the prebuilt voice to use.
    #[prost(message, tag = "1")]
    PrebuiltVoiceConfig(super::PrebuiltVoiceConfig),
}

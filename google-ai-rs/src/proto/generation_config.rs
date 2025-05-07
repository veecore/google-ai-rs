/// Supported modalities of the response.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Modality {
    /// Default value.
    Unspecified = 0,
    /// Indicates the model should return text.
    Text = 1,
    /// Indicates the model should return images.
    Image = 2,
    /// Indicates the model should return audio.
    Audio = 3,
}
impl Modality {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "MODALITY_UNSPECIFIED",
            Self::Text => "TEXT",
            Self::Image => "IMAGE",
            Self::Audio => "AUDIO",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "MODALITY_UNSPECIFIED" => Some(Self::Unspecified),
            "TEXT" => Some(Self::Text),
            "IMAGE" => Some(Self::Image),
            "AUDIO" => Some(Self::Audio),
            _ => None,
        }
    }
}

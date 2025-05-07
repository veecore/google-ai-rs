/// Block at and beyond a specified harm probability.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum HarmBlockThreshold {
    /// Threshold is unspecified.
    Unspecified = 0,
    /// Content with NEGLIGIBLE will be allowed.
    BlockLowAndAbove = 1,
    /// Content with NEGLIGIBLE and LOW will be allowed.
    BlockMediumAndAbove = 2,
    /// Content with NEGLIGIBLE, LOW, and MEDIUM will be allowed.
    BlockOnlyHigh = 3,
    /// All content will be allowed.
    BlockNone = 4,
    /// Turn off the safety filter.
    Off = 5,
}
impl HarmBlockThreshold {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "HARM_BLOCK_THRESHOLD_UNSPECIFIED",
            Self::BlockLowAndAbove => "BLOCK_LOW_AND_ABOVE",
            Self::BlockMediumAndAbove => "BLOCK_MEDIUM_AND_ABOVE",
            Self::BlockOnlyHigh => "BLOCK_ONLY_HIGH",
            Self::BlockNone => "BLOCK_NONE",
            Self::Off => "OFF",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "HARM_BLOCK_THRESHOLD_UNSPECIFIED" => Some(Self::Unspecified),
            "BLOCK_LOW_AND_ABOVE" => Some(Self::BlockLowAndAbove),
            "BLOCK_MEDIUM_AND_ABOVE" => Some(Self::BlockMediumAndAbove),
            "BLOCK_ONLY_HIGH" => Some(Self::BlockOnlyHigh),
            "BLOCK_NONE" => Some(Self::BlockNone),
            "OFF" => Some(Self::Off),
            _ => None,
        }
    }
}

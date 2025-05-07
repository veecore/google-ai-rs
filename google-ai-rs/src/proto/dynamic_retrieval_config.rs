/// The mode of the predictor to be used in dynamic retrieval.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Mode {
    /// Always trigger retrieval.
    Unspecified = 0,
    /// Run retrieval only when system decides it is necessary.
    Dynamic = 1,
}
impl Mode {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "MODE_UNSPECIFIED",
            Self::Dynamic => "MODE_DYNAMIC",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "MODE_UNSPECIFIED" => Some(Self::Unspecified),
            "MODE_DYNAMIC" => Some(Self::Dynamic),
            _ => None,
        }
    }
}

/// Supported programming languages for the generated code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Language {
    /// Unspecified language. This value should not be used.
    Unspecified = 0,
    /// Python >= 3.10, with numpy and simpy available.
    Python = 1,
}
impl Language {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "LANGUAGE_UNSPECIFIED",
            Self::Python => "PYTHON",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "LANGUAGE_UNSPECIFIED" => Some(Self::Unspecified),
            "PYTHON" => Some(Self::Python),
            _ => None,
        }
    }
}

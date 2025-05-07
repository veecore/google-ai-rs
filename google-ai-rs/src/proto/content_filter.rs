/// A list of reasons why content may have been blocked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum BlockedReason {
    /// A blocked reason was not specified.
    Unspecified = 0,
    /// Content was blocked by safety settings.
    Safety = 1,
    /// Content was blocked, but the reason is uncategorized.
    Other = 2,
}
impl BlockedReason {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "BLOCKED_REASON_UNSPECIFIED",
            Self::Safety => "SAFETY",
            Self::Other => "OTHER",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "BLOCKED_REASON_UNSPECIFIED" => Some(Self::Unspecified),
            "SAFETY" => Some(Self::Safety),
            "OTHER" => Some(Self::Other),
            _ => None,
        }
    }
}

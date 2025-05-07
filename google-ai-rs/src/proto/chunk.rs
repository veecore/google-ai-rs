/// States for the lifecycle of a `Chunk`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum State {
    /// The default value. This value is used if the state is omitted.
    Unspecified = 0,
    /// `Chunk` is being processed (embedding and vector storage).
    PendingProcessing = 1,
    /// `Chunk` is processed and available for querying.
    Active = 2,
    /// `Chunk` failed processing.
    Failed = 10,
}
impl State {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "STATE_UNSPECIFIED",
            Self::PendingProcessing => "STATE_PENDING_PROCESSING",
            Self::Active => "STATE_ACTIVE",
            Self::Failed => "STATE_FAILED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "STATE_UNSPECIFIED" => Some(Self::Unspecified),
            "STATE_PENDING_PROCESSING" => Some(Self::PendingProcessing),
            "STATE_ACTIVE" => Some(Self::Active),
            "STATE_FAILED" => Some(Self::Failed),
            _ => None,
        }
    }
}

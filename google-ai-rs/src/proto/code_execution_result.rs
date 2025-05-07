/// Enumeration of possible outcomes of the code execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Outcome {
    /// Unspecified status. This value should not be used.
    Unspecified = 0,
    /// Code execution completed successfully.
    Ok = 1,
    /// Code execution finished but with a failure. `stderr` should contain the
    /// reason.
    Failed = 2,
    /// Code execution ran for too long, and was cancelled. There may or may not
    /// be a partial output present.
    DeadlineExceeded = 3,
}
impl Outcome {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "OUTCOME_UNSPECIFIED",
            Self::Ok => "OUTCOME_OK",
            Self::Failed => "OUTCOME_FAILED",
            Self::DeadlineExceeded => "OUTCOME_DEADLINE_EXCEEDED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "OUTCOME_UNSPECIFIED" => Some(Self::Unspecified),
            "OUTCOME_OK" => Some(Self::Ok),
            "OUTCOME_FAILED" => Some(Self::Failed),
            "OUTCOME_DEADLINE_EXCEEDED" => Some(Self::DeadlineExceeded),
            _ => None,
        }
    }
}

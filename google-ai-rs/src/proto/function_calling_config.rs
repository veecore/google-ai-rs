/// Defines the execution behavior for function calling by defining the
/// execution mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Mode {
    /// Unspecified function calling mode. This value should not be used.
    Unspecified = 0,
    /// Default model behavior, model decides to predict either a function call
    /// or a natural language response.
    Auto = 1,
    /// Model is constrained to always predicting a function call only.
    /// If "allowed_function_names" are set, the predicted function call will be
    /// limited to any one of "allowed_function_names", else the predicted
    /// function call will be any one of the provided "function_declarations".
    Any = 2,
    /// Model will not predict any function call. Model behavior is same as when
    /// not passing any function declarations.
    None = 3,
}
impl Mode {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "MODE_UNSPECIFIED",
            Self::Auto => "AUTO",
            Self::Any => "ANY",
            Self::None => "NONE",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "MODE_UNSPECIFIED" => Some(Self::Unspecified),
            "AUTO" => Some(Self::Auto),
            "ANY" => Some(Self::Any),
            "NONE" => Some(Self::None),
            _ => None,
        }
    }
}

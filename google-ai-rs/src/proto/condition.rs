/// Defines the valid operators that can be applied to a key-value pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Operator {
    /// The default value. This value is unused.
    Unspecified = 0,
    /// Supported by numeric.
    Less = 1,
    /// Supported by numeric.
    LessEqual = 2,
    /// Supported by numeric & string.
    Equal = 3,
    /// Supported by numeric.
    GreaterEqual = 4,
    /// Supported by numeric.
    Greater = 5,
    /// Supported by numeric & string.
    NotEqual = 6,
    /// Supported by string only when `CustomMetadata` value type for the given
    /// key has a `string_list_value`.
    Includes = 7,
    /// Supported by string only when `CustomMetadata` value type for the given
    /// key has a `string_list_value`.
    Excludes = 8,
}
impl Operator {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "OPERATOR_UNSPECIFIED",
            Self::Less => "LESS",
            Self::LessEqual => "LESS_EQUAL",
            Self::Equal => "EQUAL",
            Self::GreaterEqual => "GREATER_EQUAL",
            Self::Greater => "GREATER",
            Self::NotEqual => "NOT_EQUAL",
            Self::Includes => "INCLUDES",
            Self::Excludes => "EXCLUDES",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "OPERATOR_UNSPECIFIED" => Some(Self::Unspecified),
            "LESS" => Some(Self::Less),
            "LESS_EQUAL" => Some(Self::LessEqual),
            "EQUAL" => Some(Self::Equal),
            "GREATER_EQUAL" => Some(Self::GreaterEqual),
            "GREATER" => Some(Self::Greater),
            "NOT_EQUAL" => Some(Self::NotEqual),
            "INCLUDES" => Some(Self::Includes),
            "EXCLUDES" => Some(Self::Excludes),
            _ => None,
        }
    }
}
/// The value type must be consistent with the value type defined in the field
/// for the corresponding key. If the value types are not consistent, the
/// result will be an empty set. When the `CustomMetadata` has a `StringList`
/// value type, the filtering condition should use `string_value` paired with
/// an INCLUDES/EXCLUDES operation, otherwise the result will also be an empty
/// set.
#[derive(Clone, PartialEq, ::prost::Oneof)]
pub enum Value {
    /// The string value to filter the metadata on.
    #[prost(string, tag = "1")]
    StringValue(::prost::alloc::string::String),
    /// The numeric value to filter the metadata on.
    #[prost(float, tag = "6")]
    NumericValue(f32),
}

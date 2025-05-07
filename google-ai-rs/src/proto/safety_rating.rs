/// The probability that a piece of content is harmful.
///
/// The classification system gives the probability of the content being
/// unsafe. This does not indicate the severity of harm for a piece of content.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum HarmProbability {
    /// Probability is unspecified.
    Unspecified = 0,
    /// Content has a negligible chance of being unsafe.
    Negligible = 1,
    /// Content has a low chance of being unsafe.
    Low = 2,
    /// Content has a medium chance of being unsafe.
    Medium = 3,
    /// Content has a high chance of being unsafe.
    High = 4,
}
impl HarmProbability {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "HARM_PROBABILITY_UNSPECIFIED",
            Self::Negligible => "NEGLIGIBLE",
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "HARM_PROBABILITY_UNSPECIFIED" => Some(Self::Unspecified),
            "NEGLIGIBLE" => Some(Self::Negligible),
            "LOW" => Some(Self::Low),
            "MEDIUM" => Some(Self::Medium),
            "HIGH" => Some(Self::High),
            _ => None,
        }
    }
}

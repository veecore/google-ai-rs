/// The state of the tuned model.
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum State {
        /// The default value. This value is unused.
        Unspecified = 0,
        /// The model is being created.
        Creating = 1,
        /// The model is ready to be used.
        Active = 2,
        /// The model failed to be created.
        Failed = 3,
    }
    impl State {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Self::Unspecified => "STATE_UNSPECIFIED",
                Self::Creating => "CREATING",
                Self::Active => "ACTIVE",
                Self::Failed => "FAILED",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "STATE_UNSPECIFIED" => Some(Self::Unspecified),
                "CREATING" => Some(Self::Creating),
                "ACTIVE" => Some(Self::Active),
                "FAILED" => Some(Self::Failed),
                _ => None,
            }
        }
    }
    /// The model used as the starting point for tuning.
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum SourceModel {
        /// Optional. TunedModel to use as the starting point for training the new
        /// model.
        #[prost(message, tag = "3")]
        TunedModelSource(super::TunedModelSource),
        /// Immutable. The name of the `Model` to tune.
        /// Example: `models/gemini-1.5-flash-001`
        #[prost(string, tag = "4")]
        BaseModel(::prost::alloc::string::String),
    }
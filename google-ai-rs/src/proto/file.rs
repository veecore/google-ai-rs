/// States for the lifecycle of a File.
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
        /// The default value. This value is used if the state is omitted.
        Unspecified = 0,
        /// File is being processed and cannot be used for inference yet.
        Processing = 1,
        /// File is processed and available for inference.
        Active = 2,
        /// File failed processing.
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
                Self::Processing => "PROCESSING",
                Self::Active => "ACTIVE",
                Self::Failed => "FAILED",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "STATE_UNSPECIFIED" => Some(Self::Unspecified),
                "PROCESSING" => Some(Self::Processing),
                "ACTIVE" => Some(Self::Active),
                "FAILED" => Some(Self::Failed),
                _ => None,
            }
        }
    }
    /// Metadata for the File.
    #[derive(Clone, Copy, PartialEq, ::prost::Oneof)]
    pub enum Metadata {
        /// Output only. Metadata for a video.
        #[prost(message, tag = "12")]
        VideoMetadata(super::VideoMetadata),
    }
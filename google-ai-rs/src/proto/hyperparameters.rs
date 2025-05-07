/// Options for specifying learning rate during tuning.
#[derive(Clone, Copy, PartialEq, ::prost::Oneof)]
pub enum LearningRateOption {
    /// Optional. Immutable. The learning rate hyperparameter for tuning.
    /// If not set, a default of 0.001 or 0.0002 will be calculated based on the
    /// number of training examples.
    #[prost(float, tag = "16")]
    LearningRate(f32),
    /// Optional. Immutable. The learning rate multiplier is used to calculate a
    /// final learning_rate based on the default (recommended) value. Actual
    /// learning rate := learning_rate_multiplier * default learning rate Default
    /// learning rate is dependent on base model and dataset size. If not set, a
    /// default of 1.0 will be used.
    #[prost(float, tag = "17")]
    LearningRateMultiplier(f32),
}

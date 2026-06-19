use crate::{
    fss::dpf::DpfKey,
    fuzzy_match::{
        dealer::SerializedFssKey,
        share_phase::SharePhaseError,
    },
    configs::{cli_config::ProtocolParameters, method_config::MethodConfig},
};

/// Method for check phase comparison
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum CheckMethod {
    GC,
    FSS,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum CheckProperty {
    Equality,
    MuBounded,
}

/// Data for check phase configuration
#[derive(Debug, Clone)]
pub enum CheckData {
    LinfGarbledCircuits,
    LinfDpf {
        fss_key: DpfKey,
        random_value: Vec<bool>,
    },
    /// Threshold value for Lp distance comparison with garbled circuits
    LpGarbledCircuits {
        /// Threshold value for comparison
        mu: u128,
    },
    /// FSS key, random value, and threshold for Lp distance comparison with IntervalFSS
    LpIntervalFSS {
        fss_key: SerializedFssKey,
        random_value: u128,
    },
}

/// Configuration for the check phase
#[derive(Debug, Clone)]
pub struct CheckConfig {
    pub h2: usize, // Input bit length of Check Phase, which is the output bit length of Share Phase
    pub h3: usize, // Output bit length of Check Phase, which is the input bit length of threshold phase
    pub d: usize, // Number of dimensions for evaluation
    pub property: CheckProperty,
    pub method: CheckMethod,
}

impl From<(ProtocolParameters, MethodConfig)> for CheckConfig {
    fn from(config: (ProtocolParameters, MethodConfig)) -> Self {
        let (protocol, methods) = config;
        let method = match methods.check_method.as_str() {
            "GC" => CheckMethod::GC,
            "FSS" => CheckMethod::FSS,
            other => panic!("Unsupported check method: {}. only support 'GC' and 'FSS'.", other),
        };

        let property = match protocol.check_property.as_str() {
            "Equality" => CheckProperty::Equality,
            "MuBounded" => CheckProperty::MuBounded,
            other => panic!("Unsupported check property: {}. Only support 'Equality' and 'MuBounded'.", other),
        };

        CheckConfig {
            h2: protocol.h2,
            h3: protocol.h3,
            d: protocol.d,
            property,
            method,
        }
    }
}

/// Error types for check phase operations
#[derive(Debug, Clone)]
pub enum CheckPhaseError {
    /// Error during share phase evaluation
    SharePhaseError(SharePhaseError),
    /// Mismatched input lengths
    InputLengthMismatch(String),
    /// Channel communication error
    ChannelError(String),
    /// Invalid configuration
    InvalidConfig(String),
}

impl From<SharePhaseError> for CheckPhaseError {
    fn from(error: SharePhaseError) -> Self {
        CheckPhaseError::SharePhaseError(error)
    }
}

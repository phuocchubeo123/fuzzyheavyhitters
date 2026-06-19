use crate::{
    fuzzy_match::dealer::SerializedFssKey,
    configs::{cli_config::ProtocolParameters, method_config::MethodConfig},
};

/// Method for threshold comparison
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ThresholdMethod {
    /// Use garbled circuits for threshold comparison
    GC,
    /// Use IntervalFSS for threshold comparison
    FSS,
}

/// Data for threshold phase configuration
#[derive(Debug, Clone)]
pub enum ThresholdData {
    /// No additional data needed for garbled circuits
    GarbledCircuits { t: u128 },
    /// FSS key and random value for IntervalFSS privacy
    IntervalFSS {
        /// FSS key for this server
        fss_key: SerializedFssKey,
        /// Random value for this server (r0 for server 0, r1 for server 1)
        random_value: u128,
    },
}

/// Configuration for the threshold phase
#[derive(Debug, Clone)]
pub struct ThresholdConfig {
    pub h3: usize,
    /// Method to use for threshold comparison
    pub method: ThresholdMethod,
}

impl From<(ProtocolParameters, MethodConfig)> for ThresholdConfig {
    fn from(config: (ProtocolParameters, MethodConfig)) -> Self {
        let (protocol, methods) = config;
        let method = match methods.threshold_method.as_str() {
            "GC" => ThresholdMethod::GC,
            "FSS" => ThresholdMethod::FSS,
            other => panic!("Unsupported check method: {}. only support 'GC' and 'FSS'.", other),
        };

        ThresholdConfig {
            h3: protocol.h3,
            method,
        }
    }
}


/// Error types for threshold phase operations
#[derive(Debug, Clone)]
pub enum ThresholdPhaseError {
    /// Channel communication error
    ChannelError(String),
    /// Invalid configuration
    InvalidConfig(String),
    /// Conversion error
    ConversionError(String),
    /// Garbled circuit error
    GarbledCircuitError(String),
}

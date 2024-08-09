use alloy::{primitives::Bytes, sol};
use eyre::{Context, Result};

// OpenZepeplin ERC20
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    ERC20,
    "./src/contracts/abi/ERC20.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    OracleRegistry,
    "./src/contracts/abi/LLMOracleRegistry.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    OracleCoordinator,
    "./src/contracts/abi/LLMOracleCoordinator.json"
);

/// Small utility to convert bytes to string.
pub fn bytes_to_string(bytes: &Bytes) -> Result<String> {
    String::from_utf8(bytes.to_vec()).wrap_err("Could not convert bytes to string")
}

/// `TaskStatus` as it appears within the coordinator.
#[derive(Debug, Clone, Copy, Default)]
pub enum TaskStatus {
    #[default]
    None,
    PendingGeneration,
    PendingValidation,
    Completed,
}

impl From<TaskStatus> for u8 {
    fn from(status: TaskStatus) -> u8 {
        status as u8
    }
}

impl TryFrom<u8> for TaskStatus {
    type Error = eyre::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TaskStatus::None),
            1 => Ok(TaskStatus::PendingGeneration),
            2 => Ok(TaskStatus::PendingValidation),
            3 => Ok(TaskStatus::Completed),
            _ => Err(eyre::eyre!("Invalid TaskStatus: {}", value)),
        }
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::None => write!(f, "None"),
            TaskStatus::PendingGeneration => write!(f, "Pending Generation"),
            TaskStatus::PendingValidation => write!(f, "Pending Validation"),
            TaskStatus::Completed => write!(f, "Completed"),
        }
    }
}

/// `OracleKind` as it appears within the registry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OracleKind {
    Generator,
    Validator,
}

impl From<OracleKind> for u8 {
    fn from(kind: OracleKind) -> u8 {
        kind as u8
    }
}

impl TryFrom<u8> for OracleKind {
    type Error = eyre::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OracleKind::Generator),
            1 => Ok(OracleKind::Validator),
            _ => Err(eyre::eyre!("Invalid OracleKind: {}", value)),
        }
    }
}

impl TryFrom<String> for OracleKind {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for OracleKind {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "g" | "gen" | "generator" => Ok(OracleKind::Generator),
            "v" | "val" | "validator" => Ok(OracleKind::Validator),
            _ => Err(eyre::eyre!("Invalid OracleKind: {}", value)),
        }
    }
}

impl std::fmt::Display for OracleKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OracleKind::Generator => write!(f, "Generator"),
            OracleKind::Validator => write!(f, "Validator"),
        }
    }
}

use alloy::primitives::U256;
use async_trait::async_trait;
use eyre::{Context, Result};
use ollama_workflows::Workflow;
use serde_json::json;

mod executor;
mod models;
mod preset;

#[async_trait]
pub trait WorkflowsExt {
    /// Executes a generation task for the given input.
    ///
    /// Returns output and metadata.
    async fn generation(&self, input: String) -> Result<(String, String)>;

    /// Executes a validation task for the given inputs.
    async fn validation(
        &self,
        input: String,
        responses: Vec<String>,
    ) -> Result<(Vec<U256>, String)>;
}

/// Utility to parse comma-separated string values, mostly read from the environment.
/// - Trims `"` from both ends at the start
/// - For each item, trims whitespace from both ends
pub fn split_comma_separated(input: Option<String>) -> Vec<String> {
    match input {
        Some(s) => s
            .trim_matches('"')
            .split(',')
            .filter_map(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(s.trim().to_string())
                }
            })
            .collect::<Vec<_>>(),
        None => vec![],
    }
}

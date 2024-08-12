use super::ProvidersExt;
use async_trait::async_trait;
use eyre::{Context, Result};
use ollama_workflows::Model;

const OPENAI_API_KEY: &str = "OPENAI_API_KEY";

/// Ollama-specific configurations.
#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    /// List of external models that are picked by the user.
    pub(crate) models: Vec<Model>,
}

impl OpenAIConfig {
    pub fn new(models: Vec<Model>) -> Self {
        Self { models }
    }
}

#[async_trait]
impl ProvidersExt for OpenAIConfig {
    async fn check_service(&self) -> Result<()> {
        log::info!("Checking OpenAI requirements");

        // just check openai
        let _ = std::env::var(OPENAI_API_KEY).wrap_err("OpenAI API key not found")?;

        Ok(())
    }

    fn describe(&self) -> String {
        format!("OpenAI with models: {:?}", self.models)
    }
}

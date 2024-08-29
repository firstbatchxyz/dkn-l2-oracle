use async_trait::async_trait;
use eyre::{eyre, Context, Result};
use ollama_rs::Ollama;
use ollama_workflows::Model;
use std::env;

use super::ProvidersExt;

const DEFAULT_OLLAMA_HOST: &str = "http://127.0.0.1";
const DEFAULT_OLLAMA_PORT: u16 = 11434;

/// Some models such as small embedding models, are hardcoded into the node.
const HARDCODED_MODELS: [&str; 1] = ["hellord/mxbai-embed-large-v1:f16"];

/// Ollama-specific configurations.
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    /// Host, usually `http://127.0.0.1`.
    pub(crate) host: String,
    /// Port, usually `11434`.
    pub(crate) port: u16,
    /// List of hardcoded models that are internally used by Ollama workflows.
    hardcoded_models: Vec<String>,
    /// List of external models that are picked by the user.
    models: Vec<Model>,
    /// Whether to automatically pull models from Ollama.
    /// This is useful for CI/CD workflows.
    auto_pull: bool,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            host: DEFAULT_OLLAMA_HOST.to_string(),
            port: DEFAULT_OLLAMA_PORT,
            hardcoded_models: HARDCODED_MODELS
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            models: Vec::default(),
            auto_pull: false,
        }
    }
}
impl OllamaConfig {
    /// Looks at the environment variables for Ollama host and port.
    ///
    /// If not found, defaults to `DEFAULT_OLLAMA_HOST` and `DEFAULT_OLLAMA_PORT`.
    pub fn new(external_models: Vec<Model>) -> Self {
        let host = env::var("OLLAMA_HOST").unwrap_or(DEFAULT_OLLAMA_HOST.to_string());
        let port = env::var("OLLAMA_PORT")
            .and_then(|port_str| port_str.parse().map_err(|_| env::VarError::NotPresent))
            .unwrap_or(DEFAULT_OLLAMA_PORT);

        // Ollama workflows may require specific models to be loaded regardless of the choices
        let hardcoded_models = HARDCODED_MODELS
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        let auto_pull = env::var("OLLAMA_AUTO_PULL").unwrap_or_default() == *"true";

        OllamaConfig {
            host,
            port,
            hardcoded_models,
            models: external_models,
            auto_pull,
        }
    }
}

#[async_trait]
impl ProvidersExt for OllamaConfig {
    /// Check if requested models exist.
    async fn check_service(&self) -> Result<()> {
        log::info!(
            "Checking Ollama requirements (auto-pull {})",
            if self.auto_pull { "on" } else { "off" }
        );

        let ollama = Ollama::new(self.host.trim_matches('"'), self.port);

        // the list of required models is those given in DKN_MODELS and the hardcoded ones
        let mut required_models = self.hardcoded_models.clone();
        required_models.extend(self.models.clone().into_iter().map(|m| m.to_string()));

        // fetch local models
        let local_models = ollama
            .list_local_models()
            .await
            .wrap_err("Could not fetch local models from Ollama, is it online?")?;

        // check that each required model exists here
        log::debug!("Checking required models: {:#?}", required_models);
        log::debug!("Found local models: {:#?}", local_models);
        for model in required_models {
            if !local_models.iter().any(|m| *m.name == model) {
                log::warn!("Model {} not found in Ollama", model);
                if self.auto_pull {
                    // if auto-pull is enabled, pull the model
                    log::info!(
                        "Downloading missing model {} (this may take a while)",
                        model
                    );
                    let status = ollama
                        .pull_model(model, false)
                        .await
                        .wrap_err("Error pulling model with Ollama")?;
                    log::debug!("Pulled model with Ollama, final status: {:#?}", status);
                } else {
                    // otherwise, give error
                    log::error!("Please download it with: ollama pull {}", model);
                    log::error!("Or, set OLLAMA_AUTO_PULL=true to pull automatically.");
                    return Err(eyre!("Required model not pulled in Ollama."));
                }
            }
        }

        Ok(())
    }

    fn describe(&self) -> String {
        format!("Ollama with models: {:?}", self.models)
    }
}

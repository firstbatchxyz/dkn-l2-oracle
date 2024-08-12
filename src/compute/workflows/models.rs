#![allow(unused)]

use super::{split_comma_separated, OllamaConfig, OpenAIConfig, ProvidersExt};
use eyre::{eyre, Result};
use ollama_workflows::{Model, ModelProvider};
use rand::seq::IteratorRandom; // provides Vec<_>.choose

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub(crate) models_providers: Vec<(ModelProvider, Model)>,
    pub(crate) providers: Vec<ModelProvider>,
    pub(crate) ollama_config: Option<OllamaConfig>,
    pub(crate) openai_config: Option<OpenAIConfig>,
}

impl std::fmt::Display for ModelConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let models_str = self
            .models_providers
            .iter()
            .map(|(provider, model)| format!("{:?}:{}", provider, model))
            .collect::<Vec<_>>()
            .join(",");
        write!(f, "{}", models_str)
    }
}

impl ModelConfig {
    /// Creates a new config with the given list of models.
    pub fn new(models: Vec<Model>) -> Self {
        // map models to (provider, model) pairs
        let models_providers = models
            .into_iter()
            .map(|m| (m.clone().into(), m))
            .collect::<Vec<_>>();

        let mut providers = Vec::new();

        // get ollama models & config
        let ollama_models = models_providers
            .iter()
            .filter_map(|(p, m)| {
                if *p == ModelProvider::Ollama {
                    Some(m.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let ollama_config = if !ollama_models.is_empty() {
            providers.push(ModelProvider::Ollama);
            Some(OllamaConfig::new(ollama_models))
        } else {
            None
        };

        // get openai models & config
        let openai_models = models_providers
            .iter()
            .filter_map(|(p, m)| {
                if *p == ModelProvider::OpenAI {
                    Some(m.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let openai_config = if !openai_models.is_empty() {
            providers.push(ModelProvider::OpenAI);
            Some(OpenAIConfig::new(openai_models))
        } else {
            None
        };

        Self {
            models_providers,
            providers,
            ollama_config,
            openai_config,
        }
    }

    /// Parses Ollama-Workflows compatible models from a comma-separated values string.
    pub fn new_from_csv(input: Option<String>) -> Self {
        let models_str = split_comma_separated(input);
        let models = models_str
            .into_iter()
            .filter_map(|s| Model::try_from(s).ok())
            .collect::<Vec<_>>();

        Self::new(models)
    }

    /// Returns the list of models in the config that match the given provider.
    pub fn get_models_for_provider(&self, provider: ModelProvider) -> Vec<Model> {
        self.models_providers
            .iter()
            .filter_map(|(p, m)| {
                if *p == provider {
                    Some(m.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Given a raw model name or provider (as a string), returns the first matching model & provider.
    ///
    /// - If input is `*` or `all`, a random model is returned.
    /// - if input is `!` the first model is returned.
    /// - If input is a model and is supported by this node, it is returned directly.
    /// - If input is a provider, the first matching model in the node config is returned.
    ///
    /// If there are no matching models with this logic, an error is returned.
    pub fn get_matching_model(&self, model_or_provider: String) -> Result<(ModelProvider, Model)> {
        if model_or_provider == "*" {
            // return a random model
            self.models_providers
                .iter()
                .choose(&mut rand::thread_rng())
                .ok_or_else(|| eyre!("No models to randomly pick for '*'."))
                .cloned()
        } else if model_or_provider == "!" {
            // return the first model
            self.models_providers
                .first()
                .ok_or_else(|| eyre!("No models to choose first for '!'."))
                .cloned()
        } else if let Ok(provider) = ModelProvider::try_from(model_or_provider.clone()) {
            // this is a valid provider, return the first matching model in the config
            self.models_providers
                .iter()
                .find(|(p, _)| *p == provider)
                .ok_or_else(|| eyre!("Provider {} is not supported by this node.", provider))
                .cloned()
        } else if let Ok(model) = Model::try_from(model_or_provider.clone()) {
            // this is a valid model, return it if it is supported by the node
            self.models_providers
                .iter()
                .find(|(_, m)| *m == model)
                .ok_or_else(|| eyre!("Model {} is not supported by this node.", model))
                .cloned()
        } else {
            // this is neither a valid provider or model for this node
            Err(eyre!(
                "Given string '{}' is not a valid model / provider identifier.",
                model_or_provider
            ))
        }
    }

    /// From a comma-separated list of model or provider names, return a random matching model & provider.
    pub fn get_any_matching_model_from_csv(
        &self,
        csv_model_or_provider: String,
    ) -> Result<(ModelProvider, Model)> {
        self.get_any_matching_model(split_comma_separated(Some(csv_model_or_provider)))
    }

    /// From a list of model or provider names, return a random matching model & provider.
    pub fn get_any_matching_model(
        &self,
        list_model_or_provider: Vec<String>,
    ) -> Result<(ModelProvider, Model)> {
        // filter models w.r.t supported ones
        let matching_models = list_model_or_provider
            .into_iter()
            .filter_map(|model_or_provider| {
                let result = self.get_matching_model(model_or_provider);
                match result {
                    Ok(result) => Some(result),
                    Err(e) => {
                        log::error!("Ignoring model: {}", e);
                        None
                    }
                }
            })
            .collect::<Vec<_>>();

        // choose random model, handles empty array case as wellk
        matching_models
            .into_iter()
            .choose(&mut rand::thread_rng())
            .ok_or_else(|| eyre!("No matching models found."))
    }

    /// Check if the required compute services are running, e.g. if Ollama
    /// is detected as a provider for the chosen models, it will check that
    /// Ollama is running.
    pub async fn check_providers(&self) -> Result<()> {
        log::info!("Checking required providers.");

        if let Some(provider) = &self.ollama_config {
            provider.check_service().await?;
            log::info!("{}", provider.describe());
        }

        if let Some(provider) = &self.openai_config {
            provider.check_service().await?;
            log::info!("{}", provider.describe());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_parser() {
        let cfg =
            ModelConfig::new_from_csv(Some("idontexist,i dont either,i332287648762".to_string()));
        assert_eq!(cfg.models_providers.len(), 0, "should have no models");

        let cfg = ModelConfig::new_from_csv(Some(
            "phi3:3.8b,phi3:14b-medium-4k-instruct-q4_1,balblablabl".to_string(),
        ));
        assert_eq!(cfg.models_providers.len(), 2, "should have some models");
    }

    #[test]
    fn test_model_matching() {
        let cfg = ModelConfig::new_from_csv(Some("gpt-3.5-turbo,phi3:3.8b".to_string()));
        assert_eq!(
            cfg.get_matching_model("openai".to_string()).unwrap().1,
            Model::GPT3_5Turbo,
            "Should find existing model"
        );

        assert_eq!(
            cfg.get_matching_model("phi3:3.8b".to_string()).unwrap().1,
            Model::Phi3Mini,
            "Should find existing model"
        );

        assert!(
            cfg.get_matching_model("gpt-4o".to_string()).is_err(),
            "Should not find anything for unsupported model"
        );

        assert!(
            cfg.get_matching_model("praise the model".to_string())
                .is_err(),
            "Should not find anything for inexisting model"
        );
    }

    #[test]
    fn test_get_any_matching_model() {
        let cfg = ModelConfig::new_from_csv(Some("gpt-3.5-turbo,phi3:3.8b".to_string()));

        let result = cfg.get_any_matching_model(vec![
            "i-dont-exist".to_string(),
            "llama3.1:latest".to_string(),
            "gpt-4o".to_string(),
            "ollama".to_string(),
        ]);
        assert_eq!(
            result.unwrap().1,
            Model::Phi3Mini,
            "Should find existing model"
        );
    }
}

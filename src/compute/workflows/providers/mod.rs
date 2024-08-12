use async_trait::async_trait;
use eyre::Result;

mod ollama;
pub use ollama::OllamaConfig;

mod openai;
pub use openai::OpenAIConfig;

#[async_trait]
pub trait ProvidersExt {
    /// Ensures that the required provider is online & ready.
    async fn check_service(&self) -> Result<()>;
}

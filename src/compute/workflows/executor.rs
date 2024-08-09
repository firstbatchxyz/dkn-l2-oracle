use alloy::primitives::U256;
use async_trait::async_trait;
use bytes::Bytes;
use eyre::{eyre, Result};
use ollama_workflows::{Entry, Executor, ProgramMemory, Workflow};

use crate::data::{arweave::Arweave, traits::OracleExternalData};

use super::preset::get_generation_workflow;

#[async_trait]
pub trait WorkflowsExt {
    async fn prepare_input(&self, input_bytes: &Bytes) -> Result<(Option<Entry>, Workflow)>;
    async fn execute_raw(&self, input_bytes: &Bytes) -> Result<(String, String)>;
}

#[async_trait]
impl WorkflowsExt for Executor {
    /// Given an input, prepares it for the executer by providing the entry and workflow.
    ///
    /// - If input is a JSON txid string (64-char hex), the entry is fetched from Arweave, and then we recurse.
    /// - If input is a JSON string, it is converted to an entry and a generation workflow is returned.
    /// - If input is a JSON workflow, entry is `None` and input is casted to a workflow
    /// - Otherwise, error is returned.
    async fn prepare_input(&self, input_bytes: &Bytes) -> Result<(Option<Entry>, Workflow)> {
        if let Ok(input_str) = serde_json::from_slice::<String>(input_bytes) {
            // this is a string, lets see if its a txid
            if Arweave::is_key(input_str.clone()) {
                // if its a txid, we download the data and parse it again
                // we dont expect to recurse here too much, because there would have to txid within txid
                // but still it is possible
                let input_bytes = Arweave::default().get(input_str).await?;
                self.prepare_input(&input_bytes).await
            } else {
                // it is not a key, so we treat it as a generation request with plaintext input
                let entry = Some(Entry::String(input_str));
                let workflow = get_generation_workflow()?;
                Ok((entry, workflow))
            }
        } else if let Ok(workflow) = serde_json::from_slice::<Workflow>(input_bytes) {
            // it is a workflow, so we can directly use it with no entry
            Ok((None, workflow))
        } else {
            Err(eyre!(
                "Could not parse input: {}",
                String::from_utf8_lossy(input_bytes)
            ))
        }
    }

    /// Executes a generation task for the given input.
    /// The workflow & entry is derived from the input.
    ///
    /// Returns output and metadata.
    async fn execute_raw(&self, input_bytes: &Bytes) -> Result<(String, String)> {
        let (entry, workflow) = self.prepare_input(input_bytes).await?;
        let mut memory = ProgramMemory::new();
        let output = self.execute(entry.as_ref(), workflow, &mut memory).await;
        Ok((output, String::default()))
    }
}

#[cfg(test)]
mod tests {
    use ollama_workflows::Model;
    use serde_json::json;

    use super::*;

    #[tokio::test]
    #[ignore = "run this manually"]
    async fn test_ollama_generation() {
        let executor = Executor::new(Model::Phi3Mini);
        let (output, _) = executor
            .execute_raw(&Bytes::from(
                json!("What is the result of 2 + 2?").to_string(),
            ))
            .await
            .unwrap();

        // funny test but it should pass
        assert!(output.contains('4'));
    }
}

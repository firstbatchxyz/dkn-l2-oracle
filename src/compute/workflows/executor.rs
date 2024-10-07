use alloy::primitives::Bytes;
use async_trait::async_trait;
use dkn_workflows::ollama_workflows::{Entry, Executor, ProgramMemory, Workflow};
use eyre::{Context, Result};

use super::postprocess::*;
use crate::data::{Arweave, OracleExternalData};

#[async_trait(?Send)]
pub trait WorkflowsExt {
    async fn prepare_input(&self, input_bytes: &Bytes) -> Result<(Option<Entry>, Workflow)>;
    async fn execute_raw(&self, input_bytes: &Bytes, protocol: &str) -> Result<(Bytes, Bytes)>;

    /// Returns a generation workflow for the executor.
    #[inline]
    fn get_generation_workflow(&self) -> Result<Workflow> {
        Ok(serde_json::from_str(include_str!(
            "presets/generation.json"
        ))?)
    }
}

#[async_trait(?Send)]
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
                let input_bytes = Arweave::default()
                    .get(input_str)
                    .await
                    .wrap_err("could not download from Arweave")?;
                self.prepare_input(&input_bytes).await
            } else {
                // it is not a key, so we treat it as a generation request with plaintext input
                let entry = Some(Entry::String(input_str));
                let workflow = self.get_generation_workflow()?;
                Ok((entry, workflow))
            }
        } else if let Ok(workflow) = serde_json::from_slice::<Workflow>(input_bytes) {
            // it is a workflow, so we can directly use it with no entry
            Ok((None, workflow))
        } else {
            // it is unparsable, return as lossy-converted string
            let input_string = String::from_utf8_lossy(input_bytes);
            let entry = Some(Entry::String(input_string.into()));
            let workflow = self.get_generation_workflow()?;
            Ok((entry, workflow))
        }
    }

    /// Executes a generation task for the given input.
    /// The workflow & entry is derived from the input.
    ///
    /// Returns output and metadata.
    async fn execute_raw(&self, input_bytes: &Bytes, protocol: &str) -> Result<(Bytes, Bytes)> {
        // parse & prepare input
        let (entry, workflow) = self.prepare_input(input_bytes).await?;

        // obtain raw output
        let mut memory = ProgramMemory::new();
        let output = self.execute(entry.as_ref(), workflow, &mut memory).await?;
        log::debug!("Output: {}", output);

        // post-process output w.r.t protocol
        match protocol.split('/').next().unwrap_or_default() {
            SwanPurchasePostProcessor::PROTOCOL => {
                SwanPurchasePostProcessor::new("<shop_list>", "</shop_list>").post_process(output)
            }
            _ => IdentityPostProcessor.post_process(output),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_parse_input_string() {
        let executor = Executor::new(Default::default());
        let input_str = "foobar";

        let (entry, _) = executor
            .prepare_input(&input_str.as_bytes().into())
            .await
            .unwrap();
        assert_eq!(entry.unwrap(), Entry::String(input_str.into()));
    }

    #[tokio::test]
    async fn test_parse_input_arweave() {
        let executor = Executor::new(Default::default());

        // hex for: Zg6CZYfxXCWYnCuKEpnZCYfy7ghit1_v4-BCe53iWuA
        // contains the string: "\"Hello, Arweave!\"" which will be parsed again within
        let arweave_key = "\"660e826587f15c25989c2b8a1299d90987f2ee0862b75fefe3e0427b9de25ae0\"";
        let expected_str = "Hello, Arweave!";

        let (entry, _) = executor
            .prepare_input(&arweave_key.as_bytes().into())
            .await
            .unwrap();
        assert_eq!(entry.unwrap(), Entry::String(expected_str.into()));
    }

    #[tokio::test]
    async fn test_parse_input_workflow() {
        let executor = Executor::new(Default::default());

        let workflow_str = include_str!("presets/generation.json");
        let (entry, _) = executor
            .prepare_input(&workflow_str.as_bytes().into())
            .await
            .unwrap();

        // if Workflow was parsed correctly, entry should be None
        assert!(entry.is_none());
    }
}

use alloy::primitives::Bytes;
use async_trait::async_trait;
use dkn_workflows::{Entry, Executor, ProgramMemory, Workflow};
use eyre::{Context, Result};

use super::postprocess::*;
use crate::data::{Arweave, OracleExternalData};

#[async_trait(?Send)]
pub trait WorkflowsExt {
    async fn parse_input_bytes(&self, input_bytes: &Bytes) -> Result<(Option<Entry>, Workflow)>;
    async fn parse_input_string(&self, input_str: String) -> Result<(Option<Entry>, Workflow)>;
    async fn execute_raw(
        &self,
        input_bytes: &Bytes,
        protocol: &str,
    ) -> Result<(Bytes, Bytes, bool)>;

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
    /// Given an input of byte-slice, parses it to an entry and workflow.
    ///
    /// - If input is a byteslice of JSON string, it is passed to `parse_input_string`.
    /// - If input is a byteslice of JSON workflow, entry is `None` and input is casted to a workflow.
    async fn parse_input_bytes(&self, input_bytes: &Bytes) -> Result<(Option<Entry>, Workflow)> {
        if let Ok(input_str) = serde_json::from_slice::<String>(input_bytes) {
            self.parse_input_string(input_str).await
        } else if let Ok(workflow) = serde_json::from_slice::<Workflow>(input_bytes) {
            // it is a workflow, so we can directly use it with no entry
            Ok((None, workflow))
        } else {
            // it is unparsable, return as lossy-converted string
            let input_str = String::from_utf8_lossy(input_bytes);
            self.parse_input_string(input_str.into()).await
        }
    }

    /// Given an input of string, parses it to an entry and workflow.
    ///
    /// - If input is a txid (64-char hex, without 0x), the entry is fetched from Arweave, and then we recurse back to `parse_input_bytes`.
    /// - Otherwise, it is treated as a plaintext input and a generation workflow is returned.
    async fn parse_input_string(&self, input_string: String) -> Result<(Option<Entry>, Workflow)> {
        if Arweave::is_key(input_string.clone()) {
            // if its a txid, we download the data and parse it again
            let input_bytes = Arweave::default()
                .get(input_string)
                .await
                .wrap_err("could not download from Arweave")?;

            // we dont expect to recurse here again too much, because there would have to txid within txid
            self.parse_input_bytes(&input_bytes).await
        } else {
            // it is not a key, so we treat it as a generation request with plaintext input
            let entry = Some(Entry::String(input_string));
            let workflow = self.get_generation_workflow()?;
            Ok((entry, workflow))
        }
    }

    /// Executes a generation task for the given input.
    /// The workflow & entry is derived from the input.
    ///
    /// Returns output, metadata, and a boolean indicating whether we shall upload the `output` to storage if large enough.
    async fn execute_raw(
        &self,
        input_bytes: &Bytes,
        protocol: &str,
    ) -> Result<(Bytes, Bytes, bool)> {
        // parse & prepare input
        let (entry, workflow) = self.parse_input_bytes(input_bytes).await?;

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
            .parse_input_bytes(&input_str.as_bytes().into())
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
            .parse_input_bytes(&arweave_key.as_bytes().into())
            .await
            .unwrap();
        assert_eq!(entry.unwrap(), Entry::String(expected_str.into()));

        // without `"`s
        let (entry, _) = executor
            .parse_input_bytes(&arweave_key.trim_matches('"').as_bytes().into())
            .await
            .unwrap();
        assert_eq!(entry.unwrap(), Entry::String(expected_str.into()));
    }

    #[tokio::test]
    async fn test_parse_input_workflow() {
        let executor = Executor::new(Default::default());

        let workflow_str = include_str!("presets/generation.json");
        let (entry, _) = executor
            .parse_input_bytes(&workflow_str.as_bytes().into())
            .await
            .unwrap();

        // if Workflow was parsed correctly, entry should be None
        assert!(entry.is_none());
    }
}

use alloy::primitives::{Bytes, U256};
use dkn_workflows::{Entry, Executor, MessageInput, Model, ProgramMemory, Workflow};
use eyre::{eyre, Context, Result};

use super::parse_downloadable;
use crate::DriaOracle;

/// An oracle validation request.
#[derive(Debug)]
pub struct ValidationRequest {
    task_id: U256,
    num_generations: usize,
}

impl ValidationRequest {
    /// Executes a request using the given model, and optionally a node.
    /// Returns the raw string output.
    pub async fn retrieve_generation_metadatas(
        &mut self,
        node: &DriaOracle,
    ) -> Result<Vec<String>> {
        let responses = node
            .get_task_responses(self.task_id.clone())
            .await
            .wrap_err("could not get generation")?;

        // parse metadata from all responses
        let mut metadatas = Vec::new();
        for response in responses {
            let metadata_str = parse_downloadable(&response.metadata).await?;
            metadatas.push(metadata_str);
        }

        Ok(metadatas)
    }

    /// Executes a validation request using the given model.
    pub async fn execute(&mut self, model: Model) -> Result<String> {
        log::debug!("Executing validation request with: {}", model);
        let mut memory = ProgramMemory::new();
        let executor = Executor::new(model);

        // TODO: !!!

        Ok(Default::default())
    }
}

#[cfg(test)]
mod tests {
    use alloy::hex::FromHex;

    use super::*;

    #[tokio::test]
    async fn test_validation() {
        let input = "What is 2 + 2";
        let generations = [
            "2 + 2 is 4.",
            "2 + 2 is 889992.",
            "Bonito applebum",
            "2 + 2 is 4 because apples are green.",
        ];
    }
}

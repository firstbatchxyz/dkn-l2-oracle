use alloy::primitives::Bytes;
use dkn_workflows::{Entry, Executor, Model, ProgramMemory, Workflow};
use eyre::{Context, Result};

use super::{chat::ChatHistoryRequest, postprocess::*};
use crate::{
    bytes_to_string,
    data::{Arweave, OracleExternalData},
};

/// Returns a generation workflow for the executor.
///
/// This should be used when the input is simple a string.
#[inline]
pub fn generation_workflow() -> Result<Workflow> {
    serde_json::from_str(include_str!("presets/generation.json"))
        .wrap_err("could not parse workflow")
}

#[derive(Debug)]
pub enum InputType {
    ChatHistory(ChatHistoryRequest),
    Workflow(Workflow),
    String(String),
}

impl InputType {
    pub async fn post_process(output: String, protocol: &str) -> Result<(Bytes, Bytes, bool)> {
        match protocol.split('/').next().unwrap_or_default() {
            SwanPurchasePostProcessor::PROTOCOL => {
                SwanPurchasePostProcessor::new("<shop_list>", "</shop_list>").post_process(output)
            }
            _ => IdentityPostProcessor.post_process(output),
        }
    }

    pub async fn execute(&self, model: Model) -> Result<String> {
        log::debug!("Executing workflow with: {}", model);
        let mut memory = ProgramMemory::new();
        let executor = Executor::new(model);

        match self {
            Self::Workflow(workflow) => executor.execute(None, workflow, &mut memory).await,
            Self::String(input) => {
                // TODO: can use `lazy_static` for this guy
                let workflow = generation_workflow()?;
                let entry = Entry::String(input.clone());
                executor.execute(Some(&entry), &workflow, &mut memory).await
            }
            Self::ChatHistory(chat_history) => {
                todo!("TODO: history execution will be implemented")
            }
        }
        .wrap_err("could not execute workflow")
    }

    /// Given an input of byte-slice, parses it into a valid input type.
    pub async fn try_parse_bytes(input_bytes: &Bytes) -> Result<Self> {
        // first, convert to string
        let mut input_string = bytes_to_string(input_bytes)?;

        // then, check storage
        if Arweave::is_key(input_string.clone()) {
            // if its a txid, we download the data and parse it again
            let input_bytes_from_arweave = Arweave::default()
                .get(input_string)
                .await
                .wrap_err("could not download from Arweave")?;

            // convert the input to string
            input_string = bytes_to_string(&input_bytes_from_arweave)?;
        }

        // parse input again
        if let Ok(chat_input) = serde_json::from_str::<ChatHistoryRequest>(&input_string) {
            Ok(InputType::ChatHistory(chat_input))
        } else if let Ok(workflow) = serde_json::from_str::<Workflow>(&input_string) {
            Ok(InputType::Workflow(workflow))
        } else {
            Ok(InputType::String(input_string))
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_parse_input_string() {
        let input_str = "foobar";
        let entry = InputType::try_parse_bytes(&input_str.as_bytes().into()).await;
        assert_eq!(entry.unwrap(), InputType::String(input_str.into()));
    }

    #[tokio::test]
    async fn test_parse_input_arweave() {
        // contains the string: "\"Hello, Arweave!\""
        // hex for: Zg6CZYfxXCWYnCuKEpnZCYfy7ghit1_v4-BCe53iWuA
        let arweave_key = "660e826587f15c25989c2b8a1299d90987f2ee0862b75fefe3e0427b9de25ae0";
        let expected_str = "\"Hello, Arweave!\"";

        let (entry, _) = executor
            .parse_input_bytes(&arweave_key.as_bytes().into())
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

    #[tokio::test]
    #[ignore = "run this manually"]
    async fn test_ollama_generation() {
        dotenvy::dotenv().unwrap();
        let executor = Executor::new(Model::Llama3_1_8B);
        let (output, _, _) = executor
            .execute_raw(&Bytes::from_static(b"What is the result of 2 + 2?"), "")
            .await
            .unwrap();

        // funny test but it should pass
        println!("Output:\n{}", output);
        // assert!(output.contains('4')); // FIXME: make this use bytes
    }

    #[tokio::test]
    #[ignore = "run this manually"]
    async fn test_openai_generation() {
        dotenvy::dotenv().unwrap();
        let executor = Executor::new(Model::Llama3_1_8B);
        let (output, _, _) = executor
            .ex(&Bytes::from_static(b"What is the result of 2 + 2?"), "")
            .await
            .unwrap();

        // funny test but it should pass
        println!("Output:\n{}", output);
        // assert!(output.contains('4')); // FIXME: make this use bytes
    }

    /// Test the generation workflow with a plain input.
    #[tokio::test]
    async fn test_workflow_plain() {
        dotenvy::dotenv().unwrap();
        let executor = Executor::new(Model::GPT4o);
        let mut memory = ProgramMemory::new();
        let workflow = executor.get_generation_workflow().unwrap();
        let input = Entry::try_value_or_str("What is 2 + 2?");
        let output = executor
            .execute(Some(&input), workflow, &mut memory)
            .await
            .unwrap();
        println!("Output:\n{}", output);
    }
}

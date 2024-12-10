use alloy::primitives::{Bytes, U256};
use dkn_workflows::{Executor, MessageInput, Model, ProgramMemory, Workflow};
use eyre::{eyre, Context, Result};

use super::{
    postprocess::{self, *},
    workflow::*,
};
use crate::{compute::parse_downloadable, DriaOracle};

/// A request with chat history.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ChatHistoryRequest {
    /// Task Id of which the output will act like history.
    pub history_id: usize,
    /// Message content.
    pub content: String,
}

/// An oracle request.
#[derive(Debug)]
pub enum GenerationRequest {
    /// A chat-history request.
    ChatHistory(ChatHistoryRequest),
    /// The request itself is a Workflow object, we execute it directly.
    Workflow(Workflow),
    /// The request is a plain string, we execute it within a generation workflow.
    String(String),
}

impl GenerationRequest {
    pub fn request_type(&self) -> &str {
        match self {
            Self::ChatHistory(_) => "chat",
            Self::Workflow(_) => "workflow",
            Self::String(_) => "string",
        }
    }

    /// Given an input of byte-slice, parses it into a valid request type.
    pub async fn try_parse_bytes(input_bytes: &Bytes) -> Result<Self> {
        let input_string = parse_downloadable(input_bytes).await?;
        log::debug!("Parsing input string: {}", input_string);
        Ok(Self::try_parse_string(input_string).await)
    }

    /// Given an input of string, parses it into a valid request type.
    pub async fn try_parse_string(input_string: String) -> Self {
        if let Ok(chat_input) = serde_json::from_str::<ChatHistoryRequest>(&input_string) {
            GenerationRequest::ChatHistory(chat_input)
        } else if let Ok(workflow) = serde_json::from_str::<Workflow>(&input_string) {
            GenerationRequest::Workflow(workflow)
        } else {
            GenerationRequest::String(input_string)
        }
    }

    /// Executes a request using the given model, and optionally a node.
    /// Returns the raw string output.
    pub async fn execute(&mut self, model: Model, node: Option<&DriaOracle>) -> Result<String> {
        log::debug!(
            "Executing {} generation request with: {}",
            self.request_type(),
            model
        );
        let mut memory = ProgramMemory::new();
        let executor = Executor::new(model);

        match self {
            // workflows are executed directly without any prompts
            // as we expect their memory to be pre-filled
            Self::Workflow(workflow) => executor
                .execute(None, workflow, &mut memory)
                .await
                .wrap_err("could not execute worfklow input"),

            // string requests are used with the generation workflow with a given prompt
            Self::String(input) => {
                let workflow = make_generation_workflow(input.clone())?;
                executor
                    .execute(None, &workflow, &mut memory)
                    .await
                    .wrap_err("could not execute worfklow for string input")
            }

            // chat history requests are used with the chat workflow
            // and the existing history is fetched & parsed from previous requests
            Self::ChatHistory(chat_request) => {
                let mut history = if chat_request.history_id == 0 {
                    // if task id is zero, there is no prior history
                    Vec::new()
                } else if let Some(node) = node {
                    // if task id is non-zero, we need the node to get the history
                    let history_task = node
                        .get_task_best_response(U256::from(chat_request.history_id))
                        .await
                        .wrap_err("could not get chat history task from contract")?;

                    // parse it as chat history output
                    let history_str = parse_downloadable(&history_task.output).await?;

                    serde_json::from_str::<Vec<MessageInput>>(&history_str)?
                } else {
                    return Err(eyre!("node is required for chat history"));
                };

                // prepare the workflow with chat history
                let workflow = make_chat_workflow(history.clone(), chat_request.content.clone())?;
                let output = executor
                    .execute(None, &workflow, &mut memory)
                    .await
                    .wrap_err("could not execute chat worfklow")?;

                // append user input to chat history
                history.push(MessageInput {
                    role: "assistant".to_string(),
                    content: output,
                });

                // return the stringified output
                let out =
                    serde_json::to_string(&history).wrap_err("could not serialize chat history")?;

                Ok(out)
            }
        }
    }

    /// Post-processes the output string based on the given protocol.
    /// Returns the output, metadata, and a boolean indicating if storage is allowed or not.
    ///
    /// If the protocol is not recognized, it defaults to `IdentityPostProcessor`.
    pub async fn post_process(output: String, protocol: &str) -> Result<(Bytes, Bytes, bool)> {
        match protocol.split('/').next().unwrap_or_default() {
            SwanPurchasePostProcessor::PROTOCOL => {
                SwanPurchasePostProcessor::new("<shop_list>", "</shop_list>").post_process(output)
            }
            _ => postprocess::IdentityPostProcessor.post_process(output),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy::hex::FromHex;

    use super::*;

    // only implemented for testing purposes
    // because `Workflow` does not implement `PartialEq`
    impl PartialEq for GenerationRequest {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::ChatHistory(a), Self::ChatHistory(b)) => {
                    a.content == b.content && a.history_id == b.history_id
                }
                (Self::Workflow(_), Self::Workflow(_)) => true, // not implemented
                (Self::String(a), Self::String(b)) => a == b,
                _ => false,
            }
        }
    }

    #[tokio::test]
    async fn test_parse_request_string() {
        let request_str = "foobar";
        let entry = GenerationRequest::try_parse_bytes(&request_str.as_bytes().into()).await;
        assert_eq!(
            entry.unwrap(),
            GenerationRequest::String(request_str.into())
        );
    }

    #[tokio::test]
    async fn test_parse_request_arweave() {
        // contains the string: "\"Hello, Arweave!\""
        // hex for: Zg6CZYfxXCWYnCuKEpnZCYfy7ghit1_v4-BCe53iWuA
        let arweave_key = "660e826587f15c25989c2b8a1299d90987f2ee0862b75fefe3e0427b9de25ae0";
        let expected_str = "\"Hello, Arweave!\"";

        let entry = GenerationRequest::try_parse_bytes(&arweave_key.as_bytes().into()).await;
        assert_eq!(
            entry.unwrap(),
            GenerationRequest::String(expected_str.into())
        );
    }

    #[tokio::test]
    async fn test_parse_request_chat() {
        let request = ChatHistoryRequest {
            history_id: 0,
            content: "foobar".to_string(),
        };
        let request_bytes = serde_json::to_vec(&request).unwrap();
        let entry = GenerationRequest::try_parse_bytes(&request_bytes.into()).await;
        assert_eq!(entry.unwrap(), GenerationRequest::ChatHistory(request));
    }

    #[tokio::test]
    async fn test_parse_request_workflow() {
        // task 21402 input
        // 0x30306234343365613266393739626263353263613565363131376534646366353634366662316365343265663566643363643564646638373533643538323463
        let input_bytes = Bytes::from_hex("30306234343365613266393739626263353263613565363131376534646366353634366662316365343265663566643363643564646638373533643538323463").unwrap();
        let workflow = GenerationRequest::try_parse_bytes(&input_bytes)
            .await
            .unwrap();
        if let GenerationRequest::Workflow(_) = workflow {
            /* do nothing */
        } else {
            panic!("Expected workflow, got something else");
        }
    }

    #[tokio::test]
    #[ignore = "run this manually"]
    async fn test_ollama_generation() {
        dotenvy::dotenv().unwrap();
        let mut request = GenerationRequest::String("What is the result of 2 + 2?".to_string());
        let output = request.execute(Model::Llama3_1_8B, None).await.unwrap();

        println!("Output:\n{}", output);
        assert!(output.contains('4'));
    }

    #[tokio::test]
    #[ignore = "run this manually"]
    async fn test_openai_generation() {
        dotenvy::dotenv().unwrap();
        let mut request = GenerationRequest::String("What is the result of 2 + 2?".to_string());
        let output = request.execute(Model::GPT4Turbo, None).await.unwrap();

        println!("Output:\n{}", output);
        assert!(output.contains('4'));
    }

    #[tokio::test]
    #[ignore = "run this manually"]
    async fn test_openai_chat() {
        dotenvy::dotenv().unwrap();
        let request = ChatHistoryRequest {
            history_id: 0,
            content: "What is 2+2?".to_string(),
        };
        let request_bytes = serde_json::to_vec(&request).unwrap();
        let mut request = GenerationRequest::try_parse_bytes(&request_bytes.into())
            .await
            .unwrap();
        let output = request.execute(Model::GPT4Turbo, None).await.unwrap();

        println!("Output:\n{}", output);
        assert!(output.contains('4'));
    }
}

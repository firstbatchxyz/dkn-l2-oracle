use crate::{storage::ArweaveStorage, DriaOracle};
use alloy::primitives::U256;
use dkn_workflows::{Executor, MessageInput, Model, ProgramMemory};
use eyre::{eyre, Context, Result};

use super::request::GenerationRequest;
use super::workflow::*;

/// Executes a request using the given model, and optionally a node.
/// Returns the raw string output.
pub async fn execute_generation(
    request: &GenerationRequest,
    model: Model,
    node: Option<&DriaOracle>,
) -> Result<String> {
    log::debug!(
        "Executing {} generation request with: {}",
        request.request_type(),
        model
    );
    let mut memory = ProgramMemory::new();
    let executor = Executor::new(model);

    match request {
        // workflows are executed directly without any prompts
        // as we expect their memory to be pre-filled
        GenerationRequest::Workflow(workflow) => executor
            .execute(None, workflow, &mut memory)
            .await
            .wrap_err("could not execute worfklow input"),

        // string requests are used with the generation workflow with a given prompt
        GenerationRequest::String(input) => {
            let (workflow, duration) = make_generation_workflow(input.clone())?;
            tokio::select! {
                result = executor.execute(None, &workflow, &mut memory) => result.wrap_err("could not execute worfklow for string input"),
                _ = tokio::time::sleep(duration) => {
                    Err(eyre!("Generation workflow timed out"))
                }
            }
        }

        // chat history requests are used with the chat workflow
        // and the existing history is fetched & parsed from previous requests
        GenerationRequest::ChatHistory(chat_request) => {
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
                let history_str = ArweaveStorage::parse_downloadable(&history_task.output).await?;

                // if its a previous message array, we can parse it directly
                if let Ok(messages) = serde_json::from_str::<Vec<MessageInput>>(&history_str) {
                    messages
                } else {
                    // otherwise, we can fallback to fetching input manually and creating a new history on-the-fly
                    let request = node
                        .get_task_request(U256::from(chat_request.history_id))
                        .await?;
                    let input = ArweaveStorage::parse_downloadable(&request.input).await?;

                    // create a new history with the input
                    vec![
                        MessageInput::new_user_message(input),
                        MessageInput::new_assistant_message(history_str),
                    ]
                }
            } else {
                return Err(eyre!("node is required for chat history"));
            };

            // prepare the workflow with chat history
            let (workflow, duration) =
                make_chat_workflow(history.clone(), chat_request.content.clone())?;
            let output = tokio::select! {
                result = executor.execute(None, &workflow, &mut memory) => result.wrap_err("could not execute chat worfklow")?,
                _ = tokio::time::sleep(duration) => {
                    return Err(eyre!("Generation workflow timed out"));
                }
            };

            // append user input to chat history
            history.push(MessageInput::new_assistant_message(output));

            // return the stringified output
            let out =
                serde_json::to_string(&history).wrap_err("could not serialize chat history")?;

            Ok(out)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compute::generation::request::{ChatHistoryRequest, GenerationRequest};
    use dkn_workflows::Model;

    #[tokio::test]
    #[ignore = "run this manually"]
    async fn test_ollama_generation() {
        dotenvy::dotenv().unwrap();
        let request = GenerationRequest::String("What is the result of 2 + 2?".to_string());
        let output = execute_generation(&request, Model::Llama3_1_8B, None)
            .await
            .unwrap();

        println!("Output:\n{}", output);
        assert!(output.contains('4'));
    }

    #[tokio::test]
    #[ignore = "run this manually"]
    async fn test_openai_generation() {
        dotenvy::dotenv().unwrap();
        let request = GenerationRequest::String("What is the result of 2 + 2?".to_string());
        let output = execute_generation(&request, Model::GPT4Turbo, None)
            .await
            .unwrap();

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
        let request = GenerationRequest::try_parse_bytes(&request_bytes.into())
            .await
            .unwrap();
        let output = execute_generation(&request, Model::GPT4Turbo, None)
            .await
            .unwrap();

        println!("Output:\n{}", output);
        assert!(output.contains('4'));
    }
}

use alloy::primitives::{Bytes, U256};
use dkn_workflows::{Entry, Executor, MessageInput, Model, ProgramMemory, Workflow};
use eyre::{eyre, Context, Result};

use super::{postprocess::*, presets::*};
use crate::{
    bytes_to_string,
    data::{Arweave, OracleExternalData},
    DriaOracle,
};

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
pub enum Request {
    /// A chat-history request.
    ChatHistory(ChatHistoryRequest),
    /// The request itself is a Workflow object, we execute it directly.
    Workflow(Workflow),
    /// The request is a plain string, we execute it within a generation workflow.
    String(String),
}

impl Request {
    pub fn request_type(&self) -> &str {
        match self {
            Self::ChatHistory(_) => "chat",
            Self::Workflow(_) => "workflow",
            Self::String(_) => "string",
        }
    }

    /// Given an input of byte-slice, parses it into a valid request type.
    pub async fn try_parse_bytes(input_bytes: &Bytes) -> Result<Self> {
        let input_string = Self::parse_downloadable(input_bytes).await?;
        log::debug!("Parsing input string: {}", input_string);
        Ok(Self::try_parse_string(input_string).await)
    }

    /// Given an input of string, parses it into a valid request type.
    pub async fn try_parse_string(input_string: String) -> Self {
        if let Ok(chat_input) = serde_json::from_str::<ChatHistoryRequest>(&input_string) {
            Request::ChatHistory(chat_input)
        } else if let Ok(workflow) = serde_json::from_str::<Workflow>(&input_string) {
            Request::Workflow(workflow)
        } else {
            Request::String(input_string)
        }
    }

    /// Executes a request using the given model, and optionally a node.
    /// Returns the raw string output.
    pub async fn execute(&mut self, model: Model, node: Option<&DriaOracle>) -> Result<String> {
        log::debug!("Executing {} request with: {}", self.request_type(), model);
        let mut memory = ProgramMemory::new();
        let executor = Executor::new(model);

        match self {
            Self::Workflow(workflow) => executor
                .execute(None, workflow, &mut memory)
                .await
                .wrap_err("could not execute worfklow input"),

            Self::String(input) => {
                let entry = Entry::String(input.clone());
                executor
                    .execute(Some(&entry), &GENERATION_WORKFLOW, &mut memory)
                    .await
                    .wrap_err("could not execute worfklow for string input")
            }

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
                    let history_str = Self::parse_downloadable(&history_task.output).await?;

                    serde_json::from_str::<Vec<MessageInput>>(&history_str)?
                } else {
                    return Err(eyre!("node is required for chat history"));
                };

                // append user input to chat history
                history.push(MessageInput {
                    role: "user".to_string(),
                    content: chat_request.content.clone(),
                });

                // prepare the workflow with chat history
                let mut workflow = get_chat_workflow();
                let task = workflow.get_tasks_by_id_mut("A").unwrap();
                task.messages = history.clone();

                // call workflow
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

    pub async fn post_process(output: String, protocol: &str) -> Result<(Bytes, Bytes, bool)> {
        match protocol.split('/').next().unwrap_or_default() {
            SwanPurchasePostProcessor::PROTOCOL => {
                SwanPurchasePostProcessor::new("<shop_list>", "</shop_list>").post_process(output)
            }
            _ => IdentityPostProcessor.post_process(output),
        }
    }

    /// Parses a given bytes input to a string, and if it is a storage key identifier it automatically
    /// downloads the data from Arweave.
    pub async fn parse_downloadable(input_bytes: &Bytes) -> Result<String> {
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

        Ok(input_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // only implemented for testing purposes
    // because `Workflow` does not implement `PartialEq`
    impl PartialEq for Request {
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
        let entry = Request::try_parse_bytes(&request_str.as_bytes().into()).await;
        assert_eq!(entry.unwrap(), Request::String(request_str.into()));
    }

    #[tokio::test]
    async fn test_parse_request_arweave() {
        // contains the string: "\"Hello, Arweave!\""
        // hex for: Zg6CZYfxXCWYnCuKEpnZCYfy7ghit1_v4-BCe53iWuA
        let arweave_key = "660e826587f15c25989c2b8a1299d90987f2ee0862b75fefe3e0427b9de25ae0";
        let expected_str = "\"Hello, Arweave!\"";

        let entry = Request::try_parse_bytes(&arweave_key.as_bytes().into()).await;
        assert_eq!(entry.unwrap(), Request::String(expected_str.into()));
    }

    #[tokio::test]
    async fn test_parse_request_workflow() {
        let workflow_str = include_str!("../presets/generation.json");
        let expected_workflow = serde_json::from_str::<Workflow>(&workflow_str).unwrap();

        let entry = Request::try_parse_bytes(&workflow_str.as_bytes().into()).await;
        assert_eq!(entry.unwrap(), Request::Workflow(expected_workflow));
    }

    #[tokio::test]
    async fn test_parse_request_chat() {
        let request = ChatHistoryRequest {
            history_id: 0,
            content: "foobar".to_string(),
        };
        let request_bytes = serde_json::to_vec(&request).unwrap();
        let entry = Request::try_parse_bytes(&request_bytes.into()).await;
        assert_eq!(entry.unwrap(), Request::ChatHistory(request));
    }

    #[tokio::test]
    #[ignore = "run this manually"]
    async fn test_ollama_generation() {
        dotenvy::dotenv().unwrap();
        let mut request = Request::String("What is the result of 2 + 2?".to_string());
        let output = request.execute(Model::Llama3_1_8B, None).await.unwrap();

        println!("Output:\n{}", output);
        assert!(output.contains('4'));
    }

    #[tokio::test]
    #[ignore = "run this manually"]
    async fn test_openai_generation() {
        dotenvy::dotenv().unwrap();
        let mut request = Request::String("What is the result of 2 + 2?".to_string());
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
        let mut request = Request::try_parse_bytes(&request_bytes.into())
            .await
            .unwrap();
        let output = request.execute(Model::GPT4Turbo, None).await.unwrap();

        println!("Output:\n{}", output);
        assert!(output.contains('4'));
    }

    #[tokio::test]
    #[ignore = "run this manually with GPT4oMini vs GPT4o"]
    async fn test_erroneous() {
        dotenvy::dotenv().unwrap();
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init();

        // this looks like a workflow, but its not parseable so it should give an errror
        // let input = "{\"config\":{\"max_steps\":50,\"max_time\":200,\"tools\":[\"ALL\"]},\"external_memory\":{\"backstory\":\"dark hacker\\nlives in the basement of his parents' house\",\"objective\":\"Hacking systems\",\"behaviour\":\"closed but strong\",\"state\":\"\",\"inventory\":[\"Empty Inventory\"]},\"tasks\":[{\"id\":\"simulate\",\"name\":\"State\",\"description\":\"Simulates from the given state to obtain a new state with respect to the given inputs.\",\"prompt\":\"You are a sophisticated 317-dimensional alien world simulator capable of simulating any fictional or non-fictional world with excellent detail. Your task is to simulate one day in the life of a character based on the provided inputs, taking into account every given detail to accurately mimic the created world.\\n\\n---------------------\\n\\nYou just woke up to a new day. When you look at mirror as you wake up, you reflect on yourself and who you are. You are:\\n<backstory>\\n{{backstory}}\\n</backstory>\\n\\nYou remember vividly what drove you in your life. You feel a strong urge to:\\n<objective>\\n{{objective}}\\n</objective>\\n\\n\\nTo be strong and coherent, you repeat out loud how you behave in front of the mirror.\\n<behaviour>\\n{{behaviour}}\\n</behaviour>\\n\\nAs you recall who you are, what you do and your drive is, you write down to a notebook your current progress with your goal:\\n<current_state>\\n{{state}}\\n</current_state>\\n\\nYou look through and see the items in your inventory.\\n<inventory>\\n{{inventory}}\\n</inventory>\\n\\nFirst, an omnipotent being watches you through out the day outlining what you've been through today within your world in <observe> tags. This being is beyond time and space can understand slightest intentions also the complex infinite parameter world around you.\\n\\nYou live another day... It's been a long day and you write down your journal what you've achieved so far today, and what is left with your ambitions. It's only been a day, so you know that you can achieve as much that is possible within a day. \\n\\nWrite this between <journal> tags.\\nStart now:\\n\",\"inputs\":[{\"name\":\"backstory\",\"value\":{\"type\":\"read\",\"key\":\"backstory\"},\"required\":true},{\"name\":\"state\",\"value\":{\"type\":\"read\",\"key\":\"state\"},\"required\":true},{\"name\":\"inventory\",\"value\":{\"type\":\"get_all\",\"key\":\"inventory\"},\"required\":true},{\"name\":\"behaviour\",\"value\":{\"type\":\"read\",\"key\":\"behaviour\"},\"required\":true},{\"name\":\"objective\",\"value\":{\"type\":\"read\",\"key\":\"objective\"},\"required\":true}],\"operator\":\"generation\",\"outputs\":[{\"type\":\"write\",\"key\":\"new_state\",\"value\":\"__result\"}]},{\"id\":\"_end\",\"name\":\"Task\",\"description\":\"Task Description\",\"prompt\":\"\",\"inputs\":[],\"operator\":\"end\",\"outputs\":[]}],\"steps\":[{\"source\":\"simulate\",\"target\":\"_end\"}],\"return_value\":{\"input\":{\"type\":\"read\",\"key\":\"new_state\"},\"to_json\":false}}";
        let input = Bytes::from_static(&hex_literal::hex!("36623630613364623161396663353163313532383663396539393664363531626633306535626438363730386262396134636339633863636632393236623266"));

        let mut request = Request::try_parse_bytes(&input).await.unwrap();
        // this is a wrong Workflow object, so instead of being parsed as Request::Workflow it is parsed as Request::String!
        // small models like GPT4oMini will not be able to handle this, and will output mumbo-jumbo at random times, sometimes will return the input itself
        // Gpt4o will be able to handle this, it actually understands the task

        let output = request.execute(Model::GPT4o, None).await.unwrap();

        println!("Output:\n{}", output);
    }
}

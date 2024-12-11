use crate::storage::ArweaveStorage;
use alloy::primitives::Bytes;
use dkn_workflows::Workflow;
use eyre::Result;

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
    /// Returns hte name of the request type, mostly for diagnostics.
    pub fn request_type(&self) -> &str {
        match self {
            Self::ChatHistory(_) => "chat",
            Self::Workflow(_) => "workflow",
            Self::String(_) => "string",
        }
    }

    /// Given an input of byte-slice, parses it into a valid request type.
    pub async fn try_parse_bytes(input_bytes: &Bytes) -> Result<Self> {
        let input_string = ArweaveStorage::parse_downloadable(input_bytes).await?;
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
}

#[cfg(test)]
mod tests {
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
        use alloy::hex::FromHex;

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
}

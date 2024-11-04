use serde::{Deserialize, Serialize};

/// A chat history entry.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatHistoryResponse {
    /// Role, usually `user`, `assistant` or `system`.
    pub role: String,
    /// Message content.
    pub content: String,
}

impl ChatHistoryResponse {
    /// Creates a new chat history entry with the given content for `assistant` role.
    pub fn assistant(content: String) -> Self {
        Self {
            role: "assistant".to_string(),
            content,
        }
    }

    /// Creates a new chat history entry with the given content for `user` role.
    pub fn user(content: String) -> Self {
        Self {
            role: "user".to_string(),
            content,
        }
    }
}

/// A request with chat history.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatHistoryRequest {
    /// Task Id of which the output will act like history.
    pub history_id: usize,
    /// Message content.
    pub content: String,
}

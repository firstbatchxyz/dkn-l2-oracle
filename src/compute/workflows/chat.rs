use serde::{Deserialize, Serialize};

/// A chat history entry.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatHistoryResponse {
    /// Role, usually `user`, `assistant` or `system`.
    pub role: String,
    /// Message content.
    pub content: String,
    /// Task Id of this entry.
    pub id: usize,
}

/// A request with chat history.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatHistoryRequest {
    /// Task Id of which the output will act like history.
    pub history_id: usize,
    /// Message content.
    pub content: String,
}

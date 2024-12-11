use dkn_workflows::{MessageInput, Workflow};
use serde_json::json;

/// Creates a generation workflow with the given input.
///
/// It is an alias for `make_chat_workflow` with a single message alone.
pub fn make_generation_workflow(input: String) -> Result<Workflow, serde_json::Error> {
    make_chat_workflow(Vec::new(), input)
}

/// Creates a chat workflow with the given input.
///
/// `messages` is the existing message history, which will be used as context for the `input` message.
pub fn make_chat_workflow(
    mut messages: Vec<MessageInput>,
    input: String,
) -> Result<Workflow, serde_json::Error> {
    // add the new input to the message history as a user message
    messages.push(MessageInput {
        role: "user".to_string(),
        content: input,
    });

    let workflow = json!({
        "config": {
            "max_steps": 10,
            "max_time": 50,
            "tools": [""]
        },
        "tasks": [
            {
                "id": "A",
                "name": "Generate with history",
                "description": "Expects an array of messages for generation",
                "operator": "generation",
                "messages": messages,
                "inputs": [],
                "outputs": [
                    {
                        "type": "write",
                        "key": "result",
                        "value": "__result"
                    }
                ]
            },
            {
                "id": "__end",
                "name": "end",
                "description": "End of the task",
                "operator": "end",
                "messages": [{ "role": "user", "content": "End of the task" }],
                "inputs": [],
                "outputs": []
            }
        ],
        "steps": [
            {
                "source": "A",
                "target": "__end"
            }
        ],
        "return_value": {
            "input": {
                "type": "read",
                "key": "result"
            }
        }
    });

    serde_json::from_value(workflow)
}

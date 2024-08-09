use eyre::{Context, Result};
use ollama_workflows::Workflow;
use serde_json::json;

/// Generation workflow for simple input-output tasks.
///
/// TODO: `lazy_static!` did not work for some reason.
pub fn get_generation_workflow() -> Result<Workflow> {
    serde_json::from_value(json!({
      "name": "LLM generation",
      "description": "Directly generate text with input",
      "config":{
          "max_steps": 1,
          "max_time": 50,
          "tools": [""]
      },
      "external_memory":{
          "context":[""],
          "question":[""],
          "answer":[""]
      },
      "tasks":[
          {
              "id": "A",
              "name": "Generate",
              "description": "",
              "prompt": "{text}",
              "inputs":[
                    {
                      "name": "text",
                      "value": {
                          "type": "input",
                          "key": ""
                      },
                      "required": true
                  }
              ],
              "operator": "generation",
              "outputs":[
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
              "prompt": "End of the task",
              "inputs": [],
              "operator": "end",
              "outputs": []
          }
      ],
      "steps":[
          {
              "source":"A",
              "target":"__end"
          }
      ],
      "return_value":{
          "input":{
              "type":"read",
              "key":"result"
          }
      }
    }))
    .wrap_err("Could not parse hardcoded workflow.")
}

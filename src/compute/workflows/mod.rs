mod executor;
pub use executor::WorkflowsExt;

mod models;
pub use models::ModelConfig;

mod providers;
pub use providers::*;

// TODO: move this elsewhere
/// Utility to parse comma-separated string values, mostly read from the environment.
/// - Trims `"` from both ends at the start
/// - For each item, trims whitespace from both ends
pub fn split_comma_separated(input: Option<&str>) -> Vec<String> {
    match input {
        Some(s) => s
            .trim_matches('"')
            .split(',')
            .filter_map(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(s.trim().to_string())
                }
            })
            .collect::<Vec<_>>(),
        None => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Bytes;
    use ollama_workflows::{Executor, Model, ProgramMemory};

    #[tokio::test]
    #[ignore = "run this manually"]
    async fn test_ollama_generation() {
        let executor = Executor::new(Model::Llama3_1_8B);
        let (output, _) = executor
            .execute_raw(&Bytes::from_static(
                "What is the result of 2 + 2?".as_bytes(),
            ))
            .await
            .unwrap();

        // funny test but it should pass
        println!("Output:\n{}", output);
        // assert!(output.contains('4'));
    }

    #[tokio::test]
    #[ignore = "run this manually"]
    async fn test_openai_generation() {
        let executor = Executor::new(Model::GPT3_5Turbo);
        let (output, _) = executor
            .execute_raw(&Bytes::from_static(
                "What is the result of 2 + 2?".as_bytes(),
            ))
            .await
            .unwrap();

        // funny test but it should pass
        println!("Output:\n{}", output);
        // assert!(output.contains('4'));
    }

    /// Test the generation workflow with a plain input.
    #[tokio::test]
    async fn test_workflow_plain() {
        let executor = Executor::new(Model::GPT4o);
        let workflow = executor.get_generation_workflow().unwrap();
        let mut memory = ProgramMemory::new();
        let output = executor.execute(None, workflow, &mut memory).await;
        println!("Output:\n{}", output);
    }
}

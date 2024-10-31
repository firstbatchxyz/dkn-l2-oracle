mod executor;
pub use executor::WorkflowsExt;

mod postprocess;

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Bytes;
    use dkn_workflows::{Entry, Executor, Model, ProgramMemory};

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
            .execute_raw(&Bytes::from_static(b"What is the result of 2 + 2?"), "")
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

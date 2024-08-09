use alloy::primitives::U256;
use async_trait::async_trait;
use eyre::Result;
use ollama_workflows::{Entry, Executor, ProgramMemory, Workflow};

use super::{preset::get_generation_workflow, WorkflowsExt};

#[async_trait]
impl WorkflowsExt for Executor {
    async fn generation(&self, input: String) -> Result<(String, String)> {
        let workflow = get_generation_workflow()?;
        let mut memory = ProgramMemory::new();
        let input = Entry::try_value_or_str(&input);
        let output = self.execute(Some(&input), workflow, &mut memory).await;
        Ok((output, String::default()))
    }

    async fn validation(
        &self,
        input: String,
        responses: Vec<String>,
    ) -> Result<(Vec<U256>, String)> {
        todo!()
    }
}

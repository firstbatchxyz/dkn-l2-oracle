use crate::{
    compute::WorkflowsExt,
    contracts::{bytes32_to_string, bytes_to_string},
    data::Arweave,
    mine_nonce, DriaOracle, ModelConfig,
};
use alloy::{
    primitives::{FixedBytes, U256},
    rpc::types::TransactionReceipt,
};
use eyre::{Context, Result};
use ollama_workflows::Executor;

/// Handles a generation request.
///
/// 1. First, we check if we have already responded to the task. Contract will revert even if we dont do this check ourselves,
/// but its better to provide the error here.
///
/// 2. Then, we check if our models are compatible with the request. If not, we return an error.
pub async fn handle_generation(
    node: &DriaOracle,
    models: &ModelConfig,
    task_id: U256,
    protocol: FixedBytes<32>,
) -> Result<Option<TransactionReceipt>> {
    let responses = node.get_task_responses(task_id).await?;
    if responses.iter().any(|r| r.responder == node.address()) {
        log::debug!("Already responded to {} with generation", task_id);
        return Ok(None);
    }

    let request = node
        .get_task_request(task_id)
        .await
        .wrap_err("Could not get task")?;

    // choose model based on the request
    let models_string = bytes_to_string(&request.models)?;
    let (_, model) = models.get_any_matching_model_from_csv(&models_string)?;
    log::debug!("Using model: {} from {}", model, models_string);

    // execute task
    let protocol_string = bytes32_to_string(&protocol)?;
    let executor = Executor::new(model);
    let (output, metadata) = executor
        .execute_raw(&request.input, &protocol_string)
        .await?;

    // do the Arweave trick for large inputs
    let arweave = Arweave::new_from_env()?;
    let output = arweave.put_if_large(output).await?;
    let metadata = arweave.put_if_large(metadata).await?;

    // mine nonce
    let nonce = mine_nonce(
        request.parameters.difficulty,
        &request.requester,
        &node.address(),
        &request.input,
        &task_id,
    )
    .0;

    // respond
    let tx_hash = node
        .respond_generation(task_id, output, metadata, nonce)
        .await?;
    Ok(Some(tx_hash))
}

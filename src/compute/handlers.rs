use super::WorkflowsExt;
use crate::DriaOracle;
use alloy::primitives::{Bytes, TxHash, U256};
use eyre::{Context, Result};
use ollama_workflows::{Executor, Model};
use std::str::FromStr;

use super::mine_nonce;

/// Handles a generation request.
pub async fn handle_generation(node: &DriaOracle, task_id: U256) -> Result<TxHash> {
    let request = node
        .get_task_request(task_id)
        .await
        .wrap_err("Could not get task")?;

    // create executor
    let executor = Executor::new(Model::GPT4oMini);

    // parse input
    // let input_str = bytes_to_string(&request.input).wrap_err("Could not read input")?;
    let (output_str, metadata_str) = executor.generation(&request.input).await?;
    let output = Bytes::from_str(&output_str)?;
    let metadata = Bytes::from_str(&metadata_str)?;

    // mine nonce
    let nonce = mine_nonce(
        request.difficulty,
        &request.requester,
        &node.address,
        &request.input,
        &task_id,
    )
    .0;

    // respond
    let tx_hash = node
        .respond_generation(task_id, output, metadata, nonce)
        .await
        .wrap_err("Could not respond to generation")?;
    Ok(tx_hash)
}

/// Handles a validation request.
pub async fn handle_validation(node: &DriaOracle, task_id: U256) -> Result<TxHash> {
    let request = node
        .get_task_request(task_id)
        .await
        .wrap_err("Could not get task")?;

    // TODO: validate responses
    let scores = vec![];
    let metadata = Bytes::default();

    // mine nonce
    let nonce = mine_nonce(
        request.difficulty,
        &request.requester,
        &node.address,
        &request.input,
        &task_id,
    )
    .0;

    let tx_hash = node
        .respond_validation(task_id, scores, metadata, nonce)
        .await
        .wrap_err("Could not respond to generation")?;
    Ok(tx_hash)
}

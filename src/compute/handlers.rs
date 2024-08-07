use std::str::FromStr;

use crate::DriaOracle;
use alloy::primitives::{Bytes, TxHash, U256};
use eyre::{Context, Result};

use super::mine_nonce;

/// Handles a generation request.
pub async fn handle_generation(node: &DriaOracle, task_id: U256) -> Result<TxHash> {
    let task = node
        .get_task_request(task_id)
        .await
        .wrap_err("Could not get task")?;

    // TODO: generate response
    let response = Bytes::from_str("hi there")?;
    let metadata = Bytes::default();

    // mine nonce
    let nonce = mine_nonce(
        task.difficulty,
        task.requester,
        node.address,
        task.input,
        task_id,
    );

    let tx_hash = node
        .respond_generation(task_id, response, metadata, nonce)
        .await
        .wrap_err("Could not respond to generation")?;
    Ok(tx_hash)
}

/// Handles a validation request.
pub async fn handle_validation(node: &DriaOracle, task_id: U256) -> Result<TxHash> {
    let task = node
        .get_task_request(task_id)
        .await
        .wrap_err("Could not get task")?;

    // TODO: validate responses
    let scores = vec![];
    let metadata = Bytes::default();

    // mine nonce
    let nonce = mine_nonce(
        task.difficulty,
        task.requester,
        node.address,
        task.input,
        task_id,
    );

    let tx_hash = node
        .respond_validation(task_id, scores, metadata, nonce)
        .await
        .wrap_err("Could not respond to generation")?;
    Ok(tx_hash)
}

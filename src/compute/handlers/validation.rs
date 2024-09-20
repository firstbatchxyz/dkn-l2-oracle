use super::ModelConfig;
use crate::{mine_nonce, DriaOracle};
use alloy::{
    primitives::{utils::parse_ether, Bytes, U256},
    rpc::types::TransactionReceipt,
};
use eyre::{eyre, Result};

/// Handles a validation request.
#[allow(unused)]
pub async fn handle_validation(
    node: &DriaOracle,
    models: &ModelConfig,
    task_id: U256,
) -> Result<Option<TransactionReceipt>> {
    // check if already responded as generator, because we cant validate our own answer
    let responses = node.get_task_responses(task_id).await?;
    if responses.iter().any(|r| r.responder == node.address()) {
        log::debug!(
            "Cant validate {} with your own generation response",
            task_id
        );
        return Ok(None);
    }

    // check if we have validated anyways
    let validations = node.get_task_validations(task_id).await?;
    if validations.iter().any(|v| v.validator == node.address()) {
        return Err(eyre!("Already validated {}", task_id));
    }

    let request = node.get_task_request(task_id).await?;

    // FIXME: will add validation prompt here
    let scores = (0..request.parameters.numGenerations)
        .map(|_| parse_ether("1.0").unwrap())
        .collect::<Vec<_>>();
    let metadata = Bytes::default();

    // FIXME: can add Arweave trick for metadata here

    // mine nonce
    let nonce = mine_nonce(
        request.parameters.difficulty,
        &request.requester,
        &node.address(),
        &request.input,
        &task_id,
    )
    .0;

    let tx_hash = node
        .respond_validation(task_id, scores, metadata, nonce)
        .await?;
    Ok(Some(tx_hash))
}

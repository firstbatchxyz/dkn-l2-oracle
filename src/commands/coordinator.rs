use crate::{
    compute::{self, ModelConfig},
    contracts::{bytes_to_string, OracleKind, TaskStatus},
    DriaOracle,
};
use alloy::{eips::BlockNumberOrTag, primitives::U256};
use eyre::{eyre, Context, Result};
use futures_util::StreamExt;
use ollama_workflows::Model;

// TODO: add cancellation here
/// Runs the main loop of the oracle node.
pub async fn run_oracle(
    node: &DriaOracle,
    kinds: Vec<OracleKind>,
    models: Vec<Model>,
    from_block: impl Into<BlockNumberOrTag>,
) -> Result<()> {
    // make sure we are registered to required kinds
    for kind in &kinds {
        if !node.is_registered(*kind).await? {
            return Err(eyre!("You need to register as {} first.", kind))?;
        }
    }
    let is_generator = kinds.contains(&OracleKind::Generator);
    let is_validator = kinds.contains(&OracleKind::Validator);

    // prepare model config
    let model_config = ModelConfig::new(models);
    if model_config.models_providers.is_empty() {
        return Err(eyre!("No models provided."))?;
    }
    model_config.check_providers().await?;

    let task_poller = node
        .subscribe_to_tasks(from_block)
        .await
        .wrap_err("Could not subscribe")?;
    log::info!(
        "Subscribed to LLMOracleCoordinator at {}",
        node.contract_addresses.coordinator
    );

    task_poller
        .into_stream()
        .for_each(|log| async {
            match log {
                Ok((event, log)) => {
                    log::debug!(
                        "Received event for tx: {}",
                        log.transaction_hash.unwrap_or_default()
                    );
                    log::info!("Received event for task: {}", event.taskId);

                    // handle the event based on task status
                    let task_status = match TaskStatus::try_from(event.statusAfter) {
                        Ok(status) => status,
                        Err(e) => {
                            log::error!(
                                "Could not parse task status: {}, skipping task {}",
                                e,
                                event.taskId
                            );
                            return;
                        }
                    };

                    let response_tx_hash = match task_status {
                        TaskStatus::PendingGeneration => {
                            if is_generator {
                                compute::handle_generation(node, &model_config, event.taskId).await
                            } else {
                                return;
                            }
                        }
                        TaskStatus::PendingValidation => {
                            if is_validator {
                                compute::handle_validation(node, &model_config, event.taskId).await
                            } else {
                                return;
                            }
                        }
                        _ => {
                            log::debug!(
                                "Ignoring task {} with status: {}",
                                event.taskId,
                                event.statusAfter
                            );
                            return;
                        }
                    };

                    match response_tx_hash {
                        Ok(tx_hash) => {
                            log::info!(
                                "Task {} processed successfully. (tx: {})",
                                event.taskId,
                                tx_hash
                            )
                        }
                        Err(e) => log::error!("Could not process task: {}", e),
                    }
                }
                Err(e) => log::error!("Could not handle event: {}", e),
            }
        })
        .await;

    Ok(())
}

pub async fn view_task_events(
    node: &DriaOracle,
    from_block: impl Into<BlockNumberOrTag> + Clone,
    to_block: impl Into<BlockNumberOrTag> + Clone,
) -> Result<()> {
    log::info!(
        "Viewing task ids & statuses between blocks: {} - {}",
        from_block.clone().into(),
        to_block.clone().into()
    );

    let task_events = node.get_tasks(from_block, to_block).await?;

    for (event, _) in task_events {
        log::info!(
            "Task: {} ({} -> {})",
            event.taskId,
            TaskStatus::try_from(event.statusBefore).unwrap_or_default(),
            TaskStatus::try_from(event.statusAfter).unwrap_or_default()
        );
    }

    Ok(())
}

pub async fn view_task(node: &DriaOracle, task_id: U256) -> Result<()> {
    log::info!("Viewing task {}.", task_id);
    let (request, responses, validations) = node.get_task(task_id).await?;

    log::info!("Request Information:");
    log::info!("Requester: {}", request.requester);
    log::info!("Status:    {}", TaskStatus::try_from(request.status)?);
    log::info!("Input:     {}", bytes_to_string(&request.input)?);
    log::info!("Models:    {}", bytes_to_string(&request.models)?);

    log::info!("Responses:");
    if responses._0.is_empty() {
        log::warn!("There are no responses yet.");
    } else {
        for (idx, response) in responses._0.iter().enumerate() {
            log::info!("Response  #{}", idx);
            log::info!("Output:    {}", bytes_to_string(&response.output)?);
            log::info!("Metadata:  {}", bytes_to_string(&response.metadata)?);
            log::info!("Generator: {}", response.responder);
        }
    }

    log::info!("Validations:");
    if validations._0.is_empty() {
        log::warn!("There are no validations yet.");
    } else {
        for (idx, validation) in validations._0.iter().enumerate() {
            log::info!("Validation #{}", idx);
            log::info!("Scores:     {:?}", validation.scores);
            log::info!("Metadata:   {}", bytes_to_string(&validation.metadata)?);
            log::info!("Validator:  {}", validation.validator);
        }
    }

    Ok(())
}

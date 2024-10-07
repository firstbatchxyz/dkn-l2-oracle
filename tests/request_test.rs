//! Tests the request command, resulting in a task being created in the coordinator contract.
//!
//! 1. Requester buys some WETH, and it is approved within the request command.
//! 2. Requester requests a task with a given input, models, difficulty, num_gens, and num_vals.
//! 3. The task is created in the coordinator contract.

use alloy::primitives::utils::parse_ether;
use dkn_oracle::{bytes_to_string, commands, DriaOracle, DriaOracleConfig, WETH};
use dkn_workflows::Model;
use eyre::Result;

#[tokio::test]
async fn test_request() -> Result<()> {
    dotenvy::dotenv().unwrap();

    // task setup
    let difficulty = 1;
    let num_gens = 1;
    let num_vals = 1;
    let protocol = format!("test/{}", env!("CARGO_PKG_VERSION"));
    let models = vec![Model::GPT4Turbo];
    let input = "What is the result of 2 + 2?".to_string();

    // node setup
    let config = DriaOracleConfig::new_from_env()?;
    let (node, _anvil) = DriaOracle::anvil_new(config).await?;

    // setup account & buy some WETH
    let requester = node.connect(node.anvil_funded_wallet(None).await?);
    let token = WETH::new(requester.addresses.token, &requester.provider);
    let _ = token.deposit().value(parse_ether("100")?).send().await?;

    // request a task, and see it in the coordinator
    let task_id = node.get_next_task_id().await?;
    commands::request_task(
        &requester, &input, models, difficulty, num_gens, num_vals, protocol,
    )
    .await?;

    // get the task info
    let (request, _, _) = node.get_task(task_id).await?;
    assert_eq!(input, bytes_to_string(&request.input).unwrap());
    assert_eq!(difficulty, request.parameters.difficulty);
    assert_eq!(num_gens, request.parameters.numGenerations);
    assert_eq!(num_vals, request.parameters.numValidations);

    Ok(())
}

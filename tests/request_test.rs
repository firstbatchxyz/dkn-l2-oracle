use alloy::primitives::utils::parse_ether;
use dkn_oracle::{bytes_to_string, commands, DriaOracle, DriaOracleConfig, WETH};
use eyre::Result;
use ollama_workflows::Model;

#[tokio::test]
async fn test_request() -> Result<()> {
    // task setup
    let difficulty = 1;
    let num_gens = 1;
    let num_vals = 1;
    let models = vec![Model::GPT3_5Turbo];
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
    commands::request_task(&requester, &input, models, difficulty, num_gens, num_vals).await?;

    // get the task info
    let (request, _, _) = node.get_task(task_id).await?;
    assert_eq!(input, bytes_to_string(&request.input).unwrap());
    assert_eq!(difficulty, request.difficulty);
    assert_eq!(num_gens, request.numGenerations);
    assert_eq!(num_vals, request.numValidations);

    Ok(())
}

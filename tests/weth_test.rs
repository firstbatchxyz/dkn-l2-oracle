use alloy::primitives::utils::parse_ether;
use dkn_oracle::{DriaOracle, DriaOracleConfig, WETH};
use eyre::Result;

/// Using the forked blockchain, creates two accounts (alice, bob) and then,
///
/// 1. Alice buys WETH
/// 2. Alice approves Bob
/// 3. Bob transfers WETH from Alice
#[tokio::test]
async fn test_weth_transfer() -> Result<()> {
    // amount of WETH that will be transferred
    let amount = parse_ether("100")?;

    let config = DriaOracleConfig::new_from_env()?;
    let (node, _anvil) = DriaOracle::anvil_new(config).await?;

    // setup alice
    let alice = node.connect(node.anvil_funded_wallet(None).await?);
    let alice_token = WETH::new(node.contract_addresses.token, &alice.provider);

    // setup bob
    let bob = node.connect(node.anvil_funded_wallet(None).await?);
    let bob_token = WETH::new(node.contract_addresses.token, &bob.provider);

    // record existing balances
    let alice_balance_before = node.get_token_balance(alice.address()).await?;
    let bob_balance_before = node.get_token_balance(bob.address()).await?;

    // alice buys WETH
    let _ = alice_token.deposit().value(amount).send().await?;
    let alice_balance_after = node.get_token_balance(alice.address()).await?;
    assert_eq!(
        alice_balance_after.amount - alice_balance_before.amount,
        amount
    );

    // alice approves bob
    let _ = alice_token.approve(bob.address(), amount).send().await?;

    // bob transfers WETH from alice
    let _ = bob_token
        .transferFrom(alice.address(), bob.address(), amount)
        .send()
        .await?;
    let alice_balance_after = node.get_token_balance(alice.address()).await?;
    let bob_balance_after = node.get_token_balance(bob.address()).await?;
    assert_eq!(alice_balance_after.amount, alice_balance_before.amount);
    assert_eq!(bob_balance_after.amount - bob_balance_before.amount, amount);

    Ok(())
}

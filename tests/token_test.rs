//! Demonstrate transferring tokens in a forked blockchain.

use alloy::{
    network::{EthereumWallet, NetworkWallet},
    node_bindings::Anvil,
    primitives::{address, keccak256, utils::format_ether, Bytes, U256},
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
    sol,
    sol_types::SolValue,
};
use dkn_oracle::{DriaOracle, DriaOracleConfig};
use eyre::Result;

#[tokio::test]
async fn test_token_transfer() -> Result<()> {
    let config = DriaOracleConfig::new_from_env()?;
    println!("Using config: {:?}", config);
    let (node, anvil) = DriaOracle::new_anvil(config).await?;

    let alice: PrivateKeySigner = anvil.keys()[0].clone().into();
    let alice_address = alice.address();
    let alice = EthereumWallet::from(alice);

    let bob: PrivateKeySigner = anvil.keys()[1].clone().into();
    let bob_address = bob.address();
    let bob = EthereumWallet::from(bob);

    let alice_balance = node.get_native_balance(alice_address).await?;
    let bob_balance = node.get_native_balance(bob_address).await?;
    let node_balance = node.get_native_balance(node.address).await?;
    println!("Alice balance: {}", alice_balance);
    println!("Bob   balance: {}", bob_balance);
    println!("Node  balance: {}", node_balance);

    let alice_balance = node.get_token_balance(alice_address).await?;
    let bob_balance = node.get_token_balance(bob_address).await?;
    let node_balance = node.get_token_balance(node.address).await?;
    println!("Alice balance: {}", alice_balance);
    println!("Bob   balance: {}", bob_balance);
    println!("Node  balance: {}", node_balance);

    Ok(())
}

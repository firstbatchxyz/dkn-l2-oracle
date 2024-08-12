use alloy::{
    network::EthereumWallet,
    node_bindings::Anvil,
    primitives::{address, keccak256, Bytes, U256},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
    sol,
    sol_types::SolValue,
};
use eyre::Result;

#[tokio::test]
async fn test_oracle_request() -> Result<()> {
    todo!("fork test oracle")
}

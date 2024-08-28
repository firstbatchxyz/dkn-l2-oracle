//! Anvil-related utilities.
//!
//! This module is only available when the `anvil` feature is enabled.

use super::{DriaOracle, DriaOracleConfig};
use alloy::network::EthereumWallet;
use alloy::node_bindings::{Anvil, AnvilInstance};
use alloy::primitives::{utils::parse_ether, U256};
use alloy::providers::ext::AnvilApi;
use alloy::signers::local::PrivateKeySigner;
use eyre::Result;

impl DriaOracle {
    /// Default ETH funding amount for generated wallets.
    const ANVIL_FUND_ETHER: &'static str = "10000";

    /// Creates a new Anvil instance that forks the chain at the configured RPC URL.
    ///
    /// Return the node instance and the Anvil instance.
    /// Note that when Anvil instance is dropped, you will lose the forked chain.
    pub async fn anvil_new(config: DriaOracleConfig) -> Result<(Self, AnvilInstance)> {
        let anvil = Anvil::new().fork(config.rpc_url.to_string()).try_spawn()?;
        let node = Self::new(config.with_rpc_url(anvil.endpoint_url())).await?;

        Ok((node, anvil))
    }

    /// Generates a random wallet, funded with the given `fund` amount.
    ///
    /// If `fund` is not provided, 10K ETH is used.
    pub async fn anvil_funded_wallet(&self, fund: Option<U256>) -> Result<EthereumWallet> {
        let fund = fund.unwrap_or_else(|| parse_ether(Self::ANVIL_FUND_ETHER).unwrap());
        let signer = PrivateKeySigner::random();
        self.provider
            .anvil_set_balance(signer.address(), fund)
            .await?;
        let wallet = EthereumWallet::from(signer);
        Ok(wallet)
    }
}

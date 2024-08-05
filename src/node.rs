use crate::contracts::*;
use alloy::primitives::{TxHash, U256};
use alloy::providers::fillers::{
    ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller,
};
use alloy::{
    network::EthereumWallet,
    primitives::Address,
    providers::{Identity, Provider, ProviderBuilder, RootProvider},
    signers::local::PrivateKeySigner,
    transports::http::{reqwest::Url, Client, Http},
};
use alloy_chains::Chain;
use eyre::{eyre, Context, Result};

// TODO: use a better type for this
type DriaOracleProvider = FillProvider<
    JoinFill<
        JoinFill<JoinFill<JoinFill<Identity, GasFiller>, NonceFiller>, ChainIdFiller>,
        WalletFiller<EthereumWallet>,
    >,
    RootProvider<Http<Client>>,
    Http<Client>,
    alloy::network::Ethereum,
>;

pub struct DriaOracle {
    pub wallet: EthereumWallet,
    pub address: Address,
    pub rpc_url: Url,
    pub contract_addresses: ContractAddresses,
    pub provider: DriaOracleProvider,
}

impl DriaOracle {
    pub async fn new(private_key: &[u8; 32], rpc_url: Url) -> Result<Self> {
        let signer = PrivateKeySigner::from_bytes(private_key.into())
            .wrap_err("Could not parse private key")?;
        let address = signer.address();
        let wallet = EthereumWallet::from(signer);
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet.clone())
            .on_http(rpc_url.clone());

        let chain_id_u64 = provider
            .get_chain_id()
            .await
            .wrap_err("Could not get chain id")?;
        let chain = Chain::from_id(chain_id_u64);
        let contract_addresses = ADDRESSES[&chain].clone();

        Ok(DriaOracle {
            wallet,
            address,
            rpc_url,
            contract_addresses,
            provider,
        })
    }

    /// Returns ETH balance and configured token balance.
    pub async fn balances(&self) -> Result<((U256, String), (U256, String))> {
        let token = ERC20::new(self.contract_addresses.token, self.provider.clone());
        let token_balance = token.balanceOf(self.address).call().await?._0;
        let token_symbol = token.symbol().call().await?._0;

        let eth_balance = self.provider.get_balance(self.address).await?;

        Ok((
            (eth_balance, "ETH".to_string()),
            (token_balance, token_symbol),
        ))
    }

    /// Register the oracle with the registry.
    ///
    /// - Checks the required approval for the registry, and approves the necessary amount.
    /// - Checks if the oracle is already registered as well.
    pub async fn register(&self) -> Result<TxHash> {
        let registry = OracleRegistry::new(self.contract_addresses.registry, self.provider.clone());

        let req = registry.register();
        let tx = req
            .send()
            .await
            .map_err(contract_error_report)
            .wrap_err(eyre!("Could not register."))?;

        log::info!("Hash: {:?}", tx.tx_hash());
        let awaited_tx_hash = tx.watch().await?;
        Ok(awaited_tx_hash)
    }

    pub async fn unregister(&self) -> Result<TxHash> {
        let registry = OracleRegistry::new(self.contract_addresses.registry, self.provider.clone());

        let req = registry.unregister();
        let tx = req
            .send()
            .await
            .map_err(contract_error_report)
            .wrap_err("Could not unregister.")?;

        log::info!("Hash: {:?}", tx.tx_hash());
        let awaited_tx_hash = tx.watch().await?;
        Ok(awaited_tx_hash)
    }

    pub async fn is_registered(&self) -> Result<bool> {
        let registry = OracleRegistry::new(self.contract_addresses.registry, self.provider.clone());

        let is_registered = registry.isRegistered(self.address).call().await?;
        Ok(is_registered._0)
    }

    pub async fn transfer_from(&self, from: Address, to: Address, amount: U256) -> Result<TxHash> {
        let token = ERC20::new(self.contract_addresses.token, self.provider.clone());

        let req = token.transferFrom(from, to, amount);
        let tx = req
            .send()
            .await
            .map_err(contract_error_report)
            .wrap_err("Could not transfer tokens.")?;

        log::info!("Hash: {:?}", tx.tx_hash());
        let awaited_tx_hash = tx.watch().await?;
        Ok(awaited_tx_hash)
    }

    pub async fn approve(&self, spender: Address, amount: U256) -> Result<TxHash> {
        let token = ERC20::new(self.contract_addresses.token, self.provider.clone());

        let req = token.approve(spender, amount);
        let tx = req
            .send()
            .await
            .map_err(contract_error_report)
            .wrap_err("Could not approve tokens.")?;

        log::info!("Hash: {:?}", tx.tx_hash());
        let awaited_tx_hash = tx.watch().await?;
        Ok(awaited_tx_hash)
    }

    pub async fn allowance(&self, owner: Address, spender: Address) -> Result<U256> {
        let token = ERC20::new(self.contract_addresses.token, self.provider.clone());

        let allowance = token.allowance(owner, spender).call().await?._0;
        Ok(allowance)
    }

    /// Returns the amount of tokens to be staked to registry.
    pub async fn registry_stake_amount(&self) -> Result<(U256, String)> {
        let registry = OracleRegistry::new(self.contract_addresses.registry, self.provider.clone());

        let stake_amount = registry.stakeAmount().call().await?._0;

        // return the symbol as well
        let token_address = registry.token().call().await?._0;
        let token = ERC20::new(token_address, self.provider.clone());
        let token_symbol = token.symbol().call().await?._0;

        Ok((stake_amount, token_symbol))
    }
}

impl core::fmt::Display for DriaOracle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Dria Oracle Node v{}\nAddress: {}\nRPC URL: {}",
            env!("CARGO_PKG_VERSION"),
            self.address,
            self.rpc_url
        )
    }
}

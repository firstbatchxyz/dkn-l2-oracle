use crate::contracts::*;
use alloy::contract::Error;
use alloy::sol_types::SolInterface;
use alloy::transports::RpcError;
use alloy::{
    network::EthereumWallet,
    primitives::Address,
    providers::{Provider, ProviderBuilder, RootProvider},
    signers::local::PrivateKeySigner,
    transports::http::{reqwest::Url, Client, Http},
};
use alloy_chains::Chain;

pub struct DriaOracle {
    pub wallet: EthereumWallet,
    pub address: Address,
    pub rpc_url: Url,
    pub contract_addresses: ContractAddresses,

    // TODO: use a better type for this
    pub provider: alloy::providers::fillers::FillProvider<
        alloy::providers::fillers::JoinFill<
            alloy::providers::fillers::JoinFill<
                alloy::providers::fillers::JoinFill<
                    alloy::providers::fillers::JoinFill<
                        alloy::providers::Identity,
                        alloy::providers::fillers::GasFiller,
                    >,
                    alloy::providers::fillers::NonceFiller,
                >,
                alloy::providers::fillers::ChainIdFiller,
            >,
            alloy::providers::fillers::WalletFiller<EthereumWallet>,
        >,
        RootProvider<Http<Client>>,
        Http<Client>,
        alloy::network::Ethereum,
    >,
}

impl DriaOracle {
    pub async fn new(private_key: &[u8; 32], rpc_url: String) -> Self {
        let signer = PrivateKeySigner::from_bytes(private_key.into()).expect("should parse");
        let address = signer.address();
        let wallet = EthereumWallet::from(signer);

        let rpc_url = Url::parse(&rpc_url).expect("should parse url");
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet.clone())
            .on_http(rpc_url.clone());

        let chain_id_u64 = provider.get_chain_id().await.expect("should get chain id");
        let chain = Chain::from_id(chain_id_u64);
        let contract_addresses = ADDRESSES[&chain].clone();

        DriaOracle {
            wallet,
            address,
            rpc_url,
            contract_addresses,
            provider,
        }
    }

    /// Register the oracle with the registry.
    ///
    /// - Checks the required approval for the registry, and approves the necessary amount.
    /// - Checks if the oracle is already registered as well.
    pub async fn register(&self) {
        let token = ERC20::new(self.contract_addresses.weth, self.provider.clone());
        let registry = OracleRegistry::new(self.contract_addresses.registry, self.provider.clone());

        // see if registered
        let req = registry.register();
        match req.send().await {
            Ok(tx) => {
                log::info!("Hash: {:?}", tx.tx_hash());
                tx.watch().await.expect("should watch");
            }
            Err(e) => {
                if let Error::TransportError(RpcError::ErrorResp(e)) = e {
                    let error = e
                        .as_decoded_error::<OracleRegistry::OracleRegistryErrors>(false)
                        .unwrap();

                    if let OracleRegistry::OracleRegistryErrors::AlreadyRegistered(e) = error {
                        println!("Already registered: {}", e._0);
                    } else {
                        println!(
                            "Transport error: {:?}",
                            String::from_utf8(error.abi_encode())
                        );
                    }
                } else {
                    println!("Unknown error: {:#?}", e);
                }
            }
        }
    }

    pub async fn unregister(&self) {
        let registry = OracleRegistry::new(self.contract_addresses.registry, self.provider.clone());

        // see if registered
        let is_registered = registry
            .isRegistered(self.address)
            .call()
            .await
            .expect("should call")
            ._0;

        println!("is registered: {}", is_registered);
    }
}

impl core::fmt::Display for DriaOracle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DriaOracle {{ address: {}, rpc_url: {} }}",
            self.address, self.rpc_url
        )
    }
}

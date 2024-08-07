use alloy::transports::http::reqwest::Url;
use dkn_oracle::DriaOracle;
use dotenvy::dotenv;
use eyre::{Context, Result};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().expect(".env file not found");
    env_logger::init();
    color_eyre::install()?;

    // parse private key
    let private_key_hex = env::var("SECRET_KEY").wrap_err("SECRET_KEY is not set")?;
    let private_key_decoded =
        hex::decode(&private_key_hex).wrap_err("Could not decode private key")?;
    let mut private_key = [0u8; 32];
    private_key.clone_from_slice(&private_key_decoded);

    // parse rpc url
    let rpc_url_env = env::var("RPC_URL").wrap_err("RPC_URL is not set")?;
    let rpc_url = Url::parse(&rpc_url_env).wrap_err("Could not parse RPC URL.")?;

    // create node
    let node = DriaOracle::new(&private_key, rpc_url).await?;
    log::info!("{}", node);
    log::info!("{}", node.contract_addresses);

    // launch CLI
    dkn_oracle::cli(node).await?;

    Ok(())
}

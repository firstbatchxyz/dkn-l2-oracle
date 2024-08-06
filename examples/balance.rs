use alloy::{primitives::utils::format_ether, transports::http::reqwest::Url};
use dkn_oracle::DriaOracle;
use eyre::{Context, Result};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let private_key_hex: String = env::var("SECRET_KEY").wrap_err("SECRET_KEY is not set")?;
    let private_key_decoded =
        hex::decode(&private_key_hex).wrap_err("Could not decode private key")?;
    let mut private_key = [0u8; 32];
    private_key.clone_from_slice(&private_key_decoded);

    let rpc_url_env = env::var("RPC_URL").wrap_err("RPC_URL is not set")?;
    let rpc_url = Url::parse(&rpc_url_env).wrap_err("Could not parse RPC URL.")?;
    let node = DriaOracle::new(&private_key, rpc_url).await?;

    let balances = node.balances().await?;
    println!("Your balances:");
    println!("{} {}", format_ether(balances[0].0), balances[0].1);
    println!("{} {}", format_ether(balances[1].0), balances[1].1);

    Ok(())
}

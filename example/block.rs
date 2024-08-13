use alloy::providers::Provider;
use dkn_oracle::{DriaOracle, DriaOracleConfig};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let config = DriaOracleConfig::new_from_env()?;
    let node = DriaOracle::new(config).await?;

    let block = node.provider.get_block_number().await?;

    println!("Current block number: 0x{:x}", block);

    Ok(())
}

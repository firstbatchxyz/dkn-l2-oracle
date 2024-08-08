use dkn_oracle::{DriaOracle, DriaOracleConfig};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // create node
    let config = DriaOracleConfig::new_from_env()?
        .enable_logs()
        .enable_color_eyre()?;
    let node = DriaOracle::new(config).await?;
    log::info!("{}", node);
    log::info!("{}", node.contract_addresses);

    // launch CLI
    dkn_oracle::cli(node).await?;

    Ok(())
}

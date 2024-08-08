use dkn_oracle::DriaOracle;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenvy::dotenv()?;
    env_logger::init();
    color_eyre::install()?;

    // create node
    let node = DriaOracle::new_from_env().await?;
    log::info!("{}", node);
    log::info!("{}", node.contract_addresses);

    // launch CLI
    dkn_oracle::cli(node).await?;

    Ok(())
}

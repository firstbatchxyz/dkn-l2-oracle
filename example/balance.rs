use dkn_oracle::{commands, DriaOracle, DriaOracleConfig};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let config = DriaOracleConfig::new_from_env()?
        .enable_logs()
        .enable_color_eyre()?;
    let node = DriaOracle::new(config).await?;

    commands::display_balance(&node).await?;

    Ok(())
}

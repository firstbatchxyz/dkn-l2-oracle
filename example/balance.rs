use dkn_oracle::{commands, DriaOracle};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenvy::dotenv()?;
    env_logger::init();
    color_eyre::install()?;
    let node = DriaOracle::new_from_env().await?;

    commands::display_balance(&node).await?;

    Ok(())
}

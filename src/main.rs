#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenvy::dotenv()?;
    env_logger::try_init()?;
    color_eyre::install()?;
    dkn_oracle::cli().await?;
    Ok(())
}

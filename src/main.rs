use eyre::{Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().wrap_err("could not read .env")?;
    env_logger::try_init().wrap_err("could not initialize env_logger")?;
    color_eyre::install()?;
    dkn_oracle::cli().await?;

    Ok(())
}

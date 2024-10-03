use eyre::{Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = dotenvy::dotenv() {
        log::warn!("Could not load .env file: {}", e);
    }

    env_logger::try_init().wrap_err("could not initialize env_logger")?;
    color_eyre::install()?;
    dkn_oracle::cli().await?;

    Ok(())
}

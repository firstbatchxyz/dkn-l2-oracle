use eyre::{Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::try_init().wrap_err("could not initialize env_logger")?;
    color_eyre::install().wrap_err("could not install color_eyre")?;

    if let Err(e) = dotenvy::dotenv() {
        log::warn!("Could not load .env file: {}", e);
    }

    dkn_oracle::cli().await?;

    Ok(())
}

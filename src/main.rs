use eyre::{Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let dotenv_result = dotenvy::dotenv();
    env_logger::try_init().wrap_err("could not initialize env_logger")?;
    if let Err(err) = dotenv_result {
        log::warn!("Could not load .env file: {}", err);
    }

    dkn_oracle::cli().await?;

    Ok(())
}

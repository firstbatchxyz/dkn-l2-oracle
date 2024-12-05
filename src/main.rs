#[tokio::main]
async fn main() -> eyre::Result<()> {
    let dotenv_result = dotenvy::dotenv();

    env_logger::builder()
        .format_timestamp(Some(env_logger::TimestampPrecision::Millis))
        .filter(None, log::LevelFilter::Off)
        .filter_module("dkn_oracle", log::LevelFilter::Info)
        .filter_module("dkn_workflows", log::LevelFilter::Info)
        .parse_default_env() // reads RUST_LOG variable
        .init();

    // log about env usage
    match dotenv_result {
        Ok(path) => log::info!("Loaded .env file at: {}", path.display()),
        Err(e) => log::warn!("Could not load .env file: {}", e),
    }

    // launch CLI
    dkn_oracle::cli().await?;

    log::info!("Bye!");
    Ok(())
}

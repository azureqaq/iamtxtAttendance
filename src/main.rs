use anyhow::Result;
use simple_logger::SimpleLogger;

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_colors(true)
        .with_level(log::LevelFilter::Debug)
        .with_module_level("reqwest", log::LevelFilter::Error)
        .with_module_level("cookie_store", log::LevelFilter::Error)
        // .with_module_level("reqwest", log::LevelFilter::Error)
        // .with_module_level("reqwest", log::LevelFilter::Error)
        .init()
        .unwrap();

    match mma().await {
        Ok(_) => log::debug!("Done!"),
        Err(e) => log::error!("Error: {}", e),
    }
}

async fn mma() -> Result<()> {
    let conf = libs::config::get_config("./config_local.toml")?;
    libs::bot::att_now_all(conf).await?;
    Ok(())
}

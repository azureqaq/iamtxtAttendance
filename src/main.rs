use anyhow::Result;
use clap::{crate_authors, crate_name, crate_version, Arg, Command};
use libs::{botdir::BotDirs, status::get_status};
use simple_logger::SimpleLogger;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_colors(true)
        .with_level(log::LevelFilter::Debug)
        .with_module_level("reqwest", log::LevelFilter::Error)
        .with_module_level("cookie_store", log::LevelFilter::Error)
        .init()
        .unwrap();

    match mma().await {
        Ok(_) => log::debug!("Done!"),
        Err(e) => log::error!("Error: {}", e),
    }
}

async fn mma() -> Result<()> {
    let mat = Command::new(crate_name!())
        .about("iamtxt.com tool")
        .version(crate_version!())
        .author(crate_authors!())
        .arg_required_else_help(true)
        .subcommand_required(false)
        .arg(
            Arg::new("uninstall")
                .long("uninstall")
                .help("删除所有相关文件")
                .action(clap::ArgAction::SetTrue)
                .num_args(0),
        )
        .arg(
            Arg::new("init")
                .long("init")
                .help("初始化")
                .action(clap::ArgAction::SetTrue)
                .num_args(0)
                .conflicts_with("uninstall"),
        )
        .arg(
            Arg::new("clean")
                .long("clean")
                .help("清理垃圾")
                .action(clap::ArgAction::SetTrue)
                .num_args(0)
                .conflicts_with_all(&["uninstall", "init"]),
        )
        .arg(
            Arg::new("att")
                .short('a')
                .long("att")
                .help("签到")
                .action(clap::ArgAction::SetTrue)
                .num_args(0)
                .conflicts_with_all(&["uninstall", "clean", "init"]),
        )
        .get_matches();

    let botdirs = BotDirs::new()?;

    if mat.get_flag("uninstall") {
        log::info!("uninstalling...");
        botdirs.uninstall()?;
    } else if mat.get_flag("clean") {
        let stat = libs::status::get_status(botdirs.status_path())?;
        let stat = Arc::new(Mutex::new(stat));
        let config = libs::config::get_config(botdirs.config_path())?;
        libs::bot::clean_stat(stat.clone(), config)?;
        libs::status::save_status(stat, botdirs.status_path())?;
    } else if mat.get_flag("init") {
        botdirs.init()?;
        let _stat = libs::status::get_status(botdirs.status_path())?;
        let _config = libs::config::get_config(botdirs.config_path())?;
    } else if mat.get_flag("att") {
        let status = Arc::new(Mutex::new(get_status(botdirs.status_path())?));
        let config = libs::config::get_config(botdirs.config_path())?;
        libs::bot::att_now_all(config, status.clone()).await?;
        libs::status::save_status(status, botdirs.status_path())?;
    } else {
        return Err(anyhow::anyhow!("unreachable!"));
    }

    Ok(())
}

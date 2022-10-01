use std::sync::{Arc, Mutex};

use anyhow::Result;
use clap::{crate_authors, crate_name, crate_version, Arg, Command};
use libs::{botdir::BotDirs, status::get_status};
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
    let mat = Command::new(crate_name!())
        .about("iamtxt.com tool")
        .version(crate_version!())
        .author(crate_authors!())
        .arg_required_else_help(true)
        .args_conflicts_with_subcommands(true)
        .arg(
            Arg::new("uninstall")
                .long("uninstall")
                .help("删除所有相关文件")
                .action(clap::ArgAction::SetTrue)
                .takes_value(false),
        )
        .arg(
            Arg::new("init")
                .long("init")
                .help("初始化")
                .action(clap::ArgAction::SetTrue)
                .takes_value(false),
        )
        .arg(
            Arg::new("clean")
                .long("clean")
                .help("清理垃圾")
                .action(clap::ArgAction::SetTrue)
                .takes_value(false),
        )
        .subcommand(
            Command::new("att")
                .about("attendance")
                .short_flag('a')
                .long_flag("att")
                .arg_required_else_help(true)
                .args_conflicts_with_subcommands(true)
                .arg(
                    Arg::new("now")
                        .long("now")
                        .action(clap::ArgAction::SetTrue)
                        .takes_value(false),
                )
                .arg(
                    Arg::new("run")
                        .long("run")
                        .action(clap::ArgAction::SetTrue)
                        .takes_value(true)
                        .conflicts_with("now"),
                ),
        )
        .get_matches();

    let botdirs = BotDirs::new()?;
    // let config = libs::config::get_config(botdirs.config_path())?;

    if mat.get_flag("uninstall") {
        log::info!("uninstalling...");
        botdirs.uninstall()?;
    } else if mat.get_flag("clean") {
        let mut stat = libs::status::get_status(botdirs.status_path())?;
        let config = libs::config::get_config(botdirs.config_path())?;
        libs::bot::clean_stat(&mut stat, config)?;
        libs::status::save_status(Arc::new(Mutex::new(stat)), botdirs.status_path())?;
    } else if mat.get_flag("init") {
        botdirs.init()?;
        let _stat = libs::status::get_status(botdirs.status_path())?;
        let _config = libs::config::get_config(botdirs.config_path())?;
    } else {
        let status = Arc::new(Mutex::new(get_status(botdirs.status_path())?));
        match mat.subcommand() {
            Some(("att", att_sub)) => {
                if att_sub.get_flag("now") {
                    let config = libs::config::get_config(botdirs.config_path())?;
                    libs::bot::att_now_all(config, status.clone()).await?;
                } else if att_sub.get_flag("run") {
                    // 一直运行
                } else {
                }
            }
            _ => {}
        }
        libs::status::save_status(status, botdirs.status_path())?;
    }

    Ok(())
}

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
        Ok(_) => {}
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
                .exclusive(true)
                .num_args(0),
        )
        .arg(
            Arg::new("init")
                .long("init")
                .help("初始化")
                .action(clap::ArgAction::SetTrue)
                .num_args(0)
                .exclusive(true),
        )
        .arg(
            Arg::new("clean")
                .long("clean")
                .help("清理垃圾")
                .action(clap::ArgAction::SetTrue)
                .exclusive(true)
                .num_args(0),
        )
        .arg(
            Arg::new("att")
                .short('a')
                .long("att")
                .exclusive(true)
                .help("签到")
                .action(clap::ArgAction::SetTrue)
                .num_args(0),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .exclusive(true)
                .help("查看配置文件内容")
                .action(clap::ArgAction::SetTrue)
                .num_args(0),
        )
        .arg(
            Arg::new("status")
                .short('s')
                .long("status")
                .exclusive(true)
                .help("查看状态文件内容")
                .action(clap::ArgAction::SetTrue)
                .num_args(0),
        )
        .arg(
            Arg::new("info")
                .short('i')
                .long("info")
                .exclusive(true)
                .help("查询积分信息")
                .action(clap::ArgAction::SetTrue)
                .num_args(0),
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
    } else if mat.get_flag("config") {
        let config = libs::config::get_config(botdirs.config_path())?;
        log::info!("配置文件: {}", botdirs.config_path().display());
        let mut t_num = 0;
        let config_len = config.len();
        for i in config.values() {
            log::info!("{}-{}", i.name(), i.enable());
            if i.enable() {
                t_num += 1;
            }
        }
        log::info!(
            "{}个已启用，{}个已关闭，共{}个",
            t_num,
            config_len - t_num,
            config_len
        );
    } else if mat.get_flag("status") {
        log::info!("状态文件: {}", botdirs.status_path().display());
        let status = libs::status::get_status(botdirs.status_path())?;
        let mut t_num = 0;
        let status_len = status.len();
        for (x, (y, z)) in &status {
            log::info!("{}-[{}]-{}", x, y, z);
            if *z {
                t_num += 1;
            }
        }
        log::info!(
            "{}个已完成，{}个失败，共{}个",
            t_num,
            status_len - t_num,
            status_len
        );
    } else if mat.get_flag("info") {
        log::info!("查询积分信息");
        let config = libs::config::get_config(botdirs.config_path())?;
        let mut handles = vec![];
        for (_, v) in config.into_iter() {
            handles.push(tokio::spawn(async move {
                let session = libs::bot::get_session(v);
                if session.login().await.is_err() {
                    log::warn!("{} 登陆失败", session.userconf().name());
                }
                session.get_info().await
            }))
        }
        for i in handles.into_iter() {
            let i = i.await;
            if i.is_ok() {
                let i = i.unwrap();
                match i {
                    Ok(e) => log::info!("{e}"),
                    Err(e) => log::warn!("{e}"),
                }
            } else {
                log::error!("内部错误");
            }
        }
    } else {
        return Err(anyhow::anyhow!("unreachable!"));
    }

    Ok(())
}

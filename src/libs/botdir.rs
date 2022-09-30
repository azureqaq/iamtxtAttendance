use anyhow::Result;
use clap::crate_name;
use platform_dirs::AppDirs;
use std::fs::remove_dir_all;
use std::path::Path;
use std::path::PathBuf;

pub struct BotDirs {
    config_dir: PathBuf,
    config_path: PathBuf,
    status_dir: PathBuf,
    status_path: PathBuf,
}

impl BotDirs {
    pub fn new() -> Result<Self> {
        let app = AppDirs::new(Some(crate_name!()), false).unwrap();
        let config_path = app.config_dir.join("config.toml");
        let status_path = app.state_dir.join("iamtxtool_status.json");
        Ok(Self {
            config_dir: app.config_dir,
            status_dir: app.state_dir,
            config_path,
            status_path,
        })
    }

    pub fn config_path(&self) -> &Path {
        self.config_path.as_path()
    }

    pub fn status_path(&self) -> &Path {
        self.status_path.as_path()
    }

    pub fn uninstall(&self) -> Result<()> {
        let dirlist = [&self.config_dir, &self.status_dir];
        for dir in dirlist {
            if dir.is_dir() {
                remove_dir_all(dir)?;
                log::info!("remove dir: {}", dir.display());
            }
        }
        Ok(())
    }

    pub fn init(&self) -> Result<()> {
        let dirlist = [&self.config_dir, &self.status_dir];
        for dir in dirlist {
            if !dir.is_dir() {
                std::fs::create_dir_all(dir)?;
                log::info!("crate dir: {}", dir.display());
            }
        }
        Ok(())
    }
}

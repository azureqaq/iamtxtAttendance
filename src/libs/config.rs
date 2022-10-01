// #![allow(unused)]
use ahash::AHashMap;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::{
    fs::{read_to_string, File},
    path::Path,
};

use crate::status::StatusFile;

/// 配置文件
pub type Config = AHashMap<String, UserConf>;

/// 用户配置
#[derive(Serialize, Deserialize, Debug)]
pub struct UserConf {
    /// 是否开启
    enable: bool,
    /// 登录名
    name: String,
    /// 登录密码
    pwd: String,
    /// 重试次数
    retry_times: Option<u8>,
}

impl UserConf {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn pwd(&self) -> &str {
        &self.pwd
    }

    /// 0 或者不写 就是 1
    pub fn retry_times(&self) -> u8 {
        match self.retry_times {
            None => 1,
            Some(0) => 1,
            Some(n) => n,
        }
    }

    pub fn enable(&self) -> bool {
        self.enable
    }

    pub fn need_att(&self, stat: Arc<Mutex<StatusFile>>) -> bool {
        let lock = stat.lock().unwrap();
        let stat = lock.get(self.name());
        if stat.is_none() && self.enable() {
            return true;
        } else if stat.is_some() && self.enable() {
            let stat = stat.unwrap();
            if !stat.1 {
                return true;
            } else {
                let date = time::Date::parse(
                    &stat.0,
                    &time::format_description::parse("[year]-[month]-[day]").unwrap(),
                );
                if date.is_err() {
                    return true;
                } else {
                    let date = date.unwrap();
                    let today = time::OffsetDateTime::now_local().unwrap().date();
                    return today != date;
                }
            }
        } else if !self.enable() {
            return false;
        }
        false
    }
}

impl Default for UserConf {
    fn default() -> Self {
        Self {
            enable: false,
            name: "myname".into(),
            pwd: "mypwd".into(),
            retry_times: None,
        }
    }
}

/// 转换
pub fn to_time_conf(con: &str) -> Result<time::Time> {
    Ok(time::Time::parse(
        con,
        &time::format_description::parse("[hour]:[minute]:[second]").unwrap(),
    )?)
}

/// 转换
pub fn to_time_stat(con: &str) -> Result<time::Time> {
    Ok(time::Time::parse(
        con,
        &time::format_description::parse("[year]-[month]-[day]").unwrap(),
    )?)
}

/// 读取配置文件，如果不存在就新建一个默认的
pub fn get_config(path: impl AsRef<Path>) -> Result<Config> {
    let path = path.as_ref();
    if path.is_file() {
        let content_str = read_to_string(path)?;
        let config: Config = toml::from_str(&content_str)?;
        Ok(config)
    } else {
        let mut config = Config::new();
        let id = "mynote".to_string();
        let user = UserConf::default();
        config.insert(id, user);
        let mut f = File::create(path).context(anyhow!("可以尝试 init"))?;
        log::info!("crate file: {}", path.display());
        write!(f, "{}", toml::to_string(&config)?)?;
        Err(anyhow!(
            "find a config with default settings: {}",
            path.display()
        ))
    }
}

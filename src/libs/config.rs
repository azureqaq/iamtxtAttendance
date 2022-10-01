// #![allow(unused)]
use ahash::AHashMap;
use anyhow::{anyhow, Result};
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
    /// 签到时间
    time: Option<String>,
    /// 重试次数
    retry_times: Option<u8>,
    /// 是否开启邮件通知
    email_enable: bool,
    /// 邮件
    email: Option<String>,
}

impl UserConf {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn pwd(&self) -> &str {
        &self.pwd
    }

    pub fn retry_times(&self) -> Option<u8> {
        match self.retry_times {
            None => None,
            Some(0) => None,
            Some(n) => Some(n),
        }
    }

    pub fn email_enable(&self) -> bool {
        self.email.is_some() && self.email_enable
    }

    pub fn enable(&self) -> bool {
        self.enable
    }

    pub fn time_str(&self) -> &str {
        match &self.time {
            None => "00:00",
            Some(s) => s,
        }
    }

    pub fn time(&self) -> Result<time::Time> {
        to_time_conf(self.time_str())
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
        }
        true
    }
}

impl Default for UserConf {
    fn default() -> Self {
        Self {
            enable: false,
            email: Some("123@qq.com".into()),
            email_enable: true,
            name: "myname".into(),
            pwd: "mypwd".into(),
            retry_times: None,
            time: Some("00:09".into()),
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
        let mut f = File::create(path)?;
        log::info!("crate file: {}", path.display());
        write!(f, "{}", toml::to_string(&config)?)?;
        Err(anyhow!(
            "find a config with default settings: {}",
            path.display()
        ))
    }
}

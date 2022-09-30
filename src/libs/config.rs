// #![allow(unused)]
use ahash::AHashMap;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::{
    fs::{read_to_string, File},
    path::Path,
};

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
        self.retry_times
    }

    pub fn email_enable(&self) -> bool {
        self.email.is_some() && self.email_enable
    }

    pub fn enable(&self) -> bool {
        self.enable
    }

    pub fn time(&self) -> &str {
        match &self.time {
            None => "00:00",
            Some(s) => s,
        }
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

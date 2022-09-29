use crate::config::{Config, UserConf};
use anyhow::{anyhow, Result};
use reqwest::{cookie::Jar, header, Client, ClientBuilder};
use serde_json::json;
use std::{sync::Arc, time::Duration};

pub struct Session {
    client: Client,
    userconf: UserConf,
}

impl Session {
    pub fn new(userconf: UserConf) -> Self {
        let mut head = header::HeaderMap::new();
        head.insert("User-Agent", 
        header::HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/105.0.0.0 Safari/537.36 Edg/105.0.1343.53"));
        let client = ClientBuilder::new()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(10))
            .default_headers(head)
            .cookie_store(true)
            .cookie_provider(Arc::new(Jar::default()))
            .connection_verbose(false)
            .build()
            .unwrap();
        Self { client, userconf }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    /// 登录
    pub async fn login(&self) -> Result<()> {
        let _res = self
            .client
            .post("https://www.iamtxt.com/e/member/doaction.php")
            .form(&json!(
                {
                    "ecmsfrom": "https://www.iamtxt.com/",
                    "enews": "login",
                    "tobind": "0",
                    "username": self.userconf.name(),
                    "password": self.userconf.pwd(),
                    "lifetime": "315360000",
                    "Submit": " 登 录 "
                }
            ))
            .send()
            .await?;

        // if !res.text().await?.contains(self.userconf.name()) {
        //     return Err(anyhow!("登陆失败！ {}", self.userconf.name()));
        // }

        Ok(())
    }

    /// 登陆后 签到一次
    pub async fn att_once(&self) -> Result<()> {
        // https://www.iamtxt.com/e/extend/signin.php
        let res = self
            .client
            .post("https://www.iamtxt.com/e/extend/signin.php")
            .form(&json!({
                "userid": "0"
            }))
            .send()
            .await?;
        log::debug!("{} 签到结果: {}", self.userconf.name(), res.text().await?);
        Ok(())
    }
}

/// get a session
pub fn get_session(userconf: UserConf) -> Session {
    let ss = Session::new(userconf);
    ss
}

pub async fn att_now_all(config: Config) -> Result<()> {
    let mut result = vec![];
    for (_, userconf) in config.into_iter() {
        result.push(tokio::spawn(async move {
            let ss = get_session(userconf);
            if let Err(e) = ss.login().await {
                return Err(anyhow!("can't login: {}, error: {}", ss.userconf.name(), e));
            }
            if let Err(e) = ss.att_once().await {
                return Err(anyhow!("can't att: {}, error: {}", ss.userconf.name(), e));
            }
            Ok(())
        }))
    }

    for i in result {
        let ii = i.await;
        if let Err(e) = ii {
            log::error!("can't await, error: {}", e);
            continue;
        } else {
            if let Err(e) = ii.unwrap() {
                log::error!("att error: {}", e);
            }
        }
    }

    Ok(())
}
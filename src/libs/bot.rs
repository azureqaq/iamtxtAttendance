use crate::{
    config::{Config, UserConf},
    status::StatusFile,
};
use anyhow::{anyhow, Result};
use reqwest::{cookie::Jar, header, Client, ClientBuilder};
use serde_json::json;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

pub struct Session {
    client: Client,
    userconf: UserConf,
}

impl Session {
    pub fn new(userconf: UserConf) -> Self {
        let mut head = header::HeaderMap::new();
        head.insert(
            "User-Agent",
            header::HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
                AppleWebKit/537.36 (KHTML, like Gecko) \
                Chrome/105.0.0.0 Safari/537.36 Edg/105.0.1343.53",
            ),
        );
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

    /// 用户设置
    pub fn userconf(&self) -> &UserConf {
        &self.userconf
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

        Ok(())
    }

    /// 登陆后 签到一次 是否签到应该在调用前判断 此不包括 login
    /// 并更新状态
    async fn att_once(&self, status: Arc<Mutex<StatusFile>>) -> Result<()> {
        // https://www.iamtxt.com/e/extend/signin.php
        let res = self
            .client
            .post("https://www.iamtxt.com/e/extend/signin.php")
            .form(&json!({
                "userid": "0"
            }))
            .send()
            .await?;
        let text = res.text().await?;
        // log::debug!(r#"签到结果[{}]: "{}""#, self.userconf.name(), &text);
        let today = crate::config::get_today().to_string();

        // 更新状态
        {
            if text.contains("已连签") || text.contains("今天已经") {
                let mut lock = status.lock().unwrap();
                lock.insert(self.userconf.name().into(), (today, true));
                Ok(())
            } else if text.contains("nolog") {
                // log::warn!("{}, 登陆失败!", self.userconf.name());
                // 如果是true就跳过
                let mut lock = status.lock().unwrap();
                let old = lock.get(self.userconf.name());
                if old.is_some() {
                    let old = old.unwrap();
                    if old.1 {
                        Ok(())
                    } else {
                        Err(anyhow!("登陆失败!"))
                    }
                } else {
                    lock.insert(self.userconf.name().into(), (today, false));
                    Err(anyhow!(" 登陆失败!"))
                }
            } else {
                // log::warn!("未处理的情况: {}", text);
                Err(anyhow!("未处理的情况: {}", text))
            }
        }
    }

    /// 尝试几次
    async fn att_times(&self, status: Arc<Mutex<StatusFile>>) -> Result<()> {
        let mut this_error = String::new();
        for i in 0..self.userconf.retry_times() {
            let status = status.clone();
            match self.att_once(status).await {
                Ok(_) => {
                    log::info!(
                        "{}/{}签到成功！{}",
                        i + 1,
                        self.userconf.retry_times(),
                        self.userconf.name()
                    );
                    return Ok(());
                }
                Err(e) => {
                    /*
                    log::debug!(
                        "第{}/{}次尝试签到失败, error: {}",
                        i + 1,
                        self.userconf.retry_times(),
                        e
                    );
                    */
                    this_error = e.to_string();
                    continue;
                }
            }
        }
        Err(anyhow!(
            "{}签到失败, error: {}",
            self.userconf.name(),
            this_error
        ))
    }

    /// 查询积分
    pub async fn get_info(&self) -> Result<String> {
        let url = "https://www.iamtxt.com/e/member/cp/";
        let html = self.client().get(url).send().await?.text().await?;
        let re = regex::Regex::new(r#"\d+"#).unwrap();
        for line in html.split('\n') {
            let line = line.trim();
            if line.contains(r#"点 ["#) {
                let res = line.trim_end_matches(r#"点 ["#);
                let res = re.captures(res).and_then(|e| e.get(0));
                if res.is_none() {
                    continue;
                } else {
                    let res = res.unwrap().as_str();
                    return Ok(res.to_string() + "点 " + self.userconf.name());
                }
            }
        }
        Err(anyhow!("无法找到 {} 的积分信息", self.userconf.name()))
    }
}

/// get a session
pub fn get_session(userconf: UserConf) -> Session {
    
    Session::new(userconf)
}

pub async fn att_now_all(config: Config, status: Arc<Mutex<StatusFile>>) -> Result<()> {
    let mut result = vec![];
    for (_, userconf) in config.into_iter() {
        if !userconf.need_att(status.clone()) {
            continue;
        }
        let status = status.clone();
        result.push(tokio::spawn(async move {
            let ss = get_session(userconf);
            if let Err(e) = ss.login().await {
                return Err(anyhow!("登陆失败: {}, 错误: {}", ss.userconf.name(), e));
            }
            if let Err(e) = ss.att_times(status).await {
                return Err(anyhow!("{}, 错误: {}", ss.userconf.name(), e));
            }
            Ok(())
        }))
    }

    for i in result {
        let ii = i.await;
        if let Err(e) = ii {
            log::error!("can't await, error: {}", e);
            continue;
        } else if let Err(e) = ii.unwrap() {
            log::error!("{}", e);
        }
    }

    Ok(())
}

pub async fn att_times(_config: Config, _status: Arc<Mutex<StatusFile>>) -> Result<()> {
    Err(anyhow!("unimplemented!"))
}

/// 在状态文件中清理不存在的
pub fn clean_stat(status: Arc<Mutex<StatusFile>>, config: Config) -> Result<()> {
    let mut res = vec![];
    {
        let status = status.lock().unwrap();
        let ids: ahash::AHashSet<String> = config
            .into_iter()
            .map(|(_s, conf)| conf.name().to_owned())
            .collect();
        for i in status.keys() {
            if !ids.contains(i) {
                res.push(i.to_owned());
            }
        }
    }

    {
        let mut status = status.lock().unwrap();
        for i in res {
            status.remove(&i);
            log::debug!("删除不需要的状态: {}", i);
        }
    }

    Ok(())
}

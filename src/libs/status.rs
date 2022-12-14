use anyhow::{anyhow, Context, Result};
use std::io::Write;
use std::{
    fs::{read_to_string, File},
    path::Path,
    sync::{Arc, Mutex},
};

/// 状态文件
///
/// id，日期，是否签到成功，今天签到的结果
pub type StatusFile = ahash::AHashMap<String, (String, bool, String)>;

pub fn get_status(path: impl AsRef<Path>) -> Result<StatusFile> {
    let path = path.as_ref();
    if path.is_file() {
        let content = read_to_string(path)?;
        let content: StatusFile = serde_json::from_str(&content).context(anyhow!(
            "无法解析!, 可以尝试删除文件后重试: {}",
            path.display()
        ))?;
        Ok(content)
    } else {
        let content = StatusFile::new();
        let mut f = File::create(path).context(anyhow!("可能需要先init"))?;
        log::info!("crate file: {}", path.display());
        write!(f, "{}", serde_json::to_string(&content)?)?;
        Ok(content)
    }
}

pub fn save_status(content: Arc<Mutex<StatusFile>>, path: impl AsRef<Path>) -> Result<()> {
    let content = content.as_ref();
    let path = path.as_ref();
    let buf = File::create(path).map(std::io::BufWriter::new)?;
    let res = serde_json::to_writer_pretty(buf, &content);
    if let Err(e) = res {
        Err(anyhow!("无法保存状态: {}, 错误: {}", path.display(), e))
    } else {
        // log::debug!("保存结果到: {}", path.display());
        Ok(())
    }
}

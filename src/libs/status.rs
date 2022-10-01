use std::{
    fs::{read_to_string, File},
    path::Path,
    sync::{Arc, Mutex},
};
// use serde::{Serialize, Deserialize};
use anyhow::{anyhow, Context, Result};
use std::io::Write;

pub type StatusFile = ahash::AHashMap<String, (String, bool)>;

pub fn get_status(path: impl AsRef<Path>) -> Result<StatusFile> {
    let path = path.as_ref();
    if path.is_file() {
        let content = read_to_string(path)?;
        let content: StatusFile = serde_json::from_str(&content).context(anyhow!(
            "无法解析!, 可以尝试删除文件后重试: {}",
            path.display()
        ))?;
        return Ok(content);
    } else {
        let content = StatusFile::new();
        let mut f = File::create(path)?;
        log::info!("crate file: {}", path.display());
        write!(f, "{}", serde_json::to_string(&content)?)?;
        return Ok(content);
    }
}

pub fn save_status(content: Arc<Mutex<StatusFile>>, path: impl AsRef<Path>) -> Result<()> {
    let content = content.as_ref();
    let path = path.as_ref();
    let buf = File::create(path).map(std::io::BufWriter::new)?;
    let res = serde_json::to_writer_pretty(buf, &content);
    if let Err(e) = res {
        return Err(anyhow!("无法保存状态: {}, 错误: {}", path.display(), e));
    } else {
        log::debug!("保存结果到: {}", path.display());
        Ok(())
    }
}

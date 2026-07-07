use crate::detect::exe_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 持久化设置（仅记住少量用户偏好，其余每次启动用默认值）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    pub exe_path: String,
    pub save_dir: String,
    pub proxy_address: String,
    pub headers: String,
    pub external_console: bool,
}

impl Settings {
    /// 配置文件默认位于「可执行文件所在目录/config/config.json」。
    fn path() -> PathBuf {
        match exe_dir() {
            Some(dir) => dir.join("config").join("config.json"),
            None => PathBuf::from("config").join("config.json"),
        }
    }

    pub fn load() -> Settings {
        let p = Self::path();
        if let Ok(s) = fs::read_to_string(&p) {
            if let Ok(s) = serde_json::from_str::<Settings>(&s) {
                return s;
            }
        }
        Settings::default()
    }

    pub fn save(&self) {
        let p = Self::path();
        if let Some(dir) = p.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Ok(s) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&p, s);
        }
    }
}

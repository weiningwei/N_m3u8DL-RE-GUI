use crate::app::ThemeMode;
use crate::detect::exe_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 持久化设置（仅记住少量用户偏好，其余每次启动用默认值）
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub exe_path: String,
    pub save_dir: String,
    pub proxy_address: String,
    pub headers: String,
    pub external_console: bool,
    #[serde(default)]
    pub theme_mode: ThemeMode,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Settings {
        Settings {
            exe_path: "C:/x/N_m3u8DL-RE.exe".into(),
            save_dir: "D:/out".into(),
            proxy_address: "http://127.0.0.1:8888".into(),
            headers: "X-Test: 1".into(),
            external_console: true,
            theme_mode: ThemeMode::Dark,
        }
    }

    #[test]
    fn serialize_round_trip() {
        let s = sample();
        let json = serde_json::to_string(&s).expect("serialize");
        let back: Settings = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(s, back);
    }

    #[test]
    fn missing_theme_mode_defaults_to_system() {
        // 旧版配置（无 theme_mode 字段）反序列化时应回退到 System，
        // 由 #[serde(default)] 保证向后兼容。
        let json = r#"{"exe_path":"x","save_dir":"","proxy_address":"","headers":"","external_console":false}"#;
        let s: Settings = serde_json::from_str(json).expect("deserialize");
        assert_eq!(s.theme_mode, ThemeMode::System);
    }
}

use std::path::PathBuf;

/// 当前 GUI 可执行文件所在目录。
pub fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
}

/// 日志文件默认路径：可执行文件所在目录下的 `log/n_m3u8dl-re-gui.log`。
pub fn default_log_path() -> String {
    let p = match exe_dir() {
        Some(dir) => dir.join("log").join("n_m3u8dl-re-gui.log"),
        None => PathBuf::from("log").join("n_m3u8dl-re-gui.log"),
    };
    p.to_string_lossy().into_owned()
}

/// 临时文件目录默认路径：可执行文件所在目录下的 `temp`。
pub fn default_temp_path() -> String {
    let p = match exe_dir() {
        Some(dir) => dir.join("temp"),
        None => PathBuf::from("temp"),
    };
    p.to_string_lossy().into_owned()
}

/// 保存目录默认路径：可执行文件所在目录下的 `downloads`。
pub fn default_save_dir() -> String {
    let p = match exe_dir() {
        Some(dir) => dir.join("downloads"),
        None => PathBuf::from("downloads"),
    };
    p.to_string_lossy().into_owned()
}

/// 定位 N_m3u8DL-RE.exe：优先用用户指定的路径，否则尝试与 GUI 同目录、再尝试 PATH。
pub fn locate_exe(preferred: &str) -> Option<String> {
    let name = if cfg!(windows) { "N_m3u8DL-RE.exe" } else { "N_m3u8DL-RE" };

    // 1. 用户指定且存在
    if !preferred.trim().is_empty() {
        if std::path::Path::new(preferred).is_file() {
            return Some(preferred.to_string());
        }
    }

    // 2. 与当前可执行文件同目录
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate.to_string_lossy().into_owned());
            }
        }
    }

    // 3. PATH 中查找
    if let Ok(path_var) = std::env::var("PATH") {
        for p in std::env::split_paths(&path_var) {
            let candidate = p.join(name);
            if candidate.is_file() {
                return Some(candidate.to_string_lossy().into_owned());
            }
        }
    }

    None
}

/// ffmpeg 可执行文件名（按平台）。
pub(crate) fn ffmpeg_file_name() -> &'static str {
    if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" }
}

/// 在目录列表中按给定顺序查找 ffmpeg，返回第一个命中的完整路径。
/// 抽取为纯函数以便对检测顺序做单元测试（不依赖全局 PATH / exe 状态）。
fn find_ffmpeg_in_dirs(preferred: &str, dirs: &[std::path::PathBuf]) -> Option<String> {
    let name = ffmpeg_file_name();
    if !preferred.trim().is_empty() && std::path::Path::new(preferred).is_file() {
        return Some(preferred.to_string());
    }
    for d in dirs {
        let candidate = d.join(name);
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().into_owned());
        }
    }
    None
}

/// 定位 ffmpeg：优先用用户指定路径，否则按 RE 同级目录 → GUI 同级目录 →
/// 环境变量 PATH 的顺序查找。本地随附的 ffmpeg 优先于全局 PATH。
/// ffmpeg 是 RE 合并/混流的必需依赖，缺失会导致下载直接失败。
pub fn locate_ffmpeg(preferred: &str) -> Option<String> {
    let mut dirs: Vec<std::path::PathBuf> = Vec::new();

    // 1. RE（N_m3u8DL-RE.exe）同级目录：本地随附的 ffmpeg 优先
    if let Some(exe) = locate_exe("") {
        if let Some(dir) = std::path::Path::new(&exe).parent() {
            dirs.push(dir.to_path_buf());
        }
    }

    // 2. GUI 同级目录
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            dirs.push(dir.to_path_buf());
        }
    }

    // 3. 环境变量 PATH
    if let Ok(path_var) = std::env::var("PATH") {
        for p in std::env::split_paths(&path_var) {
            dirs.push(p.to_path_buf());
        }
    }

    find_ffmpeg_in_dirs(preferred, &dirs)
}

/// 校验文件路径字符串的“格式”合法性（非存在性检查）。
/// - 空串视为合法：可选路径留空由调用方判断必填。
/// - 拒绝包含控制字符，或 Windows 非法文件名字符（< > " | ? *）。
///   注意：不拒绝 `:`（盘符如 `C:\` 需要）与路径分隔符 `\` `/`。
pub fn validate_path_format(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Ok(());
    }
    if path.contains(|c: char| c.is_control()) {
        return Err("包含非法控制字符".to_string());
    }
    #[cfg(windows)]
    {
        for c in ['<', '>', '"', '|', '?', '*'] {
            if path.contains(c) {
                return Err(format!("包含 Windows 非法字符 '{c}'"));
            }
        }
    }
    #[cfg(not(windows))]
    {
        if path.contains('\0') {
            return Err("包含非法空字符".to_string());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn mk_temp(tag: u32) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("nm3u8_ff_{}_{}", std::process::id(), tag));
        let _ = fs::create_dir_all(&d);
        d
    }

    fn rm(d: &std::path::PathBuf) {
        let _ = fs::remove_dir_all(d);
    }

    /// 验证不同目录下的检测顺序：RE 目录 > GUI 目录 > PATH 目录。
    #[test]
    fn detection_order_across_dirs() {
        let re = mk_temp(1);
        let gui = mk_temp(2);
        let path = mk_temp(3);
        let name = ffmpeg_file_name();

        // 仅 PATH 目录有 ffmpeg → 选中 PATH
        fs::write(path.join(name), b"").unwrap();
        let dirs = vec![re.clone(), gui.clone(), path.clone()];
        assert_eq!(find_ffmpeg_in_dirs("", &dirs), Some(path.join(name).to_string_lossy().into_owned()));

        // GUI 目录也放入 → 选中更靠前的 GUI
        fs::write(gui.join(name), b"").unwrap();
        assert_eq!(find_ffmpeg_in_dirs("", &dirs), Some(gui.join(name).to_string_lossy().into_owned()));

        // RE 目录也放入 → 选中最靠前的 RE
        fs::write(re.join(name), b"").unwrap();
        assert_eq!(find_ffmpeg_in_dirs("", &dirs), Some(re.join(name).to_string_lossy().into_owned()));

        // 用户指定路径优先于所有目录
        let pref = mk_temp(4);
        fs::write(pref.join(name), b"").unwrap();
        assert_eq!(
            find_ffmpeg_in_dirs(pref.join(name).to_str().unwrap(), &dirs),
            Some(pref.join(name).to_string_lossy().into_owned())
        );
        rm(&pref);

        // 全部不存在 → None
        rm(&re);
        rm(&gui);
        rm(&path);
        assert_eq!(find_ffmpeg_in_dirs("", &dirs), None);
    }

    #[test]
    fn no_ffmpeg_returns_none() {
        let dirs = vec![mk_temp(5), mk_temp(6)];
        assert!(find_ffmpeg_in_dirs("", &dirs).is_none());
        for d in &dirs {
            rm(d);
        }
    }

    #[test]
    fn path_format_accepts_valid() {
        // 空串（可选路径留空）合法
        assert!(validate_path_format("").is_ok());
        // 含盘符与分隔符的合法 Windows 路径
        assert!(validate_path_format("D:\\Downloads\\视频").is_ok());
        assert!(validate_path_format("C:/a/b/c.mp4").is_ok());
        assert!(validate_path_format("/usr/local/bin/ffmpeg").is_ok());
    }

    #[test]
    fn path_format_rejects_invalid() {
        assert!(validate_path_format("a/b?c").is_err()); // ?
        assert!(validate_path_format("a*b").is_err()); // *
        assert!(validate_path_format("a<b").is_err()); // <
        assert!(validate_path_format("a>b").is_err()); // >
        assert!(validate_path_format("a|b").is_err()); // |
        assert!(validate_path_format("a\"b").is_err()); // "
        assert!(validate_path_format("a\x01b").is_err()); // 控制字符
    }
}

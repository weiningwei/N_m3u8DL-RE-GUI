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

/// 定位 ffmpeg：优先用用户指定路径，否则在 PATH、RE 同目录、GUI 同目录查找。
/// ffmpeg 是 RE 合并/混流的必需依赖，缺失会导致下载直接失败。
pub fn locate_ffmpeg(preferred: &str) -> Option<String> {
    let name = if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" };

    if !preferred.trim().is_empty() {
        if std::path::Path::new(preferred).is_file() {
            return Some(preferred.to_string());
        }
    }

    // 1. PATH
    if let Ok(path_var) = std::env::var("PATH") {
        for p in std::env::split_paths(&path_var) {
            let candidate = p.join(name);
            if candidate.is_file() {
                return Some(candidate.to_string_lossy().into_owned());
            }
        }
    }

    // 2. RE 同目录
    if let Some(exe) = locate_exe("") {
        if let Some(dir) = std::path::Path::new(&exe).parent() {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate.to_string_lossy().into_owned());
            }
        }
    }

    // 3. GUI 同目录
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate.to_string_lossy().into_owned());
            }
        }
    }

    None
}

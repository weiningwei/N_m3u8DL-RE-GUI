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

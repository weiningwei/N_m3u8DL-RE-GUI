//! 仅在 Windows 上把仓库内的静态图标嵌入可执行文件资源（任务管理器/资源管理器读取的是 PE 资源图标）。
//! 图标文件为 `assets/icon.ico`（48×48 + 256×256 两档），不再在构建时生成。

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let icon = std::path::Path::new(&manifest).join("assets/icon.ico");
        let mut res = winresource::WindowsResource::new();
        res.set_icon(icon.to_str().unwrap());
        res.compile().expect("failed to embed Windows resources");
    }
}

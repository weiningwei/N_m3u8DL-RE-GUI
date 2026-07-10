#![windows_subsystem = "windows"]

mod app;
mod args;
mod config;
mod detect;
mod runner;
mod ui;

use app::App;
use iced::window::icon;
use iced::Font;

fn main() -> iced::Result {
    let font = load_cjk_font();
    let mut app = iced::application(App::new, App::update, App::view)
        .title(|_app: &App| "N_m3u8DL-RE GUI".to_string())
        .subscription(|app: &App| {
            iced::Subscription::batch([
                runner::make_subscription(app.run_gen, app.running),
                // 用 listen_with 监听“所有”键盘事件（含被控件捕获的，如 text_editor 的 Ctrl+C），
                // 这样日志区内 Ctrl+C 原生复制时也能补一个“已复制”提示。
                iced::event::listen_with(|event, _status, _window| match event {
                    iced::Event::Keyboard(ke) => Some(app::Message::KeyEvent(ke.clone())),
                    _ => None,
                }),
                // 复制命令/日志后，用定时订阅在约 500ms 后自动清除“已复制”提示
                if app.copied_at.is_some() || app.log_copied_at.is_some() {
                    iced::time::every(std::time::Duration::from_millis(100))
                        .map(|_| app::Message::Tick)
                } else {
                    iced::Subscription::none()
                },
            ])
        })
        .theme(|app: &App| app.theme())
        .default_font(font)
        .window(iced::window::Settings {
            size: iced::Size::new(1100.0, 760.0),
            min_size: Some(iced::Size::new(1000.0, 620.0)),
            icon: generate_icon(),
            ..Default::default()
        });
    // 若读到系统中文字体文件，注册到 iced 字体列表（确保 CJK 字形可用）
    if let Some(bytes) = read_cjk_font_bytes() {
        app = app.font(bytes);
    }
    app.run()
}

/// 指定中文字体族（微软雅黑），否则中文会显示为方块。
fn load_cjk_font() -> Font {
    Font::with_name("Microsoft YaHei")
}

/// 读取系统中文字体文件字节，用于注册到 iced 字体列表。
fn read_cjk_font_bytes() -> Option<Vec<u8>> {
    let candidates = [
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\msyh.ttf",
        "C:\\Windows\\Fonts\\simsun.ttc",
    ];
    for c in candidates {
        if let Ok(bytes) = std::fs::read(c) {
            return Some(bytes);
        }
    }
    None
}

/// 窗口图标：从仓库内的 `assets/icon.ico` 读取（与 build.rs 嵌入 PE 资源用的是同一个文件）。
/// 该 .ico 含多档尺寸（48×48 + 256×256），这里挑最大的一档给窗口用，清晰度最好。
fn generate_icon() -> Option<icon::Icon> {
    const ICON_BYTES: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icon.ico"));
    let cursor = std::io::Cursor::new(ICON_BYTES);
    let dir = ico::IconDir::read(cursor).ok()?;
    let entry = dir
        .entries()
        .iter()
        .max_by_key(|e| e.width().max(e.height()))?;
    let image = entry.decode().ok()?;
    icon::from_rgba(image.rgba_data().to_vec(), image.width(), image.height()).ok()
}

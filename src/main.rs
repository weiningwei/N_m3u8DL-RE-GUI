mod app;
mod args;
mod config;
mod detect;
mod runner;
mod ui;

use app::App;
use iced::Font;

fn main() -> iced::Result {
    let font = load_cjk_font();
    let mut app = iced::application(App::new, App::update, App::view)
        .title(|_app: &App| "N_m3u8DL-RE GUI".to_string())
        .subscription(|app: &App| runner::make_subscription(app.run_gen, app.running))
        .default_font(font)
        .window(iced::window::Settings {
            size: iced::Size::new(1100.0, 760.0),
            min_size: Some(iced::Size::new(1000.0, 620.0)),
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

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
        .subscription(|app: &App| runner::make_subscription(app.run_gen, app.running))
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

fn generate_icon() -> Option<icon::Icon> {
    let (data, w, h) = icon_pixels();
    icon::from_rgba(data, w, h).ok()
}

fn icon_pixels() -> (Vec<u8>, u32, u32) {
    let s: u32 = 64;
    let mut p = vec![0u8; (s * s * 4) as usize];
    let cx = s as f64 / 2.0;
    let r = 13.0;
    let hw = s as f64 / 2.0;

    for y in 0..s {
        for x in 0..s {
            let i = ((y * s + x) * 4) as usize;
            let fx = x as f64 + 0.5;
            let fy = y as f64 + 0.5;
            let lx = fx - cx;
            let ly = fy - cx;

            let adx = lx.abs() - (hw - r);
            let ady = ly.abs() - (hw - r);
            let inside = if adx > r || ady > r {
                false
            } else if adx > 0.0 && ady > 0.0 {
                adx.hypot(ady) <= r
            } else {
                true
            };

            if !inside {
                p[i + 3] = 0;
                continue;
            }

            let grad = 0.6 + 0.4 * (1.0 - fy / s as f64);
            let edge_d = adx.max(ady);
            if edge_d > r - 1.5 {
                p[i] = 18; p[i + 1] = 24; p[i + 2] = 88;
            } else if edge_d > r - 3.0 {
                p[i] = 22; p[i + 1] = 30; p[i + 2] = 106;
            } else {
                p[i] = (26.0 * grad) as u8;
                p[i + 1] = (35.0 * grad) as u8;
                p[i + 2] = (126.0 * grad) as u8;
            }
            p[i + 3] = 255;

            let in_shaft = lx >= -4.0 && lx <= 4.0 && ly >= -20.0 && ly <= 9.0;
            let in_head = ly >= 6.0 && ly <= 22.0 && {
                let t = (ly - 6.0) / 16.0;
                lx.abs() <= 16.0 * (1.0 - t)
            };

            if in_shaft || in_head {
                p[i] = 255; p[i + 1] = 255; p[i + 2] = 255;
            }
        }
    }
    (p, s, s)
}

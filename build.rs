fn main() {
    // 仅在 Windows 上嵌入图标资源（任务管理器、文件资源管理器等使用 PE 资源中的图标）
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let ico = generate_ico();
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let ico_path = std::path::Path::new(&out_dir).join("icon.ico");
        std::fs::write(&ico_path, &ico).expect("write icon.ico");

        let mut res = winresource::WindowsResource::new();
        res.set_icon(ico_path.to_str().unwrap());
        res.compile().expect("embed windows resources");
    }
}

/// 与 src/main.rs::icon_pixels() 一致的像素生成逻辑，输出 BGRA（自下而上行序）。
fn make_pixels(size: u32) -> Vec<u8> {
    let s = size;
    let mut p = vec![0u8; (s * s * 4) as usize];
    let cx = s as f64 / 2.0;
    let r = 13.0 * (s as f64 / 64.0);
    let hw = s as f64 / 2.0;

    for y in 0..s {
        for x in 0..s {
            // ICO BMP 是自下而上行序
            let ry = s - 1 - y;
            let i = ((ry * s + x) * 4) as usize;
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
            if edge_d > r - 1.5 * (s as f64 / 64.0) {
                p[i] = 18; p[i + 1] = 24; p[i + 2] = 88;
            } else if edge_d > r - 3.0 * (s as f64 / 64.0) {
                p[i] = 22; p[i + 1] = 30; p[i + 2] = 106;
            } else {
                p[i] = (26.0 * grad) as u8;
                p[i + 1] = (35.0 * grad) as u8;
                p[i + 2] = (126.0 * grad) as u8;
            }
            p[i + 3] = 255;

            let in_shaft = lx >= -4.0 * (s as f64 / 64.0)
                && lx <= 4.0 * (s as f64 / 64.0)
                && ly >= -20.0 * (s as f64 / 64.0)
                && ly <= 9.0 * (s as f64 / 64.0);
            let in_head = ly >= 6.0 * (s as f64 / 64.0)
                && ly <= 22.0 * (s as f64 / 64.0)
                && {
                    let t = (ly - 6.0 * (s as f64 / 64.0)) / (16.0 * (s as f64 / 64.0));
                    lx.abs() <= 16.0 * (s as f64 / 64.0) * (1.0 - t)
                };

            if in_shaft || in_head {
                p[i] = 255; p[i + 1] = 255; p[i + 2] = 255;
            }
        }
    }
    p
}

/// 将 RGBA 像素数据转为 ICO 所需的 BMP 数据段（含 BITMAPINFOHEADER + BGRA 像素 + AND 掩码）。
/// 返回 (bmp_data, and_mask_size)。
fn pixels_to_bmp_entry(pixels: &[u8], w: u32, h: u32) -> (Vec<u8>, usize) {
    use std::io::Write;

    // BITMAPINFOHEADER (40 bytes)
    let header_size: u32 = 40;
    let pixel_data_len = w as usize * h as usize * 4;
    // AND mask: 每行对齐到 4 字节边界，总大小为 ceil(w/32)*4 * h
    let and_stride = ((w as usize + 31) / 32) * 4;
    let and_mask_len = and_stride * h as usize;

    let image_size = pixel_data_len + and_mask_len;
    let total = header_size as usize + image_size;
    let mut bmp = Vec::with_capacity(total);

    bmp.write_all(&(header_size as u32).to_le_bytes()).unwrap();   // biSize
    bmp.write_all(&w.to_le_bytes()).unwrap();                     // biWidth
    bmp.write_all(&(h * 2).to_le_bytes()).unwrap();               // biHeight (doubled: color + mask)
    bmp.write_all(&1u16.to_le_bytes()).unwrap();                  // biPlanes
    bmp.write_all(&32u16.to_le_bytes()).unwrap();                 // biBitCount (BGRA)
    bmp.write_all(&0u32.to_le_bytes()).unwrap();                  // biCompression (BI_RGB)
    bmp.write_all(&image_size.to_le_bytes()).unwrap();            // biSizeImage
    bmp.extend_from_slice(&[0u8; 16]);                           // biXPelsPerMeter .. biClrImportant

    // BGRA pixel data (already bottom-up from make_pixels)
    bmp.extend_from_slice(pixels);

    // AND mask — 全 0 表示完全不透明（alpha 已在 BGRA 中编码）
    bmp.resize(total, 0);

    (bmp, and_mask_len)
}

/// 生成完整的 .ico 文件内容，包含 48×48 和 256×256 两档尺寸以适配各种场景（任务管理器、文件管理器等）。
fn generate_ico() -> Vec<u8> {
    use std::io::Write;

    let sizes: Vec<u32> = vec![48, 256];
    let entries: Vec<(Vec<u8>, u32)> = sizes
        .into_iter()
        .map(|sz| {
            let px = make_pixels(sz);
            let (bmp, _) = pixels_to_bmp_entry(&px, sz, sz);
            (bmp, sz)
        })
        .collect();

    let count = entries.len() as u16;
    // ICONDIR: 6 bytes + ICONDIRENTRY per image
    let dir_size = 6 + count as usize * 16;
    let data_offset = dir_size;
    let total_data: usize = entries.iter().map(|(b, _)| b.len()).sum();
    let total = dir_size + total_data;
    let mut ico = Vec::with_capacity(total);

    // ICONDIR
    ico.write_all(&[0u8, 0]).unwrap();              // reserved
    ico.write_all(&1u16.to_le_bytes()).unwrap();      // type (ICON)
    ico.write_all(&count.to_le_bytes()).unwrap();     // count

    let mut offset = data_offset as u32;
    for &(ref bmp, w) in &entries {
        let sz_u8 = if w >= 256 { 0 } else { w as u8 }; // 0 means 256 in ICO spec
        ico.write_all(&[sz_u8]).unwrap();             // width
        ico.write_all(&[sz_u8]).unwrap();             // height
        ico.write_all(&[0u8]).unwrap();               // color count (0 for >= 256 colors)
        ico.write_all(&[0u8]).unwrap();               // reserved
        ico.write_all(&1u16.to_le_bytes()).unwrap();   // planes
        ico.write_all(&32u16.to_le_bytes()).unwrap();  // bpp
        let _ = ico.write_all(&(bmp.len() as u32).to_le_bytes()); // size of image data
        let _ = ico.write_all(&offset.to_le_bytes());          // offset to image data
        offset += bmp.len() as u32;
    }

    for (bmp, _) in &entries {
        ico.extend_from_slice(bmp);
    }

    ico
}

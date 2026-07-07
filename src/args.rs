use crate::app::App;

/// 布尔选项：仅当与 RE 默认值不同时输出。
/// default=true 且当前为 false → 输出 `name false`；default=false 且当前为 true → 输出 `name`。
fn bool_opt(v: &mut Vec<String>, name: &str, on: bool, default: bool) {
    if on == default {
        return;
    }
    v.push(name.to_string());
    if !on {
        v.push("false".to_string());
    }
}

fn opt(v: &mut Vec<String>, name: &str, value: &str) {
    if !value.trim().is_empty() {
        v.push(name.to_string());
        v.push(value.trim().to_string());
    }
}

/// 构建 N_m3u8DL-RE 的命令行参数（不含 exe 与 input 位置参数）。
pub fn build_args(app: &App) -> Vec<String> {
    let mut v = Vec::new();

    // 保存相关
    opt(&mut v, "--save-dir", &app.save_dir);
    opt(&mut v, "--save-name", &app.save_name);
    opt(&mut v, "--save-pattern", &app.save_pattern);
    opt(&mut v, "--tmp-dir", &app.tmp_dir);
    opt(&mut v, "--log-file-path", &app.log_file_path);
    opt(&mut v, "--base-url", &app.base_url);

    // 线程/重试/超时
    opt(&mut v, "--thread-count", &app.thread_count);
    opt(&mut v, "--download-retry-count", &app.retry_count);
    opt(&mut v, "--http-request-timeout", &app.http_timeout);

    // 请求头（每行一条 -H）
    let headers = app.headers.text();
    let headers = headers.trim();
    if !headers.is_empty() {
        for line in headers.lines() {
            let line = line.trim();
            if !line.is_empty() {
                v.push("-H".to_string());
                v.push(line.to_string());
            }
        }
    }

    // 代理
    match app.proxy_mode {
        crate::app::ProxyMode::System => v.push("--use-system-proxy".to_string()),
        crate::app::ProxyMode::Custom => opt(&mut v, "--custom-proxy", &app.proxy_address),
        crate::app::ProxyMode::None => { /* RE 无显式禁用代理选项，留空 */ }
    }

    // 限速/并发/语言
    opt(&mut v, "-R", &app.max_speed);
    bool_opt(&mut v, "--concurrent-download", app.concurrent_download, false);
    if !matches!(app.ui_language, crate::app::UiLanguage::Default) {
        let lang = match app.ui_language {
            crate::app::UiLanguage::EnUs => "en-US",
            crate::app::UiLanguage::ZhCn => "zh-CN",
            crate::app::UiLanguage::ZhTw => "zh-TW",
            _ => "",
        };
        opt(&mut v, "--ui-language", lang);
    }

    // 二进制路径
    opt(&mut v, "--ffmpeg-binary-path", &app.ffmpeg_path);
    if !matches!(app.decryption_engine, crate::app::DecryptionEngine::Default) {
        let e = match app.decryption_engine {
            crate::app::DecryptionEngine::Ffmpeg => "FFMPEG",
            crate::app::DecryptionEngine::Mp4decrypt => "MP4DECRYPT",
            crate::app::DecryptionEngine::ShakaPackager => "SHAKA_PACKAGER",
            _ => "",
        };
        opt(&mut v, "--decryption-engine", e);
    }
    opt(&mut v, "--decryption-binary-path", &app.decryption_binary);

    // 流选择
    bool_opt(&mut v, "--auto-select", app.auto_select, false);
    opt(&mut v, "-sv", &app.select_video);
    opt(&mut v, "-sa", &app.select_audio);
    opt(&mut v, "-ss", &app.select_subtitle);
    opt(&mut v, "-dv", &app.drop_video);
    opt(&mut v, "-da", &app.drop_audio);
    opt(&mut v, "-ds", &app.drop_subtitle);
    bool_opt(&mut v, "--sub-only", app.sub_only, false);
    if !matches!(app.sub_format, crate::app::SubFormat::Srt) {
        opt(&mut v, "--sub-format", "VTT");
    }

    // 解密
    opt(&mut v, "--key", &app.key);
    opt(&mut v, "--key-text-file", &app.key_text_file);
    if !matches!(app.custom_hls_method, crate::app::CustomHlsMethod::Default) {
        let m = match app.custom_hls_method {
            crate::app::CustomHlsMethod::None => "NONE",
            crate::app::CustomHlsMethod::Aes128 => "AES_128",
            crate::app::CustomHlsMethod::Aes128Ecb => "AES_128_ECB",
            crate::app::CustomHlsMethod::Cenc => "CENC",
            crate::app::CustomHlsMethod::ChaCha20 => "CHACHA20",
            crate::app::CustomHlsMethod::SampleAes => "SAMPLE_AES",
            crate::app::CustomHlsMethod::SampleAesCtr => "SAMPLE_AES_CTR",
            crate::app::CustomHlsMethod::Unknown => "UNKNOWN",
            _ => "",
        };
        opt(&mut v, "--custom-hls-method", m);
    }
    opt(&mut v, "--custom-hls-key", &app.custom_hls_key);
    opt(&mut v, "--custom-hls-iv", &app.custom_hls_iv);
    bool_opt(&mut v, "--mp4-real-time-decryption", app.mp4_realtime_decryption, false);

    // 合并
    bool_opt(&mut v, "--binary-merge", app.binary_merge, false);
    bool_opt(&mut v, "--skip-merge", app.skip_merge, false);
    bool_opt(&mut v, "--del-after-done", app.del_after_done, true);
    bool_opt(&mut v, "--no-date-info", app.no_date_info, false);
    opt(&mut v, "-M", &app.mux_after_done);
    opt(&mut v, "--mux-import", &app.mux_import);

    // 直播
    bool_opt(&mut v, "--live-perform-as-vod", app.live_perform_as_vod, false);
    bool_opt(&mut v, "--live-real-time-merge", app.live_realtime_merge, false);
    bool_opt(&mut v, "--live-keep-segments", app.live_keep_segments, true);
    bool_opt(&mut v, "--live-pipe-mux", app.live_pipe_mux, false);
    bool_opt(&mut v, "--live-fix-vtt-by-audio", app.live_fix_vtt_by_audio, false);
    opt(&mut v, "--live-record-limit", &app.live_record_limit);
    opt(&mut v, "--live-wait-time", &app.live_wait_time);
    opt(&mut v, "--live-take-count", &app.live_take_count);

    // 高级
    bool_opt(&mut v, "--append-url-params", app.append_url_params, false);
    bool_opt(&mut v, "--check-segments-count", app.check_segments_count, true);
    bool_opt(&mut v, "--write-meta-json", app.write_meta_json, true);
    bool_opt(&mut v, "--no-log", app.no_log, false);
    if !matches!(app.log_level, crate::app::LogLevel::Info) {
        let l = match app.log_level {
            crate::app::LogLevel::Debug => "DEBUG",
            crate::app::LogLevel::Error => "ERROR",
            crate::app::LogLevel::Off => "OFF",
            crate::app::LogLevel::Warn => "WARN",
            _ => "INFO",
        };
        opt(&mut v, "--log-level", l);
    }
    opt(&mut v, "--urlprocessor-args", &app.urlprocessor_args);
    opt(&mut v, "--custom-range", &app.custom_range);
    opt(&mut v, "--task-start-at", &app.task_start_at);
    opt(&mut v, "--ad-keyword", &app.ad_keyword);
    bool_opt(&mut v, "--allow-hls-multi-ext-map", app.allow_hls_multi_ext_map, false);
    bool_opt(&mut v, "--disable-update-check", app.disable_update_check, false);
    bool_opt(&mut v, "--force-ansi-console", app.force_ansi_console, false);
    bool_opt(&mut v, "--use-ffmpeg-concat-demuxer", app.use_ffmpeg_concat_demuxer, false);

    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, ProxyMode, SubFormat, Tab};
    use iced::widget::text_editor;

    fn sample_app() -> App {
        // 仅设置少量字段，验证参数拼装
        let mut a = App {
            exe_path: String::new(),
            input: String::new(),
            save_dir: String::new(),
            save_name: String::new(),
            save_pattern: String::new(),
            tmp_dir: String::new(),
            log_file_path: String::new(),
            base_url: String::new(),
            thread_count: String::new(),
            retry_count: String::new(),
            http_timeout: String::new(),
            headers: text_editor::Content::new(),
            proxy_mode: ProxyMode::System,
            proxy_address: String::new(),
            max_speed: String::new(),
            concurrent_download: false,
            ui_language: crate::app::UiLanguage::Default,
            ffmpeg_path: String::new(),
            decryption_engine: crate::app::DecryptionEngine::Default,
            decryption_binary: String::new(),
            auto_select: false,
            select_video: String::new(),
            select_audio: String::new(),
            select_subtitle: String::new(),
            drop_video: String::new(),
            drop_audio: String::new(),
            drop_subtitle: String::new(),
            sub_only: false,
            sub_format: SubFormat::Srt,
            key: String::new(),
            key_text_file: String::new(),
            custom_hls_method: crate::app::CustomHlsMethod::Default,
            custom_hls_key: String::new(),
            custom_hls_iv: String::new(),
            mp4_realtime_decryption: false,
            binary_merge: false,
            skip_merge: false,
            del_after_done: true,
            no_date_info: false,
            mux_after_done: String::new(),
            mux_import: String::new(),
            live_perform_as_vod: false,
            live_realtime_merge: false,
            live_keep_segments: true,
            live_pipe_mux: false,
            live_fix_vtt_by_audio: false,
            live_record_limit: String::new(),
            live_wait_time: String::new(),
            live_take_count: String::new(),
            append_url_params: false,
            check_segments_count: true,
            write_meta_json: true,
            no_log: false,
            log_level: crate::app::LogLevel::Info,
            urlprocessor_args: String::new(),
            custom_range: String::new(),
            task_start_at: String::new(),
            ad_keyword: String::new(),
            allow_hls_multi_ext_map: false,
            disable_update_check: false,
            force_ansi_console: false,
            use_ffmpeg_concat_demuxer: false,
            external_console: false,
            tab: Tab::Basic,
            log: String::new(),
            running: false,
            run_gen: 0,
            exe_error: String::new(),
        };
        a.save_name = "test".into();
        a.proxy_mode = ProxyMode::Custom;
        a.proxy_address = "http://127.0.0.1:8888".into();
        a.binary_merge = true;
        a.del_after_done = false;
        a
    }

    #[test]
    fn builds_expected_args() {
        let a = sample_app();
        let args = build_args(&a);
        assert!(args.contains(&"--save-name".to_string()));
        assert!(args.contains(&"test".to_string()));
        // Custom 模式：应出现自定义代理，且不应出现系统代理
        assert!(!args.contains(&"--use-system-proxy".to_string()));
        assert!(args.contains(&"--custom-proxy".to_string()));
        assert!(args.contains(&"http://127.0.0.1:8888".to_string()));
        assert!(args.contains(&"--binary-merge".to_string()));
        // del-after-done 默认 true，这里 false → 应出现 false
        let idx = args.iter().position(|x| x == "--del-after-done").unwrap();
        assert_eq!(args[idx + 1], "false");
    }

    #[test]
    fn empty_app_has_no_args() {
        // default-true 的开关在默认值下不应输出
        let a = sample_app();
        let mut a2 = a;
        a2.save_name = String::new();
        a2.proxy_mode = ProxyMode::System;
        a2.proxy_address = String::new();
        a2.binary_merge = false;
        a2.del_after_done = true;
        let args = build_args(&a2);
        assert!(!args.contains(&"--binary-merge".to_string()));
        assert!(!args.contains(&"--del-after-done".to_string()));
    }
}

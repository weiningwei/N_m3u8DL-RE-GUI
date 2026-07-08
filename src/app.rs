use crate::args::build_args;
use crate::config::Settings;
use crate::detect::{default_log_path, default_save_dir, default_temp_path, locate_exe, locate_ffmpeg};
use crate::runner;
use crate::ui;
use iced::widget::{operation, text_editor, Id};
use iced::{Element, Task};
use iced::keyboard;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Tab {
    #[default]
    Basic,
    Streams,
    Decrypt,
    Live,
    Advanced,
    Log,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum ProxyMode {
    #[default]
    System,
    None,
    Custom,
}

impl std::fmt::Display for ProxyMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ProxyMode::System => "使用系统代理",
            ProxyMode::None => "不使用代理",
            ProxyMode::Custom => "自定义代理",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum UiLanguage {
    #[default]
    Default,
    EnUs,
    ZhCn,
    ZhTw,
}

impl std::fmt::Display for UiLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            UiLanguage::Default => "默认",
            UiLanguage::EnUs => "en-US",
            UiLanguage::ZhCn => "zh-CN",
            UiLanguage::ZhTw => "zh-TW",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum DecryptionEngine {
    #[default]
    Default,
    Ffmpeg,
    Mp4decrypt,
    ShakaPackager,
}

impl std::fmt::Display for DecryptionEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DecryptionEngine::Default => "默认 (mp4decrypt)",
            DecryptionEngine::Ffmpeg => "FFMPEG",
            DecryptionEngine::Mp4decrypt => "MP4DECRYPT",
            DecryptionEngine::ShakaPackager => "SHAKA_PACKAGER",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum SubFormat {
    #[default]
    Srt,
    Vtt,
}

impl std::fmt::Display for SubFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SubFormat::Srt => "SRT",
            SubFormat::Vtt => "VTT",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum CustomHlsMethod {
    #[default]
    Default,
    None,
    Aes128,
    Aes128Ecb,
    Cenc,
    ChaCha20,
    SampleAes,
    SampleAesCtr,
    Unknown,
}

impl std::fmt::Display for CustomHlsMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CustomHlsMethod::Default => "默认",
            CustomHlsMethod::None => "NONE",
            CustomHlsMethod::Aes128 => "AES_128",
            CustomHlsMethod::Aes128Ecb => "AES_128_ECB",
            CustomHlsMethod::Cenc => "CENC",
            CustomHlsMethod::ChaCha20 => "CHACHA20",
            CustomHlsMethod::SampleAes => "SAMPLE_AES",
            CustomHlsMethod::SampleAesCtr => "SAMPLE_AES_CTR",
            CustomHlsMethod::Unknown => "UNKNOWN",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum LogLevel {
    Debug,
    Error,
    #[default]
    Info,
    Off,
    Warn,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Error => "ERROR",
            LogLevel::Info => "INFO",
            LogLevel::Off => "OFF",
            LogLevel::Warn => "WARN",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum ThemeMode {
    /// 跟随系统（Windows 读取“应用使用浅色/深色”设置）
    #[default]
    System,
    Light,
    Dark,
}

impl std::fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ThemeMode::System => "跟随系统",
            ThemeMode::Light => "浅色",
            ThemeMode::Dark => "深色",
        })
    }
}

#[derive(Debug, Clone)]
pub enum LogEvent {
    Line(String),
    Done(Result<u32, String>),
}

#[derive(Debug, Clone)]
pub enum Message {
    // 输入/可执行文件
    ExePathChanged(String),
    BrowseExe,
    ExePathChosen(Option<String>),
    InputChanged(String),
    SaveDirChanged(String),
    BrowseSaveDir,
    SaveDirChosen(Option<String>),
    SaveNameChanged(String),
    SavePatternChanged(String),
    TmpDirChanged(String),
    LogFilePathChanged(String),
    BrowseTmpDir,
    TmpDirChosen(Option<String>),
    BrowseLogFile,
    LogFileChosen(Option<String>),
    BaseUrlChanged(String),
    // 线程/重试/超时
    ThreadCountChanged(String),
    RetryCountChanged(String),
    HttpTimeoutChanged(String),
    // 请求头
    HeadersEdited(text_editor::Action),
    // 代理
    ProxyModeSelected(ProxyMode),
    ProxyAddressChanged(String),
    // 限速/并发/语言
    MaxSpeedChanged(String),
    ConcurrentToggled(bool),
    UiLanguageSelected(UiLanguage),
    // 二进制路径
    FfmpegPathChanged(String),
    DecryptionEngineSelected(DecryptionEngine),
    DecryptionBinaryChanged(String),
    // 流选择
    AutoSelectToggled(bool),
    SelectVideoChanged(String),
    SelectAudioChanged(String),
    SelectSubtitleChanged(String),
    DropVideoChanged(String),
    DropAudioChanged(String),
    DropSubtitleChanged(String),
    SubOnlyToggled(bool),
    SubFormatSelected(SubFormat),
    // 解密
    KeyChanged(String),
    KeyTextFileChanged(String),
    CustomHlsMethodSelected(CustomHlsMethod),
    CustomHlsKeyChanged(String),
    CustomHlsIvChanged(String),
    Mp4RealtimeDecryptionToggled(bool),
    // 合并
    BinaryMergeToggled(bool),
    SkipMergeToggled(bool),
    DelAfterDoneToggled(bool),
    NoDateInfoToggled(bool),
    MuxAfterDoneChanged(String),
    MuxImportChanged(String),
    // 直播
    LivePerformAsVodToggled(bool),
    LiveRealtimeMergeToggled(bool),
    LiveKeepSegmentsToggled(bool),
    LivePipeMuxToggled(bool),
    LiveFixVttByAudioToggled(bool),
    LiveRecordLimitChanged(String),
    LiveWaitTimeChanged(String),
    LiveTakeCountChanged(String),
    // 高级
    AppendUrlParamsToggled(bool),
    CheckSegmentsCountToggled(bool),
    WriteMetaJsonToggled(bool),
    NoLogToggled(bool),
    LogLevelSelected(LogLevel),
    UrlprocessorArgsChanged(String),
    CustomRangeChanged(String),
    TaskStartAtChanged(String),
    AdKeywordChanged(String),
    AllowHlsMultiExtMapToggled(bool),
    DisableUpdateCheckToggled(bool),
    ForceAnsiConsoleToggled(bool),
    UseFfmpegConcatDemuxerToggled(bool),
    // 运行/界面
    ExternalConsoleToggled(bool),
    ThemeModeSelected(ThemeMode),
    TabSelected(Tab),
    Start,
    LogEvent(LogEvent),
    CopyPreview,
    /// 复制日志区文本（优先复制选中部分，无选中则复制全部）
    CopyLog,
    /// 日志区只读文本编辑器的用户操作（仅响游标/选中，忽略编辑以保持只读）
    LogEditorAction(text_editor::Action),
    OpenOutputFolder,
    ClearLog,
    /// 全局键盘事件（用于 ESC 退出等）
    KeyEvent(keyboard::Event),
    /// 复制命令后的“已复制”提示自动消失计时
    Tick,
}

pub struct App {
    // 输入/可执行文件
    pub exe_path: String,
    pub input: String,
    // 保存
    pub save_dir: String,
    pub save_name: String,
    pub save_pattern: String,
    pub tmp_dir: String,
    pub log_file_path: String,
    pub base_url: String,
    // 线程/重试/超时
    pub thread_count: String,
    pub retry_count: String,
    pub http_timeout: String,
    // 请求头
    pub headers: text_editor::Content,
    // 代理
    pub proxy_mode: ProxyMode,
    pub proxy_address: String,
    // 限速/并发/语言
    pub max_speed: String,
    pub concurrent_download: bool,
    pub ui_language: UiLanguage,
    // 二进制路径
    pub ffmpeg_path: String,
    pub decryption_engine: DecryptionEngine,
    pub decryption_binary: String,
    // 流选择
    pub auto_select: bool,
    pub select_video: String,
    pub select_audio: String,
    pub select_subtitle: String,
    pub drop_video: String,
    pub drop_audio: String,
    pub drop_subtitle: String,
    pub sub_only: bool,
    pub sub_format: SubFormat,
    // 解密
    pub key: String,
    pub key_text_file: String,
    pub custom_hls_method: CustomHlsMethod,
    pub custom_hls_key: String,
    pub custom_hls_iv: String,
    pub mp4_realtime_decryption: bool,
    // 合并
    pub binary_merge: bool,
    pub skip_merge: bool,
    pub del_after_done: bool,
    pub no_date_info: bool,
    pub mux_after_done: String,
    pub mux_import: String,
    // 直播
    pub live_perform_as_vod: bool,
    pub live_realtime_merge: bool,
    pub live_keep_segments: bool,
    pub live_pipe_mux: bool,
    pub live_fix_vtt_by_audio: bool,
    pub live_record_limit: String,
    pub live_wait_time: String,
    pub live_take_count: String,
    // 高级
    pub append_url_params: bool,
    pub check_segments_count: bool,
    pub write_meta_json: bool,
    pub no_log: bool,
    pub log_level: LogLevel,
    pub urlprocessor_args: String,
    pub custom_range: String,
    pub task_start_at: String,
    pub ad_keyword: String,
    pub allow_hls_multi_ext_map: bool,
    pub disable_update_check: bool,
    pub force_ansi_console: bool,
    pub use_ffmpeg_concat_demuxer: bool,
    // 运行/界面
    pub external_console: bool,
    pub theme_mode: ThemeMode,
    pub tab: Tab,
    pub log: String,
    /// 日志区可选择的只读文本编辑器内容（与 `log` 同步）
    pub log_content: text_editor::Content,
    pub running: bool,
    pub run_gen: u64,
    pub exe_error: String,
    pub input_error: String,
    /// 复制命令后显示“已复制”提示的起始时刻；None 表示当前不显示
    pub copied_at: Option<Instant>,
    /// 复制日志后显示“已复制”提示的起始时刻；None 表示当前不显示
    pub log_copied_at: Option<Instant>,
}

impl App {
    pub fn new() -> (Self, iced::Task<Message>) {
        let settings = Settings::load();
        let mut app = App {
            exe_path: settings.exe_path.clone(),
            input: String::new(),
            save_dir: if settings.save_dir.is_empty() {
                default_save_dir()
            } else {
                settings.save_dir.clone()
            },
            save_name: String::new(),
            save_pattern: String::new(),
            tmp_dir: default_temp_path(),
            log_file_path: default_log_path(),
            base_url: String::new(),
            thread_count: String::new(),
            retry_count: String::new(),
            http_timeout: String::new(),
            headers: text_editor::Content::with_text(&settings.headers),
            proxy_mode: ProxyMode::System,
            proxy_address: settings.proxy_address.clone(),
            max_speed: String::new(),
            concurrent_download: false,
            ui_language: UiLanguage::Default,
            ffmpeg_path: settings.ffmpeg_path.clone(),
            decryption_engine: DecryptionEngine::Default,
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
            custom_hls_method: CustomHlsMethod::Default,
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
            log_level: LogLevel::Info,
            urlprocessor_args: String::new(),
            custom_range: String::new(),
            task_start_at: String::new(),
            ad_keyword: String::new(),
            allow_hls_multi_ext_map: false,
            disable_update_check: false,
            force_ansi_console: false,
            use_ffmpeg_concat_demuxer: false,
            external_console: settings.external_console,
            theme_mode: settings.theme_mode,
            tab: Tab::Basic,
            log: String::new(),
            log_content: text_editor::Content::with_text(""),
            running: false,
            run_gen: 0,
            exe_error: String::new(),
            input_error: String::new(),
            copied_at: None,
            log_copied_at: None,
        };
        // 若用户未指定 exe，尝试自动探测
        if app.exe_path.is_empty() {
            if let Some(found) = locate_exe("") {
                app.exe_path = found;
            }
        }
        // 若未配置 ffmpeg，自动探测并直接显示在文本框
        if app.ffmpeg_path.trim().is_empty() {
            if let Some(f) = locate_ffmpeg("") {
                app.ffmpeg_path = f;
            }
        }
        // 启动后将焦点置于 URL 输入框，方便直接粘贴/输入地址后回车下载
        (app, operation::focus(Id::new("url-input")))
    }

    /// 实时命令预览（不含 exe 本身）
    pub fn command_preview(&self) -> String {
        let exe = locate_exe(&self.exe_path).unwrap_or_else(|| self.exe_path.clone());
        let args = build_args(self);
        let mut s = String::new();
        s.push_str(&quote(&exe));
        s.push(' ');
        if !self.input.is_empty() {
            s.push_str(&quote(&self.input));
            s.push(' ');
        }
        s.push_str(&args.join(" "));
        s.trim().to_string()
    }

    /// 根据当前主题模式返回 iced 主题
    pub fn theme(&self) -> iced::Theme {
        match self.theme_mode {
            ThemeMode::Light => iced::Theme::Light,
            ThemeMode::Dark => iced::Theme::Dark,
            ThemeMode::System => {
                if system_is_dark() {
                    iced::Theme::Dark
                } else {
                    iced::Theme::Light
                }
            }
        }
    }

    pub fn save_settings(&self) {
        let s = Settings {
            exe_path: self.exe_path.clone(),
            save_dir: self.save_dir.clone(),
            proxy_address: self.proxy_address.clone(),
            headers: self.headers.text().to_string(),
            external_console: self.external_console,
            theme_mode: self.theme_mode,
            ffmpeg_path: self.ffmpeg_path.clone(),
        };
        s.save();
    }
}

/// 给含空格的参数加引号
pub fn quote(s: &str) -> String {
    if s.contains(' ') || s.contains('"') {
        format!("\"{}\"", s.replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

/// 检查各路径字段的格式合法性，返回首个非法路径的错误提示（无则 None）。
/// 空串视为合法（可选路径），只校验用户实际填入的内容。
fn invalid_path_message(app: &App) -> Option<String> {
    use crate::detect::validate_path_format;
    let fields = [
        ("可执行文件", &app.exe_path),
        ("保存目录", &app.save_dir),
        ("临时目录", &app.tmp_dir),
        ("日志文件", &app.log_file_path),
        ("ffmpeg", &app.ffmpeg_path),
        ("解密工具", &app.decryption_binary),
        ("密钥文件", &app.key_text_file),
    ];
    for (name, p) in fields {
        if let Err(e) = validate_path_format(p) {
            return Some(format!("{name}路径格式无效：{e}"));
        }
    }
    None
}

/// 判断系统是否处于深色模式（仅 Windows 有效，其他平台返回 false 即浅色）。
#[cfg(target_os = "windows")]
fn system_is_dark() -> bool {
    use std::process::Command;
    let out = Command::new("reg")
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
            "/v",
            "AppsUseLightTheme",
        ])
        .output();
    if let Ok(o) = out {
        let s = String::from_utf8_lossy(&o.stdout);
        for line in s.lines() {
            if line.contains("AppsUseLightTheme") {
                // 典型输出如：AppsUseLightTheme    REG_DWORD    0x0
                if let Some(v) = line.split_whitespace().last() {
                    return v.eq_ignore_ascii_case("0x0");
                }
            }
        }
    }
    false
}

#[cfg(not(target_os = "windows"))]
fn system_is_dark() -> bool {
    false
}

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ExePathChanged(s) => self.exe_path = s,
            Message::BrowseExe => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .add_filter("可执行文件", &["exe"])
                            .pick_file()
                            .await
                            .map(|h| h.path().to_string_lossy().into_owned())
                    },
                    Message::ExePathChosen,
                )
            }
            Message::ExePathChosen(p) => {
                if let Some(p) = p {
                    self.exe_path = p;
                }
            }
            Message::InputChanged(s) => self.input = s,
            Message::SaveDirChanged(s) => self.save_dir = s,
            Message::BrowseSaveDir => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .pick_folder()
                            .await
                            .map(|h| h.path().to_string_lossy().into_owned())
                    },
                    Message::SaveDirChosen,
                )
            }
            Message::SaveDirChosen(p) => {
                if let Some(p) = p {
                    self.save_dir = p;
                }
            }
            Message::SaveNameChanged(s) => self.save_name = s,
            Message::SavePatternChanged(s) => self.save_pattern = s,
            Message::TmpDirChanged(s) => self.tmp_dir = s,
            Message::LogFilePathChanged(s) => self.log_file_path = s,
            Message::BrowseTmpDir => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .pick_folder()
                            .await
                            .map(|h| h.path().to_string_lossy().into_owned())
                    },
                    Message::TmpDirChosen,
                )
            }
            Message::TmpDirChosen(p) => {
                if let Some(p) = p {
                    self.tmp_dir = p;
                }
            }
            Message::BrowseLogFile => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .pick_folder()
                            .await
                            .map(|h| h.path().to_string_lossy().into_owned())
                    },
                    Message::LogFileChosen,
                )
            }
            Message::LogFileChosen(p) => {
                if let Some(p) = p {
                    let path = PathBuf::from(&p);
                    // 若选择的是目录，则默认拼接一个日志文件名；
                    // 若选择的是文件，则直接使用该文件。
                    let resolved = if path.is_dir() {
                        let name = PathBuf::from(default_log_path())
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_else(|| "n_m3u8dl-re-gui.log".to_string());
                        path.join(name)
                    } else {
                        path
                    };
                    self.log_file_path = resolved.to_string_lossy().into_owned();
                }
            }
            Message::BaseUrlChanged(s) => self.base_url = s,
            Message::ThreadCountChanged(s) => self.thread_count = s,
            Message::RetryCountChanged(s) => self.retry_count = s,
            Message::HttpTimeoutChanged(s) => self.http_timeout = s,
            Message::HeadersEdited(a) => self.headers.perform(a),
            Message::ProxyModeSelected(m) => self.proxy_mode = m,
            Message::ProxyAddressChanged(s) => self.proxy_address = s,
            Message::MaxSpeedChanged(s) => self.max_speed = s,
            Message::ConcurrentToggled(b) => self.concurrent_download = b,
            Message::UiLanguageSelected(m) => self.ui_language = m,
            Message::FfmpegPathChanged(s) => self.ffmpeg_path = s,
            Message::DecryptionEngineSelected(m) => self.decryption_engine = m,
            Message::DecryptionBinaryChanged(s) => self.decryption_binary = s,
            Message::AutoSelectToggled(b) => self.auto_select = b,
            Message::SelectVideoChanged(s) => self.select_video = s,
            Message::SelectAudioChanged(s) => self.select_audio = s,
            Message::SelectSubtitleChanged(s) => self.select_subtitle = s,
            Message::DropVideoChanged(s) => self.drop_video = s,
            Message::DropAudioChanged(s) => self.drop_audio = s,
            Message::DropSubtitleChanged(s) => self.drop_subtitle = s,
            Message::SubOnlyToggled(b) => self.sub_only = b,
            Message::SubFormatSelected(m) => self.sub_format = m,
            Message::KeyChanged(s) => self.key = s,
            Message::KeyTextFileChanged(s) => self.key_text_file = s,
            Message::CustomHlsMethodSelected(m) => self.custom_hls_method = m,
            Message::CustomHlsKeyChanged(s) => self.custom_hls_key = s,
            Message::CustomHlsIvChanged(s) => self.custom_hls_iv = s,
            Message::Mp4RealtimeDecryptionToggled(b) => self.mp4_realtime_decryption = b,
            Message::BinaryMergeToggled(b) => self.binary_merge = b,
            Message::SkipMergeToggled(b) => self.skip_merge = b,
            Message::DelAfterDoneToggled(b) => self.del_after_done = b,
            Message::NoDateInfoToggled(b) => self.no_date_info = b,
            Message::MuxAfterDoneChanged(s) => self.mux_after_done = s,
            Message::MuxImportChanged(s) => self.mux_import = s,
            Message::LivePerformAsVodToggled(b) => self.live_perform_as_vod = b,
            Message::LiveRealtimeMergeToggled(b) => self.live_realtime_merge = b,
            Message::LiveKeepSegmentsToggled(b) => self.live_keep_segments = b,
            Message::LivePipeMuxToggled(b) => self.live_pipe_mux = b,
            Message::LiveFixVttByAudioToggled(b) => self.live_fix_vtt_by_audio = b,
            Message::LiveRecordLimitChanged(s) => self.live_record_limit = s,
            Message::LiveWaitTimeChanged(s) => self.live_wait_time = s,
            Message::LiveTakeCountChanged(s) => self.live_take_count = s,
            Message::AppendUrlParamsToggled(b) => self.append_url_params = b,
            Message::CheckSegmentsCountToggled(b) => self.check_segments_count = b,
            Message::WriteMetaJsonToggled(b) => self.write_meta_json = b,
            Message::NoLogToggled(b) => self.no_log = b,
            Message::LogLevelSelected(m) => self.log_level = m,
            Message::UrlprocessorArgsChanged(s) => self.urlprocessor_args = s,
            Message::CustomRangeChanged(s) => self.custom_range = s,
            Message::TaskStartAtChanged(s) => self.task_start_at = s,
            Message::AdKeywordChanged(s) => self.ad_keyword = s,
            Message::AllowHlsMultiExtMapToggled(b) => self.allow_hls_multi_ext_map = b,
            Message::DisableUpdateCheckToggled(b) => self.disable_update_check = b,
            Message::ForceAnsiConsoleToggled(b) => self.force_ansi_console = b,
            Message::UseFfmpegConcatDemuxerToggled(b) => self.use_ffmpeg_concat_demuxer = b,
            Message::ExternalConsoleToggled(b) => {
                self.external_console = b;
                self.save_settings();
            }
            Message::ThemeModeSelected(m) => {
                self.theme_mode = m;
                self.save_settings();
            }
            Message::TabSelected(t) => self.tab = t,
            Message::Start => {
                if self.running {
                    return Task::none();
                }
                self.exe_error.clear();
                self.input_error.clear();
                // 路径格式预检：非空但含非法字符的路径直接拦下，避免把坏路径传给 RE。
                if let Some(msg) = invalid_path_message(self) {
                    self.exe_error = msg;
                    return Task::none();
                }
                match locate_exe(&self.exe_path) {
                    None => {
                        self.exe_error =
                            "未找到 N_m3u8DL-RE 可执行文件，请在“基本”页指定其路径。".to_string();
                    }
                    Some(exe) => {
                        if self.input.trim().is_empty() {
                            self.input_error = "请输入下载地址或文件路径。".to_string();
                        } else {
                            // ffmpeg 预检：RE 合并/混流必需。自动定位后用之；
                            // 若仍找不到，给出清晰指引而非让 RE 崩溃退出。
                            if self.ffmpeg_path.trim().is_empty() {
                                if let Some(f) = locate_ffmpeg("") {
                                    self.ffmpeg_path = f;
                                }
                            }
                            if self.ffmpeg_path.trim().is_empty() {
                                self.exe_error = "未找到 ffmpeg（RE 合并/混流必需）。请安装 ffmpeg 并在“高级”页指定其路径，或将其放到 PATH / N_m3u8DL-RE 同目录。下载地址：https://ffmpeg.org/download.html".to_string();
                                return Task::none();
                            }
                            self.run_gen += 1;
                            let run_id = self.run_gen;
                            let input = self.input.trim().to_string();
                            let mut args = build_args(self);
                            let capture = !self.external_console;
                            if capture {
                                args.push("--no-ansi-color".to_string());
                            }
                            runner::start_run(run_id, exe, input, args, capture);
                            self.running = true;
                            self.tab = Tab::Log;
                            self.log.clear();
                            self.log.push_str("开始运行...\n");
                            self.log_content = text_editor::Content::with_text(&self.log);
                        }
                    }
                }
            }
            Message::LogEvent(ev) => {
                let append = match ev {
                    LogEvent::Line(s) => format!("{s}\n"),
                    LogEvent::Done(Ok(code)) => {
                        self.running = false;
                        format!("进程结束，退出码 {code}\n")
                    }
                    LogEvent::Done(Err(e)) => {
                        self.running = false;
                        format!("进程异常：{e}\n")
                    }
                };
                self.log.push_str(&append);
                self.log_content = text_editor::Content::with_text(&self.log);
            }
            Message::CopyPreview => {
                let text = self.command_preview();
                std::thread::spawn(move || {
                    if let Ok(mut cb) = arboard::Clipboard::new() {
                        let _ = cb.set_text(text);
                    }
                });
                self.copied_at = Some(Instant::now());
            }
            Message::CopyLog => {
                // 优先复制选中文本；未选中任何内容时复制全部日志
                let to_copy = match self.log_content.selection() {
                    Some(s) if !s.trim().is_empty() => s,
                    _ => self.log.clone(),
                };
                std::thread::spawn(move || {
                    if let Ok(mut cb) = arboard::Clipboard::new() {
                        let _ = cb.set_text(to_copy);
                    }
                });
                self.log_copied_at = Some(Instant::now());
            }
            Message::LogEditorAction(action) => {
                // 日志区为只读：忽略文本编辑动作，仅保留游标/选中状态
                if !matches!(action, text_editor::Action::Edit(_)) {
                    self.log_content.perform(action);
                }
            }
            Message::Tick => {
                if let Some(at) = self.copied_at {
                    if at.elapsed() >= Duration::from_millis(500) {
                        self.copied_at = None;
                    }
                }
                if let Some(at) = self.log_copied_at {
                    if at.elapsed() >= Duration::from_millis(500) {
                        self.log_copied_at = None;
                    }
                }
            }
            Message::OpenOutputFolder => {
                if !self.save_dir.is_empty() {
                    let _ = std::process::Command::new("explorer")
                        .arg(&self.save_dir)
                        .spawn();
                }
            }
            Message::ClearLog => {
                self.log.clear();
                self.log_content = text_editor::Content::with_text("");
            }
            Message::KeyEvent(event) => {
                if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                    // ESC 退出
                    if let keyboard::Key::Named(keyboard::key::Named::Escape) = key {
                        return iced::exit();
                    }
                    // 日志区内 Ctrl+C：text_editor 原生已复制选中文本，这里补一个“已复制”提示
                    // 注意：Ctrl 按下时不同平台可能把按键报为 "c" 或控制字符 ETX(\u{3})
                    if modifiers.contains(keyboard::Modifiers::CTRL)
                        && !modifiers.contains(keyboard::Modifiers::ALT)
                        && matches!(key, keyboard::Key::Character(c) if {
                            let s = c.as_ref();
                            s.eq_ignore_ascii_case("c") || s == "\u{3}"
                        })
                    {
                        if let Some(sel) = self.log_content.selection() {
                            if !sel.trim().is_empty() {
                                self.log_copied_at = Some(Instant::now());
                            }
                        }
                    }
                }
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        ui::view(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::tests::sample_app;

    #[test]
    fn theme_maps_light_and_dark_explicitly() {
        let mut a = sample_app();
        a.theme_mode = ThemeMode::Light;
        assert!(matches!(a.theme(), iced::Theme::Light));
        a.theme_mode = ThemeMode::Dark;
        assert!(matches!(a.theme(), iced::Theme::Dark));
    }

    #[test]
    fn theme_system_resolves_to_a_valid_theme() {
        let mut a = sample_app();
        a.theme_mode = ThemeMode::System;
        match a.theme() {
            iced::Theme::Light | iced::Theme::Dark => {}
            other => panic!("unexpected theme from system mode: {:?}", other),
        }
    }
}

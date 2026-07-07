use crate::app::{
    App, CustomHlsMethod, DecryptionEngine, LogLevel, Message, ProxyMode, SubFormat, Tab,
    UiLanguage,
};
use iced::widget::{
    button, checkbox, column, pick_list, row, rule, scrollable, space, text, text_editor,
    text_input,
};
use iced::{Element, Length};

const PROXY_MODES: [ProxyMode; 3] = [ProxyMode::System, ProxyMode::None, ProxyMode::Custom];
const UI_LANGS: [UiLanguage; 4] = [
    UiLanguage::Default,
    UiLanguage::EnUs,
    UiLanguage::ZhCn,
    UiLanguage::ZhTw,
];
const DEC_ENGINES: [DecryptionEngine; 4] = [
    DecryptionEngine::Default,
    DecryptionEngine::Ffmpeg,
    DecryptionEngine::Mp4decrypt,
    DecryptionEngine::ShakaPackager,
];
const SUB_FORMATS: [SubFormat; 2] = [SubFormat::Srt, SubFormat::Vtt];
const HLS_METHODS: [CustomHlsMethod; 9] = [
    CustomHlsMethod::Default,
    CustomHlsMethod::None,
    CustomHlsMethod::Aes128,
    CustomHlsMethod::Aes128Ecb,
    CustomHlsMethod::Cenc,
    CustomHlsMethod::ChaCha20,
    CustomHlsMethod::SampleAes,
    CustomHlsMethod::SampleAesCtr,
    CustomHlsMethod::Unknown,
];
const LOG_LEVELS: [LogLevel; 5] = [
    LogLevel::Debug,
    LogLevel::Error,
    LogLevel::Info,
    LogLevel::Off,
    LogLevel::Warn,
];

fn lab<'a>(label: &'a str, control: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    let control: Element<'a, Message> = control.into();
    row![text(label).width(120), control]
        .spacing(6)
        .width(Length::Fill)
        .align_y(iced::Alignment::Center)
        // 输入框右边框绘制在控件边界上，会被父级裁剪掉约半像素，
        // 在非最大化（分数 DPI 缩放）时尤其明显。右侧留 2px 内缩，
        // 使右边框落在裁剪区域内部，完整显示。
        .padding(iced::Padding::new(0.0).right(2.0))
        .into()
}

fn tab_bar(app: &App) -> Element<'_, Message> {
    let tabs = [
        (Tab::Basic, "基本"),
        (Tab::Streams, "流选择"),
        (Tab::Decrypt, "解密"),
        (Tab::Live, "直播"),
        (Tab::Advanced, "高级"),
        (Tab::Log, "日志"),
    ];
    let mut r = row![].spacing(4);
    for (tab, label) in tabs {
        let label = if app.tab == tab {
            format!("▶ {label}")
        } else {
            label.to_string()
        };
        r = r.push(button(text(label).size(14)).on_press(Message::TabSelected(tab)));
    }
    r.into()
}

fn bottom_bar(app: &App) -> Element<'_, Message> {
    let start_btn = if app.running {
        button(text("运行中…").size(16))
    } else {
        button(text("开始下载").size(16)).on_press(Message::Start)
    };
    column![
        text("命令预览：").size(12),
        scrollable(text(app.command_preview()).size(12))
            .width(Length::Fill)
            .height(50.0),
        row![
            button(text("复制命令").size(13)).on_press(Message::CopyPreview),
            checkbox(app.external_console)
                .label("使用外部控制台窗口（不捕获日志，类似 SimpleG）")
                .on_toggle(Message::ExternalConsoleToggled),
            space::horizontal(),
            start_btn,
        ]
        .spacing(10)
        .width(Length::Fill),
    ]
    .spacing(4)
    .width(Length::Fill)
    .into()
}

pub fn view(app: &App) -> Element<'_, Message> {
    let content = match app.tab {
        Tab::Basic => basic_tab(app),
        Tab::Streams => streams_tab(app),
        Tab::Decrypt => decrypt_tab(app),
        Tab::Live => live_tab(app),
        Tab::Advanced => advanced_tab(app),
        Tab::Log => log_tab(app),
    };
    column![
        tab_bar(app),
        rule::horizontal(1.0),
        scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(4.0),
        bottom_bar(app),
    ]
    .spacing(6)
    .padding(10)
    .width(Length::Fill)
    .into()
}

fn basic_tab(app: &App) -> Element<'_, Message> {
    let mut col = column![].spacing(8).width(Length::Fill);
    col = col.push(lab(
        "可执行文件",
        row![
            text_input("N_m3u8DL-RE.exe 路径", &app.exe_path)
                .on_input(Message::ExePathChanged)
                .width(Length::Fill),
            button(text("浏览")).on_press(Message::BrowseExe),
        ]
        .spacing(6)
        .width(Length::Fill),
    ));
    if !app.exe_error.is_empty() {
        col = col.push(text(&app.exe_error).color([0.9, 0.3, 0.3]).size(13));
    }
    col = col.push(lab(
        "保存目录",
        row![
            text_input("默认 exe 同目录 downloads", &app.save_dir)
                .on_input(Message::SaveDirChanged)
                .width(Length::Fill),
            button(text("浏览")).on_press(Message::BrowseSaveDir),
        ]
        .spacing(6)
        .width(Length::Fill),
    ));
    col = col.push(lab(
        "临时目录",
        row![
            text_input("默认位于 exe 同目录 temp", &app.tmp_dir)
                .on_input(Message::TmpDirChanged)
                .width(Length::Fill),
            button(text("浏览")).on_press(Message::BrowseTmpDir),
        ]
        .spacing(6)
        .width(Length::Fill),
    ));
    col = col.push(lab(
        "日志文件路径",
        row![
            text_input("如 C:\\Logs\\log.txt", &app.log_file_path)
                .on_input(Message::LogFilePathChanged)
                .width(Length::Fill),
            button(text("浏览")).on_press(Message::BrowseLogFile),
        ]
        .spacing(6)
        .width(Length::Fill),
    ));
    col = col.push(lab(
        "下载地址/文件",
        text_input("URL 或本地 m3u8/mpd 路径", &app.input).on_input(Message::InputChanged),
    ));
    col = col.push(lab(
        "保存文件名",
        text_input("不含后缀，留空自动", &app.save_name).on_input(Message::SaveNameChanged),
    ));
    col = col.push(lab(
        "命名模板",
        text_input("如 <SaveName>_<Resolution>", &app.save_pattern)
            .on_input(Message::SavePatternChanged),
    ));
    col = col.push(lab(
        "BaseURL",
        text_input("设置 BaseURL", &app.base_url).on_input(Message::BaseUrlChanged),
    ));
    col = col.push(
        row![
            lab(
                "线程数",
                text_input("", &app.thread_count).on_input(Message::ThreadCountChanged)
            ),
            lab(
                "重试次数",
                text_input("", &app.retry_count).on_input(Message::RetryCountChanged)
            ),
            lab(
                "超时(秒)",
                text_input("", &app.http_timeout).on_input(Message::HttpTimeoutChanged)
            ),
        ]
        .spacing(10)
        .width(Length::Fill),
    );
    col = col.push(lab(
        "请求头",
        text_editor(&app.headers)
            .on_action(Message::HeadersEdited)
            .height(80.0),
    ));
    col = col.push(text("每行一条，如 Cookie: xxx 或 User-Agent: iOS").size(11));
    col = col.push(lab(
        "代理",
        row![
            pick_list(&PROXY_MODES[..], Some(&app.proxy_mode), Message::ProxyModeSelected)
                .width(160),
            text_input("自定义代理地址", &app.proxy_address).on_input(Message::ProxyAddressChanged),
        ]
        .spacing(6)
        .width(Length::Fill),
    ));
    col = col.push(lab(
        "限速",
        text_input("如 15M / 100K，留空不限", &app.max_speed).on_input(Message::MaxSpeedChanged),
    ));
    col = col.push(
        checkbox(app.concurrent_download)
            .label("并发下载已选择的音视频字幕")
            .on_toggle(Message::ConcurrentToggled),
    );
    col = col.push(lab(
        "界面语言",
        pick_list(&UI_LANGS[..], Some(&app.ui_language), Message::UiLanguageSelected).width(160),
    ));
    col = col.push(lab(
        "ffmpeg 路径",
        text_input("留空自动寻找", &app.ffmpeg_path).on_input(Message::FfmpegPathChanged),
    ));
    col = col.push(lab(
        "解密引擎",
        pick_list(
            &DEC_ENGINES[..],
            Some(&app.decryption_engine),
            Message::DecryptionEngineSelected
        )
        .width(200),
    ));
    col = col.push(lab(
        "解密工具路径",
        text_input("mp4decrypt/shaka 路径", &app.decryption_binary)
            .on_input(Message::DecryptionBinaryChanged),
    ));
    col.into()
}

fn streams_tab(app: &App) -> Element<'_, Message> {
    let mut col = column![].spacing(8).width(Length::Fill);
    col = col.push(
        checkbox(app.auto_select)
            .label("自动选择所有类型的最佳轨道")
            .on_toggle(Message::AutoSelectToggled),
    );
    col = col.push(lab(
        "选择视频 -sv",
        text_input("如 res=\"3840*\":codecs=hvc1:for=best", &app.select_video)
            .on_input(Message::SelectVideoChanged),
    ));
    col = col.push(lab(
        "选择音频 -sa",
        text_input("如 lang=en:for=best", &app.select_audio).on_input(Message::SelectAudioChanged),
    ));
    col = col.push(lab(
        "选择字幕 -ss",
        text_input("如 name=\"中文\":for=all", &app.select_subtitle)
            .on_input(Message::SelectSubtitleChanged),
    ));
    col = col.push(lab(
        "去除视频 -dv",
        text_input("正则", &app.drop_video).on_input(Message::DropVideoChanged),
    ));
    col = col.push(lab(
        "去除音频 -da",
        text_input("正则", &app.drop_audio).on_input(Message::DropAudioChanged),
    ));
    col = col.push(lab(
        "去除字幕 -ds",
        text_input("正则", &app.drop_subtitle).on_input(Message::DropSubtitleChanged),
    ));
    col = col.push(
        checkbox(app.sub_only)
            .label("只选取字幕轨道")
            .on_toggle(Message::SubOnlyToggled),
    );
    col = col.push(lab(
        "字幕格式",
        pick_list(&SUB_FORMATS[..], Some(&app.sub_format), Message::SubFormatSelected).width(120),
    ));
    col.into()
}

fn decrypt_tab(app: &App) -> Element<'_, Message> {
    let mut col = column![].spacing(8).width(Length::Fill);
    col = col.push(lab(
        "解密密钥 --key",
        text_input("KID:KEY 或 KEY", &app.key).on_input(Message::KeyChanged),
    ));
    col = col.push(lab(
        "密钥文件 --key-text-file",
        text_input("密钥文件路径", &app.key_text_file).on_input(Message::KeyTextFileChanged),
    ));
    col = col.push(lab(
        "HLS 加密方式",
        pick_list(
            &HLS_METHODS[..],
            Some(&app.custom_hls_method),
            Message::CustomHlsMethodSelected
        )
        .width(200),
    ));
    col = col.push(lab(
        "HLS 解密 KEY",
        text_input("文件/HEX/Base64", &app.custom_hls_key).on_input(Message::CustomHlsKeyChanged),
    ));
    col = col.push(lab(
        "HLS 解密 IV",
        text_input("文件/HEX/Base64", &app.custom_hls_iv).on_input(Message::CustomHlsIvChanged),
    ));
    col = col.push(
        checkbox(app.mp4_realtime_decryption)
            .label("实时解密 MP4 分片")
            .on_toggle(Message::Mp4RealtimeDecryptionToggled),
    );
    col.into()
}

fn live_tab(app: &App) -> Element<'_, Message> {
    let mut col = column![].spacing(8).width(Length::Fill);
    col = col.push(
        checkbox(app.live_perform_as_vod)
            .label("以点播方式下载直播流")
            .on_toggle(Message::LivePerformAsVodToggled),
    );
    col = col.push(
        checkbox(app.live_realtime_merge)
            .label("录制直播时实时合并")
            .on_toggle(Message::LiveRealtimeMergeToggled),
    );
    col = col.push(
        checkbox(app.live_keep_segments)
            .label("实时合并时保留分片")
            .on_toggle(Message::LiveKeepSegmentsToggled),
    );
    col = col.push(
        checkbox(app.live_pipe_mux)
            .label("通过管道+ffmpeg 实时混流到 TS")
            .on_toggle(Message::LivePipeMuxToggled),
    );
    col = col.push(
        checkbox(app.live_fix_vtt_by_audio)
            .label("通过音频起始时间修正 VTT 字幕")
            .on_toggle(Message::LiveFixVttByAudioToggled),
    );
    col = col.push(lab(
        "录制时长限制",
        text_input("HH:mm:ss", &app.live_record_limit).on_input(Message::LiveRecordLimitChanged),
    ));
    col = col.push(lab(
        "列表刷新间隔(秒)",
        text_input("手动设置", &app.live_wait_time).on_input(Message::LiveWaitTimeChanged),
    ));
    col = col.push(lab(
        "首次获取分片数",
        text_input("默认 16", &app.live_take_count).on_input(Message::LiveTakeCountChanged),
    ));
    col.into()
}

fn advanced_tab(app: &App) -> Element<'_, Message> {
    let mut col = column![].spacing(6).width(Length::Fill);
    col = col.push(
        checkbox(app.append_url_params)
            .label("将输入 URL 的 Params 添加到分片")
            .on_toggle(Message::AppendUrlParamsToggled),
    );
    col = col.push(
        checkbox(app.check_segments_count)
            .label("检测实际分片数量是否匹配")
            .on_toggle(Message::CheckSegmentsCountToggled),
    );
    col = col.push(
        checkbox(app.write_meta_json)
            .label("解析后输出 meta json")
            .on_toggle(Message::WriteMetaJsonToggled),
    );
    col = col.push(
        checkbox(app.no_log)
            .label("关闭日志文件输出")
            .on_toggle(Message::NoLogToggled),
    );
    col = col.push(lab(
        "日志级别",
        pick_list(&LOG_LEVELS[..], Some(&app.log_level), Message::LogLevelSelected).width(120),
    ));
    col = col.push(lab(
        "urlprocessor 参数",
        text_input("", &app.urlprocessor_args).on_input(Message::UrlprocessorArgsChanged),
    ));
    col = col.push(lab(
        "仅下载部分分片",
        text_input("如 0-10 / 05:00-20:00", &app.custom_range).on_input(Message::CustomRangeChanged),
    ));
    col = col.push(lab(
        "延时开始",
        text_input("yyyyMMddHHmmss", &app.task_start_at).on_input(Message::TaskStartAtChanged),
    ));
    col = col.push(lab(
        "广告分片关键字(正则)",
        text_input("", &app.ad_keyword).on_input(Message::AdKeywordChanged),
    ));
    col = col.push(lab(
        "混流参数 -M",
        text_input("如 format=mp4", &app.mux_after_done).on_input(Message::MuxAfterDoneChanged),
    ));
    col = col.push(lab(
        "引入外部媒体 --mux-import",
        text_input("path=...:lang=...", &app.mux_import).on_input(Message::MuxImportChanged),
    ));
    col = col.push(
        checkbox(app.binary_merge)
            .label("二进制合并")
            .on_toggle(Message::BinaryMergeToggled),
    );
    col = col.push(
        checkbox(app.skip_merge)
            .label("跳过合并")
            .on_toggle(Message::SkipMergeToggled),
    );
    col = col.push(
        checkbox(app.del_after_done)
            .label("完成后删除临时文件")
            .on_toggle(Message::DelAfterDoneToggled),
    );
    col = col.push(
        checkbox(app.no_date_info)
            .label("混流时不写入日期信息")
            .on_toggle(Message::NoDateInfoToggled),
    );
    col = col.push(
        checkbox(app.allow_hls_multi_ext_map)
            .label("允许 HLS 多个 #EXT-X-MAP(实验)")
            .on_toggle(Message::AllowHlsMultiExtMapToggled),
    );
    col = col.push(
        checkbox(app.disable_update_check)
            .label("禁用版本更新检测")
            .on_toggle(Message::DisableUpdateCheckToggled),
    );
    col = col.push(
        checkbox(app.force_ansi_console)
            .label("强制 ANSI 控制台")
            .on_toggle(Message::ForceAnsiConsoleToggled),
    );
    col = col.push(
        checkbox(app.use_ffmpeg_concat_demuxer)
            .label("ffmpeg 合并使用 concat 分离器")
            .on_toggle(Message::UseFfmpegConcatDemuxerToggled),
    );
    col.into()
}

fn log_tab(app: &App) -> Element<'_, Message> {
    column![
        row![
            button(text("清空日志").size(13)).on_press(Message::ClearLog),
            button(text("打开输出文件夹").size(13)).on_press(Message::OpenOutputFolder),
            space::horizontal(),
            text(if app.running { "● 运行中" } else { "○ 空闲" }).size(13),
        ]
        .spacing(10),
        scrollable(text(app.log.clone()).size(13))
            .width(Length::Fill)
            .height(Length::Fill),
    ]
    .spacing(6)
    .width(Length::Fill)
    .into()
}

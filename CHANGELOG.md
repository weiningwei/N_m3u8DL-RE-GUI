# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [1.0.0] - 2026-07-08

第一个可用版本。

### Added

- 基于 Rust + iced 的 `N_m3u8DL-RE` 图形界面：参数可视化拼接 + 启动器。
- 多标签页选项整理：基本 / 流选择 / 解密 / 直播 / 高级 / 日志。
- 实时捕获日志视图（复制、滚动、清空、打开输出文件夹）；外部控制台模式（类 SimpleG 弹窗运行）。
- 命令预览与一键复制。
- 配置持久化（可执行文件路径、保存目录、代理地址、请求头、外部控制台、主题、ffmpeg 路径）。
- 自动探测 `N_m3u8DL-RE` 与 `ffmpeg`（用户路径 → 同目录 → PATH）。
- 主题切换（跟随系统 / 浅色 / 深色），通过 Windows 注册表读取系统深色模式。
- 键盘快捷键：ESC 退出、日志区 Ctrl+C 复制、Tab / Shift+Tab 切换焦点、回车开始下载。
- 线程数（默认 8）、重试次数（默认 10）、HTTP 超时（默认 10s）已预填并始终传入。

### Fixed

- 捕获日志改为实时流式显示（增量更新 `text_editor` 内容，消除每行整体重建导致的 O(n²) 卡顿）。
- 修复中文乱码：`N_m3u8DL-RE` 管道输出按 UTF-8 解码、失败回退 GBK（CP936）；`strip_ansi` 改为按字符处理，避免多字节中文被二次拆成 Latin-1。

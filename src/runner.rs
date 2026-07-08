use crate::app::{LogEvent, Message};
use iced::futures::sink::SinkExt;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::sync::{LazyLock, Mutex};

/// 每次运行的日志接收端，按 run_gen 索引。runner 启动进程时写入，订阅消费后移除。
static RUN_CHANNELS: LazyLock<Mutex<HashMap<u64, mpsc::Receiver<LogEvent>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// 启动一次下载。capture=true 时捕获输出到内置日志；false 时开新控制台窗口（SimpleG 式）。
pub fn start_run(run_id: u64, exe: String, input: String, args: Vec<String>, capture: bool) {
    let (tx, rx) = mpsc::channel::<LogEvent>();
    RUN_CHANNELS.lock().unwrap().insert(run_id, rx);

    std::thread::spawn(move || {
        let mut cmd = Command::new(&exe);
        cmd.arg(&input);
        cmd.args(&args);

        if capture {
            cmd.stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
        } else {
            // SimpleG 式：继承标准流并开独立控制台窗口
            cmd.stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
                cmd.creation_flags(CREATE_NEW_CONSOLE);
            }
        }

        match cmd.spawn() {
            Ok(mut child) => {
                let mut readers = Vec::new();
                if capture {
                    if let Some(out) = child.stdout.take() {
                        readers.push(spawn_reader(out, tx.clone()));
                    }
                    if let Some(err) = child.stderr.take() {
                        readers.push(spawn_reader(err, tx.clone()));
                    }
                }
                let status = child.wait();
                // 必须等读取线程把输出全部发送完，再发 Done，
                // 否则订阅关闭后剩余行会丢失。
                for h in readers {
                    let _ = h.join();
                }
                let done = match status {
                    Ok(s) => Ok(s.code().unwrap_or(0) as u32),
                    Err(e) => Err(e.to_string()),
                };
                let _ = tx.send(LogEvent::Done(done));
            }
            Err(e) => {
                let _ = tx.send(LogEvent::Line(format!("启动失败：{e}")));
                let _ = tx.send(LogEvent::Done(Err(e.to_string())));
            }
        }
    });
}

fn spawn_reader<R: std::io::Read + Send + 'static>(
    mut reader: R,
    tx: mpsc::Sender<LogEvent>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut buf: Vec<u8> = Vec::with_capacity(256);
        let mut byte = [0u8; 1];
        loop {
            match reader.read(&mut byte) {
                Ok(0) => {
                    // EOF：把残留的半行也发出去
                    send_chunk(&mut buf, &tx, false);
                    break;
                }
                Ok(_) => {
                    let b = byte[0];
                    if b == b'\n' {
                        // 普通换行：作为新的一行
                        send_chunk(&mut buf, &tx, false);
                    } else if b == b'\r' {
                        // 回车：进度刷新，覆盖上一行
                        send_chunk(&mut buf, &tx, true);
                    } else {
                        buf.push(b);
                    }
                }
                Err(_) => break,
            }
        }
    })
}

/// 把缓冲的字节作为一行发送；progress=true 表示用 Progress 变体（覆盖上一行）。
fn send_chunk(buf: &mut Vec<u8>, tx: &mpsc::Sender<LogEvent>, progress: bool) {
    if buf.is_empty() {
        return;
    }
    // N_m3u8DL-RE 在管道输出时使用系统 ANSI 代码页（中文 Windows 为 GBK），
    // 直接按 UTF-8 解码会产生乱码，因此优先 UTF-8、失败再回退系统代码页。
    let raw = decode_bytes(buf);
    buf.clear();
    let line = strip_ansi(&raw);
    let line = line.trim_end_matches(['\r', '\n']).to_string();
    if line.is_empty() {
        return;
    }
    let _ = tx.send(if progress {
        LogEvent::Progress(line)
    } else {
        LogEvent::Line(line)
    });
}

/// 解码一行原始字节：
/// 1. 优先按 UTF-8 解码（无 BOM、且不容忍替换字符）；
/// 2. 失败则按 GBK（中文 Windows 系统 ANSI 代码页 CP936）解码。
fn decode_bytes(buf: &[u8]) -> String {
    if let Some(s) = encoding_rs::UTF_8.decode_without_bom_handling_and_without_replacement(buf) {
        return s.into_owned();
    }
    encoding_rs::GBK
        .decode_without_bom_handling(buf)
        .0
        .into_owned()
}

/// 移除 ANSI 转义序列（捕获模式下 RE 可能输出彩色控制符）。
///
/// 必须在按 UTF-8/GBK 解码后的字符串上、以「字符」为单位处理：
/// 若按字节把每个字节 `as char` 推入，多字节中文（如 UTF-8 的 `下`=E4 B8 8B）
/// 会被拆成多个 Latin-1 字符而再次乱码。ANSI 序列本身均为 ASCII，
/// 故跳过 ESC[...字母] 这段、其余字符原样保留即可。
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1B}' {
            // CSI 序列：ESC [ ... 终结字母
            if chars.peek() == Some(&'[') {
                chars.next(); // 消费 '['
                while let Some(&n) = chars.peek() {
                    if n.is_ascii_alphabetic() {
                        break;
                    }
                    chars.next();
                }
                chars.next(); // 消费终结字母
            }
            continue;
        }
        out.push(c);
    }
    out
}

/// 为指定 run_gen 创建日志订阅（运行结束时自动结束）。
pub fn make_subscription(run_id: u64, running: bool) -> iced::Subscription<Message> {
    if !running || run_id == 0 {
        return iced::Subscription::none();
    }
    iced::Subscription::run_with(run_id, |&rid: &u64| {
        iced::stream::channel(64, move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
            let rx = RUN_CHANNELS.lock().unwrap().remove(&rid);
            if let Some(rx) = rx {
                while let Ok(ev) = rx.recv() {
                    let _ = output.send(Message::LogEvent(ev)).await;
                }
            }
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_chinese_passes_through() {
        // “下载URL” 的 UTF-8 字节
        let bytes = "下载URL".as_bytes();
        assert_eq!(decode_bytes(bytes), "下载URL");
    }

    #[test]
    fn gbk_chinese_decoded_via_fallback() {
        // “下载URL” 按 GBK 编码后的字节，应能被回退解码还原
        let (gbk, _, _) = encoding_rs::GBK.encode("下载URL");
        assert_eq!(decode_bytes(&gbk), "下载URL");
    }

    #[test]
    fn strip_ansi_keeps_chinese_and_drops_escape() {
        // ANSI 颜色序列夹在中文之间，中文字符必须原样保留
        let input = "\u{1B}[32m下载\u{1B}[0m完成";
        assert_eq!(strip_ansi(input), "下载完成");
    }
}

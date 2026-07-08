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
        let mut data = String::new();
        // 读到 EOF（进程结束、管道关闭）。用 read_to_string 而非 lines()，
        // 避免进程退出时最后一行没有换行符而被丢弃。
        if reader.read_to_string(&mut data).is_ok() {
            for raw in data.split('\n') {
                let line = raw.strip_suffix('\r').unwrap_or(raw);
                if !line.is_empty() {
                    let _ = tx.send(LogEvent::Line(strip_ansi(line)));
                }
            }
        }
    })
}

/// 移除 ANSI 转义序列（捕获模式下 RE 可能输出彩色控制符）
fn strip_ansi(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == 0x1b && i + 1 < b.len() && b[i + 1] == b'[' {
            let mut j = i + 2;
            while j < b.len() && !(b[j].is_ascii_uppercase() || b[j].is_ascii_lowercase()) {
                j += 1;
            }
            i = j + 1;
        } else {
            out.push(b[i] as char);
            i += 1;
        }
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

//! 剪贴板专用单线程：Windows 上所有 OpenClipboard 读写均在此线程执行。

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, SyncSender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use once_cell::sync::OnceCell;

use crate::clipboard::database::Database;
use crate::clipboard::monitor::read_and_save_clipboard;
use crate::clipboard::{CLIPBOARD_WRITING, SKIP_NEXT_IMAGE};

#[cfg(target_os = "windows")]
use crate::clipboard::win_io;

static TASK_TX: OnceCell<Sender<WorkerTask>> = OnceCell::new();
#[cfg(target_os = "windows")]
static LAST_SEQUENCE: AtomicU32 = AtomicU32::new(0);

enum WorkerTask {
    MonitorRead,
    #[cfg(target_os = "windows")]
    Command(WorkerCommand),
}

#[cfg(target_os = "windows")]
enum WorkerCommand {
    WriteText {
        text: String,
        reply: SyncSender<Result<(), String>>,
    },
    WritePng {
        data: Vec<u8>,
        reply: SyncSender<Result<(), String>>,
    },
    WriteGif {
        data: Vec<u8>,
        reply: SyncSender<Result<(), String>>,
    },
    WriteFiles {
        paths: Vec<String>,
        reply: SyncSender<Result<(), String>>,
    },
    ReadText {
        reply: SyncSender<Result<String, String>>,
    },
}

/// 启动全局剪贴板 worker（幂等）。
pub fn start_clipboard_worker(db: Arc<Mutex<Database>>) {
    TASK_TX.get_or_init(|| {
        let (tx, rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("clipboard-worker".into())
            .spawn(move || worker_loop(rx, db))
            .expect("failed to spawn clipboard worker");
        tx
    });
}

pub fn schedule_clipboard_read() {
    if SKIP_NEXT_IMAGE.load(Ordering::SeqCst) || CLIPBOARD_WRITING.load(Ordering::SeqCst) {
        return;
    }
    if let Some(tx) = TASK_TX.get() {
        let _ = tx.send(WorkerTask::MonitorRead);
    }
}

#[cfg(target_os = "windows")]
pub fn clipboard_set_text(text: String) -> Result<(), String> {
    let (reply_tx, reply_rx) = mpsc::sync_channel(1);
    send_command(WorkerCommand::WriteText { text, reply: reply_tx })?;
    reply_rx
        .recv()
        .map_err(|e| format!("clipboard worker response lost: {e}"))?
}

#[cfg(target_os = "windows")]
pub fn clipboard_get_text() -> Result<String, String> {
    let (reply_tx, reply_rx) = mpsc::sync_channel(1);
    send_command(WorkerCommand::ReadText { reply: reply_tx })?;
    reply_rx
        .recv()
        .map_err(|e| format!("clipboard worker response lost: {e}"))?
}

#[cfg(target_os = "windows")]
pub fn clipboard_set_png(data: Vec<u8>) -> Result<(), String> {
    let (reply_tx, reply_rx) = mpsc::sync_channel(1);
    send_command(WorkerCommand::WritePng { data, reply: reply_tx })?;
    reply_rx
        .recv()
        .map_err(|e| format!("clipboard worker response lost: {e}"))?
}

#[cfg(target_os = "windows")]
pub fn clipboard_set_gif(data: Vec<u8>) -> Result<(), String> {
    let (reply_tx, reply_rx) = mpsc::sync_channel(1);
    send_command(WorkerCommand::WriteGif { data, reply: reply_tx })?;
    reply_rx
        .recv()
        .map_err(|e| format!("clipboard worker response lost: {e}"))?
}

#[cfg(target_os = "windows")]
pub fn clipboard_set_files(paths: Vec<String>) -> Result<(), String> {
    let (reply_tx, reply_rx) = mpsc::sync_channel(1);
    send_command(WorkerCommand::WriteFiles {
        paths,
        reply: reply_tx,
    })?;
    reply_rx
        .recv()
        .map_err(|e| format!("clipboard worker response lost: {e}"))?
}

#[cfg(target_os = "windows")]
fn send_command(cmd: WorkerCommand) -> Result<(), String> {
    TASK_TX
        .get()
        .ok_or("clipboard worker not started")?
        .send(WorkerTask::Command(cmd))
        .map_err(|e| format!("clipboard worker unavailable: {e}"))
}

fn worker_loop(rx: Receiver<WorkerTask>, db: Arc<Mutex<Database>>) {
    while let Ok(task) = rx.recv() {
        match task {
            WorkerTask::MonitorRead => handle_monitor_read(&db, &rx),
            #[cfg(target_os = "windows")]
            WorkerTask::Command(cmd) => handle_command(cmd),
        }
    }
    log::info!("Clipboard worker stopped.");
}

fn handle_monitor_read(db: &Arc<Mutex<Database>>, rx: &Receiver<WorkerTask>) {
    std::thread::sleep(Duration::from_millis(150));
    while matches!(rx.try_recv(), Ok(WorkerTask::MonitorRead)) {}

    if SKIP_NEXT_IMAGE.load(Ordering::SeqCst) || CLIPBOARD_WRITING.load(Ordering::SeqCst) {
        return;
    }

    #[cfg(target_os = "windows")]
    if !sequence_changed() {
        return;
    }

    for attempt in 0..5 {
        if SKIP_NEXT_IMAGE.load(Ordering::SeqCst) || CLIPBOARD_WRITING.load(Ordering::SeqCst) {
            break;
        }
        match read_and_save_clipboard(db) {
            Ok(()) => break,
            Err(e) if is_transient_clipboard_error(&e) => {
                std::thread::sleep(Duration::from_millis(50 * (attempt + 1) as u64));
            }
            Err(e) => {
                log::trace!("Clipboard read error: {e}");
                break;
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn sequence_changed() -> bool {
    let seq = win_io::sequence_number();
    if seq == 0 {
        return true;
    }
    let last = LAST_SEQUENCE.load(Ordering::SeqCst);
    if seq == last {
        return false;
    }
    LAST_SEQUENCE.store(seq, Ordering::SeqCst);
    true
}

#[cfg(target_os = "windows")]
fn handle_command(cmd: WorkerCommand) {
    match cmd {
        WorkerCommand::WriteText { text, reply } => {
            let _ = reply.send(win_io::write_text(&text));
        }
        WorkerCommand::WritePng { data, reply } => {
            let _ = reply.send(win_io::write_png(&data));
        }
        WorkerCommand::WriteGif { data, reply } => {
            let _ = reply.send(win_io::write_gif(&data));
        }
        WorkerCommand::WriteFiles { paths, reply } => {
            let _ = reply.send(win_io::write_files(&paths));
        }
        WorkerCommand::ReadText { reply } => {
            let result = win_io::open_clipboard().and_then(|_clip| {
                win_io::read_unicode_text().ok_or_else(|| "剪贴板无文本".into())
            });
            let _ = reply.send(result);
        }
    }
}

fn is_transient_clipboard_error(err: &str) -> bool {
    err.contains("ClipboardOccupied")
        || err.contains("Failed to open clipboard")
        || err.contains("clipboard is currently owned")
}

use crate::app_paths;

pub fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("unknown");
        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Box<dyn Any>".to_string()
        };
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_default();
        let backtrace = std::backtrace::Backtrace::force_capture();
        let msg = format!(
            "thread: {thread_name}\nmessage: {payload}\nlocation: {location}\nbacktrace:\n{backtrace}"
        );
        log::error!("PANIC: {msg}");
        app_paths::append_diagnostic("crash.log", "PANIC", &msg);
    }));
}

#[tauri::command]
pub fn record_client_error(kind: String, message: String) -> Result<(), String> {
    let body = format!("[{kind}] {message}");
    log::error!("Frontend error: {body}");
    app_paths::append_diagnostic("crash.log", "FRONTEND", &body);
    Ok(())
}

/// 捕获 Windows native 异常（如堆损坏 0xC0000374），写入 crash.log。
#[cfg(target_os = "windows")]
pub fn install_native_crash_handler() {
    use std::sync::Once;
    use windows::Win32::System::Diagnostics::Debug::SetUnhandledExceptionFilter;

    static INIT: Once = Once::new();
    INIT.call_once(|| unsafe {
        let _ = SetUnhandledExceptionFilter(Some(native_exception_filter));
    });
}

#[cfg(not(target_os = "windows"))]
pub fn install_native_crash_handler() {}

#[cfg(target_os = "windows")]
unsafe extern "system" fn native_exception_filter(
    info: *const windows::Win32::System::Diagnostics::Debug::EXCEPTION_POINTERS,
) -> i32 {
    use windows::Win32::Foundation::GetLastError;
    use windows::Win32::System::Diagnostics::Debug::EXCEPTION_CONTINUE_SEARCH;

    if !info.is_null() {
        let record = (*info).ExceptionRecord;
        if !record.is_null() {
            let code = (*record).ExceptionCode.0 as u32;
            let msg = format!(
                "ExceptionCode: 0x{code:08X}\nLastError: {:?}",
                GetLastError()
            );
            log::error!("NATIVE CRASH: {msg}");
            app_paths::append_diagnostic("crash.log", "NATIVE", &msg);
        }
    }
    EXCEPTION_CONTINUE_SEARCH
}

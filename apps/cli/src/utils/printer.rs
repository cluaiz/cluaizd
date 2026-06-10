use serde::Serialize;
use anyhow::Result;

/// Global output context — set once at CLI entry, read everywhere.
/// Avoids passing flags through every function signature.
pub static JSON_MODE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub static VERBOSE_MODE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Returns true if `--json` flag was passed.
#[inline]
pub fn is_json() -> bool {
    JSON_MODE.load(std::sync::atomic::Ordering::Relaxed)
}

/// Returns true if `--verbose` flag was passed.
#[inline]
pub fn is_verbose() -> bool {
    VERBOSE_MODE.load(std::sync::atomic::Ordering::Relaxed)
}

/// Structured wrapper for all JSON output from the CLI.
#[derive(Serialize)]
struct CliJsonResponse<T: Serialize> {
    status: &'static str,
    data: T,
}

/// Structured wrapper for JSON error output.
#[derive(Serialize)]
struct CliJsonError {
    status: &'static str,
    error: String,
}

pub struct Printer;

impl Printer {
    /// Prints a serializable object — as pretty JSON in `--json` mode,
    /// or as a human-readable labeled block in normal mode.
    pub fn print_json<T: Serialize>(label: &str, data: &T) -> Result<()> {
        if is_json() {
            let response = CliJsonResponse { status: "ok", data };
            println!("{}", serde_json::to_string_pretty(&response)?);
        } else {
            let json = serde_json::to_string_pretty(data)?;
            println!("=== {} ===", label);
            println!("{}", json);
        }
        Ok(())
    }

    /// Prints a success message — suppressed in `--json` mode (data is embedded in response).
    pub fn print_success(msg: &str) {
        if !is_json() {
            // Use ANSI green for success if terminal supports it
            #[cfg(not(windows))]
            println!("\x1b[32m[OK]\x1b[0m {}", msg);
            #[cfg(windows)]
            println!("[OK] {}", msg);
        }
    }

    /// Prints an error — always shown, in `--json` mode outputs structured error JSON.
    pub fn print_error(msg: &str) {
        if is_json() {
            let response = CliJsonError { status: "error", error: msg.to_string() };
            eprintln!("{}", serde_json::to_string_pretty(&response).unwrap_or_default());
        } else {
            #[cfg(not(windows))]
            eprintln!("\x1b[31m[ERROR]\x1b[0m {}", msg);
            #[cfg(windows)]
            eprintln!("[ERROR] {}", msg);
        }
    }

    /// Prints a debug message — only shown when `--verbose` is set.
    pub fn print_verbose(msg: &str) {
        if is_verbose() {
            #[cfg(not(windows))]
            eprintln!("\x1b[90m[DEBUG]\x1b[0m {}", msg);
            #[cfg(windows)]
            eprintln!("[DEBUG] {}", msg);
        }
    }

    /// Emits a raw success message as JSON `{"status":"ok","message":"..."}` in json mode,
    /// or plain text in normal mode.
    pub fn print_message(msg: &str) {
        if is_json() {
            #[derive(Serialize)]
            struct Msg<'a> { status: &'static str, message: &'a str }
            println!("{}", serde_json::to_string(&Msg { status: "ok", message: msg }).unwrap_or_default());
        } else {
            println!("{}", msg);
        }
    }
}

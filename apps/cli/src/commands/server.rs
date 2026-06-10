use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};
use crate::utils::printer::Printer;

const PID_FILE: &str = "data/cluaizd.pid";
const LOG_FILE: &str = "data/cluaizd.log";

/// `cluaizd-cli server start` — Spawns the server binary as a background daemon.
pub fn start(server_bin: &Path) -> Result<()> {
    // Check if already running
    if let Ok(pid_str) = std::fs::read_to_string(PID_FILE) {
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            if pid > 0 && is_process_alive(pid) {
                Printer::print_error(&format!("Cluaizd server is already running (PID: {}).", pid));
                return Ok(());
            }
        }
    }

    let mut actual_bin = server_bin.to_path_buf();
    if !actual_bin.exists() && actual_bin.extension().is_none() {
        actual_bin.set_extension(std::env::consts::EXE_EXTENSION);
    }

    if !actual_bin.exists() {
        bail!(
            "Server binary not found at {:?}. Build it first with: cargo build -p cluaizd-server",
            actual_bin
        );
    }

    // Ensure data directory exists for PID and log files
    std::fs::create_dir_all("data").context("Could not create data directory")?;

    let log_file = std::fs::File::create(LOG_FILE)
        .context("Failed to create server log file at data/cluaizd.log")?;
    let log_stderr = log_file.try_clone().context("Failed to clone log file handle")?;

    // Spawn server process detached from current terminal
    let child = Command::new(&actual_bin)
        .stdout(log_file)
        .stderr(log_stderr)
        .stdin(Stdio::null())
        .spawn()
        .with_context(|| format!("Failed to spawn cluaizd-server binary at {:?}", actual_bin))?;

    let pid = child.id();
    std::fs::write(PID_FILE, pid.to_string())
        .context("Failed to write PID file at data/cluaizd.pid")?;

    Printer::print_success(&format!("Cluaizd server started in background (PID: {}).", pid));
    println!("  Log:  {}", LOG_FILE);
    println!("  Stop: cluaizd-cli server stop");
    Ok(())
}

/// `cluaizd-cli server stop` — Gracefully kills the server process using its PID.
pub fn stop() -> Result<()> {
    let pid_str = std::fs::read_to_string(PID_FILE)
        .context("No PID file found at data/cluaizd.pid. Is the server running via `cluaizd-cli server start`?")?;
    let pid: u32 = pid_str
        .trim()
        .parse()
        .context("PID file contains invalid data (expected a number)")?;

    #[cfg(unix)]
    {
        // SAFETY: `pid` is a valid positive u32 read from our own PID file.
        // SIGTERM is a standard graceful shutdown signal. If the process doesn't
        // exist, `kill` returns -1 which we handle via `is_process_alive` check below.
        let result = unsafe { libc::kill(pid as i32, libc::SIGTERM) };
        if result != 0 && is_process_alive(pid) {
            bail!("Failed to send SIGTERM to process {}. Permission denied?", pid);
        }
    }
    #[cfg(windows)]
    {
        let output = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .output()
            .context("Failed to run taskkill to terminate server process")?;
        if !output.status.success() {
            bail!(
                "taskkill failed for PID {}: {}",
                pid,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    // Remove PID file
    let _ = std::fs::remove_file(PID_FILE);
    Printer::print_success(&format!("Cluaizd server (PID: {}) stopped.", pid));
    Ok(())
}

/// `cluaizd-cli server status` — Shows live process info.
pub fn status() -> Result<()> {
    match std::fs::read_to_string(PID_FILE) {
        Ok(pid_str) => {
            let pid: u32 = pid_str.trim().parse().unwrap_or(0);
            if is_process_alive(pid) {
                println!("=== Cluaizd Server Status ===");
                Printer::print_success(&format!("RUNNING (PID: {})", pid));
                println!("  Config:   cluaizd.toml");
                println!("  Log:      {}", LOG_FILE);
                println!("  PID File: {}", PID_FILE);
            } else {
                Printer::print_error("STOPPED (stale PID file found — cleaning up).");
                let _ = std::fs::remove_file(PID_FILE);
            }
        }
        Err(_) => {
            println!("=== Cluaizd Server Status ===");
            Printer::print_error("STOPPED (no PID file found).");
            println!("  Start with: cluaizd-cli server start");
        }
    }
    Ok(())
}

/// `cluaizd-cli server logs` — Tails the server log file to stdout.
pub fn logs(lines: usize) -> Result<()> {
    let content = std::fs::read_to_string(LOG_FILE)
        .with_context(|| format!("Cannot read log file at '{}'. Is the server running?", LOG_FILE))?;

    let all_lines: Vec<&str> = content.lines().collect();
    let tail_start = all_lines.len().saturating_sub(lines);

    println!("=== {} (last {} lines) ===", LOG_FILE, lines);
    for line in &all_lines[tail_start..] {
        println!("{}", line);
    }
    Ok(())
}

/// Cross-platform check if a process is alive given its PID.
fn is_process_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    #[cfg(unix)]
    {
        // SAFETY: Sending signal 0 is a standard POSIX way to check process existence.
        // It does not actually send any signal; it only checks if the process is reachable.
        unsafe { libc::kill(pid as i32, 0) == 0 }
    }
    #[cfg(windows)]
    {
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/NH"])
            .output();
        match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()),
            Err(_) => false,
        }
    }
}

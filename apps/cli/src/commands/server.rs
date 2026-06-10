use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};
use crate::utils::printer::Printer;

const PID_FILE: &str = "data/cluaizd.pid";

/// `cluaizd-cli server start` — Spawns the server binary as a background daemon.
pub fn start(server_bin: &Path) -> Result<()> {
    // Check if already running
    if let Ok(pid_str) = std::fs::read_to_string(PID_FILE) {
        let pid: u32 = pid_str.trim().parse().unwrap_or(0);
        if pid > 0 && is_process_alive(pid) {
            Printer::print_error(&format!("Cluaizd server is already running (PID: {}).", pid));
            return Ok(());
        }
    }

    // Ensure data directory exists for PID file
    std::fs::create_dir_all("data").context("Could not create data directory")?;

    // Spawn server process detached from current terminal
    let child = Command::new(server_bin)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .spawn()
        .context("Failed to spawn cluaizd-server binary. Is it compiled?")?;

    let pid = child.id();
    std::fs::write(PID_FILE, pid.to_string())
        .context("Failed to write PID file")?;

    Printer::print_success(&format!("Cluaizd server started in background (PID: {}).", pid));
    println!("  Log: data/cluaizd.log");
    println!("  Stop: cluaizd-cli server stop");
    Ok(())
}

/// `cluaizd-cli server stop` — Gracefully kills the server process using its PID.
pub fn stop() -> Result<()> {
    let pid_str = std::fs::read_to_string(PID_FILE)
        .context("No PID file found. Is the server running via `cluaizd-cli server start`?")?;
    let pid: u32 = pid_str.trim().parse()
        .context("Invalid PID file contents")?;

    #[cfg(unix)]
    {
        unsafe { libc::kill(pid as i32, libc::SIGTERM); }
    }
    #[cfg(windows)]
    {
        Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .output()
            .context("Failed to terminate server process")?;
    }

    // Remove PID file
    let _ = std::fs::remove_file(PID_FILE);

    Printer::print_success(&format!("Cluaizd server (PID: {}) stopped gracefully.", pid));
    Ok(())
}

/// `cluaizd-cli server status` — Shows live process info: PID, port, uptime.
pub fn status() -> Result<()> {
    match std::fs::read_to_string(PID_FILE) {
        Ok(pid_str) => {
            let pid: u32 = pid_str.trim().parse().unwrap_or(0);
            if is_process_alive(pid) {
                println!("=== Cluaizd Server Status ===");
                Printer::print_success(&format!("RUNNING (PID: {})", pid));
                println!("  Default Port: 7331");
                println!("  Config:       cluaizd.toml");
                println!("  PID File:     {}", PID_FILE);
            } else {
                Printer::print_error("STOPPED (stale PID file found, cleaning up).");
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

/// Cross-platform check if a process is alive given its PID.
fn is_process_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::kill(pid as i32, 0) == 0 }
    }
    #[cfg(windows)]
    {
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/NH"])
            .output();
        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                stdout.contains(&pid.to_string())
            }
            Err(_) => false,
        }
    }
}

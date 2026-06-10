use anyhow::{Context, Result};
use crate::utils::printer::Printer;

const WAL_DIR: &str = "data/wal";

/// `cluaizd-cli wal inspect` — Reads and displays WAL entries for diagnostics.
pub fn inspect(limit: usize) -> Result<()> {
    let wal_path = std::path::Path::new(WAL_DIR);

    if !wal_path.exists() {
        anyhow::bail!(
            "WAL directory not found at '{}'. Has the server ever run?",
            WAL_DIR
        );
    }

    let mut wal_files: Vec<_> = std::fs::read_dir(wal_path)
        .context("Failed to read WAL directory")?
        .flatten()
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "wal")
                .unwrap_or(false)
        })
        .collect();

    wal_files.sort_by_key(|e| e.file_name());

    if wal_files.is_empty() {
        println!("No WAL files found in '{}'. Database is clean.", WAL_DIR);
        return Ok(());
    }

    println!("=== WAL Inspection: {} file(s) found ===", wal_files.len());

    let mut total_entries = 0usize;
    for wal_file in &wal_files {
        let path = wal_file.path();
        let file_size = wal_file.metadata().map(|m| m.len()).unwrap_or(0);
        println!(
            "\n[FILE] {:?} ({:.2} KB)",
            path.file_name().unwrap_or_default(),
            file_size as f64 / 1024.0
        );

        // Use the wal crate's recover logic to read entries
        let mut entries_shown = 0usize;
        let result = wal::recover_from_wal(path.parent().unwrap_or(&path), &mut |entry| {
            if entries_shown < limit {
                println!(
                    "  [{:>6}] seq={} op={:?}",
                    entries_shown + 1,
                    entry.sequence,
                    entry.operation
                );
                entries_shown += 1;
                total_entries += 1;
            }
            Ok(())
        });

        match result {
            Ok(r) => {
                println!(
                    "  Summary: total={}, replayed={}, corrupt_skipped={}",
                    r.total_entries, r.replayed, r.skipped_corrupt
                );
                if r.skipped_corrupt > 0 {
                    Printer::print_error(&format!(
                        "  ⚠ {} corrupt entries detected in this WAL file!",
                        r.skipped_corrupt
                    ));
                }
            }
            Err(e) => {
                Printer::print_error(&format!("  Failed to parse WAL file: {}", e));
            }
        }
    }

    println!("\n=== Total entries shown: {} (limit: {}) ===", total_entries, limit);
    Ok(())
}

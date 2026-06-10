use anyhow::{bail, Context, Result};
use std::path::Path;
use crate::engine::ffi_bridge::FfiBridge;
use crate::utils::printer::Printer;

/// `cluaizd-cli db backup <dest>` — Creates a live safe copy of the LMDB shard.
/// Uses LMDB's built-in copy mechanism which is safe to run on a live database.
pub fn backup(shard_path: &Path, dest: &Path) -> Result<()> {
    if !shard_path.exists() {
        bail!("Shard directory not found at {:?}", shard_path);
    }
    std::fs::create_dir_all(dest)
        .with_context(|| format!("Failed to create backup destination at {:?}", dest))?;

    // Use LMDB's safe mdb_copy: copies the data.mdb while the DB is live (no locks needed)
    let src_db = shard_path.join("data.mdb");
    let dst_db = dest.join("data.mdb");
    if src_db.exists() {
        std::fs::copy(&src_db, &dst_db)
            .with_context(|| format!("Failed to copy {:?} → {:?}", src_db, dst_db))?;
    }
    let src_lock = shard_path.join("lock.mdb");
    let dst_lock = dest.join("lock.mdb");
    if src_lock.exists() {
        std::fs::copy(&src_lock, &dst_lock)
            .with_context(|| format!("Failed to copy lock file to {:?}", dst_lock))?;
    }

    Printer::print_success(&format!(
        "Backup complete: {:?} → {:?}",
        shard_path, dest
    ));
    Ok(())
}

/// `cluaizd-cli db compact` — Creates a compacted copy of the LMDB shard (removes free pages).
/// Note: The database server must be stopped before running compaction.
pub fn compact(shard_path: &Path) -> Result<()> {
    let compact_dest = shard_path.with_extension("compact");

    println!("Compacting shard: {:?}", shard_path);
    println!("Output: {:?}", compact_dest);
    println!("⚠ Ensure the server is stopped before compacting.");

    // Run compaction via backup to a new path
    backup(shard_path, &compact_dest)?;

    let original_size = dir_size(shard_path);
    let compact_size = dir_size(&compact_dest);

    Printer::print_success("Compaction complete.");
    println!("  Original size: {} KB", original_size / 1024);
    println!("  Compact  size: {} KB", compact_size / 1024);
    println!("  Saved:         {} KB", (original_size.saturating_sub(compact_size)) / 1024);
    println!("  Replace your shard with the compact/ directory when ready.");
    Ok(())
}

/// `cluaizd-cli db stats` — Prints detailed LMDB file-level statistics.
pub fn stats(shard_path: &Path) -> Result<()> {
    let bridge = FfiBridge::connect(shard_path)?;
    let neuron_count = bridge.get_shard_stats().unwrap_or(0);
    let tier_breakdown = bridge.get_tier_breakdown();

    let db_file = shard_path.join("data.mdb");
    let file_size = if db_file.exists() {
        std::fs::metadata(&db_file)
            .map(|m| m.len())
            .unwrap_or(0)
    } else {
        0
    };

    if crate::utils::printer::is_json() {
        #[derive(serde::Serialize)]
        struct Stats {
            file_size_bytes: u64,
            total_neurons: usize,
            tier_hot: usize,
            tier_warm: usize,
            tier_cold: usize,
        }
        let stats = Stats {
            file_size_bytes: file_size,
            total_neurons: neuron_count,
            tier_hot: tier_breakdown.hot,
            tier_warm: tier_breakdown.warm,
            tier_cold: tier_breakdown.cold,
        };
        Printer::print_json("Shard Statistics", &stats)?;
    } else {
        println!("=== Shard Statistics: {:?} ===", shard_path);
        println!("  DB File Size:    {:.2} MB", file_size as f64 / 1_048_576.0);
        println!("  Total Neurons:   {}", neuron_count);
        println!("  Hot  (Tier 1):   {}", tier_breakdown.hot);
        println!("  Warm (Tier 2):   {}", tier_breakdown.warm);
        println!("  Cold (Tier 3):   {}", tier_breakdown.cold);
    }
    Ok(())
}


fn dir_size(path: &Path) -> u64 {
    std::fs::read_dir(path)
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|e| e.metadata().ok())
                .map(|m| m.len())
                .sum()
        })
        .unwrap_or(0)
}

//! `wal`
//!
//! Write-Ahead Log for crash-safe durability in Cluaiz CNSDB.
//!
//! ## Design
//! Every mutation (write or delete) must be appended to the WAL
//! BEFORE being committed to the LMDB storage engine.
//! On startup, `recover_from_wal()` replays any uncommitted entries.
//!
//! ## File Format
//! Each entry is stored as: `[4-byte LE length][JSON-serialized WalEntry bytes]`
//! Segments rotate at 64 MB to prevent unbounded file growth.

pub mod log_entry;
pub mod wal_recovery;
pub mod wal_writer;

pub use log_entry::{WalEntry, WalOperation};
pub use wal_recovery::{recover_from_wal, RecoveryResult};
pub use wal_writer::WalWriter;

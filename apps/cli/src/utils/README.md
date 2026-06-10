# `utils/` — CLI Output Utilities

## Purpose

This module provides **all terminal output** for the CLI. No command module is allowed to call `println!` or `eprintln!` directly — all output must go through `Printer`. This ensures that the `--json` and `--verbose` flags work correctly across every command without duplicating flag-checking logic.

## Design: Global Atomic Flags

The `--json` and `--verbose` CLI flags are stored as `AtomicBool` globals set **once at startup** in `main.rs`. This avoids threading `bool` parameters through every function signature in the call stack.

```rust
// Set once in main.rs:
JSON_MODE.store(cli.json, Ordering::Relaxed);
VERBOSE_MODE.store(cli.verbose, Ordering::Relaxed);

// Read anywhere with zero overhead:
if is_json() { ... }
```

## Significant Files

### `printer.rs`

| Method | Normal Mode | `--json` Mode |
|---|---|---|
| `print_json(label, data)` | Pretty-prints the struct with a label header | `{"status":"ok","data":{...}}` |
| `print_success(msg)` | `[OK] msg` with ANSI green | Suppressed (data already in response) |
| `print_error(msg)` | `[ERROR] msg` with ANSI red to stderr | `{"status":"error","error":"msg"}` to stderr |
| `print_verbose(msg)` | `[DEBUG] msg` (only if `--verbose`) | Suppressed |
| `print_message(msg)` | Plain `println!` | `{"status":"ok","message":"msg"}` |

ANSI colors are applied via raw escape codes with `#[cfg(not(windows))]` guards to ensure clean output on Windows terminals that do not support ANSI sequences.

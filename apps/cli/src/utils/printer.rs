use serde::Serialize;
use anyhow::Result;

/// Provides consistent formatting for console output.
pub struct Printer;

impl Printer {
    /// Prints a JSON serializable object beautifully to the terminal.
    pub fn print_json<T: Serialize>(label: &str, data: &T) -> Result<()> {
        let json = serde_json::to_string_pretty(data)?;
        println!("=== {} ===", label);
        println!("{}", json);
        println!("==================");
        Ok(())
    }

    pub fn print_success(msg: &str) {
        // In a real CLI, we'd use a crate like `colored` here.
        println!("[SUCCESS] {}", msg);
    }

    pub fn print_error(msg: &str) {
        eprintln!("[ERROR] {}", msg);
    }
}

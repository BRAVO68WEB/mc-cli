use clap::Command;
use std::fs;
use std::path::Path;

/// Build the status subcommand definition
pub fn command() -> Command {
    Command::new("status").about("Show server running status using mc.lock")
}

/// Execute the status subcommand
pub async fn execute(_matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let lock_path = Path::new("mc.lock");
    if !lock_path.exists() {
        println!("Server status: stopped (mc.lock not found)");
        return Ok(());
    }

    let content = fs::read_to_string(lock_path)?;
    let pid_str = content.trim();
    if pid_str.is_empty() {
        println!("Server status: unknown (mc.lock is empty)");
        return Ok(());
    }

    println!("Server status: running (PID {})", pid_str);
    Ok(())
}

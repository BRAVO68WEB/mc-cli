use clap::Command;
use std::fs;
use std::path::PathBuf;
use std::process::Command as SysCommand;

/// Build the stop subcommand definition
pub fn command() -> Command {
    Command::new("stop").about("Stop the Minecraft server using mc.lock PID")
}

/// Execute the stop subcommand
pub async fn execute(_matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let lock_path = PathBuf::from("mc.lock");
    if !lock_path.exists() {
        println!("No mc.lock found. Server may not be running.");
        return Ok(());
    }

    let pid_str = fs::read_to_string(&lock_path)?.trim().to_string();
    if pid_str.is_empty() {
        println!("mc.lock is empty. Cannot determine PID.");
        return Ok(());
    }

    // Attempt to kill the process
    let output = SysCommand::new("kill").arg(pid_str.clone()).output()?;
    if output.status.success() {
        println!("Sent termination signal to PID {}", pid_str);
        // Remove lock file
        let _ = fs::remove_file(&lock_path);
        println!("mc.lock removed");
    } else {
        println!(
            "Failed to kill PID {}. It may have already exited.",
            pid_str
        );
        // Try removing lock anyway if process is gone
        let _ = fs::remove_file(&lock_path);
    }

    Ok(())
}

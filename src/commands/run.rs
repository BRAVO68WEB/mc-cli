use crate::utils::config_file::McConfig;
use crate::utils::runner::{run_cmd, run_cmd_with_io};
use clap::{Arg, Command};
use std::fs;
use std::path::PathBuf;

/// Build the run subcommand definition
pub fn command() -> Command {
    Command::new("run")
        .about("Run the Minecraft server using mc.toml configuration")
        .arg(
            Arg::new("nogui")
                .long("nogui")
                .help("Run server without GUI")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("demon")
                .long("demon")
                .short('d')
                .help("Run server in background (demon mode)")
                .action(clap::ArgAction::SetTrue),
        )
}

/// Execute the run subcommand
pub async fn execute(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = McConfig::load()?;
    let demon_mode = matches.get_flag("demon");

    // Build launch command from config.console.launch_cmd
    let mut cmd_args: Vec<String> = config.console.launch_cmd.clone();
    if matches.get_flag("nogui") && !cmd_args.iter().any(|a| a == "nogui") {
        cmd_args.push("nogui".to_string());
    }

    // Convert to &str vec for runner
    let cmd_slice: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();

    if demon_mode {
        // Background mode: do not inherit IO, do not wait
        let child = run_cmd_with_io(&cmd_slice, false).await?;
        let pid = child.id();
        fs::write(PathBuf::from("mc.lock"), format!("{}\n", pid))?;
        println!(
            "Server started in background. PID {} stored in mc.lock",
            pid
        );
    } else {
        // Foreground mode: inherit IO and wait for exit
        let mut child = run_cmd(&cmd_slice).await?;
        let pid = child.id();
        fs::write(PathBuf::from("mc.lock"), format!("{}\n", pid))?;
        println!(
            "Server started in foreground. PID {} stored in mc.lock",
            pid
        );

        let status = child.wait()?;
        println!("Server exited with status: {}", status);

        // Remove mc.lock when server stops
        let _ = fs::remove_file(PathBuf::from("mc.lock"));
        println!("mc.lock removed");
    }

    Ok(())
}

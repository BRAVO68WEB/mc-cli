use clap::{Command, Parser};

mod libs;
mod commands;
mod utils;

/// Minecraft CLI - A tool for managing Minecraft projects
#[derive(Parser, Debug)]
#[command(name = "mc-cli")]
#[command(version, about = "A CLI tool for managing Minecraft projects", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser, Debug)]
enum Commands {
    // We'll handle subcommands manually for more control
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build the CLI with manual subcommand handling for better async support
    let matches = Command::new("mc-cli")
        .version(env!("CARGO_PKG_VERSION"))
        .author("BRAVO68WEB")
        .about("A CLI tool for managing Minecraft projects")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(commands::init::command())
        .get_matches();

    // Handle subcommands
    match matches.subcommand() {
        Some(("init", sub_matches)) => {
            commands::init::execute(sub_matches).await?;
        }
        _ => {
            println!("Unknown command. Use --help for more information.");
        }
    }

    Ok(())
}
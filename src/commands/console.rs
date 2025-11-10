use clap::{Arg, Command};
use std::{io::{self, Write}, path::PathBuf};

use crate::utils::mc_server_props::ServerProperties;
use crate::utils::rcon::RconClient;

/// Build the console subcommand definition
pub fn command() -> Command {
    Command::new("console")
        .about("Interact with the Minecraft server console via RCON")
}

/// Execute the console subcommand
pub async fn execute(_: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    // Resolve config from args or server.properties
    let (host, port, password) = get_rcon_config().await?;

    println!("Connecting to RCON at {}:{} ...", host, port);
    let mut client = match RconClient::connect(&host, port, &password).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect/authenticate: {}", e);
            return Err(e);
        }
    };

    println!("Logged in. Type 'Q' or Ctrl-D to exit.");
    loop {
        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        let read = io::stdin().read_line(&mut input)?;
        if read == 0 { // EOF
            println!("Exiting console.");
            break;
        }
        let cmd = input.trim();
        if cmd.is_empty() { continue; }
        if cmd.eq_ignore_ascii_case("Q") { break; }

        match client.cmd(cmd).await {
            Ok(reply) => println!("{}", reply),
            Err(e) => eprintln!("Error: {}", e),
        }

        // Special-case stop to avoid server-side bug
        if cmd.eq_ignore_ascii_case("stop") { break; }
    }

    Ok(())
}

async fn get_rcon_config() -> Result<(String, u16, String), Box<dyn std::error::Error>> {
    // Defaults
    let mut host = String::new();
    let mut port = String::new();
    let mut password = String::new();

    // Server properties fallback
    let props = ServerProperties::from_file(PathBuf::from("server.properties"));
    if let Ok(p) = props {
            host = p.get("rcon.host").or_else(|| p.get("rcon_host")).unwrap_or_else(|| "127.0.0.1".to_string());
            port = p.get("rcon.port").or_else(|| p.get("rcon_port")).unwrap_or_else(|| "25575".to_string());
            password = p.get("rcon.password").or_else(|| p.get("rcon_password")).unwrap_or_default();
    } else {
        // If server.properties missing, apply hard defaults
        host = "127.0.0.1".to_string();
        port = "25575".to_string();
    }

    Ok((host, port.parse::<u16>().unwrap_or(25575), password))
}

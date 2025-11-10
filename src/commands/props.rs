use crate::utils::mc_server_props::ServerProperties;
use clap::Command;
use std::path::PathBuf;

/// Build the props subcommand
pub fn command() -> Command {
    Command::new("props")
        .about("Get or set values in server.properties")
        .arg(
            clap::Arg::new("key")
                .value_name("KEY")
                .help("Property key to read or set")
                .required(true),
        )
        .arg(
            clap::Arg::new("value")
                .value_name("VALUE")
                .help("Optional value to set for the property")
                .required(false),
        )
        .arg(
            clap::Arg::new("file")
                .long("file")
                .short('f')
                .value_name("PATH")
                .help("Path to server.properties (defaults to ./server.properties)")
                .required(false),
        )
}

/// Execute the props subcommand
pub async fn execute(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let key = matches.get_one::<String>("key").unwrap().to_string();
    let value = matches.get_one::<String>("value").cloned();

    let path = matches
        .get_one::<String>("file")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("server.properties"));
    let mut props = ServerProperties::from_file(&path)?;

    match value {
        Some(v) => {
            props.set(&key, v.clone());
            props.save(&path)?;
            println!("{}={}", key, v);
        }
        None => match props.get(&key) {
            Some(v) => println!("{}", v),
            None => {
                eprintln!("Key '{}' not found in server.properties", key);
            }
        },
    }

    Ok(())
}

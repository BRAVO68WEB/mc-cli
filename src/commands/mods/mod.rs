use clap::Command;

pub mod search;
pub mod add;
pub mod remove;
pub mod list;
pub mod update;

pub fn command() -> Command {
    Command::new("mods")
        .about("Manage mods via Modrinth")
        .subcommand(search::command())
        .subcommand(add::command())
        .subcommand(remove::command())
        .subcommand(list::command())
        .subcommand(update::command())
}

pub async fn execute(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    match matches.subcommand() {
        Some(("search", sub_matches)) => {
            search::execute(sub_matches).await?
        }
        Some(("add", sub_matches)) => {
            add::execute(sub_matches).await?
        }
        Some(("remove", sub_matches)) => {
            remove::execute(sub_matches).await?
        }
        Some(("list", sub_matches)) => {
            list::execute(sub_matches).await?
        }
        Some(("update", sub_matches)) => {
            update::execute(sub_matches).await?
        }
        _ => {
            println!("Use a subcommand, e.g., 'mods search --help'.");
        }
    }
    Ok(())
}

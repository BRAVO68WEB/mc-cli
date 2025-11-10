pub mod console;
pub mod init;
pub mod mods;
pub mod props;
pub mod run;
pub mod status;
pub mod stop;

// Central dispatcher mirroring mods/mod.rs style
pub async fn execute(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    match matches.subcommand() {
        Some(("init", sub_matches)) => init::execute(sub_matches).await?,
        Some(("run", sub_matches)) => run::execute(sub_matches).await?,
        Some(("console", sub_matches)) => console::execute(sub_matches).await?,
        Some(("props", sub_matches)) => props::execute(sub_matches).await?,
        Some(("status", sub_matches)) => status::execute(sub_matches).await?,
        Some(("stop", sub_matches)) => stop::execute(sub_matches).await?,
        Some(("mods", sub_matches)) => mods::execute(sub_matches).await?,
        _ => {
            println!("Unknown command. Use --help for more information.");
        }
    }
    Ok(())
}

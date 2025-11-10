use crate::libs::modrinth::ModrinthClient;
use crate::utils::config_file::McConfig;
use clap::{Arg, Command};
use std::fs;
use std::path::PathBuf;

pub fn command() -> Command {
    Command::new("remove")
        .about("Remove a mod entry from mc.toml [mods]")
        .arg(
            Arg::new("name")
                .help("Mod slug/name to remove")
                .required(true)
                .index(1),
        )
}

pub async fn execute(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let slug = matches.get_one::<String>("name").unwrap().to_string();

    let mut config = McConfig::load()?;

    // Determine installed version to locate jar file
    if let Some(installed_version) = config.mods.installed.get(&slug).cloned() {
        // Try to resolve file name from Modrinth for the installed version
        let client = ModrinthClient::new()?;
        let versions = client.get_project_versions(&slug).await?;

        let mut target_filename: Option<String> = None;
        for v in versions {
            if v.version_number.as_deref() == Some(installed_version.as_str())
                || v.id == installed_version
            {
                if let Some(file) = v
                    .files
                    .iter()
                    .find(|f| f.primary.unwrap_or(false))
                    .or_else(|| v.files.first())
                {
                    target_filename = Some(file.filename.clone());
                }
                break;
            }
        }

        // Delete local jar if we identified a filename
        if let Some(filename) = target_filename {
            let path = PathBuf::from("mods").join(&filename);
            if path.exists() {
                let _ = fs::remove_file(&path);
                println!("Deleted local jar: {}", path.display());
            } else {
                println!("Jar not found locally: {}", path.display());
            }
        } else {
            println!(
                "Could not resolve jar filename for installed version '{}' of '{}'.",
                installed_version, slug
            );
        }

        // Remove from config
        config.mods.installed.remove(&slug);
        config.save("mc.toml")?;
        println!("Removed mod: {}", slug);
    } else {
        println!("Mod not found: {}", slug);
    }

    Ok(())
}

use clap::{Arg, Command};
use crate::utils::config_file::McConfig;
use crate::libs::modrinth::ModrinthClient;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

extern crate modern_terminal;
use modern_terminal::{
    components::table::{Size, Table},
    core::console::Console,
};
use crate::utils::console_log::{header, field};

pub fn command() -> Command {
    Command::new("update")
        .about("Check installed mods against latest and update like dnf")
        .arg(
            Arg::new("yes")
                .long("yes")
                .short('y')
                .help("Assume yes; update without confirmation")
                .action(clap::ArgAction::SetTrue)
        )
}

struct UpdateCandidate {
    slug: String,
    installed: String,
    latest: String,
    old_filename: Option<String>,
    new_filename: Option<String>,
    new_url: Option<String>,
}

pub async fn execute(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let assume_yes = matches.get_flag("yes");

    let mut config = McConfig::load()?;
    let client = ModrinthClient::new()?;

    // Collect update candidates
    let mut candidates: Vec<UpdateCandidate> = Vec::new();
    for (slug, installed_version) in config.mods.installed.clone().into_iter() {
        let versions = client.get_project_versions(&slug).await;
        let mut latest_version = String::from("-");
        let mut new_file_url: Option<String> = None;
        let mut new_filename: Option<String> = None;
        let mut old_filename: Option<String> = None;

        match versions {
            Ok(vs) => {
                // Determine latest (first entry)
                if let Some(v) = vs.get(0) {
                    latest_version = v.version_number.clone().unwrap_or_else(|| v.id.clone());
                    if let Some(file) = v.files.iter().find(|f| f.primary.unwrap_or(false)).or_else(|| v.files.first()) {
                        new_file_url = Some(file.url.clone());
                        new_filename = Some(file.filename.clone());
                    }
                }
                // Determine old filename to delete
                for v in vs.iter() {
                    if v.version_number.as_deref() == Some(installed_version.as_str()) || v.id == installed_version {
                        if let Some(file) = v.files.iter().find(|f| f.primary.unwrap_or(false)).or_else(|| v.files.first()) {
                            old_filename = Some(file.filename.clone());
                        }
                        break;
                    }
                }
            }
            Err(_) => {
                // Leave latest as "-" if query failed
            }
        }

        let needs_update = !latest_version.eq(&installed_version) && latest_version != "-";
        candidates.push(UpdateCandidate {
            slug,
            installed: installed_version,
            latest: latest_version,
            old_filename,
            new_filename,
            new_url: new_file_url,
        });
    }

    // Render table showing diffs
    let mut rows: Vec<Vec<Box<dyn modern_terminal::core::render::Render>>> = Vec::new();
    rows.push(vec![
        { let b: Box<dyn modern_terminal::core::render::Render> = header("Mod".to_string()); b },
        { let b: Box<dyn modern_terminal::core::render::Render> = header("Installed".to_string()); b },
        { let b: Box<dyn modern_terminal::core::render::Render> = header("Latest".to_string()); b },
        { let b: Box<dyn modern_terminal::core::render::Render> = header("Status".to_string()); b },
    ]);
    let mut updates_available = 0usize;
    for c in candidates.iter() {
        let status = if c.latest == "-" {
            "unknown"
        } else if c.latest == c.installed {
            "up-to-date"
        } else {
            updates_available += 1;
            "update available"
        };
        rows.push(vec![
            { let b: Box<dyn modern_terminal::core::render::Render> = field(c.slug.clone()); b },
            { let b: Box<dyn modern_terminal::core::render::Render> = field(c.installed.clone()); b },
            { let b: Box<dyn modern_terminal::core::render::Render> = field(c.latest.clone()); b },
            { let b: Box<dyn modern_terminal::core::render::Render> = field(status.to_string()); b },
        ]);
    }

    let component: Table = Table {
        column_sizes: vec![Size::Cells(20), Size::Cells(20), Size::Cells(20), Size::Cells(20)],
        rows,
    };
    let mut writer = std::io::stdout();
    let mut console = Console::from_fd(&mut writer);
    console.render(&component)?;

    if updates_available == 0 {
        println!("All mods are up-to-date.");
        return Ok(());
    }

    // Confirm update unless -y
    let proceed = if assume_yes {
        true
    } else {
        print!("Proceed to update {} mod(s)? [y/N] ", updates_available);
        io::stdout().flush()?;
        let mut input = String::new();
        let read = io::stdin().read_line(&mut input)?;
        if read == 0 { false } else {
            matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
        }
    };

    if !proceed {
        println!("Update cancelled.");
        return Ok(());
    }

    // Ensure mods directory exists
    let mods_dir = PathBuf::from("mods");
    if !mods_dir.exists() { fs::create_dir_all(&mods_dir)?; }

    // Perform updates
    let mut updated = 0usize;
    for c in candidates.into_iter() {
        if c.latest == "-" || c.latest == c.installed { continue; }

        // Delete old jar if we know the filename
        if let Some(old_fn) = c.old_filename.as_ref() {
            let old_path = mods_dir.join(old_fn);
            if old_path.exists() {
                let _ = fs::remove_file(&old_path);
                println!("Removed old jar: {}", old_path.display());
            }
        }

        // Download new jar
        if let (Some(url), Some(new_fn)) = (c.new_url.as_ref(), c.new_filename.as_ref()) {
            let bytes = reqwest::get(url).await?.bytes().await?;
            let new_path = mods_dir.join(new_fn);
            fs::write(&new_path, &bytes)?;
            println!("Downloaded new jar: {}", new_path.display());
        } else {
            println!("Skipping download for {}: no file info.", c.slug);
            continue;
        }

        // Update config
        config.mods.installed.insert(c.slug.clone(), c.latest.clone());
        updated += 1;
    }

    // Save updated config
    config.save("mc.toml")?;
    println!("Updated {} mod(s).", updated);

    Ok(())
}


use clap::{Arg, Command};
use crate::utils::config_file::McConfig;
use crate::libs::modrinth::ModrinthClient;
use std::fs;
use std::path::PathBuf;
use crate::utils::config_file::Versions;

pub fn command() -> Command {
    Command::new("add")
        .about("Add a mod entry to mc.toml [mods]")
        .arg(
            Arg::new("name")
                .help("Mod slug/name to add")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("version")
                .help("Optional version string; if omitted, latest is used")
                .required(false)
                .index(2),
        )
}

pub async fn execute(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let slug = matches.get_one::<String>("name").unwrap().to_string();
    let version_arg = matches.get_one::<String>("version").cloned();

    // Ensure mods directory exists
    let mods_dir = PathBuf::from("mods");
    if !mods_dir.exists() {
        fs::create_dir_all(&mods_dir)?;
    }

    // Load config to know current MC/fabric versions for validation
    let mut config = McConfig::load()?;

    // Resolve project details for compatibility checks
    let client = ModrinthClient::new()?;
    let project = client.get_project(&slug).await?;
    // Basic server-side compatibility check (values are often: "unsupported", "optional", "required")
    if let Some(server_side) = project.server_side.as_deref() {
        if server_side == "unsupported" {
            return Err(format!("Project '{}' is not server-compatible (server_side=unsupported).", slug).into());
        }
    }

    // Resolve version via Modrinth if not provided
    let (version_number, download_url, filename) = if let Some(vn) = version_arg.clone() {
        // Find specific version by version_number
        let versions = client.get_project_versions(&slug).await?;
        let mut found = None;
        for v in versions {
            if v.version_number.as_deref() == Some(&vn) {
                // Validate loaders and game version compatibility
                // Ensure includes fabric loader if config is using fabric
                if !v.loaders.is_empty() {
                    let uses_fabric = !config.versions.fabric_version.is_empty();
                    if uses_fabric && !v.loaders.iter().any(|l| l.eq_ignore_ascii_case("fabric")) {
                        return Err(format!("Version '{}' of '{}' does not declare Fabric loader support.", vn, slug).into());
                    }
                }
                // Validate game version match
                if !v.game_versions.is_empty() {
                    let mc_ver = &config.versions.mc_version;
                    if !v.game_versions.iter().any(|gv| gv == mc_ver) {
                        return Err(format!("Version '{}' of '{}' targets game versions {:?}, not current '{}'.", vn, slug, v.game_versions, mc_ver).into());
                    }
                }
                // pick primary file or first
                if let Some(file) = v
                    .files
                    .iter()
                    .find(|f| f.primary.unwrap_or(false))
                    .or_else(|| v.files.first())
                {
                    found = Some((vn.clone(), file.url.clone(), file.filename.clone()));
                }
                break;
            }
        }
        match found {
            Some(tuple) => tuple,
            None => return Err(format!("Version '{}' not found for project '{}'.", vn, slug).into()),
        }
    } else {
        // No explicit version: pick the latest compatible version (newest first)
        let versions = client.get_project_versions(&slug).await?;
        let uses_fabric = !config.versions.fabric_version.is_empty();
        let mc_ver = &config.versions.mc_version;

        let v = versions
            .into_iter()
            .find(|v| {
                let loader_ok = !uses_fabric || v.loaders.iter().any(|l| l.eq_ignore_ascii_case("fabric"));
                let game_ok = v.game_versions.is_empty() || v.game_versions.iter().any(|gv| gv == mc_ver);
                loader_ok && game_ok
            })
            .ok_or_else(|| {
                format!(
                    "No compatible version of '{}' found for game '{}'{}.",
                    slug,
                    mc_ver,
                    if uses_fabric { " with Fabric loader" } else { "" }
                )
            })?;

        let file = v
            .files
            .iter()
            .find(|f| f.primary.unwrap_or(false))
            .or_else(|| v.files.first())
            .ok_or_else(|| format!("No files available for compatible version of '{}'.", slug))?;
        (
            v.version_number.clone().unwrap_or_else(|| v.id.clone()),
            file.url.clone(),
            file.filename.clone(),
        )
    };

    // Download file
    let target_path = mods_dir.join(&filename);
    let bytes = reqwest::get(&download_url).await?.bytes().await?;
    fs::write(&target_path, &bytes)?;

    // Update mc.toml
    config.mods.installed.insert(slug.clone(), version_number.clone());
    config.save("mc.toml")?;

    println!(
        "Downloaded: {} -> {}",
        filename,
        target_path.display()
    );
    Ok(())
}

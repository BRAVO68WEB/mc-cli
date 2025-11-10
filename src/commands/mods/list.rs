use crate::libs::modrinth::ModrinthClient;
use crate::utils::config_file::McConfig;
use clap::Command;

extern crate modern_terminal;
use crate::utils::console_log::{field, header};
use modern_terminal::{
    components::table::{Size, Table},
    core::console::Console,
};

pub fn command() -> Command {
    Command::new("list").about("List installed mods and show latest available version")
}

pub async fn execute(_matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let config = McConfig::load()?;
    let client = ModrinthClient::new()?;

    // Prepare table rows
    let mut rows: Vec<Vec<Box<dyn modern_terminal::core::render::Render>>> = Vec::new();
    rows.push(vec![
        {
            let b: Box<dyn modern_terminal::core::render::Render> = header("Mod".to_string());
            b
        },
        {
            let b: Box<dyn modern_terminal::core::render::Render> = header("Installed".to_string());
            b
        },
        {
            let b: Box<dyn modern_terminal::core::render::Render> = header("Latest".to_string());
            b
        },
    ]);

    for (slug, installed_version) in config.mods.installed.iter() {
        // Query Modrinth to find the latest version; use first entry
        let versions = client.get_project_versions(slug).await;
        let latest_version = match versions {
            Ok(vs) => {
                if let Some(v) = vs.into_iter().next() {
                    v.version_number.clone().unwrap_or_else(|| v.id.clone())
                } else {
                    String::from("-")
                }
            }
            Err(_) => String::from("-"),
        };

        rows.push(vec![
            {
                let b: Box<dyn modern_terminal::core::render::Render> = field(slug.clone());
                b
            },
            {
                let b: Box<dyn modern_terminal::core::render::Render> =
                    field(installed_version.clone());
                b
            },
            {
                let b: Box<dyn modern_terminal::core::render::Render> = field(latest_version);
                b
            },
        ]);
    }

    let component: Table = Table {
        column_sizes: vec![Size::Cells(20), Size::Cells(20), Size::Cells(20)],
        rows,
    };

    let mut writer = std::io::stdout();
    let mut console = Console::from_fd(&mut writer);
    console.render(&component)?;

    Ok(())
}

use crate::libs::fabric::{FabricClient, GameVersion, InstallerVersion, LoaderVersion};
use crate::utils::config_file::{Console as ConsoleConfig, McConfig, Versions};
use crate::utils::mc_server_props::ServerProperties;
use crate::utils::runner::run_cmd;
use clap::{Arg, Command};
use crossterm::{
    event::{self, Event, KeyCode},
    execute, terminal,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::*,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::io::{self};
use std::path::PathBuf;

/// Build the init subcommand definition
pub fn command() -> Command {
    Command::new("init")
        .about("Initialize a new Minecraft project")
        .arg(
            Arg::new("name")
                .short('n')
                .long("name")
                .value_name("NAME")
                .help("Name of the project")
                .required(false)
                .default_value("my-minecraft-project"),
        )
}

/// Execute the init subcommand
pub async fn execute(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let project_name = matches.get_one::<String>("name").unwrap();
    println!("Initializing new Minecraft project: {}", project_name);

    // Interactive selection for Game, Loader, and Installer versions using Ratatui
    let client = FabricClient::new()?;
    let game_versions: Vec<GameVersion> = client.get_game_versions().await?;
    let loader_versions: Vec<LoaderVersion> = client.get_loader_versions().await?;
    let installer_versions: Vec<InstallerVersion> = client.get_installer_versions().await?;

    let game_idx = select_with_ratatui(
        "Select Game Version",
        &game_versions
            .iter()
            .map(|g| format!("{}{}", g.version, if g.stable { " (stable)" } else { "" }))
            .collect::<Vec<_>>(),
    )?;
    let loader_idx = select_with_ratatui(
        "Select Loader Version",
        &loader_versions
            .iter()
            .map(|l| format!("{}{}", l.version, if l.stable { " (stable)" } else { "" }))
            .collect::<Vec<_>>(),
    )?;
    let installer_idx = select_with_ratatui(
        "Select Installer Version",
        &installer_versions
            .iter()
            .map(|i| format!("{}{}", i.version, if i.stable { " (stable)" } else { "" }))
            .collect::<Vec<_>>(),
    )?;

    let fabric_versions = FabricVersion {
        game: game_versions[game_idx].version.clone(),
        loader: loader_versions[loader_idx].version.clone(),
        installer: installer_versions[installer_idx].version.clone(),
    };

    println!("Using Fabric Versions:");
    println!("  Loader:    {}", fabric_versions.loader);
    println!("  Game:      {}", fabric_versions.game);
    println!("  Installer: {}", fabric_versions.installer);

    // Create configuration file via helper
    create_config_file(project_name, &fabric_versions).await?;

    // Download Fabric server JAR via helper
    download_fabric_server_jar(&fabric_versions).await?;

    // Start server once JAR is downloaded, to generate server files
    initial_start_server().await?;

    // Initial Setup
    initial_server_setup().await?;

    println!("Initialization complete.");

    Ok(())
}

pub struct FabricVersion {
    pub loader: String,
    pub game: String,
    pub installer: String,
}
/// Fetch Fabric version information
#[allow(dead_code)]
async fn fetch_fabric_versions() -> Result<FabricVersion, Box<dyn std::error::Error>> {
    let client = FabricClient::new()?;

    // Fetch latest stable versions
    let loader = client.get_latest_loader().await?;
    let game = client.get_latest_game().await?;
    let installer = client.get_latest_installer().await?;

    // latest versions variables
    let mut lv: String = String::new();
    let mut gv: String = String::new();
    let mut iv: String = String::new();

    if let Some(l) = loader {
        lv = l.version.clone();
    }
    if let Some(g) = game {
        gv = g.version.clone();
    }
    if let Some(i) = installer {
        iv = i.version.clone();
    }

    Ok(FabricVersion {
        loader: lv,
        game: gv,
        installer: iv,
    })
}

/// Create mc.toml configuration file using McConfig helper
async fn create_config_file(
    project_name: &str,
    fabric_versions: &FabricVersion,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = McConfig::new(project_name.to_string());
    config.versions = Versions {
        mc_version: fabric_versions.game.clone(),
        fabric_version: fabric_versions.loader.clone(),
        mc_cli_version: String::from("0.1.0"),
    };
    config.console = ConsoleConfig {
        launch_cmd: vec![
            String::from("java"),
            String::from("-Xmx2G"),
            String::from("-jar"),
            String::from("server.jar"),
            String::from("nogui"),
        ],
    };

    config.save(PathBuf::from("mc.toml"))?;
    println!("Created configuration file: mc.toml");
    Ok(())
}

/// Download the Fabric server JAR for the selected versions
async fn download_fabric_server_jar(
    fabric_versions: &FabricVersion,
) -> Result<(), Box<dyn std::error::Error>> {
    let fabric_server_url = format!(
        "https://meta.fabricmc.net/v2/versions/loader/{}/{}/{}/server/jar",
        fabric_versions.game, fabric_versions.loader, fabric_versions.installer
    );
    let output_file = "server.jar".to_string();
    println!("Downloading Fabric server JAR from: {}", fabric_server_url);
    let response = reqwest::get(&fabric_server_url).await?;
    let bytes = response.bytes().await?;
    tokio::fs::write(&output_file, &bytes).await?;
    println!("Downloaded Fabric server JAR to: {}", output_file);
    Ok(())
}

// Start server once JAR is downloaded, to generate server files
async fn initial_start_server() -> Result<(), Box<dyn std::error::Error>> {
    let mut child = run_cmd(&["java", "-jar", "server.jar", "nogui"]).await?;

    // wait until both eula.txt and server.properties are created
    let eula_file = PathBuf::from("eula.txt");
    let props_file = PathBuf::from("server.properties");
    loop {
        let eula_exists = eula_file.exists();
        let props_exists = props_file.exists();
        if eula_exists && props_exists {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    // terminate process gracefully
    let _ = child.kill();

    Ok(())
}

/// Render a selectable table and prompt user for a choice, returning selected index
#[allow(unused_assignments)]
fn select_with_ratatui(
    title: &str,
    items: &[String],
) -> Result<usize, Box<dyn std::error::Error>> {
    // Setup terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    // Filtering and scrolling state
    let mut query = String::new();
    let mut filtered_indices: Vec<usize> = (0..items.len()).collect();
    let mut selected: usize = 0; // index in filtered list
    let mut scroll: usize = 0; // top row in filtered list
    let mut result: usize = 0; // final selected original index

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(5)])
                .split(size);

            // Search bar
            let search_text = format!(
                "Search: {}  ({}/{})  ↑/↓ move • PgUp/PgDn scroll • Enter select",
                query,
                filtered_indices.len(),
                items.len()
            );
            let search = Paragraph::new(Line::from(search_text))
                .block(Block::default().title(title).borders(Borders::ALL));
            f.render_widget(search, chunks[0]);

            // Determine visible window with a 2-row pre-scroll buffer for better UX
            let visible = chunks[1].height as usize; // approximate rows available
            let buffer = 2usize;
            // If near the top, keep a 2-row buffer
            if selected < scroll + buffer {
                scroll = selected.saturating_sub(buffer);
            }
            if selected + buffer >= scroll + visible && visible > 0 {
                // Move the window so that selected is buffer rows from top
                let desired_top = selected.saturating_sub(buffer);
                // Ensure we don't scroll past the end
                let max_top = filtered_indices.len().saturating_sub(visible);
                scroll = std::cmp::min(desired_top, max_top);
            }
            let start = scroll;
            let end = std::cmp::min(start + visible, filtered_indices.len());

            // Build list for visible slice
            let list_items: Vec<ListItem> = (start..end)
                .map(|i| {
                    let idx = filtered_indices[i];
                    let is_sel = i == selected;
                    let prefix = if is_sel { "> " } else { "  " };
                    let style = if is_sel {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(format!("{}{}", prefix, items[idx].as_str())).style(style)
                })
                .collect();

            let block = Block::default().borders(Borders::ALL);
            let items_widget = List::new(list_items).block(block);
            f.render_widget(items_widget, chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(200))?
            && let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up => {
                        selected = selected.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        if selected + 1 < filtered_indices.len() {
                            selected += 1;
                        }
                    }
                    KeyCode::PageUp => {
                        let visible = terminal.size()?.height as usize;
                        selected = selected.saturating_sub(visible.max(1));
                    }
                    KeyCode::PageDown => {
                        let visible = terminal.size()?.height as usize;
                        selected = std::cmp::min(
                            selected + visible.max(1),
                            filtered_indices.len().saturating_sub(1),
                        );
                    }
                    KeyCode::Home => {
                        selected = 0;
                    }
                    KeyCode::End => {
                        if !filtered_indices.is_empty() {
                            selected = filtered_indices.len() - 1;
                        }
                    }
                    KeyCode::Enter => {
                        if filtered_indices.is_empty() {
                            result = 0;
                        } else {
                            result = filtered_indices[selected];
                        }
                        break;
                    }
                    KeyCode::Esc => {
                        if query.is_empty() {
                            // Cancel selection: default to first
                            result = 0;
                            break;
                        } else {
                            // Clear search
                            query.clear();
                            filtered_indices = (0..items.len()).collect();
                            selected = 0;
                            scroll = 0;
                        }
                    }
                    KeyCode::Char('q') => {
                        result = 0;
                        break;
                    }
                    KeyCode::Backspace => {
                        if !query.is_empty() {
                            query.pop();
                            // Refilter
                            let qlower = query.to_lowercase();
                            filtered_indices = items
                                .iter()
                                .enumerate()
                                .filter(|(_, s)| s.to_lowercase().contains(&qlower))
                                .map(|(i, _)| i)
                                .collect();
                            selected = 0;
                            scroll = 0;
                        }
                    }
                    KeyCode::Char(c) => {
                        // Update search query
                        query.push(c);
                        let qlower = query.to_lowercase();
                        filtered_indices = items
                            .iter()
                            .enumerate()
                            .filter(|(_, s)| s.to_lowercase().contains(&qlower))
                            .map(|(i, _)| i)
                            .collect();
                        selected = 0;
                        scroll = 0;
                    }
                    _ => {}
                }
            }
    }

    // Restore terminal
    terminal::disable_raw_mode()?;
    let mut out = io::stdout();
    execute!(out, terminal::LeaveAlternateScreen)?;

    Ok(result)
}

/// Initial setup of the server
async fn initial_server_setup() -> Result<(), Box<dyn std::error::Error>> {
    // Read existing server.properties
    let mut server_props = ServerProperties::from_file(PathBuf::from("server.properties"))?;

    server_props.set(
        "motd",
        "A Minecraft Server initialized by mc-cli".to_string(),
    );

    // Optimizations
    server_props.set("view-distance", "8".to_string());
    server_props.set("max-tick-time", "60000".to_string());
    server_props.set("force-gamemode", "true".to_string());
    // Enable RCON defaults for console command usability
    server_props.set("enable-rcon", "true".to_string());
    server_props.set("rcon.port", "25575".to_string());
    server_props.set("rcon.password", "changeme".to_string());

    server_props.save(PathBuf::from("server.properties"))?;
    println!("Created server properties file: server.properties");

    // set eula to true, in eula.txt
    let mut eula_props = ServerProperties::from_file(PathBuf::from("eula.txt"))?;
    eula_props.set("eula", "true".to_string());
    eula_props.save(PathBuf::from("eula.txt"))?;

    println!("Created eula.txt file: eula.txt");

    Ok(())
}

use crate::{
    libs::modrinth::{ModrinthClient, SearchQuery},
    utils::console_log::{field, header},
};
use clap::{Arg, Command};
extern crate modern_terminal;

use modern_terminal::{
    components::table::{Size, Table},
    core::console::Console,
};

pub fn command() -> Command {
    Command::new("search")
        .about("Search mods on Modrinth")
        .arg(
            Arg::new("query")
                .help("Search query string")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("loaders")
                .help("Filter by loaders (comma-separated), e.g., fabric,forge")
                .long("loaders")
                .short('l')
                .num_args(1)
                .required(false),
        )
        .arg(
            Arg::new("game_versions")
                .help("Filter by game versions (comma-separated), e.g., 1.20.1")
                .long("game-versions")
                .short('g')
                .num_args(1)
                .required(false),
        )
}

pub async fn execute(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let query_str = matches.get_one::<String>("query").unwrap().to_string();
    let loaders = matches.get_one::<String>("loaders").map(|s| {
        s.split(',')
            .map(|x| x.trim().to_string())
            .collect::<Vec<_>>()
    });
    let game_versions = matches.get_one::<String>("game_versions").map(|s| {
        s.split(',')
            .map(|x| x.trim().to_string())
            .collect::<Vec<_>>()
    });

    let client = ModrinthClient::new()?;

    // Build facets JSON per Modrinth search API
    // Example: [["project_type:mod"], ["categories:fabric"], ["versions:1.20.1"]]
    let mut facets: Vec<Vec<String>> = vec![vec!["project_type:mod".to_string()]];
    if let Some(loaders) = &loaders {
        for l in loaders {
            facets.push(vec![format!("categories:{}", l)]);
        }
    }
    if let Some(game_versions) = &game_versions {
        for gv in game_versions {
            facets.push(vec![format!("versions:{}", gv)]);
        }
    }
    let facets_str = serde_json::to_string(&facets)?;

    let query = SearchQuery {
        query: Some(query_str),
        facets: Some(facets_str),
        index: None,
        offset: None,
        limit: Some(20),
        filters: None,
    };

    let results = client.search_projects(Some(query)).await?;

    let mut writer = std::io::stdout();
    let mut console = Console::from_fd(&mut writer);

    // Build table rows as Vec<Vec<Box<dyn Render>>> to match Table requirements
    let mut rows_owned: Vec<Vec<Box<dyn modern_terminal::core::render::Render>>> = Vec::new();
    rows_owned.push(vec![
        {
            let b: Box<dyn modern_terminal::core::render::Render> = header("Title".to_string());
            b
        },
        {
            let b: Box<dyn modern_terminal::core::render::Render> = header("Slug".to_string());
            b
        },
        {
            let b: Box<dyn modern_terminal::core::render::Render> = header("Author".to_string());
            b
        },
    ]);
    for p in results.hits.iter() {
        rows_owned.push(vec![
            {
                let b: Box<dyn modern_terminal::core::render::Render> = field(p.title.clone());
                b
            },
            {
                let b: Box<dyn modern_terminal::core::render::Render> = field(p.slug.clone());
                b
            },
            {
                let b: Box<dyn modern_terminal::core::render::Render> = field(p.author.clone());
                b
            },
        ]);
    }

    let component: Table = Table {
        column_sizes: vec![
            Size::Cells(20),
            Size::Cells(20),
            Size::Cells(20),
            Size::Cells(20),
        ],
        rows: rows_owned,
    };

    console.render(&component)?;

    Ok(())
}

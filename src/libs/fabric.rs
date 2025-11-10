use reqwest;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://meta.fabricmc.net/v2";
const USER_AGENT: &str = "BRAVO68WEB/mc-cli/0.1.0";

// Installer Version Response
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InstallerVersion {
    pub url: String,
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

// Loader Version Response
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoaderVersion {
    pub separator: String,
    pub build: u32,
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

// Game Version Response
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GameVersion {
    pub version: String,
    pub stable: bool,
}

// Main Fabric Meta API Client
pub struct FabricClient {
    client: reqwest::Client,
    base_url: String,
}

impl FabricClient {
    /// Create a new FabricClient with default settings
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let client = reqwest::Client::builder().user_agent(USER_AGENT).build()?;

        Ok(Self {
            client,
            base_url: BASE_URL.to_string(),
        })
    }

    /// Override the base URL (useful for testing)
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Get all available Fabric installer versions
    ///
    /// Returns a list of installer versions sorted by newest first.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use mc_cli::libs::fabric::FabricClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = FabricClient::new()?;
    ///     let installers = client.get_installer_versions().await?;
    ///     
    ///     for installer in installers.iter().take(5) {
    ///         println!("{} (stable: {})", installer.version, installer.stable);
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_installer_versions(
        &self,
    ) -> Result<Vec<InstallerVersion>, Box<dyn std::error::Error>> {
        let url = format!("{}/versions/installer", self.base_url);
        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let versions: Vec<InstallerVersion> = response.json().await?;
            Ok(versions)
        } else {
            Err(format!("API request failed with status: {}", response.status()).into())
        }
    }

    /// Get all available Fabric loader versions
    ///
    /// Returns a list of loader versions sorted by newest first.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use mc_cli::libs::fabric::FabricClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = FabricClient::new()?;
    ///     let loaders = client.get_loader_versions().await?;
    ///     
    ///     // Get only stable versions
    ///     let stable_loaders: Vec<_> = loaders.iter()
    ///         .filter(|l| l.stable)
    ///         .collect();
    ///     
    ///     println!("Found {} stable loader versions", stable_loaders.len());
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_loader_versions(
        &self,
    ) -> Result<Vec<LoaderVersion>, Box<dyn std::error::Error>> {
        let url = format!("{}/versions/loader", self.base_url);
        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let versions: Vec<LoaderVersion> = response.json().await?;
            Ok(versions)
        } else {
            Err(format!("API request failed with status: {}", response.status()).into())
        }
    }

    /// Get all available Minecraft game versions
    ///
    /// Returns a list of game versions including both stable releases and snapshots.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use mc_cli::libs::fabric::FabricClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = FabricClient::new()?;
    ///     let games = client.get_game_versions().await?;
    ///     
    ///     // Get only stable releases
    ///     let stable_versions: Vec<_> = games.iter()
    ///         .filter(|g| g.stable)
    ///         .take(10)
    ///         .collect();
    ///     
    ///     println!("Latest 10 stable Minecraft versions:");
    ///     for version in stable_versions {
    ///         println!("- {}", version.version);
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_game_versions(&self) -> Result<Vec<GameVersion>, Box<dyn std::error::Error>> {
        let url = format!("{}/versions/game", self.base_url);
        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let versions: Vec<GameVersion> = response.json().await?;
            Ok(versions)
        } else {
            Err(format!("API request failed with status: {}", response.status()).into())
        }
    }

    /// Get the latest stable installer version
    pub async fn get_latest_installer(
        &self,
    ) -> Result<Option<InstallerVersion>, Box<dyn std::error::Error>> {
        let versions = self.get_installer_versions().await?;
        Ok(versions.into_iter().find(|v| v.stable))
    }

    /// Get the latest stable loader version
    pub async fn get_latest_loader(
        &self,
    ) -> Result<Option<LoaderVersion>, Box<dyn std::error::Error>> {
        let versions = self.get_loader_versions().await?;
        Ok(versions.into_iter().find(|v| v.stable))
    }

    /// Get the latest stable game version
    pub async fn get_latest_game(&self) -> Result<Option<GameVersion>, Box<dyn std::error::Error>> {
        let versions = self.get_game_versions().await?;
        Ok(versions.into_iter().find(|v| v.stable))
    }
}

impl Default for FabricClient {
    fn default() -> Self {
        Self::new().expect("Failed to create FabricClient")
    }
}

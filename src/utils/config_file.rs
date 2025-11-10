use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

/// Main configuration structure for mc.toml
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McConfig {
    /// Project/Deployment name
    pub name: String,
    
    /// Version information
    pub versions: Versions,
    
    /// Installed mods
    pub mods: Mods,
    
    /// Installed datapacks
    pub datapacks: Datapacks,
    
    /// Installed resourcepacks
    pub resourcepacks: Resourcepacks,
    
    /// Console/server configuration
    pub console: Console,
}

/// Version information section
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Versions {
    pub mc_version: String,
    pub fabric_version: String,
    pub mc_cli_version: String,
}

/// Mods section
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mods {
    #[serde(flatten)]
    pub installed: HashMap<String, String>,
}

/// Datapacks section
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Datapacks {
    #[serde(flatten)]
    pub installed: HashMap<String, String>,
}

/// Resourcepacks section
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Resourcepacks {
    #[serde(flatten)]
    pub installed: HashMap<String, String>,
}

/// Console/server configuration section
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Console {
    pub launch_cmd: Vec<String>,
}

impl McConfig {
    /// Parse mc.toml file from the given path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e))?;
        
        Self::from_str(&content)
    }
    
    /// Parse mc.toml from a string
    pub fn from_str(content: &str) -> Result<Self, ConfigError> {
        toml::from_str(content)
            .map_err(|e| ConfigError::ParseError(e))
    }
    
    /// Save configuration to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::SerializeError(e))?;
        
        fs::write(path, content)
            .map_err(|e| ConfigError::IoError(e))
    }
    
    /// Load mc.toml from the current directory
    pub fn load() -> Result<Self, ConfigError> {
        Self::from_file("mc.toml")
    }
    
    /// Check if mc.toml exists in the current directory
    pub fn exists() -> bool {
        Path::new("mc.toml").exists()
    }
    
    /// Create a new default configuration
    pub fn new(name: String) -> Self {
        Self {
            name,
            versions: Versions {
                mc_version: String::from("1.20.1"),
                fabric_version: String::from("0.15.0"),
                mc_cli_version: String::from("0.1.0"),
            },
            mods: Mods {
                installed: HashMap::new(),
            },
            datapacks: Datapacks {
                installed: HashMap::new(),
            },
            resourcepacks: Resourcepacks {
                installed: HashMap::new(),
            },
            console: Console {
                launch_cmd: vec![
                    String::from("java"),
                    String::from("-Xmx2G"),
                    String::from("-Xms2G"),
                    String::from("-jar"),
                    String::from("server.jar"),
                    String::from("nogui"),
                ],
            },
        }
    }
}

/// Error types for configuration file operations
#[derive(Debug)]
pub enum ConfigError {
    IoError(io::Error),
    ParseError(toml::de::Error),
    SerializeError(toml::ser::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::ParseError(e) => write!(f, "Parse error: {}", e),
            ConfigError::SerializeError(e) => write!(f, "Serialize error: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let toml_content = r#"
name = "my-minecraft-server"

[versions]
mc_version = "1.20.1"
fabric_version = "0.15.0"
mc_cli_version = "0.1.0"

[mods]
fabric-api = "0.92.0"
lithium = "0.11.2"
sodium = "0.5.3"

[datapacks]
vanilla-tweaks = "1.0.0"
custom-pack = "2.1.0"

[resourcepacks]
faithful = "1.20.1"

[console]
launch_cmd = ["java", "-Xmx4G", "-jar", "server.jar", "nogui"]
"#;

        let config = McConfig::from_str(toml_content).unwrap();
        
        assert_eq!(config.name, "my-minecraft-server");
        assert_eq!(config.versions.mc_version, "1.20.1");
        assert_eq!(config.versions.fabric_version, "0.15.0");
        assert_eq!(config.mods.installed.len(), 3);
        assert_eq!(config.mods.installed.get("fabric-api"), Some(&"0.92.0".to_string()));
        assert_eq!(config.datapacks.installed.len(), 2);
        assert_eq!(config.datapacks.installed.get("vanilla-tweaks"), Some(&"1.0.0".to_string()));
        assert_eq!(config.resourcepacks.installed.len(), 1);
        assert_eq!(config.resourcepacks.installed.get("faithful"), Some(&"1.20.1".to_string()));
        assert_eq!(config.console.launch_cmd.len(), 5);
    }

    #[test]
    fn test_new_config() {
        let config = McConfig::new(String::from("test-server"));
        
        assert_eq!(config.name, "test-server");
        assert_eq!(config.versions.mc_version, "1.20.1");
        assert!(config.mods.installed.is_empty());
        assert!(config.datapacks.installed.is_empty());
        assert!(config.resourcepacks.installed.is_empty());
        assert!(!config.console.launch_cmd.is_empty());
    }

    #[test]
    fn test_serialize_config() {
        let config = McConfig::new(String::from("test"));
        let toml_string = toml::to_string_pretty(&config).unwrap();
        
        assert!(toml_string.contains("name = \"test\""));
        assert!(toml_string.contains("[versions]"));
        assert!(toml_string.contains("[mods]"));
        assert!(toml_string.contains("[datapacks]"));
        assert!(toml_string.contains("[resourcepacks]"));
        assert!(toml_string.contains("[console]"));
    }
    
    #[test]
    fn test_config_with_versions() {
        let mut config = McConfig::new(String::from("test"));
        
        // Add mods with versions
        config.mods.installed.insert("xyz".to_string(), "0.0.0".to_string());
        config.mods.installed.insert("abc".to_string(), "1.1.1".to_string());
        
        // Add datapacks with versions
        config.datapacks.installed.insert("asdf".to_string(), "1.2.3".to_string());
        
        // Add resourcepacks with versions
        config.resourcepacks.installed.insert("qwerty".to_string(), "9.9.9".to_string());
        
        let toml_string = toml::to_string_pretty(&config).unwrap();
        
        assert!(toml_string.contains("xyz = \"0.0.0\""));
        assert!(toml_string.contains("abc = \"1.1.1\""));
        assert!(toml_string.contains("asdf = \"1.2.3\""));
        assert!(toml_string.contains("qwerty = \"9.9.9\""));
    }
}

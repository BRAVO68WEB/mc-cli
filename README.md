# mc-cli

A powerful CLI tool for Minecraft Project Deployment.

## Features

- 🚀 **Quick Project Initialization** - Scaffold new Minecraft projects in seconds
- 🌐 **API Integration** - Fetch latest versions from Modrinth and Fabric Meta APIs
- 🎯 **Interactive Mode** - Guided setup process (coming soon)
- 📝 **Configuration Management** - Parse and manage `mc.toml` configuration files

## Installation

### From Source

```bash
git clone <repository-url>
cd mc-cli
cargo build --release
```

The binary will be available at `target/release/mc_cli`.

### Add to PATH (Optional)

```bash
# Linux/macOS
sudo cp target/release/mc_cli /usr/local/bin/

# Or add to your shell profile
export PATH="$PATH:/path/to/mc-cli/target/release"
```

## Usage

### Basic Commands

```bash
# Show help
mc_cli --help

# Show version
mc_cli --version
```

### Initialize a New Project

The `init` subcommand creates a new Minecraft project with the specified configuration.

#### Basic Usage

```bash
# Create a Fabric mod with default settings
mc_cli init
```

### Example Config

```toml
name = "my-minecraft-server"

[versions]
mc_version = "1.20.1"
fabric_version = "0.15.0"
mc_cli_version = "0.1.0"

[mods]
xyz = "0.0.0"
abc = "1.1.1"
fabric-api = "0.92.0"
lithium = "0.11.2"
sodium = "0.5.3"

[datapacks]
asdf = "1.2.3"
vanilla-tweaks = "1.0.0"
custom-pack = "2.1.0"

[resourcepacks]
qwerty = "9.9.9"
faithful = "1.20.1"

[console]
launch_cmd = ["java", "-Xmx4G", "-Xms2G", "-jar", "server.jar", "nogui"]
```
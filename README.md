# ❄️ lazynixos

[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange?logo=rust)](https://www.rust-lang.org/)
[![NixOS](https://img.shields.io/badge/NixOS-Flakes-blue?logo=nixos)](https://nixos.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A fast, modern TUI for managing and deploying NixOS flake configurations. Heavily inspired by the workflow of `lazygit`.

## Why lazynixos?

Managing multiple NixOS hosts in a flake can get messy. You often find yourself typing long `nixos-rebuild` commands or forgetting which hosts you've defined. 

`lazynixos` solves this by giving you a clear, interactive interface. It discovers your configurations instantly and lets you deploy them with a single keystroke. You don't have to wait for the UI to respond while a build is running. Everything happens in the background, and you see the logs live in your terminal.

## Features

- **Instant Discovery**: Automatically parses `nixosConfigurations` from your local `flake.nix` using `nix eval`.
- **Live Build Logs**: A dual-pane layout shows your hosts on the left and streams build output on the right.
- **Non-blocking UI**: Built with `tokio` and `ratatui`. Shell commands run asynchronously so the interface stays snappy.
- **Interactive Deployments**: Select a host and trigger actions immediately.
- **Lightweight**: Written in Rust for maximum performance and minimal overhead.

## Installation

### Using Nix (Recommended)

You can run it directly without installing:

```bash
nix run github:xiongchenyu6/lazynixos
```

To install it permanently to your profile:

```bash
nix profile install github:xiongchenyu6/lazynixos
```

### Using Cargo

If you have Rust installed, you can install it via cargo:

```bash
cargo install --path .
```

## Usage

### Interactive TUI

Run `lazynixos` in the root of your NixOS flake directory. With no subcommand, it launches the interactive TUI.

```bash
lazynixos
lazynixos tui
```

#### TUI Keybindings

| Key | Action |
|-----|--------|
| `↑` / `k` | Move selection up |
| `↓` / `j` | Move selection down |
| `Enter` | Run `nixos-rebuild switch` |
| `b` | Run `nixos-rebuild build` |
| `d` | Run `nixos-rebuild dry-build` |
| `q` / `Esc` | Quit |

### CLI Mode

All CLI commands output JSON to stdout, making them suitable for scripting and AI agent integration.

```bash
# List all discovered hosts
lazynixos list
# {"hosts": ["desktop", "server", "laptop"]}

# Deploy a specific host
lazynixos rebuild myhost switch
lazynixos rebuild myhost build
lazynixos rebuild myhost dry-build

# Point to a specific flake
lazynixos --flake /path/to/flake list
lazynixos --flake /path/to/flake rebuild myhost switch
```

Build logs stream to stderr in real-time. The final JSON result is printed to stdout on completion, including success status and full log history.

### Configuration

You can point to a specific flake path using the `--flake` flag or an environment variable:

```bash
# Using a flag
lazynixos --flake /path/to/your/flake list

# Using an environment variable
export LAZYNIXOS_FLAKE="/path/to/your/flake"
lazynixos list
```

## Contributing

Contributions are welcome. Feel free to open an issue or submit a pull request if you have ideas for improvements or find any bugs.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

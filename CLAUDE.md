# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`lazynixos` is a Rust TUI + CLI for deploying NixOS flake configurations, inspired by `lazygit`. It discovers `nixosConfigurations` from a local flake and runs `nixos-rebuild` against them, streaming logs live.

## Commands

```bash
# Build / run (dev)
cargo build
cargo run                       # launches the TUI in the current dir's flake
cargo run -- list               # CLI: JSON list of hosts
cargo run -- rebuild HOST switch

# Quality gates (run before committing)
cargo fmt
cargo clippy
cargo test
cargo test follows_bottom_by_default   # run a single test by name

# Nix
nix build                       # build the package (uses cargoLock.lockFile)
nix develop                     # dev shell (cargo, rustc, clippy, rust-analyzer)
```

The dev environment is provided via the Nix flake + direnv (`.envrc` runs `use flake`). Edition is Rust 2024.

## Architecture

The binary has two entry paths, both selected in `main.rs` from the parsed `clap` CLI:

- **TUI mode** (no subcommand or `tui`) → `tui::run`
- **CLI mode** (`list`, `rebuild`) → calls into `cmd` directly and prints JSON to stdout

### Module responsibilities

- `cli.rs` — `clap` arg/subcommand definitions. The `--flake` path is global and also reads the `LAZYNIXOS_FLAKE` env var. Note `RebuildActionArg` (CLI enum) is mapped to `RebuildAction` (domain enum) in `main.rs`.
- `cmd.rs` — all subprocess work. `discover_hosts` shells out to `nix eval --json .#nixosConfigurations --apply builtins.attrNames`. `run_rebuild` (TUI) and `run_rebuild_cli` (CLI) both run `nixos-rebuild`; the deploy command is hardcoded as `nixos-rebuild <action> --flake <dir>#<host> --use-substitutes --target-host root@<host> --impure`.
- `types.rs` — shared domain types. `RebuildAction`/`LogStream` are internal enums; `AppEvent` is the message passed over the channel; `ListOutput`/`RebuildOutput`/`RebuildLogEntry` are the `serde::Serialize` shapes for CLI JSON output.
- `app.rs` — TUI state (`App`) and all state-transition logic. Pure and unit-tested: event handling (`apply_event`), selection, and log scrolling all live here with no I/O, which is why the tests target this module.
- `tui.rs` — the event loop: terminal setup/teardown, `crossterm` key/mouse handling, spawning `cmd` tasks, and draining the channel into `app.apply_event`.
- `ui.rs` — pure rendering (`ui::render`) of the ratatui widgets; returns the hosts-list `Rect` so the loop can map mouse clicks back to host indices.
- `image.rs` — generates the snowflake graphic shown in the TUI (via `ratatui-image`/`Picker`); degrades gracefully when no terminal graphics protocol is available.

### Key design patterns

- **Async, non-blocking deploys.** Built on `tokio`. The TUI never blocks: `run_rebuild` is `tokio::spawn`ed, and its stdout/stderr are each read on their own spawned task, forwarding every line as an `AppEvent::Log` over an `mpsc::channel`. The main loop polls events on a 50ms tick and drains the channel each iteration.
- **`AppEvent` is the single communication contract** between background subprocess tasks and the UI state. To add a new piece of deploy feedback, add a variant to `AppEvent` and handle it in `App::apply_event`.
- **State logic is separated from I/O.** Render (`ui.rs`) and event-loop (`tui.rs`) are kept thin; testable transitions live in `app.rs`. Prefer adding logic there with a unit test rather than inside the event loop.
- **Concurrent per-host deploys.** `App::running_actions` tracks in-flight rebuilds keyed by host; `can_start_action` prevents launching a second action on a host already deploying. Logs are stored per-host in `host_logs` (capped at 1000 lines via `VecDeque`).

### CLI/TUI parity caveat

`run_rebuild` and `run_rebuild_cli` in `cmd.rs` duplicate the `nixos-rebuild` invocation. If you change deploy flags or the command shape, update **both** functions so the TUI and CLI stay consistent.

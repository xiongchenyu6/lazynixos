use anyhow::Result;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

use crate::types::{AppEvent, LogLine, LogStream, RebuildAction, RebuildLogEntry, RebuildOutput};

pub async fn discover_hosts(flake_dir: &Path) -> Result<Vec<String>> {
    let output = Command::new("nix")
        .args([
            "eval",
            "--json",
            ".#nixosConfigurations",
            "--apply",
            "builtins.attrNames",
        ])
        .current_dir(flake_dir)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to parse flake: {}", stderr);
    }

    let hosts: Vec<String> = serde_json::from_slice(&output.stdout)?;
    Ok(hosts)
}

pub async fn run_rebuild(
    flake_dir: PathBuf,
    host: String,
    action: RebuildAction,
    tx: mpsc::Sender<AppEvent>,
) {
    let _ = tx
        .send(AppEvent::CommandStarted {
            host: host.clone(),
            action: action.clone(),
        })
        .await;

    let action_str = match action {
        RebuildAction::Switch => "switch",
        RebuildAction::Build => "build",
        RebuildAction::DryBuild => "dry-build",
    };

    let flake_arg = format!("{}#{}", flake_dir.display(), host);
    let target_host = format!("root@{}", host);

    let mut cmd = Command::new("nixos-rebuild");
    cmd.args([
        action_str,
        "--flake",
        &flake_arg,
        "--use-substitutes",
        "--target-host",
        &target_host,
        "--impure",
    ])
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            let _ = tx
                .send(AppEvent::CommandErrored {
                    host,
                    action,
                    error: e.to_string(),
                })
                .await;
            return;
        }
    };

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let tx_stdout = tx.clone();
    let host_stdout = host.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = tx_stdout
                .send(AppEvent::Log {
                    host: host_stdout.clone(),
                    line: LogLine {
                        stream: LogStream::Stdout,
                        text: line,
                    },
                })
                .await;
        }
    });

    let tx_stderr = tx.clone();
    let host_stderr = host.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = tx_stderr
                .send(AppEvent::Log {
                    host: host_stderr.clone(),
                    line: LogLine {
                        stream: LogStream::Stderr,
                        text: line,
                    },
                })
                .await;
        }
    });

    let status = child.wait().await.unwrap();
    let _ = tx
        .send(AppEvent::CommandFinished {
            host,
            action,
            success: status.success(),
        })
        .await;
}

pub async fn run_rebuild_cli(
    flake_dir: &Path,
    host: &str,
    action: &RebuildAction,
) -> Result<RebuildOutput> {
    let action_str = action.to_string();
    let flake_arg = format!("{}#{}", flake_dir.display(), host);
    let target_host = format!("root@{}", host);

    let mut cmd = Command::new("nixos-rebuild");
    cmd.args([
        &action_str,
        "--flake",
        &flake_arg,
        "--use-substitutes",
        "--target-host",
        &target_host,
        "--impure",
    ])
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let stdout_handle = tokio::spawn(async move {
        let mut lines = Vec::new();
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            eprintln!("{}", line);
            lines.push(RebuildLogEntry {
                stream: "stdout".to_string(),
                text: line,
            });
        }
        lines
    });

    let stderr_handle = tokio::spawn(async move {
        let mut lines = Vec::new();
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            eprintln!("{}", line);
            lines.push(RebuildLogEntry {
                stream: "stderr".to_string(),
                text: line,
            });
        }
        lines
    });

    let status = child.wait().await?;

    let mut logs = stdout_handle.await?;
    logs.extend(stderr_handle.await?);

    Ok(RebuildOutput {
        host: host.to_string(),
        action: action_str,
        success: status.success(),
        logs,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub enum RebuildAction {
    Switch,
    Build,
    DryBuild,
}

impl std::fmt::Display for RebuildAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let action = match self {
            Self::Switch => "switch",
            Self::Build => "build",
            Self::DryBuild => "dry-build",
        };

        f.write_str(action)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogStream {
    Stdout,
    Stderr,
    System,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogLine {
    pub stream: LogStream,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppEvent {
    HostsLoaded(Result<Vec<String>, String>),
    Log(LogLine),
    CommandStarted {
        host: String,
        action: RebuildAction,
    },
    CommandFinished {
        host: String,
        action: RebuildAction,
        success: bool,
    },
    CommandErrored {
        host: String,
        action: RebuildAction,
        error: String,
    },
}

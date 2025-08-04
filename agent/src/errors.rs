use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("invalid command")]
    InvalidCommand,
    #[error("invalid arguments")]
    InvalidArguments,
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    UserApi(#[from] UserApiError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("dialoguer error: {0}")]
    Dialoguer(#[from] dialoguer::Error),
    #[error("unknown error")]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("could not find home directory")]
    HomeDirNotFound,
    #[error("could not find or create config directory")]
    ConfigDirNotFound,
    #[error("could not read config file")]
    ReadFile,
    #[error("could not parse config file")]
    Parse,
    #[error("could not write config file")]
    WriteFile,
    #[error("API key is set but empty. Please provide a valid API key.")]
    EmptyApiKey,
    #[error("unknown error")]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum ControlPlaneApiError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("not found")]
    NotFound,
    #[error("internal server error")]
    InternalServerError,
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("unknown error")]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum SystemMonitorError {
    #[error("failed to read system information")]
    ReadError,
    #[error("unknown error")]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum JobExecutorError {
    #[error("failed to start job")]
    StartError,
    #[error("failed to stop job")]
    StopError,
    #[error("job failed with exit code {0}")]
    ExitCode(i64),
    #[error("unknown error")]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum UserApiError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("not found")]
    NotFound,
    #[error("internal server error")]
    InternalServerError,
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("unknown error")]
    Unknown(#[from] anyhow::Error),
}
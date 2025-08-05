use secrecy::SecretString;
use serde::Deserialize;
use std::path::PathBuf;
use tracing::{level_filters::LevelFilter, Level};
use tracing_subscriber::filter::Directive;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TlsConfig {
    #[serde(default)]
    pub enabled: bool,
    pub cert_file: PathBuf,
    pub key_file: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LogFormat {
    #[default]
    Pretty,
    Json,
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

impl From<&LogLevel> for Directive {
    fn from(value: &LogLevel) -> Self {
        let level = match value {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        };
        LevelFilter::from_level(level).into()
    }
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct LilacConfig {
    pub database_url: SecretString,
    pub tls: Option<TlsConfig>,
    pub http_port: u16,
    pub secret_key: SecretString,
    #[serde(default)]
    pub log_format: LogFormat,
    #[serde(default)]
    pub log_level: LogLevel,
    #[serde(default)]
    pub disable_sign_up: bool,
    #[serde(default)]
    pub allowed_usernames: Option<Vec<String>>,
}

impl LilacConfig {
    pub fn new() -> Option<Self> {
        let config_file_path = std::env::var("LILAC_CONFIG_FILE");
        let mut config_builder = config::Config::builder();
        if let Ok(path) = config_file_path {
            config_builder = config_builder.add_source(config::File::with_name(&path));
        }
        config_builder
            .add_source(config::Environment::with_prefix("LILAC").separator("__"))
            .build()
            .map_err(|e| println!("{e:?}"))
            .ok()?
            .try_deserialize()
            .map_err(|e| println!("{e:?}"))
            .ok()
    }
}

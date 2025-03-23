use std::path::Path;

use anyhow::{Context as _, Result};
use serde::Deserialize;
use tokio::fs::read_to_string;

pub async fn load_config(path: impl AsRef<Path>) -> Result<AppConfig> {
    let config_str = read_to_string(path)
        .await
        .context("failed to read config file")?;

    toml::from_str(&config_str).context("failed to parse config")
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub openai: AppConfigOpenai,
    pub assistant: AppConfigAssistant,
    pub platform: AppConfigPlatform,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfigOpenai {
    pub endpoint: String,
    pub token: String,
    pub model: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfigAssistant {
    pub system_role: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AppConfigPlatform {
    #[serde(default = "Default::default")]
    pub cli: AppConfigPlatformCli,

    #[serde(default = "Default::default")]
    pub mastodon: AppConfigPlatformMastodon,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AppConfigPlatformCli {
    pub enabled: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AppConfigPlatformMastodon {
    pub enabled: bool,
    pub server_url: String,
    pub token: String,
}

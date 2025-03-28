use std::{collections::HashMap, path::PathBuf};

use serde::Deserialize;

/// config.toml
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub platform: AppConfigPlatform,
    pub tool: AppConfigTool,
    pub llm: AppConfigLlm,
    pub storage: AppConfigStorage,
    pub assistant: AppConfigAssistant,
}

/// [platform]
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AppConfigPlatform {
    #[serde(default = "Default::default")]
    pub cli: AppConfigPlatformCli,

    #[serde(default = "Default::default")]
    pub mastodon: AppConfigPlatformMastodon,
}

/// [platform.cli]
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AppConfigPlatformCli {
    pub enabled: bool,
}

/// [platform.mastodon]
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AppConfigPlatformMastodon {
    pub enabled: bool,
    pub server_url: String,
    pub token: String,
    pub sensitive_spoiler: String,
    pub max_length: usize,
}

/// [tool]
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AppConfigTool {
    #[serde(default = "Default::default")]
    pub image_generator: AppConfigToolImageGenerator,
}

/// [tool.image_generator]
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AppConfigToolImageGenerator {
    pub enabled: bool,
    pub endpoint: String,
    pub token: String,
    pub model: String,
}

/// [storage]
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfigStorage {
    pub backend: AppConfigStorageBackend,
    pub sqlite: AppConfigStorageSqlite,
}

/// [storage].backend の種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppConfigStorageBackend {
    Sqlite,
    Memory,
}

/// [storage.sqlite]
#[derive(Debug, Clone, Deserialize, Default)]
pub struct AppConfigStorageSqlite {
    pub filepath: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfigLlm {
    pub backend: AppConfigLlmBackend,
    pub openai: AppConfigLlmOpenai,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppConfigLlmBackend {
    Openai,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfigLlmOpenai {
    pub api: AppConfigLlmOpenaiApi,
    pub endpoint: String,
    pub token: String,
    pub model: String,
    pub max_token: usize,
    pub use_structured_output: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppConfigLlmOpenaiApi {
    ChatCompletion,
    Resnposes,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfigAssistant {
    pub identity: String,
    pub identities: HashMap<String, AppConfigAssistantIdentity>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfigAssistantIdentity {
    pub system_role: String,

    #[serde(default = "Default::default")]
    pub sensitive_marker: String,
}

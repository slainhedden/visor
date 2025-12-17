use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const DEFAULT_AGENT_CONFIG_PATH: &str = ".acp/agents.json";

#[derive(Debug, Clone, Deserialize)]
pub struct AgentsConfig {
    pub agents: Vec<AgentConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub label: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

impl AgentsConfig {
    pub fn find(&self, id: &str) -> Option<AgentConfig> {
        self.agents.iter().find(|agent| agent.id == id).cloned()
    }
}

pub fn default_config_path() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let direct = cwd.join(DEFAULT_AGENT_CONFIG_PATH);
    if direct.exists() {
        return direct;
    }
    if let Some(parent) = cwd.parent() {
        let fallback = parent.join(DEFAULT_AGENT_CONFIG_PATH);
        if fallback.exists() {
            return fallback;
        }
    }
    direct
}

pub fn load_agents_config(path: &Path) -> Result<AgentsConfig, String> {
    let raw = std::fs::read_to_string(path)
        .map_err(|err| format!("failed to read agent config {}: {err}", path.display()))?;
    let config = serde_json::from_str(&raw)
        .map_err(|err| format!("failed to parse agent config {}: {err}", path.display()))?;
    Ok(config)
}

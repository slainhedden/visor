use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const DEFAULT_AGENT_CONFIG_PATH: &str = ".acp/agents.json";

#[derive(Debug, Clone)]
pub struct AgentsConfig {
    pub agents: Vec<AgentConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MCPServerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawAgentsConfig {
    pub version: u32,
    pub agents: HashMap<String, RawAgentConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawAgentConfig {
    pub label: Option<String>,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    #[allow(dead_code)] /* Left for future use, remove when implemented */
    pub settings: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub mcp_servers: HashMap<String, MCPServerConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub label: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub settings: HashMap<String, serde_json::Value>,
    pub mcp_servers: HashMap<String, MCPServerConfig>,
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
    let parsed: RawAgentsConfig = serde_json::from_str(&raw)
        .map_err(|err| format!("failed to parse agent config {}: {err}", path.display()))?;

    if parsed.version != 1 {
        return Err(format!(
            "unsupported agent config version {} (expected 1)",
            parsed.version
        ));
    }

    let agents = parsed
        .agents
        .into_iter()
        .map(|(id, agent)| AgentConfig {
            id: id.clone(),
            label: agent.label.unwrap_or_else(|| id.clone()),
            command: agent.command,
            args: agent.args,
            env: agent.env,
            settings: agent.settings,
            mcp_servers: agent.mcp_servers,
        })
        .collect();

    Ok(AgentsConfig { agents })
}

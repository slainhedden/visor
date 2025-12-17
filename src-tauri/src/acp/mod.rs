pub mod config;
pub mod handler;
pub mod manager;

use config::{default_config_path, load_agents_config};
use manager::{AcpManager, AcpSessionInfo, AcpState, AgentSummary};
use tauri::{AppHandle, State};

pub fn init_state() -> AcpState {
    AcpState::new(default_config_path())
}

pub async fn load_config_into_state(state: &AcpState) -> Result<(), String> {
    let config = load_agents_config(&state.config_path)?;
    let mut guard = state.manager.write().await;
    *guard = Some(AcpManager::new(config));
    Ok(())
}

#[tauri::command]
pub async fn acp_list_agents(state: State<'_, AcpState>) -> Result<Vec<AgentSummary>, String> {
    let guard = state.manager.read().await;
    guard
        .as_ref()
        .map(|manager| manager.agents())
        .ok_or_else(|| "ACP configuration not loaded".to_string())
}

#[tauri::command]
pub async fn acp_reload_config(
    state: State<'_, AcpState>,
) -> Result<Vec<AgentSummary>, String> {
    if let Some(manager) = state.manager.read().await.as_ref() {
        if manager.session_active() {
            return Err("cannot reload config while ACP session is active".to_string());
        }
    }
    load_config_into_state(&state).await?;
    acp_list_agents(state).await
}

#[tauri::command]
pub async fn acp_start_session(
    agent_id: String,
    root_dir: String,
    app: AppHandle,
    state: State<'_, AcpState>,
) -> Result<AcpSessionInfo, String> {
    let mut guard = state.manager.write().await;
    let manager = guard
        .as_mut()
        .ok_or_else(|| "ACP configuration not loaded".to_string())?;
    manager
        .start_session(app, agent_id, root_dir.into())
        .await
}

#[tauri::command]
pub async fn acp_stop_session(state: State<'_, AcpState>) -> Result<(), String> {
    let mut guard = state.manager.write().await;
    let manager = guard
        .as_mut()
        .ok_or_else(|| "ACP configuration not loaded".to_string())?;
    manager.stop_session().await
}

#[tauri::command]
pub async fn acp_send_prompt(text: String, state: State<'_, AcpState>) -> Result<(), String> {
    let guard = state.manager.read().await;
    let manager = guard
        .as_ref()
        .ok_or_else(|| "ACP configuration not loaded".to_string())?;
    manager.send_prompt(text).await
}

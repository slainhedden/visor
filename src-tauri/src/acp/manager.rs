use crate::acp::config::{AgentConfig, AgentsConfig};
use crate::acp::handler::{default_client_capabilities, VisorClient, VisorClientState};
use agent_client_protocol::{
    Agent, ClientSideConnection, ContentBlock, InitializeRequest, McpServer, McpServerStdio,
    NewSessionRequest, PromptRequest, ProtocolVersion, SessionId, SessionMode, SessionModeId,
    SessionModeState, SetSessionModeRequest,
};
use serde::Serialize;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tauri::async_runtime::JoinHandle;
use tauri::AppHandle;
use tokio::process::Command;
use tokio::sync::{mpsc, oneshot, RwLock};
use tokio::task::LocalSet;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

#[derive(Debug, Clone, Serialize)]
pub struct AgentSummary {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AcpSessionInfo {
    pub agent_id: String,
    pub session_id: String,
    pub modes: Option<ModeSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModeSummary {
    pub current_mode_id: String,
    pub available_modes: Vec<SessionMode>,
}

pub struct AcpManager {
    config: AgentsConfig,
    session: Option<AcpSession>,
}

impl AcpManager {
    pub fn new(config: AgentsConfig) -> Self {
        Self {
            config,
            session: None,
        }
    }

    pub fn agents(&self) -> Vec<AgentSummary> {
        self.config
            .agents
            .iter()
            .map(|agent| AgentSummary {
                id: agent.id.clone(),
                label: agent.label.clone(),
            })
            .collect()
    }

    pub fn find_agent(&self, id: &str) -> Option<AgentConfig> {
        self.config.find(id)
    }

    pub fn session_active(&self) -> bool {
        self.session.is_some()
    }

    pub async fn start_session(
        &mut self,
        app: AppHandle,
        agent_id: String,
        root_dir: PathBuf,
    ) -> Result<AcpSessionInfo, String> {
        if self.session.is_some() {
            return Err("ACP session already active".to_string());
        }

        let agent = self
            .find_agent(&agent_id)
            .ok_or_else(|| format!("unknown agent id: {agent_id}"))?;

        let root_dir = root_dir
            .canonicalize()
            .map_err(|err| format!("invalid root dir: {err}"))?;

        let (session, mode_state) = spawn_session(app, agent, root_dir).await?;
        let session_info = AcpSessionInfo {
            agent_id: session.agent_id.clone(),
            session_id: session.session_id.to_string(),
            modes: mode_state.map(|modes| ModeSummary {
                current_mode_id: modes.current_mode_id.to_string(),
                available_modes: modes.available_modes.clone(),
            }),
        };

        self.session = Some(session);
        Ok(session_info)
    }

    pub async fn stop_session(&mut self) -> Result<(), String> {
        if let Some(mut session) = self.session.take() {
            session.shutdown().await;
        }
        Ok(())
    }

    pub async fn send_prompt(&self, text: String) -> Result<(), String> {
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| "ACP session not started".to_string())?;
        session.send_prompt(text).await
    }

    pub async fn set_mode(&self, mode_id: String) -> Result<(), String> {
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| "ACP session not started".to_string())?;
        session.set_mode(SessionModeId::new(mode_id)).await
    }

    pub async fn resolve_permission(
        &self,
        request_id: String,
        option_id: Option<String>,
    ) -> Result<(), String> {
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| "ACP session not started".to_string())?;
        session.resolve_permission(request_id, option_id).await
    }
}

pub struct AcpState {
    pub manager: RwLock<Option<AcpManager>>,
    pub config_path: PathBuf,
}

impl AcpState {
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            manager: RwLock::new(None),
            config_path,
        }
    }
}

enum AcpCommand {
    Prompt {
        text: String,
        respond: oneshot::Sender<Result<(), String>>,
    },
    SetMode {
        mode_id: SessionModeId,
        respond: oneshot::Sender<Result<(), String>>,
    },
    Shutdown,
}

struct AcpSession {
    agent_id: String,
    child: tokio::sync::Mutex<tokio::process::Child>,
    local_task: JoinHandle<()>,
    command_tx: mpsc::Sender<AcpCommand>,
    session_id: SessionId,
    #[allow(dead_code)] /* Left for future use, remove when implemented */
    modes: Arc<RwLock<Option<SessionModeState>>>,
    client_state: Arc<VisorClientState>,
}

impl AcpSession {
    async fn send_prompt(&self, text: String) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(AcpCommand::Prompt { text, respond: tx })
            .await
            .map_err(|_| "ACP command channel closed".to_string())?;
        rx.await.map_err(|_| "ACP prompt canceled".to_string())?
    }

    async fn set_mode(&self, mode_id: SessionModeId) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(AcpCommand::SetMode {
                mode_id,
                respond: tx,
            })
            .await
            .map_err(|_| "ACP command channel closed".to_string())?;
        rx.await.map_err(|_| "ACP set mode canceled".to_string())?
    }

    async fn shutdown(&mut self) {
        let _ = self.command_tx.send(AcpCommand::Shutdown).await;
        let _ = self.local_task.abort();
        let mut child = self.child.lock().await;
        let _ = child.kill().await;
    }

    async fn resolve_permission(
        &self,
        request_id: String,
        option_id: Option<String>,
    ) -> Result<(), String> {
        self.client_state
            .resolve_permission(&request_id, option_id)
            .await
    }
}

async fn spawn_session(
    app: AppHandle,
    agent: AgentConfig,
    root_dir: PathBuf,
) -> Result<(AcpSession, Option<SessionModeState>), String> {
    let mut command = Command::new(&agent.command);
    command.args(&agent.args);
    command.current_dir(&root_dir);
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::inherit());

    for (key, value) in &agent.env {
        command.env(key, value);
    }

    let mut child = command
        .spawn()
        .map_err(|err| format!("failed to spawn agent: {err}"))?;

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| "agent stdin unavailable".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "agent stdout unavailable".to_string())?;

    let mode_state: Arc<RwLock<Option<SessionModeState>>> = Arc::new(RwLock::new(None));
    let state = Arc::new(VisorClientState::new(
        root_dir.clone(),
        app,
        mode_state.clone(),
    ));
    let handler = VisorClient::new(state.clone());

    let (session_tx, session_rx) = oneshot::channel();
    let (command_tx, mut command_rx) = mpsc::channel::<AcpCommand>(16);

    let root_dir_for_task = root_dir.clone();
    let agent_for_task = agent.clone();
    let mode_state_for_task = mode_state.clone();
    let local_task = tauri::async_runtime::spawn_blocking(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to create ACP runtime");
        let local_set = LocalSet::new();
        runtime.block_on(local_set.run_until(async move {
            let (client, io_task) = ClientSideConnection::new(
                handler,
                stdin.compat_write(),
                stdout.compat(),
                |task| {
                    tokio::task::spawn_local(task);
                },
            );

            tokio::task::spawn_local(async move {
                if let Err(err) = io_task.await {
                    eprintln!("ACP IO task error: {err}");
                }
            });

            let init = InitializeRequest::new(ProtocolVersion::LATEST)
                .client_capabilities(default_client_capabilities());
            if let Err(err) = client.initialize(init).await {
                let _ = session_tx.send(Err(format!("initialize failed: {err}")));
                return;
            }

            let new_session = match client
                .new_session(
                    NewSessionRequest::new(&root_dir_for_task)
                        .mcp_servers(mcp_servers_from_config(&agent_for_task)),
                )
                .await
            {
                Ok(response) => response,
                Err(err) => {
                    let _ = session_tx.send(Err(format!("new_session failed: {err}")));
                    return;
                }
            };

            let session_id = new_session.session_id.clone();
            if let Some(modes) = new_session.modes.clone() {
                let mut guard = mode_state_for_task.write().await;
                *guard = Some(modes);
            }
            let _ = session_tx.send(Ok(session_id.clone()));

            loop {
                let Some(cmd) = command_rx.recv().await else {
                    break;
                };
                match cmd {
                    AcpCommand::Prompt { text, respond } => {
                        let prompt =
                            PromptRequest::new(session_id.clone(), vec![ContentBlock::from(text)]);
                        let result = client
                            .prompt(prompt)
                            .await
                            .map(|_| ())
                            .map_err(|err| format!("prompt failed: {err}"));
                        let _ = respond.send(result);
                    }
                    AcpCommand::SetMode { mode_id, respond } => {
                        let request = SetSessionModeRequest::new(session_id.clone(), mode_id);
                        let result = client
                            .set_session_mode(request)
                            .await
                            .map(|_| ())
                            .map_err(|err| format!("set mode failed: {err}"));
                        let _ = respond.send(result);
                    }
                    AcpCommand::Shutdown => break,
                }
            }
        }));
    });

    let session_id = session_rx
        .await
        .map_err(|_| "failed to establish ACP session".to_string())??;

    let session = AcpSession {
        agent_id: agent.id,
        child: tokio::sync::Mutex::new(child),
        local_task,
        command_tx,
        session_id: session_id.clone(),
        modes: mode_state.clone(),
        client_state: state,
    };

    let modes = mode_state.read().await.clone();

    Ok((session, modes))
}

fn mcp_servers_from_config(agent: &AgentConfig) -> Vec<McpServer> {
    agent
        .mcp_servers
        .iter()
        .map(|(name, config)| {
            McpServer::Stdio(
                McpServerStdio::new(name.clone(), &config.command)
                    .args(config.args.clone())
                    .env(
                        config
                            .env
                            .iter()
                            .map(|(k, v)| {
                                agent_client_protocol::EnvVariable::new(k.clone(), v.clone())
                            })
                            .collect(),
                    ),
            )
        })
        .collect()
}

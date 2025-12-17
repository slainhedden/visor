use crate::acp::config::{AgentConfig, AgentsConfig};
use crate::acp::handler::{default_client_capabilities, VisorClient, VisorClientState};
use agent_client_protocol::{
    Agent, ClientSideConnection, ContentBlock, InitializeRequest, NewSessionRequest, PromptRequest,
    ProtocolVersion, SessionId,
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

        let (session, session_id) = spawn_session(app, agent, root_dir).await?;
        let session_info = AcpSessionInfo {
            agent_id: session.agent_id.clone(),
            session_id: session_id.to_string(),
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
    Shutdown,
}

struct AcpSession {
    agent_id: String,
    child: tokio::sync::Mutex<tokio::process::Child>,
    local_task: JoinHandle<()>,
    command_tx: mpsc::Sender<AcpCommand>,
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

    async fn shutdown(&mut self) {
        let _ = self.command_tx.send(AcpCommand::Shutdown).await;
        let _ = self.local_task.abort();
        let mut child = self.child.lock().await;
        let _ = child.kill().await;
    }
}

async fn spawn_session(
    app: AppHandle,
    agent: AgentConfig,
    root_dir: PathBuf,
) -> Result<(AcpSession, SessionId), String> {
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

    let state = Arc::new(VisorClientState::new(root_dir.clone(), app));
    let handler = VisorClient::new(state);

    let (session_tx, session_rx) = oneshot::channel();
    let (command_tx, mut command_rx) = mpsc::channel::<AcpCommand>(16);

    let root_dir_for_task = root_dir.clone();
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

            let init = InitializeRequest::new(ProtocolVersion::LATEST)
                .client_capabilities(default_client_capabilities());
            if let Err(err) = client.initialize(init).await {
                let _ = session_tx.send(Err(format!("initialize failed: {err}")));
                return;
            }

            let new_session = match client
                .new_session(NewSessionRequest::new(&root_dir_for_task))
                .await
            {
                Ok(response) => response,
                Err(err) => {
                    let _ = session_tx.send(Err(format!("new_session failed: {err}")));
                    return;
                }
            };

            let session_id = new_session.session_id.clone();
            let _ = session_tx.send(Ok(session_id.clone()));

            tokio::pin!(io_task);
            loop {
                tokio::select! {
                    result = &mut io_task => {
                        if let Err(err) = result {
                            eprintln!("ACP IO task error: {err}");
                        }
                        break;
                    }
                    maybe_cmd = command_rx.recv() => {
                        let Some(cmd) = maybe_cmd else { break; };
                        match cmd {
                            AcpCommand::Prompt { text, respond } => {
                                let prompt = PromptRequest::new(session_id.clone(), vec![ContentBlock::from(text)]);
                                let result = client
                                    .prompt(prompt)
                                    .await
                                    .map(|_| ())
                                    .map_err(|err| format!("prompt failed: {err}"));
                                let _ = respond.send(result);
                            }
                            AcpCommand::Shutdown => break,
                        }
                    }
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
    };

    Ok((session, session_id))
}

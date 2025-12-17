use agent_client_protocol::{
    AgentNotification, AgentRequest, ClientCapabilities, ClientResponse, ContentBlock,
    CreateTerminalRequest, CreateTerminalResponse, FileSystemCapability, KillTerminalCommandResponse,
    MessageHandler, PermissionOptionKind, ReadTextFileResponse, ReleaseTerminalResponse,
    RequestPermissionOutcome, RequestPermissionResponse, SelectedPermissionOutcome,
    SessionNotification, SessionUpdate, TerminalExitStatus, TerminalId, TerminalOutputRequest,
    TerminalOutputResponse, WaitForTerminalExitRequest, WaitForTerminalExitResponse,
    WriteTextFileResponse,
};
use agent_client_protocol::{
    Error, KillTerminalCommandRequest, ReadTextFileRequest, ReleaseTerminalRequest,
    RequestPermissionRequest, Result, WriteTextFileRequest,
};
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tauri::Emitter;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct VisorClient {
    state: Arc<VisorClientState>,
}

impl VisorClient {
    pub fn new(state: Arc<VisorClientState>) -> Self {
        Self { state }
    }
}

pub struct VisorClientState {
    pub root_dir: PathBuf,
    app_handle: tauri::AppHandle,
    terminals: Arc<Mutex<HashMap<TerminalId, Arc<TerminalState>>>>,
    terminal_counter: AtomicUsize,
}

impl VisorClientState {
    pub fn new(root_dir: PathBuf, app_handle: tauri::AppHandle) -> Self {
        Self {
            root_dir,
            app_handle,
            terminals: Arc::new(Mutex::new(HashMap::new())),
            terminal_counter: AtomicUsize::new(1),
        }
    }

    fn next_terminal_id(&self) -> TerminalId {
        let next = self.terminal_counter.fetch_add(1, Ordering::SeqCst);
        TerminalId::new(format!("term-{next}"))
    }

    fn emit_event(&self, event: AcpUiEvent) {
        let _ = self.app_handle.emit("acp://update", event);
    }

    fn validate_path(&self, path: &Path, allow_missing: bool) -> Result<PathBuf> {
        if path.components().any(|c| matches!(c, Component::ParentDir)) {
            return Err(Error::invalid_params().data("parent paths are not allowed"));
        }

        let root = &self.root_dir;
        let candidate = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };

        let canonical = match candidate.canonicalize() {
            Ok(path) => path,
            Err(err) if allow_missing => {
                let parent = candidate
                    .parent()
                    .ok_or_else(|| Error::invalid_params().data("path has no parent"))?;
                let canonical_parent = parent
                    .canonicalize()
                    .map_err(|_| Error::invalid_params().data("invalid parent path"))?;
                let file_name = candidate
                    .file_name()
                    .ok_or_else(|| Error::invalid_params().data("path has no file name"))?;
                canonical_parent.join(file_name)
            }
            Err(err) => {
                return Err(Error::invalid_params().data(format!(
                    "failed to resolve path: {err}"
                )))
            }
        };

        if !canonical.starts_with(root) {
            return Err(Error::invalid_params().data("path is outside project root"));
        }

        Ok(canonical)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)]
pub enum AcpUiEvent {
    ChatMessage { session_id: String, content: String },
    StatusUpdate { session_id: String, content: String },
    Error { session_id: String, content: String },
}

struct TerminalState {
    output: Mutex<TerminalOutputState>,
    child: Mutex<tokio::process::Child>,
    exit_status: Mutex<Option<TerminalExitStatus>>,
    output_limit: Option<u64>,
}

struct TerminalOutputState {
    output: String,
    truncated: bool,
}

impl TerminalOutputState {
    fn new() -> Self {
        Self {
            output: String::new(),
            truncated: false,
        }
    }

    fn append(&mut self, chunk: &str, limit: Option<u64>) {
        self.output.push_str(chunk);
        if let Some(limit) = limit {
            let limit = limit as usize;
            if self.output.len() > limit {
                self.truncated = true;
                let mut start = self.output.len().saturating_sub(limit);
                while !self.output.is_char_boundary(start) {
                    start += 1;
                }
                self.output = self.output[start..].to_string();
            }
        }
    }
}

impl MessageHandler<agent_client_protocol::ClientSide> for VisorClient {
    fn handle_request(&self, request: AgentRequest) -> impl std::future::Future<Output = Result<ClientResponse>> {
        let state = self.state.clone();
        async move {
            match request {
                AgentRequest::RequestPermissionRequest(req) => {
                    Ok(ClientResponse::RequestPermissionResponse(handle_permission(req)))
                }
                AgentRequest::ReadTextFileRequest(req) => {
                    Ok(ClientResponse::ReadTextFileResponse(handle_read_text(&state, req).await?))
                }
                AgentRequest::WriteTextFileRequest(req) => {
                    handle_write_text(&state, req).await?;
                    Ok(ClientResponse::WriteTextFileResponse(WriteTextFileResponse::new()))
                }
                AgentRequest::CreateTerminalRequest(req) => {
                    let response = handle_create_terminal(&state, req).await?;
                    Ok(ClientResponse::CreateTerminalResponse(response))
                }
                AgentRequest::TerminalOutputRequest(req) => {
                    let response = handle_terminal_output(&state, req).await?;
                    Ok(ClientResponse::TerminalOutputResponse(response))
                }
                AgentRequest::ReleaseTerminalRequest(req) => {
                    handle_release_terminal(&state, req).await?;
                    Ok(ClientResponse::ReleaseTerminalResponse(ReleaseTerminalResponse::new()))
                }
                AgentRequest::WaitForTerminalExitRequest(req) => {
                    let response = handle_wait_for_exit(&state, req).await?;
                    Ok(ClientResponse::WaitForTerminalExitResponse(response))
                }
                AgentRequest::KillTerminalCommandRequest(req) => {
                    handle_kill_terminal(&state, req).await?;
                    Ok(ClientResponse::KillTerminalResponse(KillTerminalCommandResponse::new()))
                }
                AgentRequest::ExtMethodRequest(ext) => {
                    Err(Error::method_not_found().data(format!("unsupported ext method {}", ext.method)))
                }
                _ => Err(Error::method_not_found().data("unsupported request type")),
            }
        }
    }

    fn handle_notification(
        &self,
        notification: AgentNotification,
    ) -> impl std::future::Future<Output = Result<()>> {
        let state = self.state.clone();
        async move {
            match notification {
                AgentNotification::SessionNotification(note) => {
                    emit_session_update(&state, note);
                    Ok(())
                }
                AgentNotification::ExtNotification(_) => Ok(()),
                _ => Ok(()),
            }
        }
    }
}

fn handle_permission(request: RequestPermissionRequest) -> RequestPermissionResponse {
    let selected = request
        .options
        .iter()
        .find(|option| matches!(option.kind, PermissionOptionKind::AllowOnce))
        .or_else(|| request.options.iter().find(|option| matches!(option.kind, PermissionOptionKind::AllowAlways)))
        .or_else(|| request.options.first())
        .map(|option| SelectedPermissionOutcome::new(option.option_id.clone()));

    let outcome = match selected {
        Some(selection) => RequestPermissionOutcome::Selected(selection),
        None => RequestPermissionOutcome::Cancelled,
    };

    RequestPermissionResponse::new(outcome)
}

async fn handle_read_text(state: &VisorClientState, req: ReadTextFileRequest) -> Result<ReadTextFileResponse> {
    let path = state.validate_path(&req.path, false)?;
    let content = tokio::fs::read_to_string(&path)
        .await
        .map_err(|err| Error::internal_error().data(format!("failed to read file: {err}")))?;

    let content = if req.line.is_some() || req.limit.is_some() {
        let start = req.line.unwrap_or(1).saturating_sub(1) as usize;
        let limit = req.limit.unwrap_or(u32::MAX) as usize;
        content
            .lines()
            .skip(start)
            .take(limit)
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        content
    };

    Ok(ReadTextFileResponse::new(content))
}

async fn handle_write_text(state: &VisorClientState, req: WriteTextFileRequest) -> Result<()> {
    let path = state.validate_path(&req.path, true)?;
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|err| Error::internal_error().data(format!("failed to create dirs: {err}")))?;
    }
    tokio::fs::write(&path, req.content)
        .await
        .map_err(|err| Error::internal_error().data(format!("failed to write file: {err}")))?;
    Ok(())
}

async fn handle_create_terminal(
    state: &VisorClientState,
    req: CreateTerminalRequest,
) -> Result<CreateTerminalResponse> {
    let terminal_id = state.next_terminal_id();
    let cwd = match &req.cwd {
        Some(path) => state.validate_path(path, false)?,
        None => state.root_dir.clone(),
    };

    let mut command = Command::new(&req.command);
    command.args(&req.args).current_dir(cwd).stdin(std::process::Stdio::piped());
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());

    for env_var in &req.env {
        command.env(&env_var.name, &env_var.value);
    }

    let mut child = command
        .spawn()
        .map_err(|err| Error::internal_error().data(format!("failed to spawn terminal: {err}")))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let terminal_state = Arc::new(TerminalState {
        output: Mutex::new(TerminalOutputState::new()),
        child: Mutex::new(child),
        exit_status: Mutex::new(None),
        output_limit: req.output_byte_limit,
    });

    let mut terminals = state.terminals.lock().await;
    terminals.insert(terminal_id.clone(), terminal_state.clone());
    drop(terminals);

    if let Some(stdout) = stdout {
        let terminal_state = terminal_state.clone();
        tauri::async_runtime::spawn(async move {
            read_terminal_output(stdout, terminal_state).await;
        });
    }

    if let Some(stderr) = stderr {
        let terminal_state = terminal_state.clone();
        tauri::async_runtime::spawn(async move {
            read_terminal_output(stderr, terminal_state).await;
        });
    }

    Ok(CreateTerminalResponse::new(terminal_id))
}

async fn read_terminal_output<R>(mut reader: R, terminal_state: Arc<TerminalState>)
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    let mut buffer = [0u8; 1024];
    loop {
        match reader.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => {
                let chunk = String::from_utf8_lossy(&buffer[..n]);
                let mut output = terminal_state.output.lock().await;
                output.append(&chunk, terminal_state.output_limit);
            }
            Err(_) => break,
        }
    }
}

async fn handle_terminal_output(
    state: &VisorClientState,
    req: TerminalOutputRequest,
) -> Result<TerminalOutputResponse> {
    let terminal_state = {
        let terminals = state.terminals.lock().await;
        terminals
            .get(&req.terminal_id)
            .cloned()
            .ok_or_else(|| Error::invalid_params().data("terminal not found"))?
    };

    if terminal_state.exit_status.lock().await.is_none() {
        let mut child = terminal_state.child.lock().await;
        if let Ok(Some(status)) = child.try_wait() {
            let exit_status = exit_status_from_process(status);
            *terminal_state.exit_status.lock().await = Some(exit_status);
        }
    }

    let output_state = terminal_state.output.lock().await;
    let exit_status = terminal_state.exit_status.lock().await.clone();

    Ok(TerminalOutputResponse::new(
        output_state.output.clone(),
        output_state.truncated,
    )
    .exit_status(exit_status))
}

async fn handle_wait_for_exit(
    state: &VisorClientState,
    req: WaitForTerminalExitRequest,
) -> Result<WaitForTerminalExitResponse> {
    let terminal_state = {
        let terminals = state.terminals.lock().await;
        terminals
            .get(&req.terminal_id)
            .cloned()
            .ok_or_else(|| Error::invalid_params().data("terminal not found"))?
    };

    if let Some(exit_status) = terminal_state.exit_status.lock().await.clone() {
        return Ok(WaitForTerminalExitResponse::new(exit_status));
    }

    let mut child = terminal_state.child.lock().await;
    let status = child
        .wait()
        .await
        .map_err(|err| Error::internal_error().data(format!("wait failed: {err}")))?;
    let exit_status = exit_status_from_process(status);
    *terminal_state.exit_status.lock().await = Some(exit_status.clone());

    Ok(WaitForTerminalExitResponse::new(exit_status))
}

async fn handle_kill_terminal(
    state: &VisorClientState,
    req: KillTerminalCommandRequest,
) -> Result<()> {
    let terminal_state = {
        let terminals = state.terminals.lock().await;
        terminals
            .get(&req.terminal_id)
            .cloned()
            .ok_or_else(|| Error::invalid_params().data("terminal not found"))?
    };

    let mut child = terminal_state.child.lock().await;
    let _ = child.kill().await;
    Ok(())
}

async fn handle_release_terminal(
    state: &VisorClientState,
    req: ReleaseTerminalRequest,
) -> Result<()> {
    let terminal_state = {
        let mut terminals = state.terminals.lock().await;
        terminals.remove(&req.terminal_id)
    };
    if let Some(terminal_state) = terminal_state {
        let mut child = terminal_state.child.lock().await;
        let _ = child.kill().await;
    }
    Ok(())
}

fn emit_session_update(state: &VisorClientState, note: SessionNotification) {
    let session_id = note.session_id.to_string();
    match note.update {
        SessionUpdate::AgentMessageChunk(chunk) => {
            if let Some(text) = content_block_to_text(&chunk.content) {
                state.emit_event(AcpUiEvent::ChatMessage { session_id, content: text });
            }
        }
        SessionUpdate::ToolCall(tool_call) => {
            let status = format!("{} ({:?})", tool_call.title, tool_call.status);
            state.emit_event(AcpUiEvent::StatusUpdate { session_id, content: status });
        }
        SessionUpdate::ToolCallUpdate(update) => {
            let status = format!("Tool update: {:?}", update.fields.status);
            state.emit_event(AcpUiEvent::StatusUpdate { session_id, content: status });
        }
        SessionUpdate::Plan(plan) => {
            state.emit_event(AcpUiEvent::StatusUpdate {
                session_id,
                content: format!("Plan received: {} steps", plan.entries.len()),
            });
        }
        _ => {}
    }
}

fn content_block_to_text(content: &ContentBlock) -> Option<String> {
    match content {
        ContentBlock::Text(text) => Some(text.text.clone()),
        ContentBlock::ResourceLink(link) => Some(link.uri.clone()),
        _ => None,
    }
}

fn exit_status_from_process(status: std::process::ExitStatus) -> TerminalExitStatus {
    let mut exit_status = TerminalExitStatus::new();
    exit_status = exit_status.exit_code(status.code().map(|code| code as u32));

    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            exit_status = exit_status.signal(Some(signal.to_string()));
        }
    }

    exit_status
}

pub fn default_client_capabilities() -> ClientCapabilities {
    ClientCapabilities::new().fs(FileSystemCapability::new().read_text_file(true).write_text_file(true)).terminal(true)
}

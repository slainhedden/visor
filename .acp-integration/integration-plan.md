# Implementation Plan: ACP Integration (Service-Client Architecture)

## Objective
Refactor the backend to treat the AI Agent as a service using the **Agent Client Protocol (ACP)**.
Shift from "Terminal Wrapper" to "Client-Server" architecture, where Tauri acts as the ACP Client and the Agent (Claude Code/OpenCode) acts as the Server.

## Architecture Guidelines
* **Pattern:** Service-Client via JSON-RPC over stdio.
* **State Management:** `AcpManager` stored in `AppState` using `tokio::sync::RwLock`.
* **Terminal Strategy:** "Headless" execution. We do not render xterm.js for the agent's internal commands. We spawn background shells, capture output, and feed it back to the agent so it can "see" the result of its tools (e.g., `ls`, `grep`).
* **Runtime:** ACP request handling uses `LocalBoxFuture` (non-Send). Run ACP I/O + command loop inside a dedicated current-thread Tokio runtime on a blocking task; keep Tauri’s main runtime for UI commands.

## Reminders / Open Questions
* Review `agent_client_protocol_schema` types from the crate source for exact request/response shapes.
* Confirm OpenCode binary invocation and required environment variables.
* Decide initial policy for `request_permission` (auto-approve for dev vs explicit UI prompt).
* Decide session storage (in-memory vs persisted on disk).
* Define path sandboxing: restrict to the open project root by default.
* Confirmed: terminal output is requested by the agent (`terminal/output`); the client responds with `TerminalOutputResponse`.

## Phase 1: Foundation & Configuration

### 1.1 Add Dependencies
Add the following to `src-tauri/Cargo.toml`:
```toml
[dependencies]
agent-client-protocol = "0.9" # Check for latest
tokio = { version = "1", features = ["process", "io-std", "macros", "sync", "fs"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1.0"
```

### 1.2 Define Configuration Structure

Create `src-tauri/src/acp/config.rs`:

* Define `AgentConfig` struct (`name`, `command`, `args`, `env`).
* Implement a loader that reads `.acp/agents.json`.
* **Acceptance:** App starts and logs the loaded agent configuration.

### 1.3 Scaffold Manager State

Create `src-tauri/src/acp/manager.rs`:

* Define `AcpManager` struct to hold active connections.
* Integrate into Tauri's `setup` hook to initialize the state.
* **Acceptance:** `AppState` is accessible in Tauri commands.

## Phase 2: The Client Handler (The Brain)

### 2.1 Implement File System Capabilities

Create `src-tauri/src/acp/handler.rs` and define `VisorClient` struct.
Implement `MessageHandler<ClientSide>` trait methods:

* `read_text_file`: Use `tokio::fs`.
* `write_text_file`: Use `tokio::fs`.
* **Security Constraint:** Implement a `validate_path(path)` helper. Reject any path containing `..` or residing outside the project `root_uri`.

### 2.2 Implement Headless Terminal (Critical)

In `VisorClient`, implement `create_terminal`, `terminal_output`, etc.:

* **Spawn:** On `create_terminal`, spawn a `tokio::process::Command` (sh/cmd).
* **Track:** Store the child process in a `HashMap<TerminalId, Child>`.
* **Lifecycle:** One shell per `terminal_id`. Reuse the shell if the same id is requested.
* **Pipe:** Spawn a background task to read the child's `stdout/stderr` line-by-line.
* **Feedback:** `terminal_output` is an Agent->Client request; respond with `TerminalOutputResponse` containing buffered output and exit status.
* **Acceptance:** The agent can successfully run `ls` and receive the file list without a UI terminal.

### 2.3 Implement Permission Policy

* Implement `request_permission`.
* **Policy:** Auto-approve requests for now (Development Mode), but log them.
* **Acceptance:** Agent requests do not hang waiting for user input.

## Phase 3: Connection & Lifecycle

### 3.1 Spawn & Handshake

In `src-tauri/src/acp/manager.rs`:

* Implement `spawn_agent(config, working_dir)`.
* Process setup: `Stdio::piped` for both stdin and stdout.
* **Trait Satisfaction:** Wrap `tauri::async_runtime::spawn` in a closure that matches the `spawn` signature required by `ClientSideConnection::new`.
* **Acceptance:** Agent process starts and stays alive.

### 3.2 Session Initialization

* After connection, call `client.initialize` with `ProtocolVersion::LATEST` and client capabilities.
* Call `client.new_session` with `cwd` set to the open project folder.
* **Acceptance:** Agent returns a successful initialization response and a `session_id`.

## Phase 4: API & Stream Forwarding

### 4.1 Tauri Commands

Expose the following commands to the frontend:

* `acp_start_session(agent_id: String)`: Triggers spawn and init.
* `acp_stop_session()`: Graceful shutdown.
* `acp_send_prompt(text: String)`: Calls `client.prompt()`.

### 4.2 The Event Loop

* In `spawn_agent`, after connection is established, call `client.subscribe()`.
* Spawn a dedicated task to loop over the `StreamReceiver`.
* **Map & Emit:** Convert ACP events into Tauri events (`acp://update`).
* `text` -> Emit `{ type: "chat_message", content: "..." }`
* `tool_use` -> Emit `{ type: "status_update", content: "Running ls..." }`
* `error` -> Emit `{ type: "error", content: "..." }`


## Phase 5: Verification & Safety

### 5.1 Manual Smoke Test - Instruct the user to run this once here

1. **Boot:** Start app with `.acp/agents.json` configured for a simple agent (or echo script).
2. **Connect:** Run `acp_start_session`. Verify "Connected" log.
3. **Prompt:** Send "List files in this directory".
4. **Verify:**
* Agent requests `create_terminal`.
* Backend spawns shell.
* Backend sends `ls` output to agent.
* Agent receives text.
* Frontend receives streaming text response "Here are the files...".

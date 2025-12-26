## Fix menu-driven folder open, gitignore scan, and context shell polish

- Current shell: Full-screen Visor layout with pinned-context sidebar (removable items, copy bundle), codemap canvas (chips from file list), terminal docked bottom with xterm fit + dark theme. No PTY yet.
- Menu: File → Open Folder (Cmd/Ctrl+O) is wired via Tauri menu; app menu holds Quit. Folder picker required adding dialog permission in `src-tauri/capabilities/default.json` (`dialog:default`).
- Folder flow: Uses Tauri dialog to pick a dir, invokes Rust `list_files` (ignore::WalkBuilder) to respect .gitignore/.git exclude; codemap shows chips and file count. Path badge in canvas header.
- Sidebar: smaller, high-contrast context items; Copy Bundle button; items removable.
- Tests: Added Rust unit test `list_files_respects_gitignore` to ensure ignore behavior; run `cd src-tauri && cargo test list_files_respects_gitignore`.
- Deps: JS `@xterm/addon-fit`, `@tauri-apps/plugin-dialog`; Rust `ignore`, `tempfile`. Menu config handled in Rust; window title/size set to Visor.
- Known gaps / next steps:
  - Wire PTY for the terminal (Tauri command spawning a shell, hook xterm).
  - Replace chip cloud with real codemap visualization + selection/pinning flow.
  - Persist open path and pinned context; integrate context bundle export with agents.
- Consider streaming walker/FS watch for live updates; handle large repos (chunked rendering).
- Tighten theming/spacing once data overlays land.

## Add real PTY, resize, and clipboard support

- Terminal now spawns a real shell via portable-pty (default SHELL/COMSPEC), with reader thread emitting `term-data` and writer command `write_to_terminal`.
- PTY resizing works: master stored in session; `resize_terminal` resizes PTY. Frontend `ResizeObserver` fits xterm and sends cols/rows; initial resize sent on mount. `stty size` reflects UI size after resize.
- Clipboard: added `@tauri-apps/plugin-clipboard-manager`; Cmd/Ctrl+V reads clipboard and writes to PTY; Cmd/Ctrl+C copies selection (passes through to shell when no selection). Paste also handled via onPaste fallback. Requires permission in capabilities (added).
- Layout unchanged: resizable sidebar/console via thin handles; codemap chips still placeholder.
- Tests: `cd src-tauri && cargo test` passes. Build via `npm run build`.
- Remaining: richer codemap, PTY lifecycle hardening, persist open path/pins, streaming FS updates.

## Codebase onboarding pass

- Read AGENTS.md, SPEC.md, README.md, and current notes; aligned with plan/code workflow and append-only notes.
- Frontend is Tauri + React: pinned context sidebar, codemap chip canvas, and xterm.js terminal dock with resize handles.
- Backend exposes Tauri commands for file listing (gitignore-aware) and PTY spawn/write/resize via portable-pty.
- Menu uses Tauri menu events for Open Folder and Quit; dialog + clipboard plugins enabled in capabilities.
- Known gaps from notes: richer codemap visualization, persistence of open path/pins, and PTY lifecycle hardening.

## Read ACP docs and UI target request

- Reviewed ACP intro/architecture and crate docs; ACP is JSON-RPC over stdio with streaming notifications and MCP-friendly types.
- Current UI is terminal-first with pinned context sidebar + codemap chip canvas; PTY via portable-pty and menu-driven Open Folder.
- Next step is to ingest target UI reference image and map left chat + right codemap layout.

## UI reference captured and ACP integration planning

- Loaded the new UI reference image (dark, two-pane, left chat + right editor/codemap) and compared to current terminal-first layout.
- Identified needed layout shift to chat-first with right codemap panel and a tabbed subheader (Files/Terminal/Preview) feel.
- Ready to plan ACP-backed agent sessions using the Rust ACP client API with streaming events to the UI; awaiting scope decisions.

## UI overhaul to chat-first layout

- Replaced the terminal-first layout with a chat-first left pane and a right panel with tabbed Codemap/Files/Preview UI.
- Removed xterm/PTY usage from the frontend; Open Folder still populates codemap chips via list_files.
- Added context bundle chips and a composer bar styled to match the new dark theme.
- Updated global styles with dark color scheme and new font stack.
- Verify with `npm run build` and check the new layout visually.

## Remove header extras and seamless layout

- Removed the “Agent workspace”/indexed files subtext and the Connected pill from the chat header.
- Dropped the in-app Open folder button to rely on the menu action only.
- Made the UI edge-to-edge by removing the outer card container and aligning gradients to the window.
- Added `--app-bg` CSS variable for consistent background usage.
- Build check: `npm run build` (succeeds).

## Draft ACP integration plan

- Deep-dived the ACP crate docs and captured core types and responsibilities.
- Created `.acp-integration/integration-plan.md` with architecture, tasks, and acceptance criteria.
- Plan centers on a Rust ACP manager, Tauri commands, and stream events to the chat UI.
- Noted open questions: OpenCode launch details, permission policy, and schema review.
- Next: refine plan with user feedback, then implement stepwise.

## Reviewed updated ACP integration plan

- Reviewed user-updated integration plan; core phases and service-client framing align with ACP usage.
- Flagged clarifications: config path consistency, terminal output flow per ACP schema, and rootUri sourcing from open folder.
- Noted Tauri runtime + tokio usage assumptions and need to confirm schema details from agent_client_protocol_schema.

## ACP scaffolding start (config + state)

- Added ACP dependency hooks and created `.acp/agents.json` with OpenCode placeholder.
- Added ACP config loader, manager/state scaffolding, and stub Tauri commands in `src-tauri/src/acp/`.
- Wired ACP state init + config load in `src-tauri/src/lib.rs` using `tauri::async_runtime::block_on`.
- Updated the integration plan with runtime guidance, config path, and stream mapping.
- Next: implement ACP connection lifecycle, MessageHandler logic, and real session/prompt flow.

## ACP integration core implemented

- Added ACP config in `.acp/agents.json`, config loader, and manager/state wiring with Tauri commands.
- Implemented ACP client handler (permissions, file IO, headless terminal) and session update -> UI event mapping.
- Built a dedicated current-thread Tokio runtime for ACP I/O + command loop to handle non-Send ACP futures.
- Frontend now loads agents, starts/stops ACP session, sends prompts, and renders streamed updates in chat.
- Build checks: `cargo check` in `src-tauri`, `npm run build` at repo root.

## Fix ACP config lookup path

- Default ACP config now checks current dir and parent for `.acp/agents.json` so dev builds from `src-tauri` find repo-root config.
- No behavior change if the config already exists in the current directory.

## Onboarding review of ACP integration and UI state

- Read AGENTS.md, SPEC.md, ACP docs, and `.acp-integration/integration-plan.md`.
- Reviewed ACP backend implementation in `src-tauri/src/acp/` plus Tauri wiring in `src-tauri/src/lib.rs`.
- Reviewed frontend ACP flow in `src/App.tsx` (agent list, session start/stop, prompt send, event stream rendering).
- Confirmed `.acp/agents.json` placeholder for OpenCode and notes about prior UI overhaul.

## Add ACP startup diagnostics + UI error surfacing

- Added safe ACP startup logging (agent id, command/args, cwd, env keys, initialize/new_session results) in `src-tauri/src/acp/manager.rs`.
- UI now shows the ACP start error message in the chat status via `formatInvokeError` in `src/App.tsx`.
- Look for logs prefixed with `ACP` when diagnosing startup failures.

## Start ACP IO task before initialize/new_session

- Spawned ACP `io_task` immediately after `ClientSideConnection::new` to avoid handshake deadlock.
- Removed select loop that awaited `io_task` before handling prompts; prompt loop now runs independently.

## Remove ACP debug logging after successful handshake fix

- Cleaned up temporary ACP startup `eprintln!` logs from `src-tauri/src/acp/manager.rs`.
- Kept UI error surfacing in `src/App.tsx` for visibility when start failures occur.

## ACP feature pass: modes, MCP, permissions, config normalization, UI controls

- Agent config now requires versioned map schema with `settings` and `mcp_servers`; legacy list support removed (`src-tauri/src/acp/config.rs`). Updated `.acp/agents.json` accordingly.
- Manager passes MCP stdio servers into `new_session`, captures modes, returns initial mode state, and exposes `acp_set_mode` (see `src-tauri/src/acp/manager.rs`).
- Permission flow is interactive: handler emits `acp://permission_request`, waits for `acp_resolve_permission`, and updates mode changes via session notifications (`src-tauri/src/acp/handler.rs`).
- Frontend adds a mode selector dropdown and permission modal; start/stop resets mode state; invokes use snake_case args; status/error surfacing remains (`src/App.tsx`).

## Onboarding pass + ACP integration review (current state)

- Read `AGENTS.md`, `SPEC.md`, `README.md`, ACP docs, and `.acp-integration` plans; aligned on context-engineering product goal and ACP service-client architecture.
- Reviewed ACP backend in `src-tauri/src/acp/` (config loader, manager/session runtime, handler for FS/terminal/permissions, event mapping) and Tauri wiring in `src-tauri/src/lib.rs`.
- Reviewed frontend ACP flow in `src/App.tsx` (agent list, session start/stop, prompt send, mode select, permission modal, event stream rendering).
- Noted `.acp/agents.json` OpenCode placeholder and config version 1 schema with MCP servers.
- Outstanding: ACP integration still reported buggy; next pass should focus on session lifecycle/stdio I/O, permission flow edge cases, and UI event consistency.

## OpenCode info + integration strategy framing

- Looked up SST OpenCode: MIT-licensed "open source coding agent" with client/server architecture and built-in modes (plan/build).
- OpenCode supports ACP via `opencode acp` (stdio JSON-RPC), with editor configs documented; ACP is intended for IDE <-> agent interoperability.
- Decision framing: fork IDE vs plugin-first vs standalone + ACP; plugin-first aligns with "codemap algorithm as a plugin" goal and reduces surface area.
- If narrowing scope, consider OpenCode-only ACP support initially, with ACP as optional adapter later.

## Direction discussion: OpenCode-only terminal integration (no ACP)

- Considering simplifying agent layer to an embedded terminal running OpenCode only, dropping ACP/multi-agent support for now.
- This reduces complexity but loses structured events (modes, permissions, tool status) and requires treating agent output as plain terminal text.
- If chosen, plan is to restore in-app terminal (PTY) and launch `opencode` directly, while keeping codemap/context bundle as the primary value.

## Direction: sidekick tool (no ACP/agent embedding)

- Agreed direction: build a helper tool that runs alongside any coding agent (separate terminals), focusing on codemap + context workflow.
- This avoids ACP and agent embedding; tool is agnostic and can be used with OpenCode/Claude Code/etc.
- Next decision points: CLI vs GUI, how context bundles are generated/exported, and how repo indexing is triggered.

## Plan doc for manual-first codemap + context.md

- Moved original spec to `.project-plan/codemap-spec.md` for archival reference.
- Added `.project-plan/long-term-plan.md` detailing phased roadmap: manual codemap, cached context.md deltas, and agent-optimized 1-2 file artifact.
- Captured codemap algorithm direction (dependency-aware graph) and open questions.

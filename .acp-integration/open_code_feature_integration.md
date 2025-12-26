# OpenCode Feature Integration Plan (ACP "Zed-Style" Client)

## Goals
- Evolve Visor’s ACP client to match Zed-style capabilities for OpenCode and future agents.
- Support session modes (Plan/Build), MCP server pass-through, modular agent configuration, and human-in-the-loop permissions.
- Maintain backward compatibility with the existing `.acp/agents.json` while migrating to a richer schema.

## Architecture Parity (Zed Model)
- Config layer: `.acp/agents.json` + `src-tauri/src/acp/config.rs`.
- State layer: `src-tauri/src/acp/manager.rs` (session lifecycle, mode state, MCP handoff).
- Presentation layer: `src/App.tsx` (or extracted `AgentSession` component) for chat, modes, permissions UI.

## Configuration Schema (Versioned map, future-proof)
- Schema (required):
  ```json
  {
    "version": 1,
    "agents": {
      "opencode": {
        "command": "opencode",
        "args": ["acp"],
        "env": { "OPENAI_API_KEY": "..." },
        "settings": { "model": "claude-3-5-sonnet" },
        "mcp_servers": {
          "git": { "command": "git-mcp", "args": [] }
        }
      }
    }
  }
  ```
- Legacy schema: no longer supported (migrate to versioned map).
- Validation: require `version == 1` and `command`; optional `args/env/settings/mcp_servers`. Warn (not fail) on unknown keys.
- Reminder: when uncertain about ACP/MCP fields, use the Exa tool to search the ACP schema docs.

## Session Modes (Plan vs Build)
- Capture modes:
  - `initialize` → store `agent_capabilities` (for future gating).
  - `new_session` → store `SessionModeState { current_mode_id, available_modes }` if present.
- Tauri command: `acp_set_mode(agent_id: String, mode_id: String)`:
  - Checks active session; calls `set_session_mode(SetSessionModeRequest)`.
  - Emits `acp://mode_changed` with `{ session_id, current_mode_id }` on success.
- Notifications:
  - Handle `SessionUpdate::CurrentModeUpdate` to sync UI (agent-initiated changes).
- Frontend:
  - Mode selector near composer; default first mode or “Auto”.
  - Visual safety indicator (e.g., blue for Plan, orange/red for Build).
  - Disable/guard file ops in UI when in Plan (future).
- Tests:
  - Start session, switch mode via dropdown → observe `mode_changed`.
  - Agent-driven mode change → UI updates automatically.

## MCP Bridge
- Config: per-agent `mcp_servers` map; support at least stdio-type servers (command + args + env).
- `NewSessionRequest`:
  - Populate `cwd` from open path.
  - Populate `mcp_servers` from config; empty vec if none.
  - Include even if agent capabilities aren’t advertised (fail soft; agent may ignore).
- Potential future: honor `InitializeResponse.agent_capabilities` to gate MCP transport types.
- Tests:
  - Add dummy MCP server in config (echo); start session; verify agent attempts to spawn/connect (stdout/err logs).

## Permissions (Human in the Loop)
- Backend:
  - Replace auto-approve `handle_permission` with a request/response flow.
  - Generate `request_id`; emit `acp://permission_request` with `{ id, message, options }`.
  - Store pending requests in a map; await a oneshot.
  - New Tauri command `acp_resolve_permission(request_id: String, option_id: String | deny)` to respond.
- Frontend:
  - Toast/modal showing the request text and options (Allow once/always/deny if provided).
  - Block composer only if desired; otherwise allow non-blocking toast.
  - Emit decision via `invoke acp_resolve_permission`.
- Tests:
  - Agent tool call requiring permission hangs until approve; deny propagates failure.

## Agent Config and Launch
- Use normalized config for `spawn_session`: command, args, env.
- Keep `cwd` as project root (open folder).
- Preserve UI error surfacing for spawn/init failures.
- Future: surface `agent.settings` to UI for model selection.

## Frontend Integration
- Extract ACP UI into `AgentSession` component (optional refactor).
- Events to handle: `acp://update` (chat/status), `acp://mode_changed`, `acp://permission_request`.
- State to store: session_id, modes, current_mode_id, permission requests (map).
- Mode selector and permission modal UX to be dark-theme consistent with current UI.
- Provider-scalable controls:
  - Agent picker dropdown sourced from config (already present); ensure it refreshes after config reload to support new providers.
  - Mode selector button/dropdown near composer; color-coded for safety.
  - Permission prompt button set (Allow once / Allow always / Deny) in modal or toast.
  - Optional MCP status pill (per provider) showing connected/failed if we surface MCP server states later.
  - Keep layouts flexible to add future providers (list grows from config).

## Testing / Verification
- Mode switch test: start OpenCode, switch modes, see `mode_changed`, and current_mode_id updates.
- MCP smoke: add dummy MCP server; ask agent to use it; verify spawned process.
- Permission block: request file write; UI prompt appears; approve continues; deny aborts.
- Regression: session start/stop, prompt send, chat streaming still work.

## Implementation Steps (Dev Checklist)
1) Config normalization in `src-tauri/src/acp/config.rs`: support new + legacy schema; expose MCP server info.
2) Manager state updates in `src-tauri/src/acp/manager.rs`:
   - Store modes/current_mode_id per session.
   - Pass `mcp_servers` into `NewSessionRequest`.
   - Add `acp_set_mode` command; emit mode change event.
3) Permission flow in `src-tauri/src/acp/handler.rs`:
   - Replace auto-approve; add pending map + oneshot; emit/resolve events.
   - Add `acp_resolve_permission` command.
4) Frontend (`src/App.tsx` or `AgentSession.tsx`):
   - Mode dropdown + event handling.
   - Permission toast/modal + resolve wiring.
   - Keep error surfacing for start failures.
5) Tests/manual checks per above.

## Reminders
- When unsure about ACP types or fields (modes, MCP servers, permissions), search with the Exa tool against ACP docs.
- Keep backward compatibility; don’t break existing `.acp/agents.json`.
- Avoid logging secrets; only log env keys when needed.

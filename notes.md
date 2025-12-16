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

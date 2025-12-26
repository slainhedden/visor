# Long-Term Plan: Codemap + context.md workflow

## Purpose
Define the staged roadmap for a manual-first codemap tool that generates a clean `context.md`, then evolves into a smarter, agent-optimized context layer.

## Core principles
- Manual-first: users control the initial context selection and understand why each file is included.
- Deterministic output: same repo snapshot + selection => same `context.md`.
- Agent-first delivery: keep context minimal, avoid repetition, and minimize token waste.
- Interpretability: GUI explains dependencies and selections, even if the agent consumes a compact representation.

## Phase 1: Manual codemap + basic context.md

### Goal
Provide a clear codemap view of the current repo and generate a clean `context.md` from selected files.

### Scope
- Index the repo where the command is run.
- Build a codemap visualization that lists files and shows basic structure.
- Allow users to select and pin files.
- Generate `context.md` from pinned files with a clear, stable format.

### Acceptance criteria
- Codemap loads quickly for a medium-sized repo.
- File selection is explicit and visible to the user.
- `context.md` is clean, readable, and stable for the same selection.

## Phase 2: Smarter context.md with caching

### Goal
Avoid resending unchanged file content to the agent by tracking what has already been shared.

### Approach
- Maintain a cache of file digests that the agent has already seen (per session or per project).
- On each context generation, include only:
  - New files not previously sent.
  - Files with content changes since the last send.
- If a file was sent and unchanged, omit it from the next `context.md`.

### Open decisions
- Cache scope: per session vs per project vs per agent.
- Storage: on-disk cache (recommended) vs in-memory.
- Invalidation: when to reset cache (branch change, repo root change, manual reset).

### Acceptance criteria
- Context size shrinks after the first send without losing changes.
- The system correctly re-includes a file after modifications.
- Users can force-refresh the full context if needed.

## Phase 3: Agent-optimized representation

### Goal
Collapse the agent-facing context into 1-2 canonical files, while the GUI preserves a human-friendly map.

### Approach
- Generate a compact agent context artifact from selected files and cached state.
- Keep human UI as a structured codemap with drill-down views.
- Track dependency context to justify inclusion in the compact representation.

### Risks
- Over-compression can hide important details.
- Users may need a way to inspect the agent artifact for trust.

### Acceptance criteria
- Agent artifact stays under a defined size threshold for most repos.
- GUI can explain where each snippet originates.

## Codemap algorithm direction

### Long-term target
A dependency-aware codemap that exposes relationships between files, functions, and modules.

### Signals to consider
- File-level dependencies (imports, require/using statements).
- Function-level call graph (static analysis where feasible).
- File ownership boundaries (folders, modules, packages).
- Optional: git history or runtime traces (future).

### Open questions
- Language scope: start with one or two languages or language-agnostic heuristics.
- Precision vs speed: partial graph extraction vs full call graph.
- UI representation: file graph, module graph, or layered views.

## context.md format (initial direction)

### Required sections
- Project header (name, root path, timestamp, version).
- Selected files list (paths + short descriptions).
- File contents (verbatim or summarized in later phases).
- Optional: notes or user annotations.

### Versioning
- Add a format version header, e.g. `Context-Format: 1` to allow changes later.

## Milestones summary

1. Manual codemap + context.md generator.
2. Cached context to avoid resending unchanged files.
3. Agent-optimized 1-2 file artifact with full interpretability in UI.
4. Dependency-aware codemap algorithm with richer graphing.


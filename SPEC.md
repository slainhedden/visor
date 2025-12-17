# High Level Idea

ContextMap is a desktop GUI that makes context engineering for coding agents fast, visual, and repeatable. It pairs a terminal-first agent workflow with an interactive code map so users can understand the codebase structure and select the right files without guessing. Instead of manually hunting for relevant files, the user clicks through the map and pins files into a context set that can be injected into the next agent request. The UI is designed to keep the “agent loop” tight: see what matters, select it, run the agent, and understand what changed. The product is agent-agnostic at first, focusing on being the best possible context layer on top of existing agent CLIs. Over time it can evolve from “context UI for any agent” into a deeper, more integrated agent experience.

# Detailed parts

* **Terminal-first MVP:** left pane is an integrated terminal with a clean theme; initial workflow assumes manual context injection, with provider-specific shortcuts/integrations (Codex, Claude Code, Opencode, etc.) added later; long-term path can move toward a native agent platform.
* **CodeMap-first UX:** right pane is a well-designed codemap that stays smooth and readable; users can quickly select and pin files; the UI can highlight where the agent is currently operating (based on observed activity where available), and after a run it highlights which files changed.
* **Context Set as a first-class object:** users build a pinned “context set” and export it as a bundle suitable for pasting or referencing in terminal prompts; context sets can be reused across multiple iterations.
* **Fast iteration loop:** after each run, keep a clear record of what changed and make it easy to expand/refine context for the next run (without dumping the whole repo).
* **Agent-agnostic by default:** the value is the context layer and workflow, not the underlying model choice; integrations should be optional enhancements rather than requirements.

Any final notes

* The main risk to avoid is promising “true dataflow.” For v0, the map should focus on being useful and navigable; deeper semantic flow can be layered later.
* Treat responsiveness as a product requirement: indexing/caching should be incremental so the UI stays instant after the first scan.
* Make the context bundle format stable from day one so you can add integrations without breaking workflows.


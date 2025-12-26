import { useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

type PinnedItem = {
  id: string;
  label: string;
  path: string;
};

type AgentSummary = {
  id: string;
  label: string;
};

type SessionInfo = {
  agent_id: string;
  session_id: string;
  modes?: {
    current_mode_id: string;
    available_modes: SessionMode[];
  };
};

type ChatEntry = {
  id: string;
  role: "user" | "assistant" | "status";
  content: string;
  streaming?: boolean;
};

type AcpUpdateEvent = {
  type: "chat_message" | "status_update" | "error" | "mode_changed" | "permission_request";
  session_id: string;
  content?: string;
  current_mode_id?: string;
  request_id?: string;
  message?: string;
  options?: PermissionOption[];
};

type SessionMode = {
  id: string;
  name: string;
  description?: string | null;
};

type PermissionOption = {
  option_id: string;
  label: string;
};

type PermissionRequest = {
  id: string;
  sessionId: string;
  message: string;
  options: PermissionOption[];
};

const initialPinned: PinnedItem[] = [
  { id: "1", label: "core/app.ts", path: "core/app.ts" },
  { id: "2", label: "ui/map/canvas.tsx", path: "ui/map/canvas.tsx" },
  { id: "3", label: "agents/context_bundle.md", path: "agents/context_bundle.md" },
];

const tabs = [
  { id: "codemap", label: "Codemap" },
  { id: "files", label: "Files" },
  { id: "preview", label: "Preview" },
] as const;

type TabId = (typeof tabs)[number]["id"];

function createId(prefix: string) {
  return `${prefix}-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

function formatInvokeError(err: unknown) {
  if (typeof err === "string") return err;
  if (err && typeof err === "object") {
    if ("message" in err && typeof err.message === "string") {
      return err.message;
    }
    try {
      return JSON.stringify(err);
    } catch {
      return "unknown error";
    }
  }
  return String(err);
}

function App() {
  const [openPath, setOpenPath] = useState<string | null>(null);
  const [pinnedItems, setPinnedItems] =
    useState<PinnedItem[]>(initialPinned);
  const [copyState, setCopyState] = useState<"idle" | "copied" | "error">(
    "idle",
  );
  const [codemapFiles, setCodemapFiles] = useState<string[]>([]);
  const [activeTab, setActiveTab] = useState<TabId>("codemap");
  const [agents, setAgents] = useState<AgentSummary[]>([]);
  const [selectedAgentId, setSelectedAgentId] = useState<string | null>(null);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [availableModes, setAvailableModes] = useState<SessionMode[]>([]);
  const [currentModeId, setCurrentModeId] = useState<string | null>(null);
  const [sessionStatus, setSessionStatus] = useState<
    "idle" | "starting" | "active" | "error"
  >("idle");
  const [chatEntries, setChatEntries] = useState<ChatEntry[]>([]);
  const [composerText, setComposerText] = useState("");
  const [pendingPermissions, setPendingPermissions] = useState<PermissionRequest[]>([]);

  useEffect(() => {
    const unlistenPromise = listen("menu://open-folder", () => {
      handleOpenFolder();
    });
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  useEffect(() => {
    void invoke<AgentSummary[]>("acp_list_agents")
      .then((list) => {
        setAgents(list);
        if (!selectedAgentId && list.length > 0) {
          setSelectedAgentId(list[0].id);
        }
      })
      .catch((err) => console.error("Failed to load agents", err));
  }, []);

  useEffect(() => {
    const unlistenPromise = listen<AcpUpdateEvent>("acp://update", (event) => {
      const payload = event.payload;
      if (payload.type === "chat_message") {
        if (payload.content) {
          appendAssistantChunk(payload.content);
        }
      } else if (payload.type === "status_update") {
        if (payload.content) {
          appendStatus(payload.content);
        }
      } else if (payload.type === "error") {
        appendStatus(`Error: ${payload.content ?? "unknown error"}`);
      } else if (payload.type === "mode_changed" && payload.current_mode_id) {
        setCurrentModeId(payload.current_mode_id);
      } else if (payload.type === "permission_request" && payload.request_id && payload.message && payload.options) {
        setPendingPermissions((prev) => [
          ...prev,
          {
            id: payload.request_id!,
            sessionId: payload.session_id,
            message: payload.message!,
            options: payload.options!,
          },
        ]);
      }
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  const stats = useMemo(
    () => [
      { label: "Session", value: sessionStatus },
      { label: "Agent", value: selectedAgentId ?? "none" },
      { label: "Mode", value: currentModeId ?? "default" },
    ],
    [selectedAgentId, sessionStatus, currentModeId],
  );

  const truncatePath = (value: string | null, max = 48) => {
    if (!value) return "No folder selected";
    return value.length > max ? `…${value.slice(value.length - max)}` : value;
  };

  const appendAssistantChunk = (chunk: string) => {
    setChatEntries((prev) => {
      const last = prev[prev.length - 1];
      if (last && last.role === "assistant" && last.streaming) {
        return [
          ...prev.slice(0, -1),
          { ...last, content: `${last.content}${chunk}` },
        ];
      }
      return [
        ...prev,
        {
          id: createId("assistant"),
          role: "assistant",
          content: chunk,
          streaming: true,
        },
      ];
    });
  };

  const appendStatus = (message: string) => {
    setChatEntries((prev) => [
      ...prev,
      { id: createId("status"), role: "status", content: message },
    ]);
  };

  const handleOpenFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select project folder",
      });
      if (!selected) return;
      const path = Array.isArray(selected) ? selected[0] : selected;
      setOpenPath(path);
      const result = await invoke<string[]>("list_files", { path });
      setCodemapFiles(result);
      appendStatus("Folder loaded. Ready to start ACP session.");
    } catch (err) {
      console.error("Failed to open folder", err);
    }
  };

  const handleCopyBundle = async () => {
    if (!pinnedItems.length) {
      setCopyState("error");
      return;
    }
    try {
      const bundle = [
        "Context Bundle:",
        ...pinnedItems.map((item) => `- ${item.path}`),
      ].join("\n");
      await navigator.clipboard.writeText(bundle);
      setCopyState("copied");
      setTimeout(() => setCopyState("idle"), 1500);
    } catch (err) {
      console.error("Copy failed", err);
      setCopyState("error");
      setTimeout(() => setCopyState("idle"), 1500);
    }
  };

  const handleRemovePinned = (id: string) => {
    setPinnedItems((prev) => prev.filter((item) => item.id !== id));
  };

  const handleStartSession = async () => {
    if (!selectedAgentId) {
      appendStatus("Select an agent before starting a session.");
      return;
    }
    if (!openPath) {
      appendStatus("Select a folder before starting a session.");
      return;
    }
    setSessionStatus("starting");
    try {
      const info = await invoke<SessionInfo>("acp_start_session", {
        // NOTE: keys must be camelCase here because the Tauri command expects agentId/rootDir
        agentId: selectedAgentId,
        rootDir: openPath,
      });
      setSessionId(info.session_id);
      if (info.modes) {
        setAvailableModes(info.modes.available_modes);
        setCurrentModeId(info.modes.current_mode_id);
      } else {
        setAvailableModes([]);
        setCurrentModeId(null);
      }
      setSessionStatus("active");
      appendStatus(`Session started (${info.agent_id}).`);
    } catch (err) {
      console.error("Failed to start ACP session", err);
      setSessionStatus("error");
      appendStatus(`Failed to start ACP session: ${formatInvokeError(err)}`);
    }
  };

  const handleStopSession = async () => {
    try {
      await invoke("acp_stop_session");
      setSessionId(null);
      setSessionStatus("idle");
      setAvailableModes([]);
      setCurrentModeId(null);
      setPendingPermissions([]);
      appendStatus("Session stopped.");
    } catch (err) {
      console.error("Failed to stop session", err);
      appendStatus("Failed to stop session.");
    }
  };

  const handleSendPrompt = async () => {
    if (!composerText.trim()) return;
    if (!sessionId) {
      appendStatus("Start a session before sending prompts.");
      return;
    }

    const text = composerText.trim();
    setChatEntries((prev) => [
      ...prev,
      { id: createId("user"), role: "user", content: text },
    ]);
    setComposerText("");

    try {
      await invoke("acp_send_prompt", { text });
    } catch (err) {
      console.error("Failed to send prompt", err);
      appendStatus("Prompt failed to send.");
    }
  };

  const handleModeSelect = async (modeId: string) => {
    if (!sessionId) return;
    setCurrentModeId(modeId);
    try {
      await invoke("acp_set_mode", { mode_id: modeId });
      appendStatus(`Mode set to ${modeId}.`);
    } catch (err) {
      console.error("Failed to set mode", err);
      appendStatus(`Failed to set mode: ${formatInvokeError(err)}`);
    }
  };

  const handleResolvePermission = async (requestId: string, optionId: string | null) => {
    setPendingPermissions((prev) => prev.filter((req) => req.id !== requestId));
    try {
      await invoke("acp_resolve_permission", { request_id: requestId, option_id: optionId });
      appendStatus(optionId ? `Permission granted (${optionId}).` : "Permission denied.");
    } catch (err) {
      console.error("Failed to resolve permission", err);
      appendStatus(`Permission resolution failed: ${formatInvokeError(err)}`);
    }
  };

  return (
    <div className="h-screen w-screen bg-[var(--app-bg)] text-slate-100">
      <div className="relative h-full w-full bg-[radial-gradient(circle_at_top,rgba(59,130,246,0.08),transparent_55%),radial-gradient(circle_at_bottom,rgba(20,184,166,0.08),transparent_50%)]">
        <div className="absolute inset-0 bg-gradient-to-br from-slate-900/90 via-slate-900/80 to-slate-950/90" />
        <div className="relative z-10 flex h-full gap-4 p-4">
          <section className="flex w-[55%] flex-col rounded-2xl border border-white/10 bg-slate-900/40 p-4">
            <header className="flex flex-wrap items-center justify-between gap-3">
              <div>
                <p className="text-sm font-semibold text-white">Visor Session</p>
              </div>
              <div className="flex items-center gap-2">
                <select
                  value={selectedAgentId ?? ""}
                  onChange={(event) => setSelectedAgentId(event.target.value)}
                  className="rounded-full border border-white/10 bg-slate-900/60 px-3 py-1 text-[11px] text-slate-200 focus:border-emerald-400/50 focus:outline-none"
                >
                  <option value="" disabled>
                    Select agent
                  </option>
                  {agents.map((agent) => (
                    <option key={agent.id} value={agent.id}>
                      {agent.label}
                    </option>
                  ))}
                </select>
                {sessionStatus === "active" ? (
                  <button
                    onClick={handleStopSession}
                    className="rounded-full border border-white/10 bg-white/5 px-3 py-1 text-[11px] text-slate-200 hover:bg-white/10"
                  >
                    Stop
                  </button>
                ) : (
                  <button
                    onClick={handleStartSession}
                    className="rounded-full border border-emerald-400/40 bg-emerald-500/20 px-3 py-1 text-[11px] font-semibold text-emerald-100 hover:bg-emerald-500/30"
                  >
                    {sessionStatus === "starting" ? "Starting" : "Start"}
                  </button>
                )}
              </div>
            </header>

            <div className="mt-4 flex items-center gap-2 text-[11px] text-slate-400">
              {stats.map((stat) => (
                <span
                  key={stat.label}
                  className="rounded-full border border-white/10 bg-white/5 px-2 py-0.5"
                >
                  {stat.label}: {stat.value}
                </span>
              ))}
            </div>

            {availableModes.length > 0 && (
              <div className="mt-3 flex items-center gap-2 text-[11px] text-slate-300">
                <span className="rounded-full border border-white/10 bg-white/5 px-2 py-1">
                  Mode
                </span>
                <select
                  value={currentModeId ?? ""}
                  onChange={(event) => handleModeSelect(event.target.value)}
                  className="rounded-full border border-white/10 bg-slate-900/60 px-3 py-1 text-[11px] text-slate-200 focus:border-emerald-400/50 focus:outline-none"
                >
                  {availableModes.map((mode) => (
                    <option key={mode.id} value={mode.id}>
                      {mode.name}
                    </option>
                  ))}
                </select>
              </div>
            )}

            <div className="mt-5 flex-1 space-y-4 overflow-y-auto pr-1">
              {chatEntries.length === 0 ? (
                <div className="rounded-2xl border border-white/10 bg-slate-900/40 px-4 py-6 text-center text-[13px] text-slate-300">
                  Start an ACP session to begin chatting with your agent.
                </div>
              ) : (
                chatEntries.map((entry) => {
                  if (entry.role === "user") {
                    return (
                      <div key={entry.id} className="flex justify-end">
                        <div className="max-w-[78%] rounded-2xl rounded-tr-sm border border-white/10 bg-gradient-to-br from-slate-700/70 to-slate-800/70 px-4 py-3 text-sm text-slate-100">
                          {entry.content}
                        </div>
                      </div>
                    );
                  }
                  if (entry.role === "assistant") {
                    return (
                      <div key={entry.id} className="flex justify-start">
                        <div className="max-w-[80%] rounded-2xl rounded-tl-sm border border-white/10 bg-slate-900/50 px-4 py-3 text-sm text-slate-200">
                          {entry.content}
                        </div>
                      </div>
                    );
                  }
                  return (
                    <div
                      key={entry.id}
                      className="rounded-xl border border-white/10 bg-white/5 px-4 py-3 text-[12px] text-slate-300"
                    >
                      {entry.content}
                    </div>
                  );
                })
              )}
            </div>

            <div className="mt-4 rounded-xl border border-white/10 bg-slate-950/40 p-3">
              <div className="flex flex-wrap items-center gap-2">
                <span className="text-[11px] uppercase tracking-[0.2em] text-slate-500">
                  Context
                </span>
                {pinnedItems.map((item) => (
                  <span
                    key={item.id}
                    className="flex items-center gap-1 rounded-full border border-white/10 bg-white/5 px-3 py-1 text-[11px] text-slate-200"
                  >
                    <span>{item.label}</span>
                    <button
                      type="button"
                      onClick={() => handleRemovePinned(item.id)}
                      className="ml-1 rounded-full bg-white/10 px-2 py-0.5 text-[11px] text-slate-200 hover:bg-white/20"
                      aria-label={`Remove ${item.label}`}
                    >
                      ×
                    </button>
                  </span>
                ))}
                <button
                  onClick={handleCopyBundle}
                  className="ml-auto rounded-full border border-emerald-400/40 bg-emerald-500/20 px-3 py-1 text-[11px] font-semibold text-emerald-100 hover:bg-emerald-500/30"
                >
                  Copy bundle
                </button>
              </div>
              {copyState === "copied" && (
                <p className="mt-2 text-[11px] text-emerald-200">
                  Bundle copied.
                </p>
              )}
              {copyState === "error" && (
                <p className="mt-2 text-[11px] text-rose-200">
                  Nothing to copy yet.
                </p>
              )}
            </div>

            <div className="mt-3 rounded-2xl border border-white/10 bg-slate-900/60 p-3">
              <div className="flex items-center gap-2 text-[11px] text-slate-400">
                <span className="rounded-full border border-white/10 bg-white/5 px-2 py-1">
                  Attach
                </span>
                <span className="rounded-full border border-white/10 bg-white/5 px-2 py-1">
                  Context
                </span>
                <span className="rounded-full border border-white/10 bg-white/5 px-2 py-1">
                  Tools
                </span>
                <span className="rounded-full border border-white/10 bg-white/5 px-2 py-1">
                  Settings
                </span>
              </div>
              <div className="mt-3 flex items-center gap-3">
                <input
                  type="text"
                  value={composerText}
                  onChange={(event) => setComposerText(event.target.value)}
                  onKeyDown={(event) => {
                    if (event.key === "Enter" && !event.shiftKey) {
                      event.preventDefault();
                      void handleSendPrompt();
                    }
                  }}
                  placeholder="Ask Visor to refine the workspace..."
                  className="w-full rounded-xl border border-white/10 bg-slate-950/60 px-3 py-2 text-sm text-slate-100 placeholder:text-slate-500 focus:border-emerald-400/50 focus:outline-none"
                />
                <button
                  onClick={() => void handleSendPrompt()}
                  className="rounded-xl bg-emerald-500 px-4 py-2 text-sm font-semibold text-slate-950 hover:bg-emerald-400"
                >
                  Send
                </button>
              </div>
            </div>
          </section>

          <section className="flex w-[45%] flex-col rounded-2xl border border-white/10 bg-slate-900/50">
            <div className="flex items-center justify-between border-b border-white/10 px-4 py-3">
              <div className="flex items-center gap-2 text-[12px] text-slate-300">
                {tabs.map((tab) => (
                  <button
                    key={tab.id}
                    onClick={() => setActiveTab(tab.id)}
                    className={`rounded-full px-3 py-1 text-[12px] font-semibold transition ${
                      activeTab === tab.id
                        ? "bg-white/10 text-white"
                        : "text-slate-400 hover:text-slate-200"
                    }`}
                  >
                    {tab.label}
                  </button>
                ))}
              </div>
            </div>

            <div className="flex-1 overflow-hidden">
              {activeTab === "codemap" && (
                <div className="relative h-full">
                  <div className="absolute inset-0 bg-[radial-gradient(rgba(148,163,184,0.18)_1px,transparent_0)] [background-size:26px_26px]" />
                  <div className="absolute inset-0 bg-gradient-to-b from-slate-950/10 via-transparent to-slate-950/30" />
                  <div className="relative z-10 flex h-full flex-col">
                    <div className="flex items-center justify-between px-4 py-3 text-[11px] text-slate-300">
                      <span className="rounded-full border border-white/10 bg-white/5 px-3 py-1">
                        {truncatePath(openPath)}
                      </span>
                      <span className="rounded-full border border-white/10 bg-white/5 px-3 py-1">
                        Files: {codemapFiles.length}
                      </span>
                    </div>
                    <div className="flex-1 overflow-y-auto px-4 pb-4">
                      {codemapFiles.length === 0 ? (
                        <div className="mt-10 rounded-2xl border border-white/10 bg-slate-900/60 p-6 text-center text-[13px] text-slate-300">
                          Load a folder to see the codemap layout. Nodes will
                          appear here for quick selection and pinning.
                        </div>
                      ) : (
                        <div className="flex flex-wrap gap-2">
                          {codemapFiles.slice(0, 220).map((file) => (
                            <span
                              key={file}
                              className="rounded-full border border-white/10 bg-white/5 px-3 py-1 text-[11px] text-slate-200"
                            >
                              {file}
                            </span>
                          ))}
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              )}

              {activeTab !== "codemap" && (
                <div className="flex h-full items-center justify-center text-sm text-slate-400">
                  {activeTab === "files"
                    ? "File explorer is coming soon."
                    : "Preview is coming soon."}
                </div>
              )}
            </div>
          </section>
        </div>
      </div>

      {pendingPermissions.length > 0 && (
        <div className="fixed inset-0 z-20 flex items-center justify-center bg-slate-950/70 backdrop-blur">
          <div className="w-[420px] rounded-2xl border border-white/10 bg-slate-900/90 p-6 shadow-xl">
            {pendingPermissions.map((request) => (
              <div key={request.id} className="space-y-4">
                <div className="text-sm font-semibold text-white">
                  Permission required
                </div>
                <div className="text-sm text-slate-300">{request.message}</div>
                <div className="flex flex-wrap gap-2">
                  {request.options.map((option) => (
                    <button
                      key={option.option_id}
                      onClick={() => void handleResolvePermission(request.id, option.option_id)}
                      className="rounded-full border border-emerald-400/40 bg-emerald-500/20 px-3 py-1 text-[12px] font-semibold text-emerald-100 hover:bg-emerald-500/30"
                    >
                      {option.label}
                    </button>
                  ))}
                  <button
                    onClick={() => void handleResolvePermission(request.id, null)}
                    className="rounded-full border border-white/10 bg-white/10 px-3 py-1 text-[12px] text-slate-200 hover:bg-white/20"
                  >
                    Deny
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

export default App;

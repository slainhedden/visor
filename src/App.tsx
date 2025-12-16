import { useEffect, useMemo, useRef, useState } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { readText, writeText } from "@tauri-apps/plugin-clipboard-manager";
import "@xterm/xterm/css/xterm.css";

type PinnedItem = {
  id: string;
  label: string;
  path: string;
};

const initialPinned: PinnedItem[] = [
  { id: "1", label: "core/app.ts", path: "core/app.ts" },
  { id: "2", label: "ui/map/canvas.tsx", path: "ui/map/canvas.tsx" },
  { id: "3", label: "agents/context_bundle.md", path: "agents/context_bundle.md" },
];

function ContextItem({
  item,
  onRemove,
}: {
  item: PinnedItem;
  onRemove: (id: string) => void;
}) {
  return (
    <div className="flex items-center justify-between rounded-md bg-white/20 px-2 py-1 text-xs text-slate-900 ring-1 ring-white/30">
      <span className="truncate text-[11px] font-medium text-white">
        {item.label}
      </span>
      <button
        onClick={() => onRemove(item.id)}
        className="ml-2 rounded-full bg-slate-900/70 px-2 py-0.5 text-[10px] font-semibold text-white hover:bg-slate-900/80 active:bg-slate-900"
        aria-label={`Remove ${item.label}`}
      >
        ×
      </button>
    </div>
  );
}

function App() {
  const terminalRef = useRef<HTMLDivElement | null>(null);
  const terminalContainerRef = useRef<HTMLDivElement | null>(null);
  const fitAddon = useMemo(() => new FitAddon(), []);
  const [openPath, setOpenPath] = useState<string | null>(null);
  const [sidebarWidth, setSidebarWidth] = useState<number>(260);
  const [consoleHeight, setConsoleHeight] = useState<number>(240);
  const [isResizingSidebar, setIsResizingSidebar] = useState(false);
  const [isResizingConsole, setIsResizingConsole] = useState(false);
  const [pinnedItems, setPinnedItems] = useState<PinnedItem[]>(initialPinned);
  const [copyState, setCopyState] = useState<"idle" | "copied" | "error">(
    "idle",
  );
  const [codemapFiles, setCodemapFiles] = useState<string[]>([]);
  const termInstanceRef = useRef<Terminal | null>(null);
  const termDataUnlistenRef = useRef<UnlistenFn | null>(null);
  const onDataDisposableRef = useRef<{ dispose: () => void } | null>(null);
  const spawnedRef = useRef(false);
  const lastSizeRef = useRef<{ cols: number; rows: number } | null>(null);
  const resizeObserverRef = useRef<ResizeObserver | null>(null);
  const pasteFallback = async () => {
    try {
      const text = await readText();
      return text;
    } catch (err) {
      console.error("read clipboard failed", err);
      return "";
    }
  };

  useEffect(() => {
    if (!terminalRef.current) return;

    const term = new Terminal({
      convertEol: true,
      disableStdin: false,
      fontFamily: "SFMono-Regular, Menlo, Monaco, Consolas, monospace",
      fontSize: 13,
      cursorBlink: true,
      theme: {
        background: "#0d1117",
        foreground: "#e5e7eb",
        black: "#000000",
        red: "#ff7b72",
        green: "#3fb950",
        yellow: "#e3b341",
        blue: "#58a6ff",
        magenta: "#bc8cff",
        cyan: "#39c5cf",
        white: "#c9d1d9",
      },
    });

    term.loadAddon(fitAddon);
    term.open(terminalRef.current);
    termInstanceRef.current = term;
    term.attachCustomKeyEventHandler((arg) => {
      if (arg.type !== "keydown") return true;
      const isPaste = (arg.ctrlKey || arg.metaKey) && arg.key.toLowerCase() === "v";
      const isCopy = (arg.ctrlKey || arg.metaKey) && arg.key.toLowerCase() === "c";

      if (isPaste) {
        pasteFallback().then((text) => {
          if (text) {
            void invoke("write_to_terminal", { data: text }).catch((err) =>
              console.error("write failed", err),
            );
          }
        });
        return false;
      }

      if (isCopy) {
        const selection = term.getSelection();
        if (selection) {
          writeText(selection).catch((err) => console.error("write clipboard failed", err));
          return false;
        }
        return true; // allow Ctrl+C to shell when no selection
      }

      return true;
    });
    if (!spawnedRef.current) {
      spawnedRef.current = true;
      void invoke("spawn_terminal").catch((err) =>
        console.error("spawn terminal failed", err),
      );
    }
    term.writeln("\x1b[1;36mVisor terminal\x1b[0m");
    term.writeln("Shell spawned via portable-pty.");
    term.focus();
    const disposeOnData = term.onData((data) => {
      void invoke("write_to_terminal", { data }).catch((err) =>
        console.error("write failed", err),
      );
    });
    onDataDisposableRef.current = disposeOnData;
    fitAddon.fit();
    const initialDims = fitAddon.proposeDimensions();
    if (initialDims) {
      lastSizeRef.current = {
        cols: initialDims.cols,
        rows: initialDims.rows,
      };
      void invoke("resize_terminal", {
        cols: initialDims.cols,
        rows: initialDims.rows,
      }).catch((err) => console.error("resize_terminal failed", err));
    }

    let resizeObserver: ResizeObserver | null = null;
    if (terminalContainerRef.current) {
      resizeObserver = new ResizeObserver(() => {
        if (!termInstanceRef.current) return;
        fitAddon.fit();
        const dims = fitAddon.proposeDimensions();
        if (dims) {
          const { cols, rows } = dims;
          const last = lastSizeRef.current;
          if (!last || last.cols !== cols || last.rows !== rows) {
            lastSizeRef.current = { cols, rows };
            void invoke("resize_terminal", { cols, rows }).catch((err) =>
              console.error("resize_terminal failed", err),
            );
          }
        }
      });
      resizeObserver.observe(terminalContainerRef.current);
    }

    return () => {
      if (resizeObserver && terminalContainerRef.current) {
        resizeObserver.unobserve(terminalContainerRef.current);
      }
      disposeOnData.dispose();
      onDataDisposableRef.current = null;
      term.dispose();
      termInstanceRef.current = null;
    };
  }, [fitAddon]);

  useEffect(() => {
    const unlistenPromise = listen("menu://open-folder", () => {
      handleOpenFolder();
    });
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  useEffect(() => {
    const listenPromise = listen<string>("term-data", (event) => {
      const data = event.payload;
      if (termInstanceRef.current) {
        termInstanceRef.current.write(data);
      }
    });
    listenPromise.then((fn) => {
      termDataUnlistenRef.current = fn;
    });

    return () => {
      if (termDataUnlistenRef.current) {
        termDataUnlistenRef.current?.();
        termDataUnlistenRef.current = null;
      } else {
        listenPromise.then((fn) => fn()).catch(() => {});
      }
    };
  }, []);

  useEffect(() => {
    if (!terminalContainerRef.current) return;
    const observer = new ResizeObserver(() => {
      if (termInstanceRef.current) {
        fitAddon.fit();
        const dims = fitAddon.proposeDimensions();
        if (dims) {
          const { cols, rows } = dims;
          const last = lastSizeRef.current;
          if (!last || last.cols !== cols || last.rows !== rows) {
            lastSizeRef.current = { cols, rows };
            void invoke("resize_terminal", { cols, rows }).catch((err) =>
              console.error("resize_terminal failed", err),
            );
          }
        }
      }
    });
    observer.observe(terminalContainerRef.current);
    resizeObserverRef.current = observer;
    return () => {
      observer.disconnect();
      resizeObserverRef.current = null;
    };
  }, [fitAddon]);

  const truncatePath = (value: string | null, max = 60) => {
    if (!value) return "No folder selected";
    return value.length > max ? `…${value.slice(value.length - max)}` : value;
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
    } catch (err) {
      console.error("Failed to open folder", err);
    }
  };

  const handleRemove = (id: string) => {
    setPinnedItems((prev) => prev.filter((item) => item.id !== id));
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

  useEffect(() => {
    const handleMouseMove = (event: MouseEvent) => {
      if (isResizingSidebar) {
        const next = Math.min(Math.max(event.clientX, 180), 480);
        setSidebarWidth(next);
      }
      if (isResizingConsole) {
        const vh = window.innerHeight;
        const next = Math.min(Math.max(vh - event.clientY, 160), vh - 120);
        setConsoleHeight(next);
      }
    };

    const handleMouseUp = () => {
      setIsResizingSidebar(false);
      setIsResizingConsole(false);
      document.body.style.userSelect = "";
      fitAddon.fit();
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [isResizingSidebar, isResizingConsole, fitAddon]);

  const startResizeSidebar = () => {
    setIsResizingSidebar(true);
    document.body.style.userSelect = "none";
  };

  const startResizeConsole = () => {
    setIsResizingConsole(true);
    document.body.style.userSelect = "none";
  };

  return (
    <div className="h-screen w-screen overflow-hidden bg-white text-slate-900">
      {/* top bar removed per spec; menu handles File -> Open Folder */}
      <div className="flex h-full flex-col" style={{ gridTemplateRows: "auto" }}>
        <div
          className="flex min-h-0 flex-1"
          style={{ gridTemplateRows: "auto" }}
        >
          <aside
            className="flex flex-col border-r border-slate-200 bg-slate-900/95 px-4 py-4 text-white backdrop-blur"
            style={{ width: sidebarWidth }}
          >
            <p className="text-base font-semibold text-white">Visor</p>
            <p className="mt-3 text-[11px] uppercase tracking-[0.08em] text-slate-300">
              Context Actions
            </p>
            <div className="mt-3 flex items-center justify-between rounded-md bg-white/5 px-3 py-2 ring-1 ring-white/10">
              <span className="text-[11px] font-medium text-white">
                Copy Bundle
              </span>
              <button
                onClick={handleCopyBundle}
                className="rounded-md bg-emerald-500 px-2 py-1 text-[11px] font-semibold text-white shadow-sm hover:bg-emerald-400 active:bg-emerald-500"
              >
                Copy
              </button>
            </div>
            {copyState === "copied" && (
              <p className="mt-2 text-[11px] text-emerald-200">Copied!</p>
            )}
            {copyState === "error" && (
              <p className="mt-2 text-[11px] text-rose-200">
                Nothing to copy yet.
              </p>
            )}

            <p className="mt-5 text-[11px] uppercase tracking-[0.08em] text-slate-300">
              Pinned context
            </p>
            <div className="mt-3 flex-1 space-y-2 overflow-y-auto pr-1">
              {pinnedItems.length === 0 ? (
                <p className="rounded-md bg-white/5 px-3 py-2 text-[12px] text-slate-300 ring-1 ring-white/10">
                  No items pinned yet.
                </p>
              ) : (
                pinnedItems.map((item) => (
                  <ContextItem
                    key={item.id}
                    item={item}
                    onRemove={handleRemove}
                  />
                ))
              )}
            </div>
          </aside>

          <div
            className="w-[2px] cursor-col-resize bg-slate-300/70 hover:bg-slate-500/80 active:bg-slate-600"
            onMouseDown={startResizeSidebar}
            aria-label="Resize sidebar"
          />

          <section
            className="relative flex-1"
            style={{
              backgroundColor: "#54626F",
              backgroundImage:
                "radial-gradient(rgba(255,255,255,0.15) 1px, transparent 0), radial-gradient(rgba(255,255,255,0.15) 1px, transparent 0)",
              backgroundPosition: "0 0, 14px 14px",
              backgroundSize: "28px 28px",
            }}
          >
            <div className="absolute inset-x-0 top-0 flex items-center justify-between px-4 py-3 text-xs text-white/80">
              <div className="rounded-md bg-black/20 px-3 py-1 ring-1 ring-white/10">
                {truncatePath(openPath)}
              </div>
              <div className="rounded-md bg-black/20 px-3 py-1 ring-1 ring-white/10">
                Files: {codemapFiles.length}
              </div>
            </div>
            <div className="pointer-events-auto absolute inset-0 flex flex-wrap content-start items-start gap-2 p-4">
              {codemapFiles.length === 0 ? (
                <div className="rounded-full bg-white/85 px-4 py-2 text-[11px] font-medium text-slate-600 shadow-sm ring-1 ring-slate-200/60">
                  Codemap canvas placeholder
                </div>
              ) : (
                codemapFiles.slice(0, 200).map((file) => (
                  <span
                    key={file}
                    className="rounded-full bg-white/80 px-3 py-1 text-[11px] font-medium text-slate-700 shadow-sm ring-1 ring-slate-200"
                  >
                    {file}
                  </span>
                ))
              )}
            </div>
          </section>
        </div>

        <div
          className="h-[3px] cursor-row-resize bg-slate-300/70 hover:bg-slate-500/80 active:bg-slate-600"
          onMouseDown={startResizeConsole}
          aria-label="Resize console"
        />

          <section
            className="relative border-t border-slate-200 bg-slate-950"
            style={{ height: consoleHeight }}
            ref={terminalContainerRef}
          >
            <div className="pointer-events-none absolute inset-x-0 top-0 h-10 bg-gradient-to-b from-slate-900/60 to-transparent" />
            <div className="absolute inset-0 bg-[radial-gradient(circle_at_top_left,rgba(59,130,246,0.12),transparent_35%),radial-gradient(circle_at_bottom_right,rgba(16,185,129,0.12),transparent_35%)]" />
          <div
            ref={terminalRef}
            className="relative z-10 h-full w-full px-4 py-3"
            aria-label="Terminal placeholder"
            onPaste={(event) => {
              event.preventDefault();
              pasteFallback().then((text) => {
                if (text) {
                  void invoke("write_to_terminal", { data: text }).catch(
                    (err) => console.error("write failed", err),
                  );
                }
              });
            }}
          />
        </section>
      </div>
    </div>
  );
}

export default App;

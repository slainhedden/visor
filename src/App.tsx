import { useEffect, useRef } from "react";
import { Terminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";

function App() {
  const terminalRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!terminalRef.current) return;

    const term = new Terminal({
      convertEol: true,
      disableStdin: true,
      fontFamily: "SFMono-Regular, Menlo, Monaco, Consolas, monospace",
      fontSize: 13,
      cursorBlink: false,
      theme: {
        background: "#0b1220",
        foreground: "#e5e7eb",
      },
    });

    term.open(terminalRef.current);
    term.writeln("\x1b[1;36mContextMap terminal placeholder\x1b[0m");
    term.writeln("This is a non-interactive shell stub.");
    term.writeln("We'll wire it to a PTY in the next phase.");

    return () => {
      term.dispose();
    };
  }, []);

  return (
    <div className="h-screen w-screen overflow-hidden bg-white text-slate-900">
      <div className="grid h-full grid-rows-[auto_32vh]">
        <div className="flex min-h-0">
          <aside className="flex w-64 flex-col border-r border-slate-200 bg-slate-50/80 px-4 py-4 backdrop-blur">
            <p className="text-base font-semibold text-slate-800">Visor</p>
            <p className="mt-3 text-[11px] uppercase tracking-[0.08em] text-slate-500">
              Pinned context (placeholder)
            </p>
            <div className="mt-4 space-y-2 text-sm text-slate-700">
              <div className="rounded-md bg-white/80 px-3 py-2 ring-1 ring-slate-200">
                core/app.ts
              </div>
              <div className="rounded-md bg-white/80 px-3 py-2 ring-1 ring-slate-200">
                ui/map/canvas.tsx
              </div>
              <div className="rounded-md bg-white/80 px-3 py-2 ring-1 ring-slate-200">
                agents/context_bundle.md
              </div>
            </div>
          </aside>

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
            <div className="pointer-events-none absolute inset-0 flex items-start justify-start">
              <div className="m-4 rounded-full bg-white/85 px-4 py-2 text-[11px] font-medium text-slate-600 shadow-sm ring-1 ring-slate-200/60">
                Codemap canvas placeholder
              </div>
            </div>
          </section>
        </div>

        <section className="relative border-t border-slate-200 bg-slate-950">
          <div className="pointer-events-none absolute inset-x-0 top-0 h-10 bg-gradient-to-b from-slate-900/60 to-transparent" />
          <div className="absolute inset-0 bg-[radial-gradient(circle_at_top_left,rgba(59,130,246,0.12),transparent_35%),radial-gradient(circle_at_bottom_right,rgba(16,185,129,0.12),transparent_35%)]" />
          <div
            ref={terminalRef}
            className="relative z-10 h-full w-full px-4 py-3"
            aria-label="Terminal placeholder"
          />
        </section>
      </div>
    </div>
  );
}

export default App;

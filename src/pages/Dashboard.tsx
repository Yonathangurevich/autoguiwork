import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import {
  loadAutomation,
  saveAutomation,
  runAutomation,
  loadCanvasLayout,
  saveCanvasLayout,
  defaultLayout,
  layoutToActions,
  newNodeId,
} from "../services";
import type { App, ActionsKind, RunReport, CanvasLayout } from "../models";
import { ActionPalette } from "../features/dashboard/palette/ActionPalette";
import { AutomationCanvas } from "../features/dashboard/canvas/AutomationCanvas";
import { RunLog } from "../features/dashboard/panels/RunLog";
import { ServerPanel } from "../features/dashboard/panels/ServerPanel";
import { RunsPanel } from "../features/dashboard/panels/RunsPanel";
import "./Dashboard.css";

export function Dashboard() {
  const { name } = useParams<{ name: string }>();
  const navigate = useNavigate();
  const [app, setApp] = useState<App | null>(null);
  const [layout, setLayout] = useState<CanvasLayout | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [running, setRunning] = useState(false);
  const [report, setReport] = useState<RunReport | null>(null);
  const [showServer, setShowServer] = useState(false);
  const [showRuns, setShowRuns] = useState(false);

  // Load the automation, then its canvas layout. If there's no saved layout
  // (first time in the editor), lay the existing actions out as a chain.
  async function reload() {
    if (!name) return;
    try {
      const loaded = await loadAutomation(name);
      setApp(loaded);
      const saved = await loadCanvasLayout(name);
      setLayout(saved ?? defaultLayout(loaded.actions));
    } catch (e) {
      setError(String(e));
    }
  }

  useEffect(() => {
    reload();
  }, [name]);

  // THE SYNC POINT. Any canvas edit writes two files:
  //   canvas.json     — the visual layout (positions + parked nodes), UI only
  //   automation.json — actions[] derived by walking the connected chain,
  //                     which is exactly what the Rust engine runs.
  // Parked nodes are not on the chain, so they never become actions.
  async function applyLayout(next: CanvasLayout) {
    if (!app || !name) return;
    setLayout(next);

    const nextApp: App = { ...app, actions: layoutToActions(next) };
    setApp(nextApp);

    try {
      await saveCanvasLayout(name, next);
      await saveAutomation(nextApp);
    } catch (e) {
      setError(String(e));
    }
  }

  // A palette click drops a new, unconnected node on the canvas. It becomes a
  // step only once you wire it into the chain.
  function addAction(action: ActionsKind) {
    if (!layout) return;
    applyLayout({
      ...layout,
      nodes: [
        ...layout.nodes,
        { id: newNodeId(layout), x: 520, y: 120 + layout.nodes.length * 40, action },
      ],
    });
  }

  // Run the automation once and show the log. The Rust side takes over the
  // mouse/keyboard; Esc stops it.
  async function test() {
    if (!app || running) return;
    setRunning(true);
    setReport(null);
    setError(null);
    try {
      const r = await runAutomation(app.name);
      setReport(r);
    } catch (e) {
      setError(String(e));
    } finally {
      setRunning(false);
    }
  }

  if (error) return <p className="error">error: {error}</p>;
  if (!app || !layout) return <p>loading…</p>;

  // The declared input names (from a Server trigger), offered as arg choices in
  // every text field so steps can reference the caller's args.
  const declaredArgs =
    typeof app.trigger === "object" && "Server" in app.trigger
      ? app.trigger.Server.inputs.map((i) => i.name)
      : [];

  return (
    <div className="dashboard">
      <div className="dash-top">
        <button className="back" onClick={() => navigate("/")}>
          ← back
        </button>
        <span className="dash-name">{app.name}</span>
        <div className="dash-actions">
          <button
            className="server-toggle"
            onClick={() => setShowRuns((s) => !s)}
            title="endpoint run history"
          >
            ☰ Runs
          </button>
          <button
            className="server-toggle"
            onClick={() => setShowServer((s) => !s)}
            title="expose as an API endpoint"
          >
            ⇅ Server
          </button>
          <button
            className="test"
            onClick={test}
            disabled={running || app.actions.length === 0}
            title={
              app.actions.length === 0
                ? "connect at least one node to Start"
                : "run this automation"
            }
          >
            {running ? "running…" : "▶ Test"}
          </button>
        </div>
      </div>

      {showServer && <ServerPanel app={app} onChanged={reload} />}
      {showRuns && <RunsPanel appName={app.name} />}

      <div className="dash-body">
        {/* the node canvas — the connected chain IS app.actions */}
        <AutomationCanvas
          appName={app.name}
          args={declaredArgs}
          layout={layout}
          onLayoutChange={applyLayout}
        />

        {/* right nav: pick an action to drop a node on the canvas */}
        <ActionPalette onPick={addAction} />
      </div>

      <RunLog
        report={report}
        running={running}
        onClose={() => setReport(null)}
      />
    </div>
  );
}

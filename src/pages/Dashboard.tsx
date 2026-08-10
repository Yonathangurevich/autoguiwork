import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { loadAutomation, saveAutomation, runAutomation } from "../api";
import type { App, ActionsKind, RunReport } from "../types/automation";
import { ActionPalette } from "../components/dashboard/ActionPalette";
import { StepList } from "../components/dashboard/StepList";
import { RunLog } from "../components/dashboard/RunLog";
import { ServerPanel } from "../components/dashboard/ServerPanel";
import { RunsPanel } from "../components/dashboard/RunsPanel";
import "./Dashboard.css";

export function Dashboard() {
  const { name } = useParams<{ name: string }>();
  const navigate = useNavigate();
  const [app, setApp] = useState<App | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [running, setRunning] = useState(false);
  const [report, setReport] = useState<RunReport | null>(null);
  const [showServer, setShowServer] = useState(false);
  const [showRuns, setShowRuns] = useState(false);

  function reload() {
    if (!name) return;
    loadAutomation(name)
      .then(setApp)
      .catch((e) => setError(String(e)));
  }

  useEffect(() => {
    reload();
  }, [name]);

  // Persist the whole app to JSON (the source of truth). Called on meaningful
  // edits — add/remove/reorder a step. Field-level debounced saves come later.
  async function persist(next: App) {
    setApp(next);
    try {
      await saveAutomation(next);
    } catch (e) {
      setError(String(e));
    }
  }

  // A palette click appends a new step (an Action) to the automation.
  function addAction(action: ActionsKind) {
    if (!app) return;
    persist({ ...app, actions: [...app.actions, { action }] });
  }

  function removeStep(index: number) {
    if (!app) return;
    persist({
      ...app,
      actions: app.actions.filter((_, i) => i !== index),
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

  // Replace one step's action after the user edits its fields.
  function updateStep(index: number, action: ActionsKind) {
    if (!app) return;
    persist({
      ...app,
      actions: app.actions.map((step, i) => (i === index ? { action } : step)),
    });
  }

  // Reorder: move the step at `from` to position `to` (drag-and-drop).
  function moveStep(from: number, to: number) {
    if (!app || from === to) return;
    const actions = [...app.actions];
    const [moved] = actions.splice(from, 1);
    actions.splice(to, 0, moved);
    persist({ ...app, actions });
  }

  if (error) return <p className="error">error: {error}</p>;
  if (!app) return <p>loading…</p>;

  // The declared input names (from a Server trigger), offered as arg choices in
  // every text field so steps can reference the caller's args.
  const declaredArgs =
    typeof app.trigger === "object" && "Server" in app.trigger
      ? app.trigger.Server.inputs.map((i) => i.name)
      : [];

  return (
    <div className="dashboard">
      {/* top bar: back + name + Test (center) */}
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
                ? "add a step first"
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
        {/* the step canvas — each step is an Action in app.actions */}
        <StepList
          appName={app.name}
          args={declaredArgs}
          actions={app.actions}
          onRemove={removeStep}
          onUpdate={updateStep}
          onMove={moveStep}
        />

        {/* right nav: pick an action to add as a step */}
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

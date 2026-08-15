import { useEffect, useState } from "react";
import { automationRuns } from "../../../services";
import type { RunRecord, ActionStatus, RunOutcome } from "../../../models";
import "./RunsPanel.css";

function statusText(s: ActionStatus): { text: string; cls: string } {
  if (s === "Ok") return { text: "ok", cls: "ok" };
  if (s === "Skipped") return { text: "skipped", cls: "skipped" };
  return { text: s.Failed, cls: "failed" };
}

function outcomeText(o: RunOutcome): { text: string; cls: string } {
  if (o === "Completed") return { text: "completed ✓", cls: "ok" };
  if ("Failed" in o)
    return { text: `failed at step ${o.Failed.at_index + 1}`, cls: "failed" };
  return { text: `cancelled at step ${o.Cancelled.at_index + 1}`, cls: "skipped" };
}

export function RunsPanel({ appName }: { appName: string }) {
  const [runs, setRuns] = useState<RunRecord[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [openIdx, setOpenIdx] = useState<number | null>(null);

  async function load() {
    setError(null);
    try {
      setRuns(await automationRuns(appName));
    } catch (e) {
      setError(String(e));
    }
  }

  useEffect(() => {
    load();
  }, [appName]);

  return (
    <div className="runs-panel">
      <div className="runs-head">
        <h3>runs</h3>
        <button className="runs-refresh" onClick={load}>
          refresh
        </button>
      </div>

      {error && <p className="server-error">{error}</p>}
      {runs.length === 0 && !error && (
        <p className="runs-empty">no runs yet — call the endpoint to see history here.</p>
      )}

      <div className="runs-list">
        {runs.map((rec, i) => {
          const o = outcomeText(rec.report.outcome);
          const isOpen = openIdx === i;
          return (
            <div className={`run-item ${isOpen ? "open" : ""}`} key={rec.at + i}>
              <button className="run-row" onClick={() => setOpenIdx(isOpen ? null : i)}>
                <span className="run-time">{new Date(rec.at).toLocaleString()}</span>
                <span className={`run-outcome ${o.cls}`}>{o.text}</span>
              </button>

              {isOpen && (
                <ul className="run-detail">
                  {rec.report.results.map((r) => {
                    const s = statusText(r.status);
                    return (
                      <li className="run-step" key={r.index}>
                        <span className="run-idx">{r.index + 1}</span>
                        <span className="run-name">{r.name}</span>
                        <span className="run-ms">{r.duration_ms}ms</span>
                        <span className={`run-status ${s.cls}`}>{s.text}</span>
                      </li>
                    );
                  })}
                </ul>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

import type { RunReport, ActionStatus, RunOutcome } from "../../../models";
import "./RunLog.css";

function statusLabel(s: ActionStatus): { text: string; cls: string } {
  if (s === "Ok") return { text: "ok", cls: "ok" };
  if (s === "Skipped") return { text: "skipped", cls: "skipped" };
  return { text: s.Failed, cls: "failed" };
}

function outcomeLabel(o: RunOutcome): { text: string; cls: string } {
  if (o === "Completed") return { text: "completed ✓", cls: "ok" };
  if ("Failed" in o) return { text: `failed at step ${o.Failed.at_index + 1}`, cls: "failed" };
  return { text: `cancelled at step ${o.Cancelled.at_index + 1}`, cls: "skipped" };
}

export function RunLog({
  report,
  running,
  onClose,
}: {
  report: RunReport | null;
  running: boolean;
  onClose: () => void;
}) {
  if (!running && !report) return null;

  return (
    <div className="runlog">
      <div className="runlog-head">
        <span className="runlog-title">run log</span>
        {!running && (
          <button className="runlog-close" onClick={onClose}>
            ×
          </button>
        )}
      </div>

      {running && <p className="runlog-running">running… (press Esc to stop)</p>}

      {report && (
        <>
          <ul className="runlog-list">
            {report.results.map((r) => {
              const s = statusLabel(r.status);
              return (
                <li className="runlog-row" key={r.index}>
                  <span className="runlog-idx">{r.index + 1}</span>
                  <span className="runlog-name">{r.name}</span>
                  <span className="runlog-ms">{r.duration_ms}ms</span>
                  <span className={`runlog-status ${s.cls}`}>{s.text}</span>
                </li>
              );
            })}
          </ul>
          {(() => {
            const o = outcomeLabel(report.outcome);
            return <p className={`runlog-outcome ${o.cls}`}>{o.text}</p>;
          })()}
        </>
      )}
    </div>
  );
}

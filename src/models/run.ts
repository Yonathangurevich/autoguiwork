// The result of running an automation — mirrors the Rust engine's RunReport
// and the per-endpoint run history (RunRecord).

// How one action turned out.
export type ActionStatus = "Ok" | { Failed: string } | "Skipped";

export interface ActionResult {
  index: number;
  name: string;
  status: ActionStatus;
  duration_ms: number;
}

// The overall result of a run.
export type RunOutcome =
  | "Completed"
  | { Failed: { at_index: number } }
  | { Cancelled: { at_index: number } };

// What the automation produced for its caller.
export type OutputValue =
  | "None"
  | { Text: string }
  | { File: { content_type: string; base64: string } };

export interface RunReport {
  app_name: string;
  results: ActionResult[];
  outcome: RunOutcome;
  output: OutputValue;
}

// One saved endpoint run: timestamp + its report (the Runs tab history).
export interface RunRecord {
  at: string;
  report: RunReport;
}

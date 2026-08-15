// Running an automation + reading its run history.

import { invoke } from "@tauri-apps/api/core";
import type { RunReport, RunRecord } from "../models";

// Run once (the Test button). The run takes over the mouse/keyboard and can be
// stopped with Esc; resolves with the full report.
export function runAutomation(name: string): Promise<RunReport> {
  return invoke<RunReport>("run_automation", { name });
}

// Recent endpoint runs (newest first) for the Runs history panel.
export function automationRuns(name: string): Promise<RunRecord[]> {
  return invoke<RunRecord[]>("automation_runs", { name });
}

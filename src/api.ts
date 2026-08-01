// Typed wrappers around the Tauri commands. Components call these instead of
// raw invoke(), so every backend call is type-checked against the engine's
// serde shapes (see types/automation.ts).

import { invoke } from "@tauri-apps/api/core";
import type { App, RunReport, RunRecord, InputSpec } from "./types/automation";

export function listAutomations(): Promise<string[]> {
  return invoke<string[]>("list_automations");
}

export function createAutomation(name: string): Promise<App> {
  return invoke<App>("create_automation", { name });
}

export function loadAutomation(name: string): Promise<App> {
  return invoke<App>("load_automation", { name });
}

export function saveAutomation(app: App): Promise<void> {
  return invoke("save_automation", { app });
}

// Run the automation once (Test). Resolves with the full run report; the run
// takes over the mouse/keyboard and can be stopped with Esc.
export function runAutomation(name: string): Promise<RunReport> {
  return invoke<RunReport>("run_automation", { name });
}

// Save image bytes (from clipboard) into the automation's assets/ as a PNG.
// Returns the relative path to store in the FindImage step's image_path.
export function saveImage(
  appName: string,
  stepIndex: number,
  bytes: Uint8Array
): Promise<string> {
  // Tauri serializes a plain number[] to Vec<u8> on the Rust side.
  return invoke<string>("save_image", {
    appName,
    stepIndex,
    bytes: Array.from(bytes),
  });
}

// Read a saved template image back as a data: URL, so the preview survives
// navigating away and reopening the automation.
export function readImage(
  appName: string,
  relativePath: string
): Promise<string> {
  return invoke<string>("read_image", { appName, relativePath });
}

// Is the shared worker server running? Resolves to the port, or null.
export function serverStatus(): Promise<number | null> {
  return invoke<number | null>("server_status");
}

// Expose an automation as an API endpoint ("set to production"). Sets its
// Server trigger + declared inputs, starts the server, returns the URL to call.
export function exposeAsServer(
  name: string,
  inputs: InputSpec[]
): Promise<string> {
  return invoke<string>("expose_as_server", { name, inputs });
}

// Turn an exposed automation back to Manual (not served).
export function unexposeServer(name: string): Promise<void> {
  return invoke("unexpose_server", { name });
}

// Recent endpoint runs (newest first) for the Runs tab.
export function automationRuns(name: string): Promise<RunRecord[]> {
  return invoke<RunRecord[]>("automation_runs", { name });
}

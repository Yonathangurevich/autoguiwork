// Saving/loading the visual editor's layout. This is UI memory ONLY — the Rust
// engine never reads it. automation.json still holds just the connected chain
// as actions[], exactly as the engine runs it.

import { invoke } from "@tauri-apps/api/core";
import type { CanvasLayout } from "../models";

export function saveCanvasLayout(
  name: string,
  layout: CanvasLayout
): Promise<void> {
  return invoke("save_canvas_layout", { name, layout });
}

// Returns null when this automation has never been opened in the editor
// (the UI then lays the chain out top-to-bottom by default).
export function loadCanvasLayout(name: string): Promise<CanvasLayout | null> {
  return invoke<CanvasLayout | null>("load_canvas_layout", { name });
}

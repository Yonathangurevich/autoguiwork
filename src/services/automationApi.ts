// Automation CRUD — the list/create/load/save Tauri commands.
// Components call these instead of raw invoke(), so calls stay type-checked
// against the engine's serde shapes (see models/).

import { invoke } from "@tauri-apps/api/core";
import type { App } from "../models";

export function listAutomations(): Promise<string[]> {
  return invoke<string[]>("list_automations");
}

export function createAutomation(name: string): Promise<App> {
  return invoke<App>("create_automation", { name });
}

export function loadAutomation(name: string): Promise<App> {
  return invoke<App>("load_automation", { name });
}

// Persist the whole automation. Writing the JSON IS the save (source of truth).
export function saveAutomation(app: App): Promise<void> {
  return invoke("save_automation", { app });
}

// An automation — mirrors the Rust engine's App. This is the shape saved to
// automation.json (the single source of truth) and loaded to rebuild the UI.
// (Rust's base_dir is #[serde(skip)], so it's absent from the JSON.)

import type { Action } from "./action";
import type { Trigger } from "./trigger";

export interface App {
  name: string;
  trigger: Trigger;
  actions: Action[];
}

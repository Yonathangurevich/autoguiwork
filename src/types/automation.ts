// TypeScript mirrors of the Rust engine's serde-serialized types.
// These MUST match the JSON shapes serde produces, or invoke() calls break.
//
// serde's default enum representation (what the engine uses):
//   - unit variant        -> "VariantName"                 (a bare string)
//   - newtype variant      -> { "VariantName": inner }
//   - tuple variant        -> { "VariantName": [a, b] }
//   - struct variant       -> { "VariantName": { field: ... } }

// --- Context values / arguments -------------------------------------------

// PosArg: a screen position, literal or resolved from a variable at run time.
export type PosArg =
  | { Literal: [number, number] }
  | { Var: string };

// TextArg: a string, literal or resolved from a variable (e.g. an input arg).
export type TextArg =
  | { Literal: string }
  | { Var: string };

// --- Mouse / keyboard -----------------------------------------------------

export type MouseOptions =
  | { MoveTo: [PosArg, number] }
  | "LeftClick"
  | "RightClick"
  | { Drag: [PosArg, number] }
  | "DoubleClick";

export type Key =
  // common
  | "Enter" | "BackSpace" | "Space" | "Tab" | "Escape" | "Delete" | "Insert"
  // arrows
  | "LeftArrow" | "RightArrow" | "UpArrow" | "DownArrow"
  // navigation
  | "Home" | "End" | "PageUp" | "PageDown"
  // modifiers
  | "Shift" | "Ctrl" | "Alt" | "Win"
  // function keys
  | "F1" | "F2" | "F3" | "F4" | "F5" | "F6" | "F7" | "F8" | "F9" | "F10"
  | "F11" | "F12" | "F13" | "F14" | "F15" | "F16" | "F17" | "F18" | "F19"
  | "F20" | "F21" | "F22" | "F23" | "F24";

export type KeyboardOptions =
  | { Input: TextArg }
  | { PressKey: Key };

// --- Open apps ------------------------------------------------------------

export type Googles = "Drive" | "Chrome" | { Search: TextArg };

export type OpenApps =
  | "Outlook"
  | { Google: Googles }
  | { Explorer: TextArg }
  | { RandomApp: TextArg };

// --- The action set (mirrors ActionsKind) ---------------------------------

export type ActionsKind =
  | { Mouse: MouseOptions }
  | { Keyboard: { opts: KeyboardOptions; repeat: number } }
  | { Open: OpenApps }
  | { MoveLastDownload: { to: TextArg; waited_ms: number } }
  | { WaitForWindow: { title: TextArg; waited_ms: number } }
  | { FindImageLoop: { image_path: string; waited_ms: number; store_as: string } }
  | { Sleep: number }
  | { SetOutput: TextArg }
  | { ReadFileAsBase64: { path: TextArg; content_type: string } };

// One step in the automation. action_index/action_name are #[serde(skip)] on
// the Rust side (re-derived on load), so they are absent from the JSON.
export interface Action {
  action: ActionsKind;
}

// --- Triggers -------------------------------------------------------------

export interface InputSpec {
  name: string;
  required: boolean;
}

export type Trigger =
  | "Manual"
  | { Server: { endpoint_id: string; inputs: InputSpec[] } };

// --- The automation (mirrors App) -----------------------------------------
// base_dir is #[serde(skip)] on Rust, so it's not in the JSON.

export interface App {
  name: string;
  trigger: Trigger;
  actions: Action[];
}

// --- Run report (what execute returns) ------------------------------------

export type ActionStatus = "Ok" | { Failed: string } | "Skipped";

export interface ActionResult {
  index: number;
  name: string;
  status: ActionStatus;
  duration_ms: number;
}

export type RunOutcome =
  | "Completed"
  | { Failed: { at_index: number } }
  | { Cancelled: { at_index: number } };

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

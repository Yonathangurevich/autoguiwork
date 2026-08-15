// The action set — mirrors the Rust engine's ActionsKind and its sub-enums.
// An automation is an ordered list of these; the engine runs them top to bottom.

import type { PosArg, TextArg } from "./values";

// --- Mouse ---------------------------------------------------------------
export type MouseOptions =
  | { MoveTo: [PosArg, number] }
  | "LeftClick"
  | "RightClick"
  | { Drag: [PosArg, number] }
  | "DoubleClick";

// --- Keyboard ------------------------------------------------------------
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

export type KeyboardOptions = { Input: TextArg } | { PressKey: Key };

// --- Open apps -----------------------------------------------------------
export type Googles = "Drive" | "Chrome" | { Search: TextArg };

export type OpenApps =
  | "Outlook"
  | { Google: Googles }
  | { Explorer: TextArg }
  | { RandomApp: TextArg };

// --- The action union ----------------------------------------------------
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

// One step in the automation. (Rust's action_index/action_name are
// #[serde(skip)], so they're absent from the JSON.)
export interface Action {
  action: ActionsKind;
}

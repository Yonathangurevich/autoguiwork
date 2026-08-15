// Human-readable labels for actions — turns an ActionsKind into the short text
// shown on a step in the canvas. Pure formatting logic, kept out of the UI.

import type { ActionsKind, TextArg } from "../models";

// Show a TextArg: a fixed value as-is, a variable reference as {name}.
export function showArg(t: TextArg): string {
  return "Var" in t ? `{${t.Var}}` : t.Literal;
}

// A short label for any action, so each step reads clearly.
export function describeAction(action: ActionsKind): string {
  if ("Mouse" in action) {
    const m = action.Mouse;
    if (typeof m === "string") return `Mouse: ${m}`;
    if ("MoveTo" in m) return "Mouse: Move to";
    if ("Drag" in m) return "Mouse: Drag";
    return "Mouse";
  }
  if ("Keyboard" in action) {
    const { opts, repeat } = action.Keyboard;
    const rep = repeat > 1 ? ` ×${repeat}` : "";
    if ("Input" in opts) return `Keyboard: Type text${rep}`;
    if ("PressKey" in opts) return `Keyboard: Press ${opts.PressKey}${rep}`;
    return "Keyboard";
  }
  if ("Open" in action) {
    const o = action.Open;
    if (o === "Outlook") return "Open: Outlook";
    if ("Google" in o) return "Open: Google";
    if ("Explorer" in o) return "Open: File Explorer";
    return "Open: custom";
  }
  if ("MoveLastDownload" in action)
    return `Move last download → ${showArg(action.MoveLastDownload.to) || "…"}`;
  if ("WaitForWindow" in action)
    return `Wait for window: ${showArg(action.WaitForWindow.title) || "…"}`;
  if ("FindImageLoop" in action)
    return `Find image → ${action.FindImageLoop.store_as}`;
  if ("Sleep" in action) return `Sleep ${action.Sleep}s`;
  if ("SetOutput" in action) return "Set output (text)";
  if ("ReadFileAsBase64" in action) return "Return file (base64)";
  return "Unknown action";
}

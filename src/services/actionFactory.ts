// The catalog of actions the palette offers: a display label + a factory that
// produces a fresh ActionsKind with sensible defaults. Kept out of the UI so
// the action set lives in one obvious place.

import type { ActionsKind } from "../models";

export interface ActionTemplate {
  label: string;
  make: () => ActionsKind;
}

export const ACTION_TEMPLATES: ActionTemplate[] = [
  {
    label: "Find image",
    make: () => ({
      FindImageLoop: { image_path: "", waited_ms: 10000, store_as: "found" },
    }),
  },
  { label: "Move to", make: () => ({ Mouse: { MoveTo: [{ Var: "found" }, 0.2] } }) },
  { label: "Left click", make: () => ({ Mouse: "LeftClick" }) },
  { label: "Double click", make: () => ({ Mouse: "DoubleClick" }) },
  {
    label: "Type text",
    make: () => ({ Keyboard: { opts: { Input: { Literal: "" } }, repeat: 1 } }),
  },
  {
    label: "Press key",
    make: () => ({ Keyboard: { opts: { PressKey: "Enter" }, repeat: 1 } }),
  },
  {
    label: "Wait for window",
    make: () => ({ WaitForWindow: { title: { Literal: "" }, waited_ms: 15000 } }),
  },
  { label: "Sleep", make: () => ({ Sleep: 1.0 }) },
  { label: "Open app", make: () => ({ Open: { RandomApp: { Literal: "" } } }) },
  {
    label: "Move last download",
    make: () => ({ MoveLastDownload: { to: { Literal: "" }, waited_ms: 60000 } }),
  },
  { label: "Set output (text)", make: () => ({ SetOutput: { Var: "found" } }) },
  {
    label: "Return file (base64)",
    make: () => ({
      ReadFileAsBase64: { path: { Literal: "" }, content_type: "application/pdf" },
    }),
  },
];

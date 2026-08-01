import type { ActionsKind } from "../../types/automation";
import "./ActionPalette.css";

// Each palette entry is a label + a factory that produces a fresh ActionsKind
// with sensible defaults. Picking one appends it as a step; the step's fields
// get edited in the canvas afterwards.
interface PaletteItem {
  label: string;
  make: () => ActionsKind;
}

const ITEMS: PaletteItem[] = [
  {
    label: "Find image",
    make: () => ({
      FindImageLoop: { image_path: "", waited_ms: 10000, store_as: "found" },
    }),
  },
  { label: "Move to", make: () => ({ Mouse: { MoveTo: [{ Var: "found" }, 0.2] } }) },
  { label: "Left click", make: () => ({ Mouse: "LeftClick" }) },
  { label: "Double click", make: () => ({ Mouse: "DoubleClick" }) },
  { label: "Type text", make: () => ({ Keyboard: { Input: { Literal: "" } } }) },
  { label: "Press Enter", make: () => ({ Keyboard: { PressKey: "Enter" } }) },
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

export function ActionPalette({ onPick }: { onPick: (a: ActionsKind) => void }) {
  return (
    <nav className="palette">
      <h3 className="palette-title">actions</h3>
      {ITEMS.map((item) => (
        <button key={item.label} className="palette-item" onClick={() => onPick(item.make())}>
          {item.label}
        </button>
      ))}
    </nav>
  );
}

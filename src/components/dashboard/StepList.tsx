import { useState } from "react";
import type { Action, ActionsKind, TextArg } from "../../types/automation";
import { StepEditor } from "./StepEditor";
import "./StepList.css";

// Show a TextArg: a fixed value as-is, an arg reference as {name}.
function showArg(t: TextArg): string {
  return "Var" in t ? `{${t.Var}}` : t.Literal;
}

// A short human label for any action, so each step reads clearly in the canvas.
function describe(action: ActionsKind): string {
  if ("Mouse" in action) {
    const m = action.Mouse;
    if (typeof m === "string") return `Mouse: ${m}`;
    if ("MoveTo" in m) return "Mouse: Move to";
    if ("Drag" in m) return "Mouse: Drag";
    return "Mouse";
  }
  if ("Keyboard" in action) {
    const k = action.Keyboard;
    if ("Input" in k) return "Keyboard: Type text";
    if ("PressKey" in k) return `Keyboard: Press ${k.PressKey}`;
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

interface Props {
  appName: string;
  args: string[];
  actions: Action[];
  onRemove: (index: number) => void;
  onUpdate: (index: number, action: ActionsKind) => void;
}

export function StepList({ appName, args, actions, onRemove, onUpdate }: Props) {
  // Which step is expanded for editing (only one at a time).
  const [openIndex, setOpenIndex] = useState<number | null>(null);

  return (
    <div className="steps">
      {actions.length === 0 && (
        <p className="steps-empty">
          no steps yet — pick an action from the right to add one.
        </p>
      )}

      {actions.map((step, i) => {
        const isOpen = openIndex === i;
        return (
          <div className={`step ${isOpen ? "open" : ""}`} key={i}>
            <div className="step-head">
              <span className="step-index">{i + 1}</span>
              <button
                className="step-label"
                onClick={() => setOpenIndex(isOpen ? null : i)}
              >
                {describe(step.action)}
              </button>
              <button
                className="step-remove"
                title="remove step"
                onClick={() => onRemove(i)}
              >
                ×
              </button>
            </div>

            {isOpen && (
              <div className="step-body">
                <StepEditor
                  action={step.action}
                  appName={appName}
                  stepIndex={i}
                  args={args}
                  onChange={(a) => onUpdate(i, a)}
                />
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}

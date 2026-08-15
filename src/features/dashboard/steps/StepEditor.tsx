import type {
  ActionsKind,
  MouseOptions,
  KeyboardOptions,
  OpenApps,
  TextArg,
  Key,
} from "../../../models";
import { FindImageEditor } from "./FindImageEditor";
import "./StepEditor.css";

// Edits one step's fields. Switches on the action variant and renders the
// matching inputs; every change calls onChange with a fresh ActionsKind, which
// the Dashboard persists to JSON.
interface Props {
  action: ActionsKind;
  onChange: (a: ActionsKind) => void;
  // Needed by the FindImage editor to save a pasted image into the right
  // automation folder + step slot.
  appName: string;
  stepIndex: number;
  // The automation's declared input names, offered as arg choices in text fields.
  args: string[];
}

// --- small field helpers ---------------------------------------------------

function TextField({
  label,
  value,
  onChange,
  placeholder,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  placeholder?: string;
}) {
  return (
    <label className="field">
      <span>{label}</span>
      <input
        value={value}
        placeholder={placeholder}
        onChange={(e) => onChange(e.target.value)}
      />
    </label>
  );
}

function NumberField({
  label,
  value,
  onChange,
  step,
}: {
  label: string;
  value: number;
  onChange: (v: number) => void;
  step?: number;
}) {
  return (
    <label className="field">
      <span>{label}</span>
      <input
        type="number"
        value={value}
        step={step ?? 1}
        onChange={(e) => onChange(Number(e.target.value))}
      />
    </label>
  );
}

// A TextArg is either a literal string or a variable reference (an arg the
// caller sends). Toggle between "fixed text" and "from an input"; in input mode,
// pick from the automation's declared args (dropdown) — or type a name if none
// are declared yet.
function TextArgField({
  label,
  value,
  args,
  onChange,
}: {
  label: string;
  value: TextArg;
  args: string[];
  onChange: (v: TextArg) => void;
}) {
  const isVar = "Var" in value;
  const current = isVar ? value.Var : value.Literal;
  return (
    <div className="field">
      <span>{label}</span>
      <div className="textarg">
        <select
          value={isVar ? "var" : "literal"}
          onChange={(e) =>
            onChange(
              e.target.value === "var" ? { Var: current } : { Literal: current }
            )
          }
        >
          <option value="literal">fixed text</option>
          <option value="var">from an input</option>
        </select>

        {isVar && args.length > 0 ? (
          // Pick from the declared inputs.
          <select
            value={current}
            onChange={(e) => onChange({ Var: e.target.value })}
          >
            <option value="">choose an input…</option>
            {args.map((a) => (
              <option key={a} value={a}>
                {a}
              </option>
            ))}
          </select>
        ) : (
          <input
            value={current}
            placeholder={isVar ? "input name" : "text"}
            onChange={(e) =>
              onChange(isVar ? { Var: e.target.value } : { Literal: e.target.value })
            }
          />
        )}
      </div>
    </div>
  );
}

// --- per-action editors ----------------------------------------------------

function OpenEditor({
  value,
  args,
  onChange,
}: {
  value: OpenApps;
  args: string[];
  onChange: (v: OpenApps) => void;
}) {
  const empty: TextArg = { Literal: "" };
  const kind =
    value === "Outlook"
      ? "Outlook"
      : "Google" in value
        ? "Google"
        : "Explorer" in value
          ? "Explorer"
          : "RandomApp";

  return (
    <>
      <label className="field">
        <span>app</span>
        <select
          value={kind}
          onChange={(e) => {
            switch (e.target.value) {
              case "Outlook":
                return onChange("Outlook");
              case "Google":
                return onChange({ Google: "Chrome" });
              case "Explorer":
                return onChange({ Explorer: empty });
              case "RandomApp":
                return onChange({ RandomApp: empty });
            }
          }}
        >
          <option value="Outlook">Outlook</option>
          <option value="Google">Google (browser)</option>
          <option value="Explorer">File Explorer (path)</option>
          <option value="RandomApp">Custom command / URI</option>
        </select>
      </label>

      {typeof value === "object" && "Google" in value && (
        <label className="field">
          <span>which</span>
          <select
            value={typeof value.Google === "string" ? value.Google : "Search"}
            onChange={(e) => {
              const v = e.target.value;
              onChange({
                Google: v === "Search" ? { Search: empty } : (v as "Drive" | "Chrome"),
              });
            }}
          >
            <option value="Chrome">Google.com</option>
            <option value="Drive">Google Drive</option>
            <option value="Search">Custom URL</option>
          </select>
        </label>
      )}

      {typeof value === "object" &&
        "Google" in value &&
        typeof value.Google === "object" &&
        "Search" in value.Google && (
          <TextArgField
            label="URL"
            value={value.Google.Search}
            args={args}
            onChange={(v) => onChange({ Google: { Search: v } })}
          />
        )}

      {typeof value === "object" && "Explorer" in value && (
        <TextArgField
          label="folder path"
          value={value.Explorer}
          args={args}
          onChange={(v) => onChange({ Explorer: v })}
        />
      )}

      {typeof value === "object" && "RandomApp" in value && (
        <TextArgField
          label="command / URI"
          value={value.RandomApp}
          args={args}
          onChange={(v) => onChange({ RandomApp: v })}
        />
      )}
    </>
  );
}

function MouseEditor({
  value,
  onChange,
}: {
  value: MouseOptions;
  onChange: (v: MouseOptions) => void;
}) {
  const kind =
    typeof value === "string"
      ? value
      : "MoveTo" in value
        ? "MoveTo"
        : "Drag";

  return (
    <>
      <label className="field">
        <span>mouse</span>
        <select
          value={kind}
          onChange={(e) => {
            switch (e.target.value) {
              case "LeftClick":
                return onChange("LeftClick");
              case "RightClick":
                return onChange("RightClick");
              case "DoubleClick":
                return onChange("DoubleClick");
              case "MoveTo":
                return onChange({ MoveTo: [{ Var: "found" }, 0.2] });
              case "Drag":
                return onChange({ Drag: [{ Var: "found" }, 0.2] });
            }
          }}
        >
          <option value="LeftClick">Left click</option>
          <option value="RightClick">Right click</option>
          <option value="DoubleClick">Double click</option>
          <option value="MoveTo">Move to</option>
          <option value="Drag">Drag to</option>
        </select>
      </label>

      {typeof value === "object" && "MoveTo" in value && (
        <p className="hint">moves to the position stored by a Find-image step</p>
      )}
      {typeof value === "object" && "Drag" in value && (
        <p className="hint">drags to the position stored by a Find-image step</p>
      )}
    </>
  );
}

function KeyboardEditor({
  value,
  args,
  onChange,
}: {
  value: KeyboardOptions;
  args: string[];
  onChange: (v: KeyboardOptions) => void;
}) {
  const isInput = "Input" in value;
  // Grouped so the dropdown is navigable with 40+ keys.
  const KEY_GROUPS: { label: string; keys: Key[] }[] = [
    { label: "common", keys: ["Enter", "BackSpace", "Space", "Tab", "Escape", "Delete", "Insert"] },
    { label: "arrows", keys: ["LeftArrow", "RightArrow", "UpArrow", "DownArrow"] },
    { label: "navigation", keys: ["Home", "End", "PageUp", "PageDown"] },
    { label: "modifiers", keys: ["Shift", "Ctrl", "Alt", "Win"] },
    {
      label: "function",
      keys: [
        "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12",
        "F13", "F14", "F15", "F16", "F17", "F18", "F19", "F20", "F21", "F22", "F23", "F24",
      ],
    },
  ];

  return (
    <>
      <label className="field">
        <span>keyboard</span>
        <select
          value={isInput ? "Input" : "PressKey"}
          onChange={(e) =>
            onChange(
              e.target.value === "Input"
                ? { Input: { Literal: "" } }
                : { PressKey: "Enter" }
            )
          }
        >
          <option value="Input">Type text</option>
          <option value="PressKey">Press a key</option>
        </select>
      </label>

      {isInput && (
        <TextArgField
          label="text"
          value={value.Input}
          args={args}
          onChange={(v) => onChange({ Input: v })}
        />
      )}

      {"PressKey" in value && (
        <label className="field">
          <span>key</span>
          <select
            value={value.PressKey}
            onChange={(e) => onChange({ PressKey: e.target.value as Key })}
          >
            {KEY_GROUPS.map((g) => (
              <optgroup key={g.label} label={g.label}>
                {g.keys.map((k) => (
                  <option key={k} value={k}>
                    {k}
                  </option>
                ))}
              </optgroup>
            ))}
          </select>
        </label>
      )}
    </>
  );
}

// --- the switcher ----------------------------------------------------------

export function StepEditor({ action, onChange, appName, stepIndex, args }: Props) {
  if ("Open" in action) {
    return (
      <OpenEditor
        value={action.Open}
        args={args}
        onChange={(v) => onChange({ Open: v })}
      />
    );
  }
  if ("Mouse" in action) {
    return (
      <MouseEditor value={action.Mouse} onChange={(v) => onChange({ Mouse: v })} />
    );
  }
  if ("Keyboard" in action) {
    const kb = action.Keyboard;
    return (
      <>
        <KeyboardEditor
          value={kb.opts}
          args={args}
          onChange={(v) => onChange({ Keyboard: { ...kb, opts: v } })}
        />
        <NumberField
          label="repeat (times)"
          value={kb.repeat}
          onChange={(v) =>
            onChange({ Keyboard: { ...kb, repeat: Math.max(1, Math.floor(v)) } })
          }
        />
      </>
    );
  }
  if ("Sleep" in action) {
    return (
      <NumberField
        label="seconds"
        value={action.Sleep}
        step={0.1}
        onChange={(v) => onChange({ Sleep: v })}
      />
    );
  }
  if ("WaitForWindow" in action) {
    return (
      <>
        <TextArgField
          label="window title contains"
          value={action.WaitForWindow.title}
          args={args}
          onChange={(v) =>
            onChange({ WaitForWindow: { ...action.WaitForWindow, title: v } })
          }
        />
        <NumberField
          label="timeout (ms)"
          value={action.WaitForWindow.waited_ms}
          onChange={(v) =>
            onChange({ WaitForWindow: { ...action.WaitForWindow, waited_ms: v } })
          }
        />
      </>
    );
  }
  if ("MoveLastDownload" in action) {
    return (
      <>
        <TextArgField
          label="move to (destination path)"
          value={action.MoveLastDownload.to}
          args={args}
          onChange={(v) =>
            onChange({ MoveLastDownload: { ...action.MoveLastDownload, to: v } })
          }
        />
        <NumberField
          label="download timeout (ms)"
          value={action.MoveLastDownload.waited_ms}
          onChange={(v) =>
            onChange({
              MoveLastDownload: { ...action.MoveLastDownload, waited_ms: v },
            })
          }
        />
      </>
    );
  }
  if ("SetOutput" in action) {
    return (
      <TextArgField
        label="output value"
        value={action.SetOutput}
        args={args}
        onChange={(v) => onChange({ SetOutput: v })}
      />
    );
  }
  if ("ReadFileAsBase64" in action) {
    return (
      <>
        <TextArgField
          label="file path"
          value={action.ReadFileAsBase64.path}
          args={args}
          onChange={(v) =>
            onChange({
              ReadFileAsBase64: { ...action.ReadFileAsBase64, path: v },
            })
          }
        />
        <TextField
          label="content type"
          value={action.ReadFileAsBase64.content_type}
          onChange={(v) =>
            onChange({
              ReadFileAsBase64: { ...action.ReadFileAsBase64, content_type: v },
            })
          }
          placeholder="application/pdf"
        />
      </>
    );
  }
  if ("FindImageLoop" in action) {
    return (
      <FindImageEditor
        value={action.FindImageLoop}
        appName={appName}
        stepIndex={stepIndex}
        onChange={(v) => onChange({ FindImageLoop: v })}
      />
    );
  }

  return <p className="hint">no options for this action</p>;
}

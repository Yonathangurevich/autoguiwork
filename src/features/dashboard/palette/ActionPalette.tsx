import type { ActionsKind } from "../../../models";
import { ACTION_TEMPLATES } from "../../../services";
import "./ActionPalette.css";

// The right-side action nav: click a template to append it as a new step.
// The catalog itself lives in services/actionFactory.
export function ActionPalette({ onPick }: { onPick: (a: ActionsKind) => void }) {
  return (
    <nav className="palette">
      <h3 className="palette-title">actions</h3>
      {ACTION_TEMPLATES.map((item) => (
        <button
          key={item.label}
          className="palette-item"
          onClick={() => onPick(item.make())}
        >
          {item.label}
        </button>
      ))}
    </nav>
  );
}

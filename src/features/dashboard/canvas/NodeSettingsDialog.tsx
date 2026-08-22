import type { ActionsKind } from "../../../models";
import { describeAction } from "../../../services";
import { StepEditor } from "../steps/StepEditor";
import "./NodeSettingsDialog.css";

// A small modal for editing one node's settings. Reuses the existing per-action
// StepEditor forms — only the presentation moved from inline-expand to a dialog.
interface Props {
  action: ActionsKind;
  appName: string;
  stepIndex: number;
  args: string[];
  onChange: (a: ActionsKind) => void;
  onClose: () => void;
}

export function NodeSettingsDialog({
  action,
  appName,
  stepIndex,
  args,
  onChange,
  onClose,
}: Props) {
  return (
    // Click the dim backdrop to close; clicks inside the dialog don't bubble.
    <div className="dialog-backdrop" onClick={onClose}>
      <div className="dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-head">
          <span className="dialog-title">{describeAction(action)}</span>
          <button className="dialog-close" onClick={onClose}>
            ×
          </button>
        </div>

        <div className="dialog-body">
          <StepEditor
            action={action}
            appName={appName}
            stepIndex={stepIndex}
            args={args}
            onChange={onChange}
          />
        </div>

        <div className="dialog-foot">
          <button className="dialog-done" onClick={onClose}>
            done
          </button>
        </div>
      </div>
    </div>
  );
}

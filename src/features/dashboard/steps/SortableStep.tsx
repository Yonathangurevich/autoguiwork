import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import type { Action, ActionsKind } from "../../../models";
import { describeAction } from "../../../services";
import { StepEditor } from "./StepEditor";

// One step row in the sortable list. Drag is bound ONLY to the ⋮⋮ handle
// (via {...listeners}), so editing inputs inside an expanded step is never
// hijacked by the drag. dnd-kit uses pointer events, not native HTML5 drag,
// so there's no WebView2 "red-X" issue.
interface Props {
  id: string;
  index: number;
  step: Action;
  appName: string;
  args: string[];
  isOpen: boolean;
  onToggle: () => void;
  onRemove: () => void;
  onUpdate: (action: ActionsKind) => void;
}

export function SortableStep({
  id,
  index,
  step,
  appName,
  args,
  isOpen,
  onToggle,
  onRemove,
  onUpdate,
}: Props) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } =
    useSortable({ id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.4 : 1,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={`step ${isOpen ? "open" : ""} ${isDragging ? "dragging" : ""}`}
    >
      <div className="step-head">
        <span
          className="drag-handle"
          title="drag to reorder"
          {...attributes}
          {...listeners}
        >
          ⋮⋮
        </span>
        <span className="step-index">{index + 1}</span>
        <button className="step-label" onClick={onToggle}>
          {describeAction(step.action)}
        </button>
        <button className="step-remove" title="remove step" onClick={onRemove}>
          ×
        </button>
      </div>

      {isOpen && (
        <div className="step-body">
          <StepEditor
            action={step.action}
            appName={appName}
            stepIndex={index}
            args={args}
            onChange={onUpdate}
          />
        </div>
      )}
    </div>
  );
}

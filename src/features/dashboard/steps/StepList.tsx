import { useState } from "react";
import {
  DndContext,
  closestCenter,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from "@dnd-kit/core";
import {
  SortableContext,
  verticalListSortingStrategy,
  arrayMove,
} from "@dnd-kit/sortable";
import { restrictToVerticalAxis } from "@dnd-kit/modifiers";
import type { Action, ActionsKind } from "../../../models";
import { SortableStep } from "./SortableStep";
import "./StepList.css";

interface Props {
  appName: string;
  args: string[];
  actions: Action[];
  onRemove: (index: number) => void;
  onUpdate: (index: number, action: ActionsKind) => void;
  onMove: (from: number, to: number) => void;
}

export function StepList({
  appName,
  args,
  actions,
  onRemove,
  onUpdate,
  onMove,
}: Props) {
  // Which step is expanded for editing (only one at a time).
  const [openIndex, setOpenIndex] = useState<number | null>(null);

  // Require a small drag distance so a plain click on the handle doesn't
  // count as a drag.
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } })
  );

  // dnd-kit needs stable string ids; index-based ids are fine since the order
  // is what we reorder. ids map 1:1 to the current positions.
  const ids = actions.map((_, i) => `step-${i}`);

  function handleDragEnd(e: DragEndEvent) {
    const { active, over } = e;
    if (!over || active.id === over.id) return;
    const from = ids.indexOf(String(active.id));
    const to = ids.indexOf(String(over.id));
    if (from === -1 || to === -1) return;

    // Follow the open step to its new position; if a different step was open,
    // recompute where it landed after the move (so the right editor stays open).
    if (openIndex !== null) {
      const order = actions.map((_, i) => i);
      setOpenIndex(arrayMove(order, from, to).indexOf(openIndex));
    }
    onMove(from, to);
  }

  return (
    <div className="steps">
      {actions.length === 0 && (
        <p className="steps-empty">
          no steps yet — pick an action from the right to add one.
        </p>
      )}

      <DndContext
        sensors={sensors}
        collisionDetection={closestCenter}
        modifiers={[restrictToVerticalAxis]}
        onDragEnd={handleDragEnd}
      >
        <SortableContext items={ids} strategy={verticalListSortingStrategy}>
          {actions.map((step, i) => (
            <SortableStep
              key={ids[i]}
              id={ids[i]}
              index={i}
              step={step}
              appName={appName}
              args={args}
              isOpen={openIndex === i}
              onToggle={() => setOpenIndex(openIndex === i ? null : i)}
              onRemove={() => onRemove(i)}
              onUpdate={(a) => onUpdate(i, a)}
            />
          ))}
        </SortableContext>
      </DndContext>
    </div>
  );
}

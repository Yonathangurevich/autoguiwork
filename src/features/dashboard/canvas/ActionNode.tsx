import { Handle, Position, type NodeProps } from "@xyflow/react";
import { describeAction } from "../../../services";
import type { ActionsKind } from "../../../models";

// What we stash on each React Flow node.
export interface ActionNodeData {
  action?: ActionsKind;
  index: number | null; // step number when in the chain, null when parked
  onOpen: () => void;
  onRemove: () => void;
  [key: string]: unknown;
}

// One action card on the canvas.
//  - target handle (top): accepts ONE incoming wire
//  - source handle (bottom): allows ONE outgoing wire
// The single-outgoing rule is what keeps the graph a straight chain, so it
// stays equivalent to the engine's linear actions[].
export function ActionNode({ data, selected }: NodeProps) {
  const d = data as ActionNodeData;
  const parked = d.index === null;

  return (
    <div
      className={`node ${selected ? "selected" : ""} ${parked ? "parked" : ""}`}
      onDoubleClick={d.onOpen}
    >
      <Handle type="target" position={Position.Top} className="node-handle" />

      <div className="node-head">
        <span className="node-index">{parked ? "–" : d.index! + 1}</span>
        <span className="node-title">
          {d.action ? describeAction(d.action) : "empty"}
        </span>
        <button
          className="node-remove"
          title="remove node"
          onClick={(e) => {
            e.stopPropagation();
            d.onRemove();
          }}
        >
          ×
        </button>
      </div>

      <button className="node-settings" onClick={d.onOpen}>
        settings
      </button>

      <Handle type="source" position={Position.Bottom} className="node-handle" />
    </div>
  );
}

// The fixed anchor: where the chain begins. No target handle (nothing connects
// INTO Start) and it can't be removed.
export function StartNode() {
  return (
    <div className="node start-node">
      <div className="node-head">
        <span className="node-title">▶ Start</span>
      </div>
      <Handle type="source" position={Position.Bottom} className="node-handle" />
    </div>
  );
}

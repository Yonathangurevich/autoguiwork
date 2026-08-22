// The visual editor's layout — UI state only, saved to canvas.json.
// The Rust engine never sees this; it only ever reads the connected chain,
// which we derive into automation.json's actions[].
//
// Why this exists: a node can be "parked" on the canvas without being wired
// into the chain. Parked nodes are deliberately NOT actions, so they'd be lost
// on reload if we didn't remember them here. Same for x/y positions.

import type { ActionsKind } from "./action";

// The fixed anchor node. It has no action and can't be deleted or moved into
// the chain's middle — it just marks where the chain begins.
export const START_NODE_ID = "start";

export interface CanvasNode {
  /** Stable id used for edges; survives reordering. */
  id: string;
  /** Canvas position in pixels. */
  x: number;
  y: number;
  /** The action this node performs. Absent for the Start node. */
  action?: ActionsKind;
}

/** One wire: source node -> target node. A node may have at most ONE outgoing
 *  edge, which is what keeps the graph a single straight chain (no branching)
 *  and therefore equivalent to a linear actions[] array. */
export interface CanvasEdge {
  source: string;
  target: string;
}

export interface CanvasLayout {
  nodes: CanvasNode[];
  edges: CanvasEdge[];
}

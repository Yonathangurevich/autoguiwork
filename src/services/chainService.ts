// Turning the canvas graph into the linear actions[] the engine runs, and back.
//
// THE KEY IDEA: because every node may have at most ONE outgoing edge, the
// graph can only ever be a single straight chain. Walking it from the Start
// node yields an ordered list — and that list IS actions[]. Parked (unconnected)
// nodes simply aren't on the walk, so they never reach the engine.
//
//   [Start] -> A -> B -> C        parked: D
//               |    |    |
//         actions[0] [1]  [2]     (D is not an action)

import type { Action, ActionsKind, CanvasLayout, CanvasNode } from "../models";
import { START_NODE_ID } from "../models";

/** Walk the chain from Start and return the node ids in execution order
 *  (excluding Start itself). Parked nodes are not included. */
export function chainNodeIds(layout: CanvasLayout): string[] {
  // source id -> target id. One outgoing edge per node is enforced when
  // connecting, so a plain map is enough.
  const next = new Map<string, string>();
  for (const e of layout.edges) next.set(e.source, e.target);

  const order: string[] = [];
  const seen = new Set<string>([START_NODE_ID]);

  let current = next.get(START_NODE_ID);
  while (current) {
    // Defensive: a cycle would loop forever. Connecting rules prevent it, but
    // never trust the file on disk.
    if (seen.has(current)) break;
    seen.add(current);
    order.push(current);
    current = next.get(current);
  }
  return order;
}

/** The connected chain as the engine's actions[]. This is what gets written to
 *  automation.json — unchanged in shape from before the canvas existed. */
export function layoutToActions(layout: CanvasLayout): Action[] {
  const byId = new Map<string, CanvasNode>(layout.nodes.map((n) => [n.id, n]));
  return chainNodeIds(layout)
    .map((id) => byId.get(id))
    .filter((n): n is CanvasNode => !!n && !!n.action)
    .map((n) => ({ action: n.action as ActionsKind }));
}

/** True when a node is part of the chain (vs. parked on the canvas). */
export function isInChain(layout: CanvasLayout, nodeId: string): boolean {
  return chainNodeIds(layout).includes(nodeId);
}

/** Build a default layout for an automation that has no saved canvas yet:
 *  lay its existing actions out as a vertical chain under Start. */
export function defaultLayout(actions: Action[]): CanvasLayout {
  const nodes: CanvasNode[] = [
    { id: START_NODE_ID, x: 240, y: 40 },
    ...actions.map((a, i) => ({
      id: `n${i}`,
      x: 240,
      y: 160 + i * 120,
      action: a.action,
    })),
  ];

  // Wire Start -> n0 -> n1 -> ... in order.
  const edges = nodes.slice(0, -1).map((n, i) => ({
    source: n.id,
    target: nodes[i + 1].id,
  }));

  return { nodes, edges: actions.length ? edges : [] };
}

/** A fresh id that doesn't collide with existing nodes. */
export function newNodeId(layout: CanvasLayout): string {
  let i = layout.nodes.length;
  let id = `n${i}`;
  const taken = new Set(layout.nodes.map((n) => n.id));
  while (taken.has(id)) id = `n${++i}`;
  return id;
}

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  ReactFlow,
  Background,
  Controls,
  useNodesState,
  useEdgesState,
  type Connection,
  type Edge,
  type Node,
  type IsValidConnection,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";

import type { ActionsKind, CanvasLayout } from "../../../models";
import { START_NODE_ID } from "../../../models";
import { chainNodeIds } from "../../../services";
import { ActionNode, StartNode, type ActionNodeData } from "./ActionNode";
import { NodeSettingsDialog } from "./NodeSettingsDialog";
import "./AutomationCanvas.css";

interface Props {
  appName: string;
  args: string[];
  layout: CanvasLayout;
  /** Called only on COMMITTED changes (drag end, connect, disconnect, add,
   *  remove, edit) — never on every mouse-move. The parent persists and
   *  re-derives actions[] from the connected chain. */
  onLayoutChange: (next: CanvasLayout) => void;
}

const nodeTypes = { action: ActionNode, start: StartNode };

export function AutomationCanvas({
  appName,
  args,
  layout,
  onLayoutChange,
}: Props) {
  const [editingId, setEditingId] = useState<string | null>(null);

  // React Flow owns node/edge state LOCALLY so dragging and selecting are
  // instant — no parent re-render (and no disk write) per mouse-move. We push
  // upward only when a gesture completes. This is what keeps CPU flat.
  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);

  const chain = useMemo(() => chainNodeIds(layout), [layout]);

  const removeNode = useCallback(
    (id: string) => {
      onLayoutChange({
        nodes: layout.nodes.filter((n) => n.id !== id),
        edges: layout.edges.filter((e) => e.source !== id && e.target !== id),
      });
    },
    [layout, onLayoutChange]
  );

  // Rebuild React Flow's state when the layout changes STRUCTURALLY (nodes or
  // wires added/removed/edited). We deliberately key off structure, not
  // positions, so a drag in progress is never yanked out from under the user.
  const structureKey = useMemo(
    () =>
      JSON.stringify({
        n: layout.nodes.map((n) => [n.id, n.action]),
        e: layout.edges,
      }),
    [layout.nodes, layout.edges]
  );

  useEffect(() => {
    setNodes(
      layout.nodes.map((n) => {
        if (n.id === START_NODE_ID) {
          return {
            id: n.id,
            type: "start",
            position: { x: n.x, y: n.y },
            data: {},
            deletable: false,
          } as Node;
        }
        const idx = chain.indexOf(n.id);
        const data: ActionNodeData = {
          action: n.action,
          index: idx === -1 ? null : idx,
          onOpen: () => setEditingId(n.id),
          onRemove: () => removeNode(n.id),
        };
        return {
          id: n.id,
          type: "action",
          position: { x: n.x, y: n.y },
          data,
        } as Node;
      })
    );
    setEdges(
      layout.edges.map((e) => ({
        id: `${e.source}->${e.target}`,
        source: e.source,
        target: e.target,
        // NOT animated: animated edges run a forever-looping CSS animation on
        // every wire, which keeps the webview repainting and burns CPU.
        animated: false,
      }))
    );
    // structureKey covers node ids/actions and all edges; chain follows from it.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [structureKey, setNodes, setEdges]);

  // --- committing changes ------------------------------------------------

  // Keep a ref to the latest positions so commit handlers read fresh values.
  const nodesRef = useRef(nodes);
  nodesRef.current = nodes;

  /** Persist current positions plus whatever edge set we pass in. */
  const commit = useCallback(
    (nextEdges: CanvasLayout["edges"]) => {
      const positions = new Map(
        nodesRef.current.map((n) => [n.id, n.position])
      );
      onLayoutChange({
        nodes: layout.nodes.map((n) => {
          const p = positions.get(n.id);
          return p ? { ...n, x: p.x, y: p.y } : n;
        }),
        edges: nextEdges,
      });
    },
    [layout.nodes, onLayoutChange]
  );

  // Save positions ONCE, when the drag finishes — not during it.
  const onNodeDragStop = useCallback(() => {
    commit(layout.edges);
  }, [commit, layout.edges]);

  // THE RULE: one outgoing wire per node, one incoming per node, no cycles,
  // nothing into Start. React Flow calls this while you drag a wire, so an
  // invalid target simply won't connect — the UI can't create a branch.
  const isValidConnection: IsValidConnection = useCallback(
    (c) => {
      const { source, target } = c as Connection;
      if (!source || !target || source === target) return false;
      if (layout.edges.some((e) => e.source === source)) return false;
      if (layout.edges.some((e) => e.target === target)) return false;
      if (target === START_NODE_ID) return false;
      const next = new Map(layout.edges.map((e) => [e.source, e.target]));
      let cur = next.get(target);
      while (cur) {
        if (cur === source) return false;
        cur = next.get(cur);
      }
      return true;
    },
    [layout.edges]
  );

  const onConnect = useCallback(
    (c: Connection) => {
      if (!c.source || !c.target) return;
      commit([...layout.edges, { source: c.source, target: c.target }]);
    },
    [commit, layout.edges]
  );

  const onEdgesDelete = useCallback(
    (deleted: Edge[]) => {
      const gone = new Set(deleted.map((e) => `${e.source}->${e.target}`));
      commit(
        layout.edges.filter((e) => !gone.has(`${e.source}->${e.target}`))
      );
    },
    [commit, layout.edges]
  );

  const editing = layout.nodes.find((n) => n.id === editingId);

  return (
    <div className="canvas-wrap">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onNodeDragStop={onNodeDragStop}
        onConnect={onConnect}
        onEdgesDelete={onEdgesDelete}
        isValidConnection={isValidConnection}
        fitView
        proOptions={{ hideAttribution: true }}
      >
        <Background gap={16} />
        <Controls />
      </ReactFlow>

      {editing?.action && (
        <NodeSettingsDialog
          action={editing.action}
          appName={appName}
          stepIndex={Math.max(0, chain.indexOf(editing.id))}
          args={args}
          onChange={(a: ActionsKind) =>
            onLayoutChange({
              ...layout,
              nodes: layout.nodes.map((n) =>
                n.id === editing.id ? { ...n, action: a } : n
              ),
            })
          }
          onClose={() => setEditingId(null)}
        />
      )}
    </div>
  );
}

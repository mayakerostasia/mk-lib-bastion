import { useCallback, useEffect, useMemo, useState } from 'react';
import ReactFlow, {
  Node,
  Edge,
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  MarkerType,
  Position,
} from 'reactflow';
import 'reactflow/dist/style.css';
import { AgentMessage } from './types';

interface AgentGraphProps {
  messages: AgentMessage[];
}

export function AgentGraph({ messages }: AgentGraphProps) {
  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);

  // Update graph when messages change
  useEffect(() => {
    const agentMap = new Map<string, { count: number; lastSeen: number }>();
    const edgeMap = new Map<string, { count: number; performative: string; lastSeen: number }>();

    // Process all messages to build agent and edge data
    messages.forEach((msg) => {
      // Track sender
      const sender = agentMap.get(msg.from_agent) || { count: 0, lastSeen: 0 };
      agentMap.set(msg.from_agent, {
        count: sender.count + 1,
        lastSeen: Math.max(sender.lastSeen, msg.timestamp),
      });

      // Track receiver if present
      if (msg.to_agent) {
        const receiver = agentMap.get(msg.to_agent) || { count: 0, lastSeen: 0 };
        agentMap.set(msg.to_agent, {
          count: receiver.count,
          lastSeen: Math.max(receiver.lastSeen, msg.timestamp),
        });

        // Track edge
        const edgeKey = `${msg.from_agent}-${msg.to_agent}`;
        const edge = edgeMap.get(edgeKey) || { count: 0, performative: msg.performative, lastSeen: 0 };
        edgeMap.set(edgeKey, {
          count: edge.count + 1,
          performative: msg.performative,
          lastSeen: Math.max(edge.lastSeen, msg.timestamp),
        });
      }
    });

    // Convert to React Flow nodes
    const now = Date.now() / 1000;
    const newNodes: Node[] = Array.from(agentMap.entries()).map(([id, data], index) => {
      const isActive = now - data.lastSeen < 10; // Active if seen in last 10 seconds
      const angle = (index / agentMap.size) * 2 * Math.PI;
      const radius = 200;

      return {
        id,
        type: 'default',
        position: {
          x: 400 + radius * Math.cos(angle),
          y: 300 + radius * Math.sin(angle),
        },
        data: {
          label: (
            <div className="text-center">
              <div className="font-bold text-sm">{id}</div>
              <div className="text-xs text-gray-500">{data.count} msgs</div>
            </div>
          ),
        },
        style: {
          background: isActive ? '#10b981' : '#6b7280',
          color: 'white',
          border: '2px solid #374151',
          borderRadius: '8px',
          padding: '10px',
          fontSize: '12px',
        },
      };
    });

    // Convert to React Flow edges
    const newEdges: Edge[] = Array.from(edgeMap.entries()).map(([key, data]) => {
      const [source, target] = key.split('-');
      const isRecent = now - data.lastSeen < 5; // Recent if seen in last 5 seconds

      return {
        id: key,
        source,
        target,
        label: `${data.performative} (${data.count})`,
        animated: isRecent,
        style: { stroke: isRecent ? '#10b981' : '#6b7280', strokeWidth: 2 },
        markerEnd: {
          type: MarkerType.ArrowClosed,
          color: isRecent ? '#10b981' : '#6b7280',
        },
        labelStyle: { fontSize: 10, fill: '#374151' },
        labelBgStyle: { fill: 'white' },
      };
    });

    setNodes(newNodes);
    setEdges(newEdges);
  }, [messages, setNodes, setEdges]);

  return (
    <div style={{ width: '100%', height: '100%' }}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        fitView
      >
        <Background />
        <Controls />
        <MiniMap />
      </ReactFlow>
    </div>
  );
}

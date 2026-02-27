export interface AgentMessage {
  timestamp: number;
  from_agent: string;
  to_agent?: string;
  performative: string;
  message_id: string;
  conversation_id?: string;
}

export interface AgentNode {
  id: string;
  label: string;
  messageCount: number;
  lastSeen: number;
  status: 'active' | 'idle' | 'offline';
}

export interface MessageEdge {
  id: string;
  source: string;
  target: string;
  performative: string;
  count: number;
  lastMessage: number;
}

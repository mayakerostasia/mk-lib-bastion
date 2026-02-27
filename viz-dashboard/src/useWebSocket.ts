import { useEffect, useState, useCallback } from 'react';
import { AgentMessage } from './types';

export function useWebSocket(url: string) {
  const [messages, setMessages] = useState<AgentMessage[]>([]);
  const [isConnected, setIsConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const connect = useCallback(() => {
    console.log('Connecting to WebSocket:', url);
    const ws = new WebSocket(url);

    ws.onopen = () => {
      console.log('WebSocket connected');
      setIsConnected(true);
      setError(null);
    };

    ws.onmessage = (event) => {
      try {
        const message: AgentMessage = JSON.parse(event.data);
        console.log('Received message:', message);
        setMessages(prev => [...prev.slice(-99), message]); // Keep last 100 messages
      } catch (err) {
        console.error('Failed to parse message:', err);
      }
    };

    ws.onerror = (event) => {
      console.error('WebSocket error:', event);
      setError('WebSocket connection error');
    };

    ws.onclose = () => {
      console.log('WebSocket disconnected');
      setIsConnected(false);
      // Reconnect after 3 seconds
      setTimeout(connect, 3000);
    };

    return ws;
  }, [url]);

  useEffect(() => {
    const ws = connect();
    return () => {
      ws.close();
    };
  }, [connect]);

  return { messages, isConnected, error };
}

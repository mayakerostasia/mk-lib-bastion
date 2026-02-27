import { AgentGraph } from './AgentGraph';
import { useWebSocket } from './useWebSocket';

function App() {
  const WS_URL = import.meta.env.VITE_WS_URL || 'ws://localhost:3030/ws';
  const { messages, isConnected, error } = useWebSocket(WS_URL);

  return (
    <div className="w-full h-full flex flex-col">
      {/* Header */}
      <div className="bg-gray-800 text-white p-4 shadow-lg">
        <div className="flex justify-between items-center">
          <h1 className="text-2xl font-bold">Simian Agent Visualization</h1>
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-2">
              <div
                className={`w-3 h-3 rounded-full ${
                  isConnected ? 'bg-green-500' : 'bg-red-500'
                }`}
              />
              <span className="text-sm">
                {isConnected ? 'Connected' : 'Disconnected'}
              </span>
            </div>
            <div className="text-sm text-gray-300">
              {messages.length} messages
            </div>
          </div>
        </div>
        {error && (
          <div className="mt-2 text-sm text-red-400">
            Error: {error}
          </div>
        )}
      </div>

      {/* Graph */}
      <div className="flex-1 bg-gray-100">
        {isConnected || messages.length > 0 ? (
          <AgentGraph messages={messages} />
        ) : (
          <div className="flex items-center justify-center h-full text-gray-500">
            <div className="text-center">
              <div className="text-4xl mb-4">🔌</div>
              <div className="text-lg">Waiting for connection...</div>
              <div className="text-sm mt-2">WebSocket: {WS_URL}</div>
            </div>
          </div>
        )}
      </div>

      {/* Footer with legend */}
      <div className="bg-gray-800 text-white p-3 text-sm">
        <div className="flex gap-6">
          <div className="flex items-center gap-2">
            <div className="w-3 h-3 rounded-full bg-green-500" />
            <span>Active (last 10s)</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-3 h-3 rounded-full bg-gray-500" />
            <span>Idle</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-4 h-0.5 bg-green-500" />
            <span>Recent message (last 5s)</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-4 h-0.5 bg-gray-500" />
            <span>Older message</span>
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;

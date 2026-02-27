# Agent Visualization Dashboard

Real-time visualization of Simian agent communication using React Flow.

## Features

- 🔴 Live agent status (active/idle/offline)
- 📊 Message flow visualization with animated edges
- 🎯 Message count per agent
- 🔄 Auto-reconnecting WebSocket
- 📈 Mini-map and zoom controls
- 🎨 Beautiful React Flow UI

## Quick Start

```bash
# Install dependencies
npm install

# Run development server
npm run dev

# Build for production
npm run build
```

## Docker

```bash
# Build
docker build -t viz-dashboard .

# Run
docker run -p 5173:5173 -e VITE_WS_URL=ws://localhost:3030/ws viz-dashboard
```

## Environment Variables

- `VITE_WS_URL` - WebSocket URL (default: `ws://localhost:3030/ws`)

## How It Works

1. Connects to WebSocket bridge at `ws://localhost:3030/ws`
2. Receives agent messages in real-time
3. Builds graph of agents (nodes) and communication flows (edges)
4. Updates visualization with animations for recent activity

## Graph Colors

- 🟢 Green node/edge = Active (last 10s/5s)
- ⚪ Gray node/edge = Idle
- ➡️ Animated edges = Recent messages

Visit http://localhost:5173 to see the visualization!

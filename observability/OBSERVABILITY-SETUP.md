# Observability Setup Scripts

Quick PowerShell scripts to configure observability endpoints for simian-llm.

## Available Scripts

### 1. **setup-observability.ps1** - Flexible Local Setup

Most flexible option. Customize any endpoint and port.

**Usage:**
```powershell
# Default (localhost, all services)
. .\setup-observability.ps1

# Custom collector host
. .\setup-observability.ps1 -CollectorHost "192.168.1.100"

# Custom ports
. .\setup-observability.ps1 -CollectorPort 4318 -MetricPort 8888

# Docker Compose mode (uses container names)
. .\setup-observability.ps1 -Docker

# Production mode (listens on 0.0.0.0)
. .\setup-observability.ps1 -Production
```

**Parameters:**
- `-CollectorHost` (default: "localhost")
- `-CollectorPort` (default: 4317)
- `-LokiHost` (default: "localhost")
- `-LokiPort` (default: 3100)
- `-MetricBind` (default: "0.0.0.0")
- `-MetricPort` (default: 9090)
- `-Docker` (switch for Docker Compose)
- `-Production` (switch for production)

**What it sets:**
```
COLLECTOR_ENDPOINT = http://host:4317
LOKI_ENDPOINT      = http://host:3100
METRIC_BIND        = 0.0.0.0 or 127.0.0.1
METRIC_PORT        = 9090
```

---

### 2. **setup-observability-docker.ps1** - Docker Compose

Quick setup for Docker Compose stacks (uses container names).

**Usage:**
```powershell
. .\setup-observability-docker.ps1
```

**What it sets:**
```
COLLECTOR_ENDPOINT = http://otel:4317       # Docker container name
LOKI_ENDPOINT      = http://loki:3100       # Docker container name
METRIC_BIND        = 0.0.0.0
METRIC_PORT        = 9090
```

**Perfect for:**
```bash
docker-compose up -d
. .\setup-observability-docker.ps1
cargo run -p simian-llm
```

---

### 3. **setup-observability-prod.ps1** - Production Setup

Production-ready with connectivity validation.

**Usage:**
```powershell
# Production defaults (replace hostnames)
. .\setup-observability-prod.ps1 -CollectorHost "otel.example.com" -LokiHost "logs.example.com"

# With custom ports
. .\setup-observability-prod.ps1 `
    -CollectorHost "otel.example.com" `
    -CollectorPort 4317 `
    -LokiHost "logs.example.com" `
    -LokiPort 3100 `
    -MetricPort 9090
```

**What it sets:**
```
COLLECTOR_ENDPOINT = http://host:4317
LOKI_ENDPOINT      = http://host:3100
METRIC_BIND        = 0.0.0.0               # Accessible from network
METRIC_PORT        = 9090
```

**Features:**
- ✅ Validates connectivity before starting
- ✅ Warns about production requirements
- ✅ Suggests TLS configuration
- ✅ Reminds about secrets management

---

## Common Workflows

### Local Development (Console Only)
```powershell
# Services not running - everything logs to console
cargo test -p simian-llm
```

### Local Development (Docker Compose)
```powershell
# Start all services
docker-compose up -d

# Set up environment
. .\setup-observability-docker.ps1

# Run app
cargo run -p simian-llm --example end_to_end_lmstudio

# Access dashboards
# Grafana:    http://localhost:3000
# Prometheus: http://localhost:9090
```

### Local Development (Custom Network)
```powershell
# On a machine with services running at 192.168.1.100
. .\setup-observability.ps1 -CollectorHost "192.168.1.100" -LokiHost "192.168.1.100"

# Run app
cargo run -p simian-llm
```

### Production (Kubernetes/Cloud)
```powershell
# Set up for cloud infrastructure
. .\setup-observability-prod.ps1 `
    -CollectorHost "otel.observability.prod.svc.cluster.local" `
    -LokiHost "loki.observability.prod.svc.cluster.local"

# Deploy and run
docker build -t simian-llm:latest .
docker push simian-llm:latest
kubectl apply -f deployment.yaml
```

---

## What Gets Set

These scripts set PowerShell environment variables for the current session:

```powershell
$env:COLLECTOR_ENDPOINT
$env:LOKI_ENDPOINT
$env:METRIC_BIND
$env:METRIC_PORT
```

**These variables are used by:**
- `simian-tracing/src/telemetry.rs` - For OpenTelemetry and Loki
- `simian-metrics/src/lib.rs` - For Prometheus metrics

**Persistence:**
- ✅ Environment variables persist for your PowerShell session
- ❌ They're lost when you close PowerShell
- 💾 To persist between sessions, add to your PowerShell $PROFILE

---

## Making Setup Permanent

Add to your PowerShell profile (`$PROFILE`):

```powershell
# Observability setup
. "C:\path\to\repo\setup-observability.ps1" -Docker
```

Or set them directly:

```powershell
$env:COLLECTOR_ENDPOINT = "http://localhost:4317"
$env:LOKI_ENDPOINT = "http://localhost:3100"
$env:METRIC_BIND = "0.0.0.0"
$env:METRIC_PORT = 9090
```

---

## Troubleshooting

### Script Won't Run
```powershell
# Need to enable script execution
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### Endpoints Not Reachable
```powershell
# Check what was set
Write-Host $env:COLLECTOR_ENDPOINT
Write-Host $env:LOKI_ENDPOINT

# Test connectivity manually
curl http://localhost:3100/ready
curl http://localhost:9090/metrics
```

### Services Not Running
```powershell
# Check Docker containers
docker ps

# Start docker-compose stack
docker-compose up -d

# Wait for services to initialize
Start-Sleep -Seconds 10

# Re-run setup script
. .\setup-observability-docker.ps1
```

---

## Quick Reference

| Scenario | Command |
|----------|---------|
| Local dev, no services | `cargo test -p simian-llm` |
| Local dev, Docker | `. .\setup-observability-docker.ps1` |
| Local dev, custom host | `. .\setup-observability.ps1 -CollectorHost "192.168.1.100"` |
| Production | `. .\setup-observability-prod.ps1 -CollectorHost "prod.example.com"` |

---

## Environment Variables Reference

| Variable | Default | Used By | Example |
|----------|---------|---------|---------|
| COLLECTOR_ENDPOINT | `http://localhost:4317` | simian-tracing (tracer) | `http://otel.example.com:4317` |
| LOKI_ENDPOINT | `http://localhost:3100` | simian-tracing (logger) | `http://logs.example.com:3100` |
| METRIC_BIND | `127.0.0.1` | simian-metrics | `0.0.0.0` |
| METRIC_PORT | `9090` | simian-metrics | `8888` |

---

## Next Steps

1. ✅ Choose your setup script
2. ✅ Run it with appropriate parameters
3. ✅ Start observability services (if using Docker)
4. ✅ Run your simian-llm application
5. ✅ Access dashboards (Grafana, Prometheus)

Happy observing! 🔍

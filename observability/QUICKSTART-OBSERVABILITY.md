# 🚀 Quick Start: Observability Stack for simian-llm

Complete setup with PowerShell scripts and Docker Compose configuration.

## ⚡ 30-Second Quick Start

```powershell
# 1. Start all observability services
docker-compose up -d

# 2. Set environment variables
. .\setup-observability-docker.ps1

# 3. Run your app
cargo run -p simian-llm --example end_to_end_lmstudio

# 4. View traces and logs
# Open browser to:
# Grafana:     http://localhost:3000 (admin/admin)
# Prometheus:  http://localhost:9090
# App Metrics: http://localhost:9090/metrics
```

Done! Your app is now fully instrumented. ✅

---

## 📦 What's Included

### PowerShell Setup Scripts

1. **setup-observability.ps1** - Flexible, customizable
   ```powershell
   . .\setup-observability.ps1
   . .\setup-observability.ps1 -CollectorHost "192.168.1.100"
   . .\setup-observability.ps1 -Docker
   ```

2. **setup-observability-docker.ps1** - Docker Compose optimized
   ```powershell
   . .\setup-observability-docker.ps1
   ```

3. **setup-observability-prod.ps1** - Production ready
   ```powershell
   . .\setup-observability-prod.ps1 -CollectorHost "otel.example.com"
   ```

### Docker Compose Services

- **OpenTelemetry Collector** - Receives and exports traces
- **Loki** - Log aggregation
- **Prometheus** - Metrics storage
- **Grafana** - Visualization dashboard

### Configuration Files

- `docker-compose.yml` - Full stack definition
- `otel-collector-config.yaml` - OTEL configuration
- `loki-config.yaml` - Loki log storage config
- `prometheus.yml` - Prometheus scrape config
- `grafana-datasources.yaml` - Grafana data sources
- `grafana-dashboards.yaml` - Grafana dashboards

---

## 🎯 Common Workflows

### Development (No External Services)
```powershell
# Everything logs to console, no external services needed
cargo test -p simian-llm
```

### Development (Docker Stack)
```powershell
# Start all services
docker-compose up -d

# Set up environment (uses container names)
. .\setup-observability-docker.ps1

# Run app
cargo run -p simian-llm --example end_to_end_lmstudio

# View in Grafana
Start-Process "http://localhost:3000"
```

### Development (Remote Services)
```powershell
# Services running on 192.168.1.100
. .\setup-observability.ps1 -CollectorHost "192.168.1.100" -LokiHost "192.168.1.100"

# Run app
cargo run -p simian-llm
```

### Production
```powershell
# Cloud-hosted services
. .\setup-observability-prod.ps1 `
    -CollectorHost "otel.company.com" `
    -LokiHost "logs.company.com"

# Deploy
docker build -t simian-llm:latest .
```

---

## 📋 Setup Steps

### 1. Check Prerequisites
```powershell
# Verify Docker is installed and running
docker --version
docker ps

# Verify PowerShell (5.0+)
$PSVersionTable.PSVersion
```

### 2. Navigate to Repository
```powershell
cd C:\Users\njtwi\work_repos\rust_testing\mk-lib-bastion
```

### 3. Start Services
```powershell
# Start Docker Compose stack
docker-compose up -d

# Wait for services to initialize
Start-Sleep -Seconds 10

# Verify services are running
docker ps | grep -E "otel|loki|prometheus|grafana"
```

### 4. Set Environment Variables
```powershell
# Choose one based on your setup

# For Docker Compose (recommended for local dev)
. .\setup-observability-docker.ps1

# For custom hosts
. .\setup-observability.ps1 -CollectorHost "your-host"

# For production
. .\setup-observability-prod.ps1 -CollectorHost "prod-host"
```

### 5. Verify Configuration
```powershell
# Check what was set
Write-Host "COLLECTOR_ENDPOINT: $env:COLLECTOR_ENDPOINT"
Write-Host "LOKI_ENDPOINT: $env:LOKI_ENDPOINT"
Write-Host "METRIC_BIND: $env:METRIC_BIND"
Write-Host "METRIC_PORT: $env:METRIC_PORT"

# Test connectivity
curl http://localhost:3100/ready     # Loki
curl http://localhost:9090/metrics   # Prometheus
```

### 6. Run Your App
```powershell
# Examples
cargo run -p simian-llm --example end_to_end_lmstudio
cargo run -p simian-llm --example spawn_agents

# Or tests
cargo test -p simian-llm
```

### 7. Access Dashboards
```powershell
# Open Grafana (admin/admin password)
Start-Process "http://localhost:3000"

# Open Prometheus metrics browser
Start-Process "http://localhost:9090"

# View your app's metrics
Start-Process "http://localhost:9090/metrics"
```

---

## 🔍 Environment Variables

These are automatically set by the scripts:

| Variable | Default | What It Does |
|----------|---------|--------------|
| `COLLECTOR_ENDPOINT` | `http://localhost:4317` | Where to send traces (OTEL gRPC) |
| `LOKI_ENDPOINT` | `http://localhost:3100` | Where to send logs |
| `METRIC_BIND` | `0.0.0.0` | IP to listen on for metrics |
| `METRIC_PORT` | `9090` | Port to listen on for metrics |

Your app reads these and connects automatically.

---

## 📊 What Gets Collected

### Traces 🔗
- Agent creation and lifecycle
- Request handling and latency
- LLM API calls and responses
- Registry operations
- Error paths and exceptions

**View in:** Grafana → Explore → Select Trace datasource

### Logs 📝
- Structured log entries with fields
- Service name, version, environment
- Process ID
- Severity levels (DEBUG, INFO, WARN, ERROR)

**View in:** Grafana → Explore → Select Loki datasource

### Metrics 📈
- Request counters
- Response latency (histograms)
- Active connections (gauges)
- Error rates
- Token usage (for LLM)

**View in:** Prometheus → Graph tab or Grafana dashboards

---

## 🛑 Troubleshooting

### Services Won't Start
```powershell
# Check Docker
docker ps
docker logs otel
docker logs loki
docker logs prometheus

# Restart services
docker-compose restart

# Full restart (remove volumes)
docker-compose down -v
docker-compose up -d
```

### PowerShell Script Won't Run
```powershell
# Enable script execution
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser

# Run script again
. .\setup-observability-docker.ps1
```

### Can't Connect to Services
```powershell
# Check what was set
$env:COLLECTOR_ENDPOINT
$env:LOKI_ENDPOINT

# Test connectivity
curl http://localhost:3100/ready
curl http://localhost:9090/metrics

# If using container names from outside Docker:
# Use host.docker.internal instead
. .\setup-observability.ps1 -CollectorHost "host.docker.internal"
```

### No Traces/Logs/Metrics Showing
1. Verify app is running with correct env vars
2. Check app doesn't have errors (look at console)
3. Wait 10-15 seconds for data to appear
4. Refresh Grafana dashboard
5. Check COLLECTOR_ENDPOINT is reachable

---

## 🧹 Cleanup

### Stop Services (Keep Data)
```powershell
docker-compose stop
```

### Stop and Remove (Delete Data)
```powershell
docker-compose down
docker volume rm docker_compose_dir_loki-storage
docker volume rm docker_compose_dir_prometheus-storage
docker volume rm docker_compose_dir_grafana-storage
```

### Clean PowerShell Session
```powershell
# Remove env vars (or close PowerShell)
Remove-Item Env:COLLECTOR_ENDPOINT
Remove-Item Env:LOKI_ENDPOINT
Remove-Item Env:METRIC_BIND
Remove-Item Env:METRIC_PORT
```

---

## 📚 Additional Resources

- **OpenTelemetry:** https://opentelemetry.io/docs/
- **Loki:** https://grafana.com/docs/loki/
- **Prometheus:** https://prometheus.io/docs/
- **Grafana:** https://grafana.com/docs/grafana/

---

## 🎓 Next Steps

1. ✅ Run `setup-observability-docker.ps1`
2. ✅ Start your app
3. ✅ View traces in Grafana
4. ✅ Check metrics in Prometheus
5. ✅ Create custom dashboards
6. ✅ Set up alerts

---

**Ready? Let's go!**

```powershell
docker-compose up -d
. .\setup-observability-docker.ps1
cargo run -p simian-llm --example end_to_end_lmstudio
```

🚀 Welcome to observable systems!

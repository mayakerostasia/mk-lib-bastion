# 📊 Observability Setup for simian-llm

Complete observability stack with traces, logs, and metrics.

## 📁 Directory Structure

```
observability/
├── README.md                          # This file
├── setup-observability.ps1            # Main setup script (flexible)
├── setup-observability-docker.ps1     # Docker Compose quick setup
├── setup-observability-prod.ps1       # Production deployment
├── alloy-config.alloy                 # Grafana Alloy configuration
├── loki-config.yaml                   # Loki log storage config
├── prometheus.yml                     # Prometheus metrics config
├── grafana-datasources.yaml           # Grafana data sources
├── grafana-dashboards.yaml            # Grafana dashboard provisioning
├── QUICKSTART-OBSERVABILITY.md        # 30-second getting started
├── OBSERVABILITY-SETUP.md             # Detailed setup guide
└── OBSERVABILITY-CHECKLIST.md         # Verification checklist
```

## 🚀 Quick Start

### From Repository Root

```powershell
# Docker Compose setup (recommended for local dev)
. setup-observability.ps1 -Docker

# Custom host setup
. setup-observability.ps1 -CollectorHost "192.168.1.100"

# Production setup
. setup-observability.ps1 -Production -CollectorHost "prod.example.com"
```

### From Observability Directory

```powershell
cd observability

# Flexible setup
. .\setup-observability.ps1

# Docker Compose
. .\setup-observability-docker.ps1

# Production
. .\setup-observability-prod.ps1
```

## 📚 Documentation

- **QUICKSTART-OBSERVABILITY.md** - Get running in 30 seconds
- **OBSERVABILITY-SETUP.md** - Complete setup reference
- **OBSERVABILITY-CHECKLIST.md** - Verification steps

## 🔧 Configuration Files

All configuration files use relative paths from the repository root:

- `docker-compose.yml` - Uses `./observability/*.yaml`
- Services use `/etc/...` paths inside containers

No manual path updates needed - everything is configured correctly.

## 🐳 Docker Compose Services

```yaml
alloy       - Grafana Alloy telemetry collector (port 4317)
loki        - Log aggregation (port 3100)
prometheus  - Metrics storage (port 9090)
grafana     - Visualization (port 3000)
```

Start with:
```bash
docker-compose up -d
```

## ✨ Environment Variables Set

| Variable | Default | Purpose |
|----------|---------|---------|
| `COLLECTOR_ENDPOINT` | `http://alloy:4317` | Traces to Alloy |
| `LOKI_ENDPOINT` | `http://loki:3100` | Logs |
| `METRIC_BIND` | `0.0.0.0` | Metrics listen |
| `METRIC_PORT` | `9090` | Metrics port |

## 🎯 Next Steps

1. Run setup script: `. setup-observability.ps1 -Docker`
2. Start services: `docker-compose up -d`
3. Run your app: `cargo run -p simian-llm`
4. View Grafana: http://localhost:3000

## 📖 See Also

- `../docker-compose.yml` - Main Docker Compose file (references observability/)
- `../README.md` - Repository README
- `../OBSERVABILITY-SETUP.md` - In root for reference

---

Everything is clean, organized, and ready to use! 🎉

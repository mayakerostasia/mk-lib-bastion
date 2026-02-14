# End-to-End Observability Checklist

Quick checklist to verify your observability stack is working correctly.

## Pre-Flight Checklist

- [ ] Docker is installed and running
- [ ] PowerShell 5.0+ available
- [ ] At least 4GB RAM available
- [ ] Ports 3000, 3100, 4317, 9090 are available

## Setup Checklist

### Step 1: Start Services
```powershell
# Start all services
docker-compose up -d

# ✅ Check all services started
docker ps
# You should see: otel, loki, prometheus, grafana containers
```

- [ ] OTEL container running
- [ ] Loki container running
- [ ] Prometheus container running
- [ ] Grafana container running

### Step 2: Configure Environment
```powershell
# Run setup script
. .\setup-observability-docker.ps1

# ✅ Verify output
# Should show all environment variables set with values
```

- [ ] COLLECTOR_ENDPOINT set to http://otel:4317
- [ ] LOKI_ENDPOINT set to http://loki:3100
- [ ] METRIC_BIND set to 0.0.0.0
- [ ] METRIC_PORT set to 9090

### Step 3: Run Application
```powershell
# Run example
cargo run -p simian-llm --example end_to_end_lmstudio

# ✅ Application should start with no connection errors
# Look for messages like:
# [INFO] Creating LlmAgent
# [INFO] Sending completion request to LmStudio
```

- [ ] App starts without errors
- [ ] App connects to LmStudio successfully
- [ ] Traces are being sent (check logs)

### Step 4: Verify Data Collection

#### Check Prometheus Metrics
```powershell
# Open in browser
Start-Process "http://localhost:9090/metrics"

# ✅ Should see metrics like:
# agent.created
# agent.requests
# llm.completion.requests
```

- [ ] Metrics endpoint responds
- [ ] Metrics data is populated
- [ ] Your app's metrics are visible

#### Check Loki Logs
```powershell
# Open Grafana
Start-Process "http://localhost:3000"
# Login: admin / admin
# Explore → Select Loki → Run query: {service_name="simian-llm"}
```

- [ ] Grafana accessible
- [ ] Can login with admin/admin
- [ ] Loki data source configured
- [ ] Logs appear in Loki

#### Check Prometheus
```powershell
# In Prometheus UI (http://localhost:9090)
# Graph tab → Select a metric like "agent_created"
# Should show graph with data points
```

- [ ] Prometheus UI accessible
- [ ] Metrics graphs appear
- [ ] Data is being scraped

### Step 5: Create Dashboard

In Grafana:

1. Home → Create → Dashboard
2. Add panel → Graph
3. Select Prometheus as datasource
4. Choose metric: `agent_created` or `llm_completion_requests`
5. See real-time graph

- [ ] Can create new dashboard
- [ ] Metrics show in graphs
- [ ] Real-time updates work

---

## Health Checks

Run these commands to verify services:

### Loki Health
```powershell
curl http://localhost:3100/ready
# Should respond with "ready"
```
- [ ] Loki responds to health check

### Prometheus Health
```powershell
curl http://localhost:9090/-/healthy
# Should respond with HTTP 200
```
- [ ] Prometheus responds to health check

### OpenTelemetry Collector Health
```powershell
# OTEL uses gRPC, harder to test with curl
# Check logs instead:
docker logs otel
# Should not show errors
```
- [ ] OTEL logs don't show errors

### Grafana Health
```powershell
curl http://localhost:3000/api/health
# Should respond with JSON health info
```
- [ ] Grafana responds to health check

---

## Verification Workflows

### Workflow 1: End-to-End Data Flow

1. ✅ Run app
2. ✅ App sends trace to OTEL (check logs for "Initializing telemetry")
3. ✅ OTEL receives trace (check: `docker logs otel | grep -i received`)
4. ✅ App sends logs to Loki (check: Grafana Loki data source)
5. ✅ App exports metrics to Prometheus (check: http://localhost:9090/metrics)

### Workflow 2: Agent Lifecycle Tracing

1. ✅ Run example with agent creation
2. ✅ Watch for spans: `agent_lifecycle`, `agent_creation`
3. ✅ In Grafana → Explore → Traces (if OTEL exports to Jaeger)
4. ✅ See full span hierarchy and timing

### Workflow 3: Request Tracing

1. ✅ Run example that sends requests
2. ✅ Watch metrics: `agent.requests`, `llm.completion.requests`
3. ✅ Check response times: `agent.response_time_ms`
4. ✅ Verify in Prometheus graph

### Workflow 4: Error Tracing

1. ✅ Trigger an error (e.g., LmStudio not running)
2. ✅ App logs error with context
3. ✅ Error appears in Loki logs
4. ✅ Error metric incremented: `llm.chat.failures`

---

## Common Issues & Solutions

### Issue: Services won't start
**Solution:**
```powershell
docker-compose down -v
docker-compose up -d
```

### Issue: Can't connect to services
**Solution:**
```powershell
# Check if services are running
docker ps

# If not, restart
docker-compose restart

# Try again after 10 seconds
Start-Sleep -Seconds 10
```

### Issue: No data in Prometheus
**Solution:**
1. Verify METRIC_BIND=0.0.0.0 (check: `$env:METRIC_BIND`)
2. Verify app is running with correct env vars
3. Wait 15 seconds for first scrape
4. Check Prometheus targets: http://localhost:9090/targets

### Issue: No logs in Loki
**Solution:**
1. Verify LOKI_ENDPOINT is set
2. Check app logs for errors during startup
3. Wait 5 seconds and refresh
4. In Loki, use query: `{job="simian-llm"}` or `{service_name="simian-llm"}`

### Issue: PowerShell script won't run
**Solution:**
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
. .\setup-observability-docker.ps1
```

---

## Success Criteria

You've successfully set up observability when:

- ✅ All 4 containers (otel, loki, prometheus, grafana) are running
- ✅ Environment variables are set for traces, logs, and metrics
- ✅ Your app starts without connection errors
- ✅ Prometheus shows `/metrics` endpoint
- ✅ Grafana shows data from Prometheus
- ✅ Grafana shows logs from Loki
- ✅ You can see real-time data in dashboards
- ✅ You can query metrics and logs

---

## What's Being Collected

### From Your Agent
- **Traces:** Creation, requests, responses, errors
- **Logs:** INFO, DEBUG, WARN, ERROR with structured fields
- **Metrics:** Requests, latency, tokens, subscriptions, errors

### Example Queries

**Prometheus:**
```
agent.created{agent_id="analyzer-01"}
llm.completion.latency_ms
agent.response_time_ms
rate(llm.completion.requests[5m])  # 5-minute rate
```

**Loki:**
```
{service_name="simian-llm"}
{service_name="simian-llm"} |= "error"
{service_name="simian-llm", agent_id="analyzer-01"}
```

---

## Performance Baseline

With default configuration:

| Component | CPU | Memory | Disk |
|-----------|-----|--------|------|
| Prometheus | ~2% | 200MB | 100MB |
| Loki | ~1% | 150MB | 50MB |
| Grafana | ~1% | 100MB | 20MB |
| OTEL | ~2% | 100MB | 10MB |
| **Total** | **~6%** | **550MB** | **180MB** |

For production, increase resources 2-3x.

---

## Next Steps

1. ✅ Complete all checklist items
2. ✅ Create custom dashboards in Grafana
3. ✅ Set up alerts in Prometheus
4. ✅ Export dashboards for version control
5. ✅ Document SLOs and error budgets

---

## Support & Docs

- **QUICKSTART-OBSERVABILITY.md** - Quick start guide
- **OBSERVABILITY-SETUP.md** - Detailed setup options
- **observability-endpoint-config.md** - Configuration reference
- **observability-audit.md** - Detailed audit findings

---

**You're all set! Start collecting observability data from your simian-llm agents.** 🚀

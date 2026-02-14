# Set up observability environment variables for simian-llm
# Usage: . .\setup-observability.ps1
#        or .\setup-observability.ps1

param(
    [string]$CollectorHost = "localhost",
    [int]$CollectorPort = 4317,
    [string]$LokiHost = "localhost",
    [int]$LokiPort = 3100,
    [string]$MetricBind = "0.0.0.0",
    [int]$MetricPort = 9090,
    [switch]$Docker = $false,
    [switch]$Production = $false
)

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  Observability Environment Setup" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# Override for Docker Compose
if ($Docker) {
    Write-Host "📦 Docker Compose mode detected`n" -ForegroundColor Yellow
    $CollectorHost = "otel"
    $LokiHost = "loki"
}

# Override for Production
if ($Production) {
    Write-Host "🏢 Production mode detected`n" -ForegroundColor Yellow
    $MetricBind = "0.0.0.0"
}

# Build endpoints
$CollectorEndpoint = "http://${CollectorHost}:${CollectorPort}"
$LokiEndpoint = "http://${LokiHost}:${LokiPort}"

# Set environment variables
Write-Host "Setting environment variables..." -ForegroundColor Green

$env:COLLECTOR_ENDPOINT = $CollectorEndpoint
$env:LOKI_ENDPOINT = $LokiEndpoint
$env:METRIC_BIND = $MetricBind
$env:METRIC_PORT = $MetricPort

# Display what was set
Write-Host "`n✅ Environment variables configured:" -ForegroundColor Green
Write-Host "   COLLECTOR_ENDPOINT = $CollectorEndpoint" -ForegroundColor Cyan
Write-Host "   LOKI_ENDPOINT      = $LokiEndpoint" -ForegroundColor Cyan
Write-Host "   METRIC_BIND        = $MetricBind" -ForegroundColor Cyan
Write-Host "   METRIC_PORT        = $MetricPort" -ForegroundColor Cyan

# Show how to use
Write-Host "`n📋 Next steps:" -ForegroundColor Magenta
Write-Host "   1. Verify services are running:"
Write-Host "      docker ps | grep -E 'otel|loki|prometheus'" -ForegroundColor Gray
Write-Host "`n   2. Run your application:" -ForegroundColor Magenta
Write-Host "      cargo run -p simian-llm --example end_to_end_lmstudio" -ForegroundColor Gray
Write-Host "`n   3. Access observability dashboards:" -ForegroundColor Magenta
Write-Host "      Grafana:     http://localhost:3000" -ForegroundColor Gray
Write-Host "      Prometheus:  http://localhost:9090" -ForegroundColor Gray
Write-Host "      App Metrics: http://localhost:9090/metrics" -ForegroundColor Gray

# Optionally test connectivity
Write-Host "`n🔍 Testing connectivity..." -ForegroundColor Yellow

try {
    $loki_test = curl -s http://localhost:3100/ready -ErrorAction SilentlyContinue
    if ($loki_test) {
        Write-Host "   ✅ Loki is responding" -ForegroundColor Green
    } else {
        Write-Host "   ❌ Loki not responding at $LokiEndpoint" -ForegroundColor Yellow
    }
} catch {
    Write-Host "   ⚠️  Could not test Loki connectivity" -ForegroundColor Yellow
}

try {
    $prometheus_test = curl -s http://localhost:9090/metrics -ErrorAction SilentlyContinue
    if ($prometheus_test) {
        Write-Host "   ✅ Prometheus is responding" -ForegroundColor Green
    } else {
        Write-Host "   ⚠️  Prometheus not responding (may not be running yet)" -ForegroundColor Yellow
    }
} catch {
    Write-Host "   ⚠️  Could not test Prometheus connectivity" -ForegroundColor Yellow
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  Setup Complete!" -ForegroundColor Green
Write-Host "========================================`n" -ForegroundColor Cyan

# Production-mode observability setup
# Usage: . .\setup-observability-prod.ps1 -CollectorHost "otel.example.com" -LokiHost "loki.example.com"

param(
    [Parameter(Mandatory=$false)]
    [string]$CollectorHost = "observability.prod.example.com",
    
    [Parameter(Mandatory=$false)]
    [int]$CollectorPort = 4317,
    
    [Parameter(Mandatory=$false)]
    [string]$LokiHost = "logs.prod.example.com",
    
    [Parameter(Mandatory=$false)]
    [int]$LokiPort = 3100,
    
    [Parameter(Mandatory=$false)]
    [int]$MetricPort = 9090
)

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  Production Observability Setup" -ForegroundColor Red
Write-Host "========================================`n" -ForegroundColor Cyan

Write-Host "⚠️  PRODUCTION MODE" -ForegroundColor Red
Write-Host "   Ensure all hostnames are DNS-resolvable`n" -ForegroundColor Yellow

$CollectorEndpoint = "http://${CollectorHost}:${CollectorPort}"
$LokiEndpoint = "http://${LokiHost}:${LokiPort}"

# Set environment variables
$env:COLLECTOR_ENDPOINT = $CollectorEndpoint
$env:LOKI_ENDPOINT = $LokiEndpoint
$env:METRIC_BIND = "0.0.0.0"
$env:METRIC_PORT = $MetricPort

Write-Host "✅ Production environment variables configured:" -ForegroundColor Green
Write-Host "   COLLECTOR_ENDPOINT = $CollectorEndpoint" -ForegroundColor Cyan
Write-Host "   LOKI_ENDPOINT      = $LokiEndpoint" -ForegroundColor Cyan
Write-Host "   METRIC_BIND        = 0.0.0.0" -ForegroundColor Cyan
Write-Host "   METRIC_PORT        = $MetricPort" -ForegroundColor Cyan

Write-Host "`n🔍 Verifying connectivity..." -ForegroundColor Yellow

# Test OTEL
Write-Host "   Testing OpenTelemetry collector..." -ForegroundColor Gray
try {
    $test = Invoke-WebRequest "http://${CollectorHost}:${CollectorPort}" -ErrorAction SilentlyContinue
    Write-Host "   ✅ OTEL collector responding" -ForegroundColor Green
} catch {
    Write-Host "   ⚠️  OTEL collector not immediately accessible (may fail at runtime)" -ForegroundColor Yellow
}

# Test Loki
Write-Host "   Testing Loki..." -ForegroundColor Gray
try {
    $test = Invoke-WebRequest "http://${LokiHost}:${LokiPort}/ready" -ErrorAction SilentlyContinue
    Write-Host "   ✅ Loki responding" -ForegroundColor Green
} catch {
    Write-Host "   ⚠️  Loki not immediately accessible (may fail at runtime)" -ForegroundColor Yellow
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  Production Setup Complete!" -ForegroundColor Green
Write-Host "========================================`n" -ForegroundColor Cyan
Write-Host "📝 Remember to:" -ForegroundColor Magenta
Write-Host "   • Use TLS in production (update script for grpcs://)" -ForegroundColor Gray
Write-Host "   • Store secrets in a secrets manager" -ForegroundColor Gray
Write-Host "   • Monitor collector and Loki health" -ForegroundColor Gray
Write-Host "   • Set up log retention policies" -ForegroundColor Gray
Write-Host "`n" -ForegroundColor Cyan

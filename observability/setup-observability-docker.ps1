# Quick setup for Docker Compose observability stack
# Usage: . .\setup-observability-docker.ps1

param(
    [switch]$Test = $false
)

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  Docker Compose Observability Setup" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# Docker Compose defaults
$env:COLLECTOR_ENDPOINT = "http://alloy:4317"
$env:LOKI_ENDPOINT = "http://loki:3100"
$env:METRIC_BIND = "0.0.0.0"
$env:METRIC_PORT = 9090

Write-Host "✅ Environment variables set for Docker Compose:" -ForegroundColor Green
Write-Host "   COLLECTOR_ENDPOINT = http://alloy:4317" -ForegroundColor Cyan
Write-Host "   LOKI_ENDPOINT      = http://loki:3100" -ForegroundColor Cyan
Write-Host "   METRIC_BIND        = 0.0.0.0" -ForegroundColor Cyan
Write-Host "   METRIC_PORT        = 9090" -ForegroundColor Cyan

Write-Host "`n📋 Next steps:" -ForegroundColor Magenta
Write-Host "   1. Start Docker Compose services:" -ForegroundColor Magenta
Write-Host "      docker-compose up -d" -ForegroundColor Gray
Write-Host "`n   2. Wait a moment for services to start" -ForegroundColor Magenta
Write-Host "      sleep 10" -ForegroundColor Gray
Write-Host "`n   3. Run your application:" -ForegroundColor Magenta
Write-Host "      cargo run -p simian-llm --example end_to_end_lmstudio" -ForegroundColor Gray
Write-Host "`n========================================`n" -ForegroundColor Cyan

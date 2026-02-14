# Setup observability for simian-llm
# This is the main entry point - delegates to specific setup scripts in ./observability/

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

# Check if observability directory exists
if (-not (Test-Path "observability")) {
    Write-Host "❌ Error: observability directory not found!" -ForegroundColor Red
    Write-Host "Please run this script from the repository root." -ForegroundColor Yellow
    exit 1
}

# Delegate to appropriate setup script
if ($Docker) {
    Write-Host "🐳 Running Docker Compose setup..." -ForegroundColor Cyan
    & ".\observability\setup-observability-docker.ps1"
} elseif ($Production) {
    Write-Host "🏢 Running Production setup..." -ForegroundColor Cyan
    & ".\observability\setup-observability-prod.ps1" `
        -CollectorHost $CollectorHost `
        -CollectorPort $CollectorPort `
        -LokiHost $LokiHost `
        -LokiPort $LokiPort `
        -MetricPort $MetricPort
} else {
    Write-Host "🔧 Running flexible setup..." -ForegroundColor Cyan
    & ".\observability\setup-observability.ps1" `
        -CollectorHost $CollectorHost `
        -CollectorPort $CollectorPort `
        -LokiHost $LokiHost `
        -LokiPort $LokiPort `
        -MetricBind $MetricBind `
        -MetricPort $MetricPort
}

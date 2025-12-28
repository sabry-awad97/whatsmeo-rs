#!/usr/bin/env pwsh
# Build WhatsApp bridge DLL for Windows

$ErrorActionPreference = "Stop"

Write-Host "🔨 Building WhatsApp bridge DLL..." -ForegroundColor Cyan

Push-Location $PSScriptRoot/bridge

# Ensure CGO is enabled
$env:CGO_ENABLED = "1"

# Build as shared library
go build -buildmode=c-shared -o ../../target/whatsmeow.dll .

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Built target/whatsmeow.dll" -ForegroundColor Green
    Write-Host "✅ Header: target/whatsmeow.h" -ForegroundColor Green
} else {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}

Pop-Location

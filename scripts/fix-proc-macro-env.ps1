#!/usr/bin/env pwsh
# Fix proc-macro cross-compilation environment variables (PowerShell version)
# This script ensures that proc-macro compilation works correctly

param(
    [switch]$Verbose
)

Write-Host "🔧 Fixing proc-macro cross-compilation environment..." -ForegroundColor Cyan

# List of environment variables that can cause proc-macro issues
$ProblematicVars = @(
    "CARGO_BUILD_TARGET",
    "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER",
    "CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUNNER",
    "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER",
    "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUNNER"
)

# Unset problematic environment variables
foreach ($var in $ProblematicVars) {
    $value = [Environment]::GetEnvironmentVariable($var)
    if ($value) {
        Write-Host "⚠️  Unsetting $var (was: $value)" -ForegroundColor Yellow
        [Environment]::SetEnvironmentVariable($var, $null)
        Remove-Item "Env:$var" -ErrorAction SilentlyContinue
    } else {
        if ($Verbose) {
            Write-Host "✅ $var is not set" -ForegroundColor Green
        }
    }
}

# Verify the current environment
Write-Host ""
Write-Host "📋 Current Rust environment:" -ForegroundColor Blue
try {
    $rustVersion = & rustc --version 2>$null
    $cargoVersion = & cargo --version 2>$null
    $hostInfo = & rustc -vV 2>$null | Select-String "host:" | ForEach-Object { $_.ToString().Split(' ')[1] }
    
    Write-Host "  Rust version: $rustVersion" -ForegroundColor Blue
    Write-Host "  Cargo version: $cargoVersion" -ForegroundColor Blue
    Write-Host "  Default host: $hostInfo" -ForegroundColor Blue
} catch {
    Write-Host "❌ Failed to get Rust environment info: $_" -ForegroundColor Red
    exit 1
}

# Check for any remaining cross-compilation variables
Write-Host ""
Write-Host "🔍 Checking for remaining cross-compilation variables:" -ForegroundColor Blue
$crossVars = Get-ChildItem Env: | Where-Object { $_.Name -match "^CARGO_.*TARGET.*=" }
if ($crossVars) {
    Write-Host "⚠️  Found cross-compilation variables:" -ForegroundColor Yellow
    $crossVars | ForEach-Object { Write-Host "  $($_.Name)=$($_.Value)" -ForegroundColor Yellow }
} else {
    Write-Host "✅ No cross-compilation variables found" -ForegroundColor Green
}

# Verify proc-macro compilation works
Write-Host ""
Write-Host "🧪 Testing proc-macro compilation..." -ForegroundColor Blue
try {
    $output = & cargo check --all-features --workspace 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Proc-macro compilation test passed" -ForegroundColor Green
    } else {
        Write-Host "❌ Proc-macro compilation test failed" -ForegroundColor Red
        Write-Host "Attempting to diagnose the issue..." -ForegroundColor Yellow
        Write-Host $output -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "❌ Failed to run proc-macro test: $_" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "🎉 Proc-macro environment fix completed successfully!" -ForegroundColor Green

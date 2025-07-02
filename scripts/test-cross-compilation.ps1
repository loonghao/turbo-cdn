#!/usr/bin/env pwsh
# Test script for cross-compilation proc-macro fix
# This script tests the proc-macro cross-compilation fix for turbo-cdn

param(
    [string]$Target = "x86_64-unknown-linux-gnu",
    [switch]$Verbose,
    [switch]$DryRun
)

Write-Host "🔧 Testing proc-macro cross-compilation fix for target: $Target" -ForegroundColor Cyan

# Check if cross is installed
if (-not (Get-Command cross -ErrorAction SilentlyContinue)) {
    Write-Host "❌ cross is not installed. Installing..." -ForegroundColor Red
    if (-not $DryRun) {
        cargo install cross --git https://github.com/cross-rs/cross
    }
}

# Check if target is installed
Write-Host "📦 Checking if target $Target is available..." -ForegroundColor Yellow
if (-not $DryRun) {
    rustup target add $Target
}

# Set environment variables for proc-macro fix
$env:CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER = ""

Write-Host "🔨 Testing cross-compilation with proc-macro crates..." -ForegroundColor Green

if ($Verbose) {
    Write-Host "Environment variables:" -ForegroundColor Blue
    Write-Host "  CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER: '$env:CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER'" -ForegroundColor Blue
}

# Test compilation
$crossArgs = @("build", "--target", $Target)
if ($Verbose) {
    $crossArgs += "--verbose"
}

Write-Host "Running: cross $($crossArgs -join ' ')" -ForegroundColor Magenta

if (-not $DryRun) {
    try {
        & cross @crossArgs
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ Cross-compilation successful! Proc-macro fix is working." -ForegroundColor Green
        } else {
            Write-Host "❌ Cross-compilation failed with exit code: $LASTEXITCODE" -ForegroundColor Red
            exit $LASTEXITCODE
        }
    } catch {
        Write-Host "❌ Cross-compilation failed with error: $_" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "🔍 Dry run mode - would execute: cross $($crossArgs -join ' ')" -ForegroundColor Yellow
}

Write-Host "🎉 Test completed successfully!" -ForegroundColor Green

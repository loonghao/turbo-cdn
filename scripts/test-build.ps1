# Test build script to verify cross-platform compilation on Windows
# This script tests the same targets that our CI/CD uses

param(
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"

Write-Host "🚀 Testing Turbo CDN Cross-Platform Build" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

# Function to print colored output
function Write-Status {
    param(
        [string]$Status,
        [string]$Message
    )
    
    switch ($Status) {
        "success" { Write-Host "✅ $Message" -ForegroundColor Green }
        "error" { Write-Host "❌ $Message" -ForegroundColor Red }
        "info" { Write-Host "ℹ️  $Message" -ForegroundColor Yellow }
    }
}

Write-Status "info" "Detected platform: Windows"

# Define Windows targets to test
$Targets = @(
    "x86_64-pc-windows-msvc",
    "x86_64-pc-windows-gnu",
    "i686-pc-windows-msvc"
)

Write-Status "info" "Will test $($Targets.Count) targets: $($Targets -join ', ')"

# Test each target
$SuccessfulBuilds = 0
$FailedBuilds = 0

foreach ($target in $Targets) {
    Write-Host ""
    Write-Status "info" "Building for target: $target"
    
    # Add target if not already installed
    try {
        rustup target add $target 2>$null
    } catch {
        # Target might already be installed
    }
    
    # Build the target
    $buildArgs = @("build", "--release", "--target", $target)
    if ($Verbose) {
        $buildArgs += "--verbose"
    }
    
    try {
        & cargo @buildArgs
        Write-Status "success" "Build successful for $target"
        $SuccessfulBuilds++
        
        # Check if binary exists
        $BinaryPath = "target\$target\release\turbo-cdn.exe"
        
        if (Test-Path $BinaryPath) {
            $BinarySize = (Get-Item $BinaryPath).Length
            $BinarySizeMB = [math]::Round($BinarySize / 1MB, 2)
            Write-Status "info" "Binary size: $BinarySizeMB MB ($BinaryPath)"
        } else {
            Write-Status "error" "Binary not found at expected path: $BinaryPath"
        }
    } catch {
        Write-Status "error" "Build failed for $target"
        if ($Verbose) {
            Write-Host $_.Exception.Message -ForegroundColor Red
        }
        $FailedBuilds++
    }
}

# Summary
Write-Host ""
Write-Host "📊 Build Summary" -ForegroundColor Cyan
Write-Host "================" -ForegroundColor Cyan
Write-Status "success" "Successful builds: $SuccessfulBuilds"
if ($FailedBuilds -gt 0) {
    Write-Status "error" "Failed builds: $FailedBuilds"
} else {
    Write-Status "success" "Failed builds: $FailedBuilds"
}

# Test basic functionality on native target
Write-Host ""
Write-Status "info" "Testing basic functionality..."

# Build for native target
try {
    cargo build --release
    Write-Status "success" "Native build completed"
} catch {
    Write-Status "error" "Native build failed"
    $FailedBuilds++
}

# Test version command
try {
    $versionOutput = & ".\target\release\turbo-cdn.exe" --version
    Write-Status "success" "Version command works: $versionOutput"
} catch {
    Write-Status "error" "Version command failed"
    $FailedBuilds++
}

# Test help command
try {
    & ".\target\release\turbo-cdn.exe" --help | Out-Null
    Write-Status "success" "Help command works"
} catch {
    Write-Status "error" "Help command failed"
    $FailedBuilds++
}

# Test a simple download optimization
Write-Host ""
Write-Status "info" "Testing URL optimization functionality..."
try {
    $testUrl = "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip"
    $optimizeOutput = & ".\target\release\turbo-cdn.exe" optimize $testUrl
    Write-Status "success" "URL optimization works"
    if ($Verbose) {
        Write-Host "Optimization result: $optimizeOutput" -ForegroundColor Gray
    }
} catch {
    Write-Status "error" "URL optimization failed"
    if ($Verbose) {
        Write-Host $_.Exception.Message -ForegroundColor Red
    }
    $FailedBuilds++
}

# Final result
Write-Host ""
if ($FailedBuilds -eq 0) {
    Write-Status "success" "All tests passed! 🎉"
    Write-Host ""
    Write-Host "✅ Your build configuration is working correctly" -ForegroundColor Green
    Write-Host "✅ Cross-platform compilation is functional" -ForegroundColor Green
    Write-Host "✅ Binary functionality is verified" -ForegroundColor Green
    exit 0
} else {
    Write-Status "error" "Some tests failed! 💥"
    Write-Host ""
    Write-Host "❌ Please check the failed builds above" -ForegroundColor Red
    Write-Host "❌ You may need to install additional dependencies" -ForegroundColor Red
    exit 1
}

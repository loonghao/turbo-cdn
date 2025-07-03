#!/usr/bin/env pwsh
# Simple PowerShell script to fix clippy format string warnings

param(
    [switch]$DryRun = $false
)

Write-Host "Turbo CDN - Clippy Format String Fixer" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan

if ($DryRun) {
    Write-Host "DRY RUN mode - no files will be modified" -ForegroundColor Yellow
}

$totalFiles = 0
$modifiedFiles = 0

# Find all Rust source files
$rustFiles = @()
if (Test-Path "src") { $rustFiles += Get-ChildItem -Path "src" -Filter "*.rs" -Recurse }
if (Test-Path "tests") { $rustFiles += Get-ChildItem -Path "tests" -Filter "*.rs" -Recurse }
if (Test-Path "examples") { $rustFiles += Get-ChildItem -Path "examples" -Filter "*.rs" -Recurse }

Write-Host "Found $($rustFiles.Count) Rust files to check" -ForegroundColor Cyan

foreach ($file in $rustFiles) {
    $totalFiles++
    $content = Get-Content $file.FullName -Raw
    $originalContent = $content
    $changed = $false
    
    # Fix format! patterns
    $patterns = @(
        @{ Pattern = 'format!\s*\(\s*"([^"]*)\{\}([^"]*)",\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\)'; Replacement = 'format!("$1{$3}$2")' },
        @{ Pattern = 'println!\s*\(\s*"([^"]*)\{\}([^"]*)",\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\)'; Replacement = 'println!("$1{$3}$2")' },
        @{ Pattern = 'eprintln!\s*\(\s*"([^"]*)\{\}([^"]*)",\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\)'; Replacement = 'eprintln!("$1{$3}$2")' },
        @{ Pattern = 'print!\s*\(\s*"([^"]*)\{\}([^"]*)",\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\)'; Replacement = 'print!("$1{$3}$2")' }
    )
    
    foreach ($patternInfo in $patterns) {
        $newContent = [regex]::Replace($content, $patternInfo.Pattern, $patternInfo.Replacement)
        if ($newContent -ne $content) {
            $content = $newContent
            $changed = $true
        }
    }
    
    if ($changed) {
        Write-Host "FIXED: $($file.FullName)" -ForegroundColor Green
        $modifiedFiles++
        
        if (!$DryRun) {
            Set-Content $file.FullName -Value $content -NoNewline
        }
    } else {
        Write-Host "OK: $($file.FullName)" -ForegroundColor Gray
    }
}

Write-Host ""
Write-Host "Summary:" -ForegroundColor Cyan
Write-Host "  Total files checked: $totalFiles" -ForegroundColor White
Write-Host "  Files modified: $modifiedFiles" -ForegroundColor Green

if (!$DryRun -and $modifiedFiles -gt 0) {
    Write-Host ""
    Write-Host "Running cargo fmt..." -ForegroundColor Cyan
    cargo fmt --all
    
    Write-Host ""
    Write-Host "Running clippy check..." -ForegroundColor Cyan
    $result = cargo clippy --all-targets --all-features -- -D warnings
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "All clippy checks passed!" -ForegroundColor Green
    } else {
        Write-Host "Some clippy issues remain." -ForegroundColor Red
    }
} elseif ($DryRun) {
    Write-Host ""
    Write-Host "Dry run completed. Use without -DryRun to apply changes." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Format string fix completed!" -ForegroundColor Green

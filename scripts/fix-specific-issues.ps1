#!/usr/bin/env pwsh
# Safe script to fix specific clippy format string issues

Write-Host "Fixing specific clippy format string issues..." -ForegroundColor Cyan

# Fix src/progress.rs line 364
$progressFile = "src/progress.rs"
if (Test-Path $progressFile) {
    $content = Get-Content $progressFile -Raw
    $newContent = $content -replace 'format!\("(\{\})s", seconds\)', 'format!("{seconds}s")'
    if ($content -ne $newContent) {
        Set-Content $progressFile -Value $newContent -NoNewline
        Write-Host "FIXED: $progressFile" -ForegroundColor Green
    }
}

# Fix src/smart_downloader.rs line 335
$smartDownloaderFile = "src/smart_downloader.rs"
if (Test-Path $smartDownloaderFile) {
    $content = Get-Content $smartDownloaderFile -Raw
    $newContent = $content -replace 'format!\("Request failed: \{\}", e\)', 'format!("Request failed: {e}")'
    if ($content -ne $newContent) {
        Set-Content $smartDownloaderFile -Value $newContent -NoNewline
        Write-Host "FIXED: $smartDownloaderFile" -ForegroundColor Green
    }
}

Write-Host "Running cargo fmt..." -ForegroundColor Cyan
cargo fmt --all

Write-Host "Running clippy check..." -ForegroundColor Cyan
$result = cargo clippy --all-targets --all-features -- -D warnings

if ($LASTEXITCODE -eq 0) {
    Write-Host "All clippy checks passed!" -ForegroundColor Green
} else {
    Write-Host "Some clippy issues remain." -ForegroundColor Red
}

Write-Host "Specific fixes completed!" -ForegroundColor Green

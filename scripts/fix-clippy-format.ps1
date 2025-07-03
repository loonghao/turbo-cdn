#!/usr/bin/env pwsh
# PowerShell script to automatically fix all clippy format string warnings
# This script will find and replace all instances of format!("{}", var) with format!("{var}")

param(
    [switch]$DryRun = $false,
    [switch]$Verbose = $false
)

Write-Host "🔧 Turbo CDN - Clippy Format String Fixer" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

if ($DryRun) {
    Write-Host "🔍 Running in DRY RUN mode - no files will be modified" -ForegroundColor Yellow
}

# Function to log messages
function Write-Log {
    param($Message, $Color = "White")
    if ($Verbose -or $Message.Contains("OK") -or $Message.Contains("ERROR") -or $Message.Contains("FIX")) {
        Write-Host $Message -ForegroundColor $Color
    }
}

# Function to fix format strings in a file
function Fix-FormatStrings {
    param(
        [string]$FilePath
    )
    
    if (!(Test-Path $FilePath)) {
        Write-Log "❌ File not found: $FilePath" "Red"
        return $false
    }
    
    $content = Get-Content $FilePath -Raw
    $originalContent = $content
    $changeCount = 0
    
    # Pattern 1: format!("text {}", variable) -> format!("text {variable}")
    $pattern1 = 'format!\s*\(\s*"([^"]*)\{\}([^"]*)",\s*([^)]+)\)'
    $content = [regex]::Replace($content, $pattern1, {
        param($match)
        $text1 = $match.Groups[1].Value
        $text2 = $match.Groups[2].Value
        $variable = $match.Groups[3].Value.Trim()
        
        # Simple variable name (no complex expressions)
        if ($variable -match '^[a-zA-Z_][a-zA-Z0-9_]*$') {
            $script:changeCount++
            return "format!(`"$text1{$variable}$text2`")"
        } else {
            return $match.Value
        }
    })
    
    # Pattern 2: println!("text {}", variable) -> println!("text {variable}")
    $pattern2 = 'println!\s*\(\s*"([^"]*)\{\}([^"]*)",\s*([^)]+)\)'
    $content = [regex]::Replace($content, $pattern2, {
        param($match)
        $text1 = $match.Groups[1].Value
        $text2 = $match.Groups[2].Value
        $variable = $match.Groups[3].Value.Trim()
        
        # Simple variable name (no complex expressions)
        if ($variable -match '^[a-zA-Z_][a-zA-Z0-9_]*$') {
            $script:changeCount++
            return "println!(`"$text1{$variable}$text2`")"
        } else {
            return $match.Value
        }
    })
    
    # Pattern 3: eprintln!("text {}", variable) -> eprintln!("text {variable}")
    $pattern3 = 'eprintln!\s*\(\s*"([^"]*)\{\}([^"]*)",\s*([^)]+)\)'
    $content = [regex]::Replace($content, $pattern3, {
        param($match)
        $text1 = $match.Groups[1].Value
        $text2 = $match.Groups[2].Value
        $variable = $match.Groups[3].Value.Trim()
        
        # Simple variable name (no complex expressions)
        if ($variable -match '^[a-zA-Z_][a-zA-Z0-9_]*$') {
            $script:changeCount++
            return "eprintln!(`"$text1{$variable}$text2`")"
        } else {
            return $match.Value
        }
    })
    
    # Pattern 4: print!("text {}", variable) -> print!("text {variable}")
    $pattern4 = 'print!\s*\(\s*"([^"]*)\{\}([^"]*)",\s*([^)]+)\)'
    $content = [regex]::Replace($content, $pattern4, {
        param($match)
        $text1 = $match.Groups[1].Value
        $text2 = $match.Groups[2].Value
        $variable = $match.Groups[3].Value.Trim()
        
        # Simple variable name (no complex expressions)
        if ($variable -match '^[a-zA-Z_][a-zA-Z0-9_]*$') {
            $script:changeCount++
            return "print!(`"$text1{$variable}$text2`")"
        } else {
            return $match.Value
        }
    })
    
    if ($changeCount -gt 0) {
        Write-Log "🔧 Fixed $changeCount format strings in $FilePath" "Green"
        
        if (!$DryRun) {
            Set-Content $FilePath -Value $content -NoNewline
        }
        return $true
    } else {
        Write-Log "✅ No changes needed in $FilePath" "Gray"
        return $false
    }
}

# Main execution
$totalFiles = 0
$modifiedFiles = 0

# Find all Rust source files
$rustFiles = Get-ChildItem -Path "src", "tests", "examples" -Filter "*.rs" -Recurse -ErrorAction SilentlyContinue

Write-Log "🔍 Found $($rustFiles.Count) Rust files to check" "Cyan"

foreach ($file in $rustFiles) {
    $totalFiles++
    Write-Log "Checking: $($file.FullName)" "Gray"
    
    if (Fix-FormatStrings -FilePath $file.FullName) {
        $modifiedFiles++
    }
}

Write-Host ""
Write-Host "📊 Summary:" -ForegroundColor Cyan
Write-Host "  Total files checked: $totalFiles" -ForegroundColor White
Write-Host "  Files modified: $modifiedFiles" -ForegroundColor Green

if (!$DryRun -and $modifiedFiles -gt 0) {
    Write-Host ""
    Write-Host "🚀 Running cargo fmt to ensure proper formatting..." -ForegroundColor Cyan
    cargo fmt --all
    
    Write-Host ""
    Write-Host "🧪 Running clippy to verify fixes..." -ForegroundColor Cyan
    $clippyResult = cargo clippy --all-targets --all-features -- -D warnings
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ All clippy checks passed!" -ForegroundColor Green
    } else {
        Write-Host "❌ Some clippy issues remain. Please check the output above." -ForegroundColor Red
    }
} elseif ($DryRun) {
    Write-Host ""
    Write-Host "🔍 Dry run completed. Use without -DryRun to apply changes." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "✨ Format string fix completed!" -ForegroundColor Green

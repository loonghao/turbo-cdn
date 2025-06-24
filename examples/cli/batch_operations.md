# 📦 Batch Operations Examples

This guide shows how to efficiently download multiple files using Turbo CDN CLI.

## 🚀 Shell Scripting for Batch Downloads

### Basic Batch Download Script

Create a shell script for downloading multiple files:

```bash
#!/bin/bash
# download_tools.sh - Download development tools

echo "🚀 Downloading development tools with Turbo CDN..."

# Define URLs
URLS=(
    "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip"
    "https://github.com/sharkdp/fd/releases/download/v8.7.0/fd-v8.7.0-x86_64-pc-windows-msvc.zip"
    "https://github.com/sharkdp/bat/releases/download/v0.24.0/bat-v0.24.0-x86_64-pc-windows-msvc.zip"
    "https://github.com/cli/cli/releases/download/v2.40.1/gh_2.40.1_windows_amd64.zip"
)

# Download each file
for url in "${URLS[@]}"; do
    echo "📥 Downloading: $url"
    turbo-cdn dl "$url" --verbose
    echo "✅ Completed: $url"
    echo ""
done

echo "🎉 All downloads completed!"
```

### PowerShell Batch Script (Windows)

```powershell
# download_tools.ps1 - Download development tools

Write-Host "🚀 Downloading development tools with Turbo CDN..." -ForegroundColor Green

$urls = @(
    "https://github.com/microsoft/vscode/releases/download/1.85.0/VSCodeUserSetup-x64-1.85.0.exe",
    "https://github.com/git-for-windows/git/releases/download/v2.43.0.windows.1/Git-2.43.0-64-bit.exe",
    "https://github.com/microsoft/PowerToys/releases/download/v0.76.2/PowerToysUserSetup-0.76.2-x64.exe",
    "https://nodejs.org/dist/v20.10.0/node-v20.10.0-x64.msi"
)

foreach ($url in $urls) {
    Write-Host "📥 Downloading: $url" -ForegroundColor Yellow
    & turbo-cdn dl $url --verbose
    Write-Host "✅ Completed: $url" -ForegroundColor Green
    Write-Host ""
}

Write-Host "🎉 All downloads completed!" -ForegroundColor Green
```

## 📋 URL Lists and Parallel Processing

### Download from URL List File

Create a file with URLs:

```bash
# urls.txt
https://github.com/rust-lang/rust-analyzer/releases/download/2023-12-04/rust-analyzer-x86_64-pc-windows-msvc.gz
https://github.com/golang/go/releases/download/go1.21.5/go1.21.5.windows-amd64.zip
https://github.com/nodejs/node/releases/download/v20.10.0/node-v20.10.0-win-x64.zip
https://github.com/python/cpython/releases/download/v3.12.1/python-3.12.1-amd64.exe
```

Process the file:

```bash
#!/bin/bash
# download_from_list.sh

while IFS= read -r url; do
    if [[ ! -z "$url" && ! "$url" =~ ^# ]]; then
        echo "📥 Downloading: $url"
        turbo-cdn dl "$url"
    fi
done < urls.txt
```

### Parallel Downloads with xargs

```bash
# Download multiple files in parallel (Linux/macOS)
cat urls.txt | xargs -n 1 -P 4 -I {} turbo-cdn dl "{}"

# Windows equivalent with PowerShell
Get-Content urls.txt | ForEach-Object -Parallel { & turbo-cdn dl $_ } -ThrottleLimit 4
```

## 🎯 Specific Use Cases

### Development Environment Setup

```bash
#!/bin/bash
# setup_dev_env.sh - Complete development environment setup

echo "🛠️ Setting up development environment..."

# Create directories
mkdir -p ./downloads/{tools,runtimes,editors}

# Download development tools
echo "📥 Downloading development tools..."
turbo-cdn dl "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip" "./downloads/tools/"
turbo-cdn dl "https://github.com/sharkdp/fd/releases/download/v8.7.0/fd-v8.7.0-x86_64-pc-windows-msvc.zip" "./downloads/tools/"
turbo-cdn dl "https://github.com/sharkdp/bat/releases/download/v0.24.0/bat-v0.24.0-x86_64-pc-windows-msvc.zip" "./downloads/tools/"

# Download runtimes
echo "📥 Downloading runtimes..."
turbo-cdn dl "https://nodejs.org/dist/v20.10.0/node-v20.10.0-win-x64.zip" "./downloads/runtimes/"
turbo-cdn dl "https://github.com/golang/go/releases/download/go1.21.5/go1.21.5.windows-amd64.zip" "./downloads/runtimes/"
turbo-cdn dl "https://github.com/rust-lang/rust/releases/download/1.74.1/rust-1.74.1-x86_64-pc-windows-msvc.msi" "./downloads/runtimes/"

# Download editors
echo "📥 Downloading editors..."
turbo-cdn dl "https://github.com/microsoft/vscode/releases/download/1.85.0/VSCodeUserSetup-x64-1.85.0.exe" "./downloads/editors/"
turbo-cdn dl "https://github.com/neovim/neovim/releases/download/v0.9.4/nvim-win64.zip" "./downloads/editors/"

echo "✅ Development environment setup completed!"
```

### Package Manager Downloads

```bash
#!/bin/bash
# download_packages.sh - Download packages from various sources

echo "📦 Downloading packages from multiple sources..."

# npm packages
echo "📥 Downloading npm packages..."
turbo-cdn dl "https://registry.npmjs.org/react/-/react-18.2.0.tgz" "./downloads/npm/"
turbo-cdn dl "https://registry.npmjs.org/typescript/-/typescript-5.3.3.tgz" "./downloads/npm/"
turbo-cdn dl "https://registry.npmjs.org/express/-/express-4.18.2.tgz" "./downloads/npm/"

# Python packages
echo "📥 Downloading Python packages..."
turbo-cdn dl "https://files.pythonhosted.org/packages/source/r/requests/requests-2.31.0.tar.gz" "./downloads/python/"
turbo-cdn dl "https://files.pythonhosted.org/packages/source/d/django/Django-4.2.7.tar.gz" "./downloads/python/"
turbo-cdn dl "https://files.pythonhosted.org/packages/source/f/flask/Flask-3.0.0.tar.gz" "./downloads/python/"

# Rust crates (source)
echo "📥 Downloading Rust crates..."
turbo-cdn dl "https://crates.io/api/v1/crates/serde/1.0.193/download" "./downloads/rust/"
turbo-cdn dl "https://crates.io/api/v1/crates/tokio/1.35.1/download" "./downloads/rust/"
turbo-cdn dl "https://crates.io/api/v1/crates/clap/4.4.11/download" "./downloads/rust/"

echo "✅ Package downloads completed!"
```

## 🔄 Error Handling and Retry Logic

### Robust Download Script with Error Handling

```bash
#!/bin/bash
# robust_download.sh - Download with error handling and retry

download_with_retry() {
    local url="$1"
    local max_attempts=3
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        echo "📥 Attempt $attempt/$max_attempts: $url"
        
        if turbo-cdn dl "$url"; then
            echo "✅ Successfully downloaded: $url"
            return 0
        else
            echo "❌ Failed attempt $attempt for: $url"
            if [ $attempt -lt $max_attempts ]; then
                echo "⏳ Waiting 5 seconds before retry..."
                sleep 5
            fi
        fi
        
        ((attempt++))
    done
    
    echo "💥 Failed to download after $max_attempts attempts: $url"
    return 1
}

# URLs to download
URLS=(
    "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip"
    "https://github.com/sharkdp/fd/releases/download/v8.7.0/fd-v8.7.0-x86_64-pc-windows-msvc.zip"
    "https://github.com/sharkdp/bat/releases/download/v0.24.0/bat-v0.24.0-x86_64-pc-windows-msvc.zip"
)

# Track results
failed_downloads=()
successful_downloads=()

# Download each URL
for url in "${URLS[@]}"; do
    if download_with_retry "$url"; then
        successful_downloads+=("$url")
    else
        failed_downloads+=("$url")
    fi
    echo ""
done

# Report results
echo "📊 Download Summary:"
echo "✅ Successful: ${#successful_downloads[@]}"
echo "❌ Failed: ${#failed_downloads[@]}"

if [ ${#failed_downloads[@]} -gt 0 ]; then
    echo ""
    echo "💥 Failed downloads:"
    for url in "${failed_downloads[@]}"; do
        echo "  - $url"
    done
fi
```

## 📊 Progress Monitoring for Batch Operations

### Batch Download with Progress Tracking

```bash
#!/bin/bash
# batch_with_progress.sh - Batch downloads with progress tracking

URLS=(
    "https://github.com/microsoft/vscode/releases/download/1.85.0/VSCode-linux-x64.tar.gz"
    "https://github.com/docker/compose/releases/download/v2.23.3/docker-compose-linux-x86_64"
    "https://github.com/kubernetes/kubectl/releases/download/v1.29.0/kubectl-linux-amd64"
)

total_files=${#URLS[@]}
current_file=0

echo "🚀 Starting batch download of $total_files files..."
echo ""

for url in "${URLS[@]}"; do
    ((current_file++))
    echo "📥 [$current_file/$total_files] Downloading: $(basename "$url")"
    echo "🔗 URL: $url"
    
    # Download with verbose output
    if turbo-cdn dl "$url" --verbose; then
        echo "✅ [$current_file/$total_files] Completed: $(basename "$url")"
    else
        echo "❌ [$current_file/$total_files] Failed: $(basename "$url")"
    fi
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
done

echo "🎉 Batch download completed!"
```

## 🔗 Integration with Build Systems

### Makefile Integration

```makefile
# Makefile - Download dependencies with Turbo CDN

TOOLS_DIR := ./tools
DOWNLOADS_DIR := ./downloads

.PHONY: download-tools download-runtimes clean

download-tools:
	@echo "📥 Downloading development tools..."
	@mkdir -p $(TOOLS_DIR)
	turbo-cdn dl "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip" "$(TOOLS_DIR)/"
	turbo-cdn dl "https://github.com/sharkdp/fd/releases/download/v8.7.0/fd-v8.7.0-x86_64-pc-windows-msvc.zip" "$(TOOLS_DIR)/"
	@echo "✅ Tools downloaded to $(TOOLS_DIR)"

download-runtimes:
	@echo "📥 Downloading runtimes..."
	@mkdir -p $(DOWNLOADS_DIR)/runtimes
	turbo-cdn dl "https://nodejs.org/dist/v20.10.0/node-v20.10.0-win-x64.zip" "$(DOWNLOADS_DIR)/runtimes/"
	turbo-cdn dl "https://github.com/golang/go/releases/download/go1.21.5/go1.21.5.windows-amd64.zip" "$(DOWNLOADS_DIR)/runtimes/"
	@echo "✅ Runtimes downloaded to $(DOWNLOADS_DIR)/runtimes"

clean:
	@echo "🧹 Cleaning downloads..."
	rm -rf $(DOWNLOADS_DIR) $(TOOLS_DIR)
	@echo "✅ Cleaned"
```

## 🔗 Next Steps

- [API Examples](../api/) - Use Turbo CDN in your Rust applications
- [Integration Examples](../integration/) - Integrate with other tools
- [Performance Examples](../performance/) - Optimize download performance

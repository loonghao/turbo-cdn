# 🔧 Advanced CLI Usage

This guide covers advanced features and use cases for the Turbo CDN command line interface.

## 🌍 Geographic Optimization

Turbo CDN automatically detects your geographic location and selects the best CDN endpoints.

### Understanding CDN Selection
```bash
# See which CDN is selected for your location
turbo-cdn optimize "https://github.com/microsoft/vscode/releases/download/1.85.0/VSCode-linux-x64.tar.gz" --verbose
```

### Regional Examples
```bash
# These URLs will be optimized differently based on your location:

# For users in China - may use Fastly or jsDelivr
turbo-cdn optimize "https://github.com/nodejs/node/releases/download/v20.10.0/node-v20.10.0-linux-x64.tar.xz"

# For users in Europe - may use GitHub's European CDN
turbo-cdn optimize "https://github.com/golang/go/releases/download/go1.21.5/go1.21.5.linux-amd64.tar.gz"

# For users in Asia-Pacific - optimized routing
turbo-cdn optimize "https://github.com/rust-lang/rust/releases/download/1.74.1/rust-1.74.1-x86_64-unknown-linux-gnu.tar.gz"
```

## 📊 Performance Monitoring

### Detailed Download Statistics
```bash
# Download with verbose output to see performance metrics
turbo-cdn dl "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip" --verbose
```

Expected output includes:
- Geographic detection results
- CDN selection reasoning
- Download speed and progress
- Chunk download statistics
- Resume capability status

### Performance Comparison
```bash
# Compare original vs optimized URLs
echo "Original URL performance:"
turbo-cdn dl "https://github.com/sharkdp/bat/releases/download/v0.24.0/bat-v0.24.0-x86_64-pc-windows-msvc.zip" --verbose

echo "Optimized URL performance:"
OPTIMIZED_URL=$(turbo-cdn optimize "https://github.com/sharkdp/bat/releases/download/v0.24.0/bat-v0.24.0-x86_64-pc-windows-msvc.zip")
turbo-cdn dl "$OPTIMIZED_URL" --verbose
```

## 🔄 Resume and Retry Logic

### Automatic Resume
Turbo CDN automatically resumes interrupted downloads:

```bash
# Start a large download
turbo-cdn dl "https://github.com/microsoft/vscode/releases/download/1.85.0/VSCode-linux-x64.tar.gz"

# If interrupted (Ctrl+C), restart with the same command
# It will automatically resume from where it left off
turbo-cdn dl "https://github.com/microsoft/vscode/releases/download/1.85.0/VSCode-linux-x64.tar.gz"
```

### Manual Resume Testing
```bash
# Download a large file and interrupt it manually to test resume
turbo-cdn dl "https://github.com/docker/compose/releases/download/v2.23.3/docker-compose-linux-x86_64" --verbose
```

## 🚀 Performance Optimization Features

### Concurrent Chunk Downloads
Turbo CDN automatically uses multiple connections for faster downloads:

```bash
# Large files benefit most from concurrent downloading
turbo-cdn dl "https://github.com/microsoft/PowerToys/releases/download/v0.76.2/PowerToysUserSetup-0.76.2-x64.exe" --verbose
```

### Adaptive Chunk Sizing
The system automatically adjusts chunk sizes based on network conditions:

```bash
# Monitor chunk size adaptation with verbose output
turbo-cdn dl "https://github.com/obsidianmd/obsidian-releases/releases/download/v1.4.16/Obsidian-1.4.16.AppImage" --verbose
```

## 🌐 Platform-Specific Examples

### Windows Executables
```bash
# Windows development tools
turbo-cdn dl "https://github.com/microsoft/winget-cli/releases/download/v1.6.2771/Microsoft.DesktopAppInstaller_8wekyb3d8bbwe.msixbundle"

# Windows utilities
turbo-cdn dl "https://github.com/microsoft/PowerToys/releases/download/v0.76.2/PowerToysUserSetup-0.76.2-x64.exe"
```

### Linux Packages
```bash
# Linux development tools
turbo-cdn dl "https://github.com/neovim/neovim/releases/download/v0.9.4/nvim-linux64.tar.gz"

# Linux utilities
turbo-cdn dl "https://github.com/sharkdp/fd/releases/download/v8.7.0/fd-v8.7.0-x86_64-unknown-linux-gnu.tar.gz"
```

### macOS Applications
```bash
# macOS development tools
turbo-cdn dl "https://github.com/microsoft/vscode/releases/download/1.85.0/VSCode-darwin-universal.zip"

# macOS utilities
turbo-cdn dl "https://github.com/mas-cli/mas/releases/download/v1.8.6/mas.pkg"
```

## 🔍 URL Pattern Support

### GitHub Releases
```bash
# Standard GitHub release pattern
turbo-cdn optimize "https://github.com/{owner}/{repo}/releases/download/{tag}/{filename}"

# Example
turbo-cdn optimize "https://github.com/cli/cli/releases/download/v2.40.1/gh_2.40.1_windows_amd64.zip"
```

### npm Packages
```bash
# npm registry pattern
turbo-cdn optimize "https://registry.npmjs.org/{package}/-/{package}-{version}.tgz"

# Example
turbo-cdn optimize "https://registry.npmjs.org/typescript/-/typescript-5.3.3.tgz"
```

### Python Packages
```bash
# PyPI pattern
turbo-cdn optimize "https://files.pythonhosted.org/packages/source/{first_letter}/{package}/{package}-{version}.tar.gz"

# Example
turbo-cdn optimize "https://files.pythonhosted.org/packages/source/r/requests/requests-2.31.0.tar.gz"
```

## 🛡️ Security Features

### Integrity Verification
Turbo CDN includes built-in integrity verification:

```bash
# Downloads are automatically verified when possible
turbo-cdn dl "https://github.com/rust-lang/rust/releases/download/1.74.1/rust-1.74.1-x86_64-pc-windows-msvc.msi" --verbose
```

### HTTPS Enforcement
All optimized URLs use HTTPS when available:

```bash
# HTTP URLs are upgraded to HTTPS when possible
turbo-cdn optimize "http://example.com/file.zip"
```

## 📈 Benchmarking

### Speed Comparison
```bash
# Time a download to compare with other tools
time turbo-cdn dl "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip"

# Compare with curl
time curl -L -o ripgrep.zip "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip"

# Compare with wget
time wget "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip"
```

## 🔧 Environment Variables

### Configuration via Environment
```bash
# Enable debug logging
export RUST_LOG=turbo_cdn=debug
turbo-cdn dl "https://example.com/file.zip" --verbose

# Set custom user agent
export TURBO_CDN_USER_AGENT="MyApp/1.0"
turbo-cdn dl "https://example.com/file.zip"

# Configure timeout
export TURBO_CDN_TIMEOUT=60
turbo-cdn dl "https://example.com/large-file.zip"
```

## 🔗 Next Steps

- [Batch Operations](batch_operations.md) - Download multiple files efficiently
- [API Examples](../api/) - Use Turbo CDN in your applications
- [Integration Examples](../integration/) - Integrate with other tools

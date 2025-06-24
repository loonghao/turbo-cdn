# 📥 Basic Download Examples

This guide shows basic usage of the Turbo CDN command line interface.

## 🚀 Quick Start

### Simple Download
Download a file with automatic CDN optimization:

```bash
turbo-cdn dl "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip"
```

### Download to Specific Location
Specify where to save the downloaded file:

```bash
turbo-cdn download "https://github.com/oven-sh/bun/releases/download/bun-v1.2.9/bun-linux-x64.zip" "./downloads/bun.zip"
```

### Verbose Output
See detailed information about the download process:

```bash
turbo-cdn dl "https://github.com/microsoft/vscode/releases/download/1.85.0/VSCode-linux-x64.tar.gz" --verbose
```

## 🔍 URL Optimization

### Get Optimal CDN URL
Find the best CDN URL without downloading:

```bash
turbo-cdn optimize "https://github.com/rust-lang/mdBook/releases/download/v0.4.21/mdbook-v0.4.21-x86_64-pc-windows-msvc.zip"
```

### Check Multiple URLs
```bash
# GitHub release
turbo-cdn optimize "https://github.com/sharkdp/fd/releases/download/v8.7.0/fd-v8.7.0-x86_64-pc-windows-msvc.zip"

# npm package
turbo-cdn optimize "https://registry.npmjs.org/express/-/express-4.18.2.tgz"

# Python package
turbo-cdn optimize "https://files.pythonhosted.org/packages/source/r/requests/requests-2.31.0.tar.gz"
```

## 📊 Performance Information

### View Statistics
See download performance statistics:

```bash
turbo-cdn stats
```

### Version Information
Check the installed version:

```bash
turbo-cdn version
# or
turbo-cdn --version
```

## 🎯 Common Use Cases

### Development Tools
```bash
# Download Rust tools
turbo-cdn dl "https://github.com/rust-lang/rust-analyzer/releases/download/2023-12-04/rust-analyzer-x86_64-pc-windows-msvc.gz"

# Download Node.js
turbo-cdn dl "https://nodejs.org/dist/v20.10.0/node-v20.10.0-win-x64.zip"

# Download Go
turbo-cdn dl "https://go.dev/dl/go1.21.5.windows-amd64.zip"
```

### Popular Software
```bash
# Download VS Code
turbo-cdn dl "https://github.com/microsoft/vscode/releases/download/1.85.0/VSCodeUserSetup-x64-1.85.0.exe"

# Download Git
turbo-cdn dl "https://github.com/git-for-windows/git/releases/download/v2.43.0.windows.1/Git-2.43.0-64-bit.exe"

# Download Docker Desktop
turbo-cdn dl "https://desktop.docker.com/win/main/amd64/Docker%20Desktop%20Installer.exe"
```

### Package Managers
```bash
# npm packages
turbo-cdn dl "https://registry.npmjs.org/react/-/react-18.2.0.tgz"

# Python packages
turbo-cdn dl "https://files.pythonhosted.org/packages/source/d/django/Django-4.2.7.tar.gz"

# Rust crates (source)
turbo-cdn dl "https://crates.io/api/v1/crates/serde/1.0.193/download"
```

## 🛠️ Troubleshooting

### Common Issues

#### Network Timeout
If downloads are timing out, try with verbose output to see what's happening:
```bash
turbo-cdn dl "https://example.com/large-file.zip" --verbose
```

#### Permission Denied
Make sure you have write permissions to the target directory:
```bash
# Download to a directory you own
turbo-cdn dl "https://example.com/file.zip" "./my-downloads/file.zip"
```

#### Invalid URL
Ensure the URL is accessible and points to a downloadable file:
```bash
# Test URL optimization first
turbo-cdn optimize "https://example.com/file.zip"
```

### Getting Help
```bash
# Show all available commands and options
turbo-cdn --help

# Show help for specific command
turbo-cdn download --help
```

## 🔗 Next Steps

- [Advanced Usage](advanced_usage.md) - Learn about advanced CLI features
- [Batch Operations](batch_operations.md) - Download multiple files efficiently
- [API Examples](../api/) - Use Turbo CDN in your Rust applications

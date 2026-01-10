# Self Update

Turbo CDN includes an opt-in self-update feature to keep your installation up to date.

> Build with `--features self-update` (disabled by default for libraries) or use official CLI release binaries to enable this command.


## Usage

### Check for Updates

Check if a new version is available without installing:

```bash
turbo-cdn self-update --check
```

Example output:
```
🔄 Turbo CDN - Self Update
=========================
Current version: v0.7.1

🔍 Checking for updates...
✨ New version available: v0.7.2


Run 'turbo-cdn self-update' to update.
```

### Update to Latest Version

Update to the latest version:

```bash
turbo-cdn self-update
```

Or use the alias:

```bash
turbo-cdn upgrade
```

Example output:
```
🔄 Turbo CDN - Self Update
=========================
Current version: v0.7.1

🔍 Checking for updates...
Downloading...
✓ Download complete

🎉 Successfully updated to v0.7.2
   Please restart turbo-cdn to use the new version.

```

## How It Works

The self-update feature:

1. **Checks GitHub Releases** - Queries the latest release from `loonghao/turbo-cdn`
2. **Compares Versions** - Compares current version with latest available
3. **Downloads Binary** - Downloads the appropriate binary for your platform
4. **Replaces Executable** - Safely replaces the current executable

## Supported Platforms

Self-update supports all platforms with pre-built binaries:

| Platform | Architecture | Supported |
|----------|--------------|-----------|
| Linux | x86_64 | ✅ |
| Linux | x86_64 (musl) | ✅ |
| Linux | aarch64 | ✅ |
| Linux | aarch64 (musl) | ✅ |
| macOS | x86_64 | ✅ |
| macOS | aarch64 (Apple Silicon) | ✅ |
| Windows | x86_64 | ✅ |
| Windows | aarch64 | ✅ |

## Manual Update

If self-update fails, you can always update manually:

1. Visit [GitHub Releases](https://github.com/loonghao/turbo-cdn/releases)
2. Download the appropriate binary for your platform
3. Replace the existing `turbo-cdn` executable

Or reinstall via cargo:

```bash
cargo install turbo-cdn --force
```

## Troubleshooting

### Update Fails

If the update fails:

1. **Check internet connection** - Ensure you can reach GitHub
2. **Check permissions** - You may need admin/sudo rights to replace the executable
3. **Try manual update** - Download directly from GitHub Releases

### Permission Denied

On Linux/macOS, you may need elevated permissions:

```bash
sudo turbo-cdn self-update
```

### Behind Corporate Proxy

If you're behind a proxy, ensure your proxy settings are configured:

```bash
export HTTPS_PROXY=http://proxy.example.com:8080
turbo-cdn self-update
```

## Security

The self-update feature:

- Downloads only from official GitHub Releases
- Uses HTTPS for all connections
- Verifies the downloaded binary matches the expected format

For additional security, you can verify releases manually by checking the release signatures on GitHub.

#!/bin/bash
# Fix proc-macro cross-compilation environment variables
# This script ensures that proc-macro compilation works correctly in CI

set -euo pipefail

echo "🔧 Fixing proc-macro cross-compilation environment..."

# Backup and temporarily disable Cargo config that might interfere
CARGO_CONFIG_BACKUP=""
if [[ -f ".cargo/config.toml" ]]; then
    echo "📁 Backing up .cargo/config.toml to avoid interference..."
    CARGO_CONFIG_BACKUP=".cargo/config.toml.backup.$$"
    cp ".cargo/config.toml" "$CARGO_CONFIG_BACKUP"
    mv ".cargo/config.toml" ".cargo/config.toml.disabled"
    echo "✅ Cargo config temporarily disabled"
fi

# List of environment variables that can cause proc-macro issues
PROBLEMATIC_VARS=(
    "CARGO_BUILD_TARGET"
    "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER"
    "CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUNNER"
    "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER"
    "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUNNER"
    "CARGO_CFG_TARGET_ARCH"
    "CARGO_CFG_TARGET_OS"
    "CARGO_CFG_TARGET_FAMILY"
)

# Unset problematic environment variables
for var in "${PROBLEMATIC_VARS[@]}"; do
    if [[ -n "${!var:-}" ]]; then
        echo "⚠️  Unsetting $var (was: ${!var})"
        unset "$var"
    else
        echo "✅ $var is not set"
    fi
done

# Force set correct environment for native compilation
export CARGO_BUILD_TARGET=""
export CARGO_TARGET_DIR="target"

# Verify the current environment
echo ""
echo "📋 Current Rust environment:"
echo "  Rust version: $(rustc --version)"
echo "  Cargo version: $(cargo --version)"
echo "  Default host: $(rustc -vV | grep host | cut -d' ' -f2)"

# Check for any remaining cross-compilation variables
echo ""
echo "🔍 Checking for remaining cross-compilation variables:"
env | grep -E "^CARGO_.*TARGET.*=" || echo "✅ No cross-compilation variables found"

# Get the native host target
NATIVE_TARGET=$(rustc -vV | grep host | cut -d' ' -f2)
echo "🎯 Native target: $NATIVE_TARGET"

# Verify proc-macro compilation works with explicit native target
echo ""
echo "🧪 Testing proc-macro compilation with native target..."

# Try multiple approaches to fix proc-macro compilation
echo "Approach 1: Using explicit native target..."
if cargo check --target "$NATIVE_TARGET" --all-features --workspace > /dev/null 2>&1; then
    echo "✅ Proc-macro compilation test passed with explicit target"
    COMPILATION_SUCCESS=true
else
    echo "⚠️  Explicit target approach failed, trying without target..."
    COMPILATION_SUCCESS=false
fi

if [[ "$COMPILATION_SUCCESS" != "true" ]]; then
    echo "Approach 2: Using default target..."
    if cargo check --all-features --workspace > /dev/null 2>&1; then
        echo "✅ Proc-macro compilation test passed with default target"
        COMPILATION_SUCCESS=true
    else
        echo "❌ Both approaches failed. Attempting to diagnose the issue..."
        echo "Environment variables:"
        env | grep -E "^CARGO_" || echo "No CARGO_ variables found"
        echo ""
        echo "Cargo configuration:"
        cargo config get || echo "No cargo config found"
        echo ""
        echo "Detailed error output:"
        cargo check --all-features --workspace

        # Restore Cargo config before exiting
        if [[ -n "$CARGO_CONFIG_BACKUP" && -f "$CARGO_CONFIG_BACKUP" ]]; then
            echo "🔄 Restoring Cargo config..."
            mv "$CARGO_CONFIG_BACKUP" ".cargo/config.toml"
            rm -f ".cargo/config.toml.disabled"
        fi
        exit 1
    fi
fi

# Restore Cargo config
if [[ -n "$CARGO_CONFIG_BACKUP" && -f "$CARGO_CONFIG_BACKUP" ]]; then
    echo "🔄 Restoring Cargo config..."
    mv "$CARGO_CONFIG_BACKUP" ".cargo/config.toml"
    rm -f ".cargo/config.toml.disabled"
    echo "✅ Cargo config restored"
fi

echo ""
echo "🎉 Proc-macro environment fix completed successfully!"

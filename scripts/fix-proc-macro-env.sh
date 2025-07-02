#!/bin/bash
# Fix proc-macro cross-compilation environment variables
# This script ensures that proc-macro compilation works correctly in CI

set -euo pipefail

echo "🔧 Fixing proc-macro cross-compilation environment..."

# List of environment variables that can cause proc-macro issues
PROBLEMATIC_VARS=(
    "CARGO_BUILD_TARGET"
    "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER"
    "CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUNNER"
    "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER"
    "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUNNER"
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

# Verify proc-macro compilation works
echo ""
echo "🧪 Testing proc-macro compilation..."
if cargo check --all-features --workspace > /dev/null 2>&1; then
    echo "✅ Proc-macro compilation test passed"
else
    echo "❌ Proc-macro compilation test failed"
    echo "Attempting to diagnose the issue..."
    cargo check --all-features --workspace
    exit 1
fi

echo ""
echo "🎉 Proc-macro environment fix completed successfully!"

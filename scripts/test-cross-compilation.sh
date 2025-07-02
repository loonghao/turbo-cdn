#!/bin/bash
# Test script for cross-compilation proc-macro fix
# This script tests the proc-macro cross-compilation fix for turbo-cdn

set -euo pipefail

TARGET="${1:-x86_64-unknown-linux-gnu}"
VERBOSE="${VERBOSE:-false}"
DRY_RUN="${DRY_RUN:-false}"

echo "🔧 Testing proc-macro cross-compilation fix for target: $TARGET"

# Check if cross is installed
if ! command -v cross &> /dev/null; then
    echo "❌ cross is not installed. Installing..."
    if [ "$DRY_RUN" != "true" ]; then
        cargo install cross --git https://github.com/cross-rs/cross
    fi
fi

# Check if target is installed
echo "📦 Checking if target $TARGET is available..."
if [ "$DRY_RUN" != "true" ]; then
    rustup target add "$TARGET"
fi

# Set environment variables for proc-macro fix
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=""

echo "🔨 Testing cross-compilation with proc-macro crates..."

if [ "$VERBOSE" = "true" ]; then
    echo "Environment variables:"
    echo "  CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER: '$CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER'"
fi

# Test compilation
CROSS_ARGS=("build" "--target" "$TARGET")
if [ "$VERBOSE" = "true" ]; then
    CROSS_ARGS+=("--verbose")
fi

echo "Running: cross ${CROSS_ARGS[*]}"

if [ "$DRY_RUN" != "true" ]; then
    if cross "${CROSS_ARGS[@]}"; then
        echo "✅ Cross-compilation successful! Proc-macro fix is working."
    else
        echo "❌ Cross-compilation failed with exit code: $?"
        exit 1
    fi
else
    echo "🔍 Dry run mode - would execute: cross ${CROSS_ARGS[*]}"
fi

echo "🎉 Test completed successfully!"

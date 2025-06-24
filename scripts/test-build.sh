#!/bin/bash

# Test build script to verify cross-platform compilation
# This script tests the same targets that our CI/CD uses

set -e

echo "🚀 Testing Turbo CDN Cross-Platform Build"
echo "========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    case $status in
        "success")
            echo -e "${GREEN}✅ $message${NC}"
            ;;
        "error")
            echo -e "${RED}❌ $message${NC}"
            ;;
        "info")
            echo -e "${YELLOW}ℹ️  $message${NC}"
            ;;
    esac
}

# Check if we're on the right platform
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    PLATFORM="linux"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    PLATFORM="macos"
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    PLATFORM="windows"
else
    print_status "error" "Unsupported platform: $OSTYPE"
    exit 1
fi

print_status "info" "Detected platform: $PLATFORM"

# Define targets to test based on platform
case $PLATFORM in
    "linux")
        TARGETS=(
            "x86_64-unknown-linux-gnu"
            "x86_64-unknown-linux-musl"
            "aarch64-unknown-linux-gnu"
            "armv7-unknown-linux-gnueabihf"
        )
        ;;
    "macos")
        TARGETS=(
            "x86_64-apple-darwin"
            "aarch64-apple-darwin"
        )
        ;;
    "windows")
        TARGETS=(
            "x86_64-pc-windows-msvc"
            "x86_64-pc-windows-gnu"
            "i686-pc-windows-msvc"
        )
        ;;
esac

print_status "info" "Will test ${#TARGETS[@]} targets: ${TARGETS[*]}"

# Install cross if on Linux and not already installed
if [[ "$PLATFORM" == "linux" ]] && ! command -v cross &> /dev/null; then
    print_status "info" "Installing cross for cross-compilation..."
    cargo install cross --git https://github.com/cross-rs/cross
fi

# Test each target
SUCCESSFUL_BUILDS=0
FAILED_BUILDS=0

for target in "${TARGETS[@]}"; do
    echo ""
    print_status "info" "Building for target: $target"
    
    # Determine if we need cross or cargo
    if [[ "$PLATFORM" == "linux" && "$target" != "x86_64-unknown-linux-gnu" ]]; then
        BUILD_CMD="cross"
    else
        BUILD_CMD="cargo"
        # Add target if not native
        if [[ "$target" != "x86_64-unknown-linux-gnu" && "$target" != "x86_64-apple-darwin" && "$target" != "x86_64-pc-windows-msvc" ]]; then
            rustup target add "$target" 2>/dev/null || true
        fi
    fi
    
    # Build the target
    if $BUILD_CMD build --release --target "$target" --verbose; then
        print_status "success" "Build successful for $target"
        ((SUCCESSFUL_BUILDS++))
        
        # Check if binary exists
        if [[ "$PLATFORM" == "windows" ]]; then
            BINARY_PATH="target/$target/release/turbo-cdn.exe"
        else
            BINARY_PATH="target/$target/release/turbo-cdn"
        fi
        
        if [[ -f "$BINARY_PATH" ]]; then
            BINARY_SIZE=$(du -h "$BINARY_PATH" | cut -f1)
            print_status "info" "Binary size: $BINARY_SIZE ($BINARY_PATH)"
        else
            print_status "error" "Binary not found at expected path: $BINARY_PATH"
        fi
    else
        print_status "error" "Build failed for $target"
        ((FAILED_BUILDS++))
    fi
done

# Summary
echo ""
echo "📊 Build Summary"
echo "================"
print_status "success" "Successful builds: $SUCCESSFUL_BUILDS"
if [[ $FAILED_BUILDS -gt 0 ]]; then
    print_status "error" "Failed builds: $FAILED_BUILDS"
else
    print_status "success" "Failed builds: $FAILED_BUILDS"
fi

# Test basic functionality on native target
echo ""
print_status "info" "Testing basic functionality..."

# Build for native target
cargo build --release

# Test version command
if ./target/release/turbo-cdn --version; then
    print_status "success" "Version command works"
else
    print_status "error" "Version command failed"
    ((FAILED_BUILDS++))
fi

# Test help command
if ./target/release/turbo-cdn --help > /dev/null; then
    print_status "success" "Help command works"
else
    print_status "error" "Help command failed"
    ((FAILED_BUILDS++))
fi

# Final result
echo ""
if [[ $FAILED_BUILDS -eq 0 ]]; then
    print_status "success" "All tests passed! 🎉"
    echo ""
    echo "✅ Your build configuration is working correctly"
    echo "✅ Cross-platform compilation is functional"
    echo "✅ Binary functionality is verified"
    exit 0
else
    print_status "error" "Some tests failed! 💥"
    echo ""
    echo "❌ Please check the failed builds above"
    echo "❌ You may need to install additional dependencies"
    exit 1
fi

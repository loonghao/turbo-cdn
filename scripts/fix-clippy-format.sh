#!/bin/bash
# Bash script to automatically fix all clippy format string warnings
# This script will find and replace all instances of format!("{}", var) with format!("{var}")

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
GRAY='\033[0;37m'
NC='\033[0m' # No Color

# Default options
DRY_RUN=false
VERBOSE=false

# Function to print colored output
print_log() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  --dry-run    Show what would be changed without modifying files"
    echo "  --verbose    Show detailed output"
    echo "  --help       Show this help message"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            show_usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

print_log $CYAN "🔧 Turbo CDN - Clippy Format String Fixer"
print_log $CYAN "========================================="

if [ "$DRY_RUN" = true ]; then
    print_log $YELLOW "🔍 Running in DRY RUN mode - no files will be modified"
fi

# Function to fix format strings in a file
fix_format_strings() {
    local file_path=$1
    local change_count=0
    
    if [ ! -f "$file_path" ]; then
        print_log $RED "❌ File not found: $file_path"
        return 1
    fi
    
    # Create a temporary file for modifications
    local temp_file=$(mktemp)
    cp "$file_path" "$temp_file"
    
    # Pattern 1: format!("text {}", variable) -> format!("text {variable}")
    # Using sed with extended regex
    if sed -E 's/format!\s*\(\s*"([^"]*)\{\}([^"]*)",\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\)/format!("\1{\3}\2")/g' "$temp_file" > "${temp_file}.new"; then
        if ! cmp -s "$temp_file" "${temp_file}.new"; then
            ((change_count++))
            mv "${temp_file}.new" "$temp_file"
        else
            rm "${temp_file}.new"
        fi
    fi
    
    # Pattern 2: println!("text {}", variable) -> println!("text {variable}")
    if sed -E 's/println!\s*\(\s*"([^"]*)\{\}([^"]*)",\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\)/println!("\1{\3}\2")/g' "$temp_file" > "${temp_file}.new"; then
        if ! cmp -s "$temp_file" "${temp_file}.new"; then
            ((change_count++))
            mv "${temp_file}.new" "$temp_file"
        else
            rm "${temp_file}.new"
        fi
    fi
    
    # Pattern 3: eprintln!("text {}", variable) -> eprintln!("text {variable}")
    if sed -E 's/eprintln!\s*\(\s*"([^"]*)\{\}([^"]*)",\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\)/eprintln!("\1{\3}\2")/g' "$temp_file" > "${temp_file}.new"; then
        if ! cmp -s "$temp_file" "${temp_file}.new"; then
            ((change_count++))
            mv "${temp_file}.new" "$temp_file"
        else
            rm "${temp_file}.new"
        fi
    fi
    
    # Pattern 4: print!("text {}", variable) -> print!("text {variable}")
    if sed -E 's/print!\s*\(\s*"([^"]*)\{\}([^"]*)",\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\)/print!("\1{\3}\2")/g' "$temp_file" > "${temp_file}.new"; then
        if ! cmp -s "$temp_file" "${temp_file}.new"; then
            ((change_count++))
            mv "${temp_file}.new" "$temp_file"
        else
            rm "${temp_file}.new"
        fi
    fi
    
    # Check if any changes were made
    if ! cmp -s "$file_path" "$temp_file"; then
        print_log $GREEN "🔧 Fixed format strings in $file_path"
        
        if [ "$DRY_RUN" = false ]; then
            cp "$temp_file" "$file_path"
        fi
        rm "$temp_file"
        return 0
    else
        if [ "$VERBOSE" = true ]; then
            print_log $GRAY "✅ No changes needed in $file_path"
        fi
        rm "$temp_file"
        return 1
    fi
}

# Main execution
total_files=0
modified_files=0

# Find all Rust source files
if [ -d "src" ] || [ -d "tests" ] || [ -d "examples" ]; then
    rust_files=$(find src tests examples -name "*.rs" 2>/dev/null || true)
    
    if [ -n "$rust_files" ]; then
        file_count=$(echo "$rust_files" | wc -l)
        print_log $CYAN "🔍 Found $file_count Rust files to check"
        
        while IFS= read -r file; do
            if [ -n "$file" ]; then
                ((total_files++))
                if [ "$VERBOSE" = true ]; then
                    echo "Checking: $file"
                fi
                
                if fix_format_strings "$file"; then
                    ((modified_files++))
                fi
            fi
        done <<< "$rust_files"
    else
        print_log $YELLOW "⚠️  No Rust files found in src/, tests/, or examples/ directories"
    fi
else
    print_log $RED "❌ No src/, tests/, or examples/ directories found"
    exit 1
fi

echo ""
print_log $CYAN "📊 Summary:"
echo "  Total files checked: $total_files"
print_log $GREEN "  Files modified: $modified_files"

if [ "$DRY_RUN" = false ] && [ $modified_files -gt 0 ]; then
    echo ""
    print_log $CYAN "🚀 Running cargo fmt to ensure proper formatting..."
    cargo fmt --all
    
    echo ""
    print_log $CYAN "🧪 Running clippy to verify fixes..."
    if cargo clippy --all-targets --all-features -- -D warnings; then
        print_log $GREEN "✅ All clippy checks passed!"
    else
        print_log $RED "❌ Some clippy issues remain. Please check the output above."
        exit 1
    fi
elif [ "$DRY_RUN" = true ]; then
    echo ""
    print_log $YELLOW "🔍 Dry run completed. Run without --dry-run to apply changes."
fi

echo ""
print_log $GREEN "✨ Format string fix completed!"

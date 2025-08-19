#!/bin/bash

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    print_error "Not in a git repository"
    exit 1
fi

# Install json-schema-diff if not already installed
print_info "Checking if json-schema-diff is installed..."
if ! command -v json-schema-diff &> /dev/null; then
    print_info "Installing json-schema-diff with build-binary feature..."
    cargo install json-schema-diff --features=build-binary
else
    print_info "json-schema-diff is already installed"
fi

# Get the current branch
CURRENT_BRANCH=$(git branch --show-current)
print_info "Current branch: $CURRENT_BRANCH"

# Check if main branch exists
if ! git show-ref --verify --quiet refs/heads/main; then
    print_error "Main branch does not exist"
    exit 1
fi

# Find all .jsonschema files in the repository
print_info "Finding all .jsonschema files..."
SCHEMA_FILES=$(find . -name "*.jsonschema" -type f | sort)

if [ -z "$SCHEMA_FILES" ]; then
    print_warning "No .jsonschema files found in the repository"
    exit 0
fi

print_info "Found $(echo "$SCHEMA_FILES" | wc -l) schema files:"
echo "$SCHEMA_FILES"

# Create temporary directory for main branch files
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

print_info "Created temporary directory: $TEMP_DIR"

# Track if any breaking changes were found
BREAKING_CHANGES_FOUND=false
TOTAL_FILES=0
CHANGED_FILES=0
BREAKING_FILES=0

# Process each schema file
while IFS= read -r schema_file; do
    TOTAL_FILES=$((TOTAL_FILES + 1))
    print_info "Processing: $schema_file"
    
    # Get the file from main branch
    MAIN_FILE="$TEMP_DIR/$(basename "$schema_file")"
    
    # Check if file exists in main branch
    if git show "main:$schema_file" > "$MAIN_FILE" 2>/dev/null; then
        print_info "Comparing $schema_file with main branch version..."
        
        # Run json-schema-diff and capture output
        DIFF_OUTPUT=$(json-schema-diff "$MAIN_FILE" "$schema_file" 2>/dev/null || true)
        
        if [ -n "$DIFF_OUTPUT" ]; then
            CHANGED_FILES=$((CHANGED_FILES + 1))
            print_warning "Changes detected in $schema_file:"
            
            # Check if any changes are breaking
            FILE_HAS_BREAKING=false
            while IFS= read -r line; do
                if [ -n "$line" ]; then
                    echo "  $line"
                    # Check if this change is breaking
                    if echo "$line" | grep -q '"is_breaking":true'; then
                        FILE_HAS_BREAKING=true
                        BREAKING_CHANGES_FOUND=true
                    fi
                fi
            done <<< "$DIFF_OUTPUT"
            
            if [ "$FILE_HAS_BREAKING" = true ]; then
                BREAKING_FILES=$((BREAKING_FILES + 1))
                print_error "BREAKING CHANGES found in $schema_file"
            else
                print_info "Non-breaking changes in $schema_file"
            fi
            echo
        else
            print_info "No changes detected in $schema_file"
        fi
    else
        print_warning "File $schema_file does not exist in main branch (new file)"
        # New files are not considered breaking changes for this script
    fi
done <<< "$SCHEMA_FILES"

# Print summary
echo "=================================="
print_info "SUMMARY:"
print_info "Total schema files processed: $TOTAL_FILES"
print_info "Files with changes: $CHANGED_FILES"
if [ $BREAKING_FILES -gt 0 ]; then
    print_error "Files with breaking changes: $BREAKING_FILES"
else
    print_info "Files with breaking changes: $BREAKING_FILES"
fi
echo "=================================="

# Exit with error if breaking changes were found
if [ "$BREAKING_CHANGES_FOUND" = true ]; then
    print_error "Breaking changes detected! Please review the changes above."
    exit 1
else
    print_info "No breaking changes detected. All good!"
    exit 0
fi

#!/usr/bin/env bash
# Enhanced responsibility table generator with semantic analysis

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../core/lib.sh"

MODULE_DIR="$1"

if [ -z "$MODULE_DIR" ]; then
  die "Usage: $0 <module_directory>"
fi

if [ ! -d "$MODULE_DIR" ]; then
  die "Directory not found: $MODULE_DIR"
fi

README="$MODULE_DIR/readme.md"

if [ ! -f "$README" ]; then
  die "readme.md not found in $MODULE_DIR"
fi

# Analyze source files with semantic extraction
analyze_file() {
  local file="$1"
  local filename
  filename=$(basename "$file")

  local responsibility=""

  # Try //! comment first
  responsibility=$(grep -m 1 "^//!" "$file" 2>/dev/null | \
    sed 's|^//! *||; s|# ||' || echo "")

  # If empty, try extracting from pub struct/enum
  if [ -z "$responsibility" ]; then
    local type_name
    type_name=$(grep -m 1 "^pub struct\|^pub enum" "$file" 2>/dev/null | \
      awk '{print $3}' || echo "")

    if [ -n "$type_name" ]; then
      responsibility=$(echo "$type_name" | \
        sed 's/\([A-Z]\)/ \1/g; s/^ //; s/  / /g')
      responsibility="${responsibility,,} implementation"
    fi
  fi

  # Fallback to filename heuristics
  if [ -z "$responsibility" ]; then
    case "$filename" in
      lib.rs)
        responsibility="Module organization and public API exports"
        ;;
      mod.rs)
        responsibility="Submodule organization and re-exports"
        ;;
      error.rs|errors.rs)
        responsibility="Error type definitions and conversions"
        ;;
      config.rs|configuration.rs)
        responsibility="Configuration management and loading"
        ;;
      types.rs)
        responsibility="Type definitions and data structures"
        ;;
      client.rs)
        responsibility="Client implementation for external service"
        ;;
      server.rs)
        responsibility="Server implementation and request handling"
        ;;
      handler.rs|handlers.rs)
        responsibility="Request handler implementations"
        ;;
      middleware.rs)
        responsibility="Middleware components and processing"
        ;;
      routes.rs|routing.rs)
        responsibility="Route definitions and URL mapping"
        ;;
      storage.rs|store.rs)
        responsibility="Data storage and persistence layer"
        ;;
      cache.rs)
        responsibility="Caching layer implementation"
        ;;
      utils.rs|util.rs|helpers.rs|helper.rs|common.rs)
        responsibility="⚠️ ANTI-PATTERN - rename to specific responsibility"
        ;;
      *)
        local base
        base=$(basename "$filename" .rs)
        responsibility="TBD - $(echo "$base" | sed 's/_/ /g') module"
        ;;
    esac
  fi

  # Truncate if too long
  if [ ${#responsibility} -gt 60 ]; then
    responsibility="${responsibility:0:57}..."
  fi

  echo "$responsibility"
}

# Generate table
generate_table() {
  log INFO "Generating responsibility table for $MODULE_DIR"

  {
    echo ""
    echo "## Directory Structure"
    echo ""
    echo "### Source Files"
    echo ""
    echo "| File | Responsibility |"
    echo "|------|----------------|"

    # Process files in src/
    if [ -d "$MODULE_DIR/src" ]; then
      # First: lib.rs or main.rs
      for entry_file in lib.rs main.rs; do
        if [ -f "$MODULE_DIR/src/$entry_file" ]; then
          local resp
          resp=$(analyze_file "$MODULE_DIR/src/$entry_file")
          echo "| $entry_file | $resp |"
        fi
      done

      # Then: all other .rs files
      find "$MODULE_DIR/src" -maxdepth 1 -name "*.rs" -type f \
        ! -name "lib.rs" ! -name "main.rs" | sort | while read -r file; do
        local filename
        filename=$(basename "$file")
        local resp
        resp=$(analyze_file "$file")
        echo "| $filename | $resp |"
      done

      # Then: subdirectories
      find "$MODULE_DIR/src" -maxdepth 1 -type d ! -name "src" | sort | while read -r dir; do
        local dirname
        dirname=$(basename "$dir")

        local resp=""
        if [ -f "$dir/mod.rs" ]; then
          resp=$(analyze_file "$dir/mod.rs")
        else
          resp="TBD - $(echo "$dirname" | sed 's/_/ /g') module"
        fi

        echo "| $dirname/ | $resp |"
      done
    fi

    echo ""
    echo "**Notes:**"
    echo "- Entries marked 'TBD' require manual documentation"
    echo "- Entries marked '⚠️ ANTI-PATTERN' should be renamed to specific responsibilities"
    echo ""
  } >> "$README"

  log INFO "✅ Generated responsibility table in $README"
}

# Main execution
generate_table
track_progress "responsibility_table_$(basename "$MODULE_DIR")" "generated"

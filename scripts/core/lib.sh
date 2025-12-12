#!/usr/bin/env bash
# Core library for remediation automation

set -euo pipefail

# Color codes for output
readonly COLOR_RED='\033[0;31m'
readonly COLOR_GREEN='\033[0;32m'
readonly COLOR_YELLOW='\033[1;33m'
readonly COLOR_BLUE='\033[0;34m'
readonly COLOR_RESET='\033[0m'

# Logging levels
readonly LOG_LEVEL_DEBUG=0
readonly LOG_LEVEL_INFO=1
readonly LOG_LEVEL_WARN=2
readonly LOG_LEVEL_ERROR=3

# Current log level (default: INFO)
LOG_LEVEL="${LOG_LEVEL:-$LOG_LEVEL_INFO}"

# Structured logging
log() {
  local level="$1"
  shift
  local message="$*"
  local timestamp
  timestamp=$(date '+%Y-%m-%d %H:%M:%S')

  case "$level" in
    DEBUG)
      [ "$LOG_LEVEL" -le "$LOG_LEVEL_DEBUG" ] && \
        echo -e "${COLOR_BLUE}[DEBUG]${COLOR_RESET} $timestamp - $message" >&2
      ;;
    INFO)
      [ "$LOG_LEVEL" -le "$LOG_LEVEL_INFO" ] && \
        echo -e "${COLOR_GREEN}[INFO]${COLOR_RESET} $timestamp - $message" >&2
      ;;
    WARN)
      [ "$LOG_LEVEL" -le "$LOG_LEVEL_WARN" ] && \
        echo -e "${COLOR_YELLOW}[WARN]${COLOR_RESET} $timestamp - $message" >&2
      ;;
    ERROR)
      [ "$LOG_LEVEL" -le "$LOG_LEVEL_ERROR" ] && \
        echo -e "${COLOR_RED}[ERROR]${COLOR_RESET} $timestamp - $message" >&2
      ;;
  esac
}

# Error handling with context
die() {
  log ERROR "$@"
  exit 1
}

# Dry-run capability
dry_run_mode() {
  if [ "${DRY_RUN:-0}" = "1" ]; then
    log WARN "🔍 DRY RUN MODE - No changes will be applied"
    return 0
  else
    return 1
  fi
}

# Progress tracking
track_progress() {
  local task_id="$1"
  local status="$2"
  local progress_file="-remediation_progress.txt"

  echo "[$task_id] $status - $(date)" >> "$progress_file"
}

# Metrics collection
collect_metric() {
  local metric_name="$1"
  local metric_value="$2"
  local metrics_file="-remediation_metrics.txt"

  echo "$metric_name: $metric_value - $(date)" >> "$metrics_file"
}

# Export functions
export -f log die dry_run_mode track_progress collect_metric

"""Iron SDK - Pythonic API layer for Iron Cage agent protection.

This module provides decorators, context managers, and framework integrations
for protecting AI agents with budget tracking, PII detection, and reliability patterns.
"""

__version__ = "0.1.0"

# Re-export from iron_runtime
from iron_runtime import LlmRouter, Runtime

__all__ = ["LlmRouter", "Runtime"]

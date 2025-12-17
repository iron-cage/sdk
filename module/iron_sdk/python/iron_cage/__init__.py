"""Iron Cage - Python SDK for AI agent protection.

This module provides LlmRouter and Runtime for protecting AI agents
with budget tracking, safety controls, and LLM API proxying.

Example:
    from iron_cage import LlmRouter
    from openai import OpenAI

    with LlmRouter(api_key="ic_xxx", server_url="https://...") as router:
        client = OpenAI(base_url=router.base_url, api_key=router.api_key)
        response = client.chat.completions.create(
            model="gpt-4",
            messages=[{"role": "user", "content": "Hello!"}]
        )
"""

__version__ = "0.2.0"

# Import from compiled Rust module
from iron_cage.iron_cage import LlmRouter, Runtime

__all__ = ["LlmRouter", "Runtime"]

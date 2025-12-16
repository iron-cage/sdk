"""Type stubs for iron_sdk module."""

from typing import Optional, Tuple

__version__: str

class LlmRouter:
    """LLM Router - Local proxy server for OpenAI/Anthropic API requests.

    Creates a local HTTP server that intercepts LLM API requests,
    fetches real API keys from Iron Cage server, and forwards
    requests to the actual provider.
    """

    def __init__(
        self,
        api_key: Optional[str] = None,
        server_url: Optional[str] = None,
        cache_ttl_seconds: int = 300,
        budget: Optional[float] = None,
        provider_key: Optional[str] = None,
    ) -> None:
        """Create a new LlmRouter instance.

        Args:
            api_key: Iron Cage API token (required unless provider_key is set)
            server_url: Iron Cage server URL (required unless provider_key is set)
            cache_ttl_seconds: How long to cache API keys (default: 300)
            budget: Optional budget limit in USD
            provider_key: Direct provider API key (bypasses Iron Cage server)
        """
        ...

    @property
    def base_url(self) -> str:
        """Get the base URL for the OpenAI client.

        Returns URL like "http://127.0.0.1:52431/v1"
        """
        ...

    @property
    def api_key(self) -> str:
        """Get the API key to use with the OpenAI client."""
        ...

    @property
    def port(self) -> int:
        """Get the port the proxy is listening on."""
        ...

    @property
    def provider(self) -> str:
        """Get the auto-detected provider ("openai" or "anthropic")."""
        ...

    @property
    def is_running(self) -> bool:
        """Check if the proxy server is running."""
        ...

    @property
    def budget(self) -> Optional[float]:
        """Get current budget limit in USD (None if no budget set)."""
        ...

    @property
    def budget_status(self) -> Optional[Tuple[float, float]]:
        """Get budget status as (spent, limit) tuple in USD."""
        ...

    def total_spent(self) -> float:
        """Get total spent in USD (0.0 if no budget set)."""
        ...

    def set_budget(self, amount_usd: float) -> None:
        """Set budget limit in USD.

        Args:
            amount_usd: New budget limit in USD (e.g., 10.0 for $10)
        """
        ...

    def stop(self) -> None:
        """Stop the proxy server."""
        ...

    def __enter__(self) -> "LlmRouter":
        """Enter context manager."""
        ...

    def __exit__(self, exc_type: object, exc_val: object, exc_tb: object) -> None:
        """Exit context manager and stop the proxy."""
        ...

class Runtime:
    """Runtime - Agent lifecycle management.

    Manages agent processes and coordinates with Iron Cage subsystems.
    """

    def __init__(self, budget: float, verbose: Optional[bool] = None) -> None:
        """Create new runtime.

        Args:
            budget: Budget limit in USD
            verbose: Enable verbose logging (default: False)
        """
        ...

    @property
    def budget(self) -> float:
        """Get the budget limit."""
        ...

    @property
    def verbose(self) -> bool:
        """Get verbose setting."""
        ...

    def start_agent(self, script_path: str) -> str:
        """Start an agent from a Python script.

        Args:
            script_path: Path to the Python script

        Returns:
            Agent ID string
        """
        ...

    def stop_agent(self, agent_id: str) -> None:
        """Stop an agent.

        Args:
            agent_id: ID of the agent to stop
        """
        ...

    def get_metrics(self, agent_id: str) -> Optional[str]:
        """Get agent metrics as JSON string.

        Args:
            agent_id: ID of the agent

        Returns:
            JSON string with metrics or None if agent not found
        """
        ...

__all__: list[str]

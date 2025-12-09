"""Type stubs for iron_runtime."""

from typing import Optional, Any

class Runtime:
    """Agent runtime with safety and cost controls."""

    def __init__(self, budget: float, verbose: Optional[bool] = None) -> None:
        """Create a new Runtime instance.

        Args:
            budget: Budget limit in USD
            verbose: Enable verbose logging
        """
        ...

    def start_agent(self, script_path: str) -> str:
        """Start an agent from a Python script.

        Args:
            script_path: Path to the agent script

        Returns:
            Agent ID string
        """
        ...

    def stop_agent(self, agent_id: str) -> None:
        """Stop a running agent.

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


class LlmRouter:
    """LLM Router - Local proxy for OpenAI/Anthropic API requests.

    Creates a local HTTP server that intercepts LLM API requests,
    fetches real API keys from Iron Cage server, and forwards
    requests to the actual provider.

    Example:
        >>> from iron_runtime import LlmRouter
        >>> from openai import OpenAI
        >>>
        >>> router = LlmRouter(
        ...     api_key="ic_xxx",
        ...     server_url="https://api.iron-cage.io",
        ... )
        >>> client = OpenAI(base_url=router.base_url, api_key=router.api_key)
        >>> response = client.chat.completions.create(...)
        >>> router.stop()
    """

    def __init__(
        self,
        api_key: str,
        server_url: str,
        cache_ttl_seconds: int = 300,
    ) -> None:
        """Create a new LlmRouter instance.

        Args:
            api_key: Iron Cage API token (IC_TOKEN)
            server_url: Iron Cage server URL (required)
            cache_ttl_seconds: How long to cache API keys (default: 300)

        Raises:
            RuntimeError: If server fails to start
        """
        ...

    @property
    def base_url(self) -> str:
        """Get the base URL for the OpenAI client.

        Returns:
            URL like "http://127.0.0.1:52431/v1"
        """
        ...

    @property
    def api_key(self) -> str:
        """Get the API key to use with the OpenAI client.

        Returns:
            The IC_TOKEN which the proxy validates
        """
        ...

    @property
    def port(self) -> int:
        """Get the port the proxy is listening on."""
        ...

    @property
    def is_running(self) -> bool:
        """Check if the proxy server is running."""
        ...

    def stop(self) -> None:
        """Stop the proxy server."""
        ...

    def __enter__(self) -> "LlmRouter":
        """Context manager entry."""
        ...

    def __exit__(
        self,
        exc_type: Any,
        exc_val: Any,
        exc_tb: Any,
    ) -> None:
        """Context manager exit - stops the proxy server."""
        ...

"""Type stubs for Iron Cage Python bindings"""

from typing import Optional, Any

class Runtime:
    """Iron Cage runtime for AI agent safety and cost control"""

    def __init__(self, budget: float, verbose: bool = False) -> None:
        """
        Create new runtime with budget limit

        Args:
            budget: Maximum budget in dollars
            verbose: Enable verbose logging
        """
        ...

    def start_agent(self, script_path: str) -> str:
        """
        Start an AI agent from Python script

        Args:
            script_path: Path to agent Python script

        Returns:
            Agent ID (UUID string)
        """
        ...

    def stop_agent(self, agent_id: str) -> None:
        """
        Stop a running agent

        Args:
            agent_id: Agent ID returned from start_agent
        """
        ...

    def get_metrics(self, agent_id: str) -> Optional[str]:
        """
        Get agent metrics as JSON string

        Args:
            agent_id: Agent ID

        Returns:
            JSON string with metrics or None if agent not found
        """
        ...


class LlmRouter:
    """LLM Router - Local proxy for OpenAI/Anthropic API requests."""

    def __init__(
        self,
        api_key: Optional[str] = None,
        server_url: Optional[str] = None,
        cache_ttl_seconds: int = 300,
        budget: Optional[float] = None,
        provider_key: Optional[str] = None,
    ) -> None: ...

    @property
    def base_url(self) -> str: ...
    @property
    def api_key(self) -> str: ...
    @property
    def port(self) -> int: ...
    @property
    def is_running(self) -> bool: ...
    @property
    def provider(self) -> str: ...
    @property
    def budget(self) -> Optional[float]: ...
    @property
    def budget_status(self) -> Optional[tuple[float, float]]: ...

    def total_spent(self) -> float: ...
    def set_budget(self, amount_usd: float) -> None: ...
    def stop(self) -> None: ...

    def __enter__(self) -> "LlmRouter": ...
    def __exit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> None: ...

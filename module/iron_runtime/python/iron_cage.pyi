"""Type stubs for Iron Cage Python bindings"""

from typing import Optional

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

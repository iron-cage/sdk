# AI Safety Guardrails

**Concept:** Defense-in-depth protection validating inputs, filtering outputs, and authorizing actions before AI execution.

---

## User Need

AI agents in production face security risks that traditional security tools don't address:
- Prompt injection attacks manipulating agent behavior
- Sensitive data (PII, secrets) leaking in outputs
- Unauthorized actions executed without approval
- No visibility into what agents are actually doing

## Core Idea

Apply **defense-in-depth** at three control points:

```
Input → [VALIDATE] → Agent → [FILTER] → Output
                        ↓
                   [AUTHORIZE]
                        ↓
                     Action
```

1. **Input validation** - Detect and block prompt injection attempts
2. **Output filtering** - Redact PII, secrets, sensitive content before delivery
3. **Action authorization** - Whitelist allowed tools, require approval for high-risk operations

The insight: Security must be **inline** (blocking, not just detecting) and **AI-aware** (understanding LLM-specific attack vectors).

## Key Components

- **Prompt Classifier** - ML-based injection detection
- **PII Detector** - Pattern + NER-based sensitive data identification
- **Secret Scanner** - API keys, passwords, tokens in outputs
- **Policy Engine** - Tool whitelists, parameter validation, approval workflows

## Related Capabilities

- [Safe Execution](safe_execution.md) - Sandboxes complement guardrails
- [LLM Access Control](llm_access_control.md) - Budget enforcement is a form of safety
- [Observability](observability.md) - Audit trail for all safety decisions

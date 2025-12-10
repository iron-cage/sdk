# LLM Providers

**Purpose:** Multi-provider LLM access through unified API.

---

## User Need

Use any LLM provider without code changes, with automatic fallback.

## Core Idea

**Unified API abstracts provider differences:**

```
Agent --unified API--> Iron Cage --provider API--> OpenAI
                            |                  --> Anthropic
                            |                  --> Azure
                            +--fallback chain---> Google
```

## Supported Providers

| Provider | Models | Features |
|----------|--------|----------|
| OpenAI | GPT-4, GPT-3.5 | Full support |
| Anthropic | Claude 3 | Full support |
| Azure OpenAI | GPT-4, GPT-3.5 | Full support |
| Google | Gemini | Basic support |
| Local | Ollama, vLLM | Beta |

## Unified API

```python
# Same code works with any provider
response = cage.llm.chat(
    model="gpt-4",  # or "claude-3" or "gemini-pro"
    messages=[{"role": "user", "content": "Hello"}]
)
```

## IP Selection and Fallback

**Note:** In Pilot, admin binds IPs to agents. Developer selects which bound IP to use. Automatic fallback is future feature.

**Agent-Level IP Configuration (Future):**

```
Agent configured with multiple IPs:
- IP 1: OpenAI (primary, developer default)
- IP 2: Anthropic (backup, developer selects manually)
- IP 3: Azure (backup, developer selects manually)

Developer selects IP per request or configures priority.
```

**Pilot Scope:**
- Admin binds IPs to agents (minimal complexity)
- Developer selects which IP to use
- No automatic fallback (future feature)

**Developer Control:**
- Select IP/provider among allowed list
- Select model among allowed list
- High level of control for efficient development

## Cost Normalization

| Provider | GPT-4 Equivalent | Normalized Cost |
|----------|-----------------|-----------------|
| OpenAI | GPT-4 | $0.03/1K tokens |
| Anthropic | Claude 3 Opus | $0.015/1K tokens |
| Azure | GPT-4 | $0.03/1K tokens |

---

*Related: [002_secret_backends.md](002_secret_backends.md)*

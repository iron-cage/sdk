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

## Fallback Chains

```
Primary: OpenAI GPT-4
    | (if rate limited or down)
Fallback 1: Anthropic Claude 3
    | (if unavailable)
Fallback 2: Azure OpenAI GPT-4
```

## Cost Normalization

| Provider | GPT-4 Equivalent | Normalized Cost |
|----------|-----------------|-----------------|
| OpenAI | GPT-4 | $0.03/1K tokens |
| Anthropic | Claude 3 Opus | $0.015/1K tokens |
| Azure | GPT-4 | $0.03/1K tokens |

---

*Related: [secret_backends.md](secret_backends.md)*

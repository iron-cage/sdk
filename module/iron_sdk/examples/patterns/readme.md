# patterns/

Example implementations demonstrating Iron Cage protection patterns.

---

## Responsibility Table

| Entity | Responsibility | Input ’ Output | Scope | Out of Scope |
|--------|----------------|----------------|-------|--------------|
| `circuit_breaker.py` | Demonstrate circuit breaker pattern | LLM failures ’ Automatic fallback | Circuit breaker configuration, failure detection, provider fallback, recovery logic | NOT budget enforcement (’ cost_optimization.py), NOT PII handling (’ pii_handling.py), NOT rate limiting (’ rate_limiting.py) |
| `cost_optimization.py` | Show budget tracking and cost optimization | Budget constraints ’ Cost-optimized agent | Budget limits, token tracking, model selection for cost, cost attribution | NOT circuit breakers (’ circuit_breaker.py), NOT PII detection (’ pii_handling.py), NOT rate limits (’ rate_limiting.py) |
| `pii_handling.py` | Illustrate PII detection and redaction | Sensitive data ’ Protected output | PII detection patterns, redaction strategies, privacy protection, output filtering | NOT cost tracking (’ cost_optimization.py), NOT circuit breakers (’ circuit_breaker.py), NOT rate limiting (’ rate_limiting.py) |
| `rate_limiting.py` | Demonstrate rate limiting implementation | Request volume ’ Throttled execution | Rate limit configuration, token bucket algorithm, backpressure handling, quota management | NOT cost optimization (’ cost_optimization.py), NOT PII handling (’ pii_handling.py), NOT circuit breakers (’ circuit_breaker.py) |

---

## Running Examples

```bash
# Set API key
export OPENAI_API_KEY=sk-...

# Run circuit breaker example
python circuit_breaker.py

# Run cost optimization example
python cost_optimization.py

# Run PII handling example
python pii_handling.py

# Run rate limiting example
python rate_limiting.py
```

---

**Status:** Scaffolding (files created, implementation pending)

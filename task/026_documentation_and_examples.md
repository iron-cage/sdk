# Task 026: Documentation and examples

## Dependencies
- Task 019

## Context
Internal documentation is excellent (90+ docs). Public-facing documentation (README, examples, quick start) is needed for open source adoption.

Critical areas:
- `readme.md`
- `docs/`
- Examples directory (to be created)

## Implementation plan
1. Write README with quick start matching presentation flow.
2. Create examples directory: basic usage, LangChain integration, budget enforcement, PII detection.
3. Add architecture diagram (from presentation material).
4. Write contributing guide.
5. Generate CHANGELOG from git history.

## Acceptance criteria
- README quick start works on a fresh machine with `pip install iron-cage`.
- Each example runs successfully.
- Contributing guide covers: build, test, PR process.
- CHANGELOG covers all releases.

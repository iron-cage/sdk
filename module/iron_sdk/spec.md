# iron_sdk - Specification

**Module:** iron_sdk
**Layer:** 5 (Integration)
**Status:** Active

> **Specification Philosophy:** This specification focuses on architectural-level design and well-established knowledge. It describes what the module does and why, not implementation details or algorithms. Implementation constraints are minimal to allow flexibility. For detailed requirements, see spec/-archived_detailed_spec.md.

---

## Responsibility

Pythonic SDK layer for Iron Cage agent protection. Provides @protect_agent decorator, context managers, typed configurations, and framework integrations (LangChain, CrewAI, AutoGPT). Includes examples/ directory with runnable framework examples.

---

## Scope

**In Scope:**
- @protect_agent decorator for function-level protection
- Context managers (with Budget(...), with Protection(...))
- Typed configuration classes (BudgetConfig, SafetyConfig, ReliabilityConfig)
- Framework integrations (LangChain, CrewAI, AutoGPT)
- Examples directory (merged from iron_examples per ADR-006)
- Async/await support for async agents

**Out of Scope:**
- Core runtime functionality (see iron_runtime)
- OS-level sandboxing (see iron_sandbox)
- CLI functionality (see iron_cli_py)
- Direct PyO3 FFI (hidden behind Pythonic API)

---

## Dependencies

**Required Modules:**
- iron-cage - PyPI package containing iron_runtime (Rust runtime binary, automatically installed as pip dependency - users never interact with it directly)

**Required External:**
- Python 3.8+

**Installation:**
Users install ONLY `iron-sdk` - the `iron-cage` dependency is automatically installed by pip.

**Optional:**
- langchain - LangChain integration
- crewai - CrewAI integration
- autogpt - AutoGPT integration

---

## Core Concepts

**Key Components:**
- **Protect Decorator:** @protect_agent for transparent protection
- **Context Managers:** Resource management with Budget, Protection
- **Config Classes:** Typed configuration for BudgetConfig, SafetyConfig
- **Examples:** Framework integration examples in examples/ directory

---

## Integration Points

**Used by:**
- Developers - Python agents using SDK

**Uses:**
- iron_runtime - Via PyO3 FFI for actual protection

---

*For detailed API specification, see spec/-archived_detailed_spec.md*
*For examples, see examples/ directory*

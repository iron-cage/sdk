# Scope Boundaries

**Purpose:** Explicit definition of what's in and out of Iron Cage platform scope.

---

## User Need

Understand what Iron Cage does and doesn't do to avoid scope creep.

## Core Idea

**Clear boundaries prevent feature creep and maintain focus:**

## In Scope (Core Platform)

| Feature | Why In Scope |
|---------|--------------|
| **Agent Governance** | Core value prop: safety, cost, reliability |
| **Budget Control** | Prevent runaway costs |
| **Safety Guardrails** | PII detection, prompt injection blocking |
| **Multi-Provider Access** | Unified API across LLM providers |
| **Credential Management** | Secure token storage and injection |
| **Audit & Compliance** | SOC 2, GDPR, HIPAA support |
| **Real-Time Monitoring** | Dashboard with live agent state |

## Explicitly Out of Scope

| Feature | Why Excluded | Alternative |
|---------|--------------|-------------|
| **Agent Development IDE** | Not our expertise | Use VS Code, PyCharm |
| **LLM Fine-Tuning** | Different market | Use OpenAI fine-tuning |
| **Agent Marketplace** | Premature | May add in Year 2 |
| **Multi-Cloud Orchestration** | Too complex for pilot | Use Kubernetes directly |
| **Data Labeling** | Different problem domain | Use Label Studio |
| **Vector Database** | Agent responsibility | Use Pinecone, Weaviate |

## Deferred Features (Future)

| Feature | Pilot Status | Full Platform |
|---------|--------------|---------------|
| **Server-Side Execution** | Spec-only | Q2 2026 |
| **Windows Sandboxing** | Not supported | Explore alternatives |
| **Multi-Tenancy** | Single-tenant | Q3 2026 |
| **Advanced Analytics** | Basic metrics | Full BI dashboard |
| **Auto-Scaling** | Manual | K8s HPA |

## Boundary Enforcement

**How we maintain scope:**
- ADR required for new capabilities
- Spec must justify feature in pilot scope
- "Is this needed for 5-minute demo?" test
- Defer if answer is no

---

*Related: [002_business_constraints.md](002_business_constraints.md) | [004_trade_offs.md](004_trade_offs.md)*

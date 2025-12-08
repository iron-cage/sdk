# IronLang System Context Diagram

## Diagram Type

This is a **System Context Diagram** (C4 Model Level 1).

**Purpose**: Shows all actors (human and system) that interact with IronLang, without internal implementation details.

**Other names**: Context Diagram, Level 0 DFD, System Boundary Diagram

---

## System Context Diagram

```
                                    ┌─────────────────────────────────────────────────────────────────┐
                                    │                     EXTERNAL SYSTEMS                            │
                                    └─────────────────────────────────────────────────────────────────┘

    ┌──────────────┐      ┌──────────────┐      ┌──────────────┐      ┌──────────────┐      ┌──────────────┐
    │              │      │              │      │              │      │              │      │              │
    │   Database   │      │   REST API   │      │ File System  │      │    Cache     │      │   Object     │
    │  (Postgres,  │      │  (External   │      │   (Local,    │      │   (Redis,    │      │   Storage    │
    │   MySQL,     │      │   Services)  │      │    S3-like)  │      │   Memcached) │      │   (S3, GCS)  │
    │   SQLite)    │      │              │      │              │      │              │      │              │
    └──────┬───────┘      └──────┬───────┘      └──────┬───────┘      └──────┬───────┘      └──────┬───────┘
           │                     │                     │                     │                     │
           │                     │                     │                     │                     │
           │              ┌──────┴─────────────────────┴─────────────────────┴──────┐              │
           │              │                                                         │              │
           └──────────────┤                    CONNECTORS                           ├──────────────┘
                          │         (Language-agnostic via containers)              │
                          │                                                         │
                          │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐    │
                          │  │Postgres │  │  HTTP   │  │  File   │  │  Redis  │    │
                          │  │Connector│  │Connector│  │Connector│  │Connector│    │
                          │  └─────────┘  └─────────┘  └─────────┘  └─────────┘    │
                          │                                                         │
                          └────────────────────────┬────────────────────────────────┘
                                                   │
                                                   │ NDJSON over STDIN/STDOUT
                                                   │
                          ┌────────────────────────┴────────────────────────────────┐
                          │                                                         │
                          │                  IRONLANG RUNTIME                       │
                          │                                                         │
                          │  ┌─────────────────────────────────────────────────┐   │
                          │  │  - Message routing & validation                 │   │
                          │  │  - Authentication enforcement                   │   │
                          │  │  - Protocol version checking                    │   │
                          │  │  - Streaming orchestration                      │   │
                          │  │  - Error handling & logging                     │   │
                          │  └─────────────────────────────────────────────────┘   │
                          │                                                         │
                          └────────────────────────┬────────────────────────────────┘
                                                   │
                                                   │ NDJSON over STDIN/STDOUT
                                                   │
                          ┌────────────────────────┴────────────────────────────────┐
                          │                                                         │
                          │                     AI AGENTS                           │
                          │                                                         │
                          │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐    │
                          │  │ Code    │  │ Data    │  │Research │  │ Custom  │    │
                          │  │ Agent   │  │ Analyst │  │ Agent   │  │ Agent   │    │
                          │  └─────────┘  └─────────┘  └─────────┘  └─────────┘    │
                          │                                                         │
                          └────────────────────────┬────────────────────────────────┘
                                                   │
                                                   │
                    ┌──────────────────────────────┼──────────────────────────────┐
                    │                              │                              │
                    ▼                              ▼                              ▼
           ┌───────────────┐              ┌───────────────┐              ┌───────────────┐
           │               │              │               │              │               │
           │   END USER    │              │    AGENT      │              │   PLATFORM    │
           │               │              │   DEVELOPER   │              │   OPERATOR    │
           │  Interacts    │              │               │              │               │
           │  with AI      │              │  Builds AI    │              │  Deploys &    │
           │  agent via    │              │  agents that  │              │  manages      │
           │  chat/UI      │              │  use IronLang │              │  IronLang     │
           │               │              │  protocol     │              │  runtime      │
           └───────────────┘              └───────────────┘              └───────────────┘

                                    ┌─────────────────────────────────────────────────────────────────┐
                                    │                        HUMAN ACTORS                             │
                                    └─────────────────────────────────────────────────────────────────┘



                                                          ┌───────────────┐
                                                          │               │
                                           ┌──────────────│   CONNECTOR   │
                                           │              │   DEVELOPER   │
                                           │              │               │
                                           │              │  Builds data  │
                                           │              │  source       │
                                           │              │  connectors   │
                                           │              └───────────────┘
                                           │
                                           │  Publishes connectors
                                           ▼
                          ┌────────────────────────────────────────────────────────┐
                          │                                                        │
                          │                 CONNECTOR REGISTRY                     │
                          │            (Container images repository)               │
                          │                                                        │
                          └────────────────────────────────────────────────────────┘
```

---

## Actor Descriptions

### Human Actors

| Actor | Role | Interactions |
|-------|------|--------------|
| **End User** | Person who uses AI agents | Interacts via chat/UI, triggers agent operations, receives results |
| **Agent Developer** | Builds AI agents | Uses IronLang protocol to connect agents to data sources |
| **Connector Developer** | Builds data connectors | Implements IronLang protocol for specific data sources |
| **Platform Operator** | Manages infrastructure | Deploys runtime, manages connectors, monitors operations |

### System Components

| Component | Responsibility | Protocol |
|-----------|----------------|----------|
| **AI Agents** | Business logic, user interaction | NDJSON messages to Runtime |
| **IronLang Runtime** | Orchestration, security, routing | NDJSON bidirectional |
| **Connectors** | Data source interface | NDJSON messages from Runtime |
| **External Systems** | Data storage and retrieval | Native protocols (SQL, HTTP, etc.) |
| **Connector Registry** | Stores connector container images | Container pull protocol |

---

## Message Flow

```
End User                AI Agent              Runtime              Connector           Database
    │                      │                     │                     │                  │
    │──"Query sales"──────▶│                     │                     │                  │
    │                      │                     │                     │                  │
    │                      │──AUTH──────────────▶│                     │                  │
    │                      │                     │──AUTH──────────────▶│                  │
    │                      │                     │                     │──connect()──────▶│
    │                      │                     │                     │◀──OK─────────────│
    │                      │                     │◀──ACK───────────────│                  │
    │                      │◀──ACK───────────────│                     │                  │
    │                      │                     │                     │                  │
    │                      │──READ(SQL)─────────▶│                     │                  │
    │                      │                     │──READ(SQL)─────────▶│                  │
    │                      │                     │                     │──SELECT...──────▶│
    │                      │                     │                     │◀──rows───────────│
    │                      │                     │◀──ACK(data)─────────│                  │
    │                      │◀──ACK(data)─────────│                     │                  │
    │                      │                     │                     │                  │
    │◀──"Sales: $1.2M"─────│                     │                     │                  │
    │                      │                     │                     │                  │
```

---

## Trust Boundaries

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                              UNTRUSTED ZONE                                         │
│                                                                                     │
│   ┌─────────────┐                                                                   │
│   │  End User   │  - May provide malicious input                                    │
│   └─────────────┘  - Must be validated by agent                                     │
│                                                                                     │
└─────────────────────────────────────────────────────────────────────────────────────┘
                                        │
                                        ▼
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                              SEMI-TRUSTED ZONE                                      │
│                                                                                     │
│   ┌─────────────┐                                                                   │
│   │  AI Agent   │  - Trusted code, but operates on untrusted input                  │
│   └─────────────┘  - Must sanitize before sending to Runtime                        │
│                                                                                     │
└─────────────────────────────────────────────────────────────────────────────────────┘
                                        │
                                        ▼
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                               TRUSTED ZONE                                          │
│                                                                                     │
│   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐                          │
│   │   Runtime   │────▶│  Connector  │────▶│   Database  │                          │
│   └─────────────┘     └─────────────┘     └─────────────┘                          │
│                                                                                     │
│   - Runtime enforces authentication                                                 │
│   - Connector has credentials                                                       │
│   - Database contains sensitive data                                                │
│                                                                                     │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Related Diagram Types

| Diagram Type | C4 Level | Shows | When to Use |
|--------------|----------|-------|-------------|
| **System Context** | Level 1 | All actors + system boundary | High-level overview |
| **Container Diagram** | Level 2 | Deployable units (services, DBs) | Architecture decisions |
| **Component Diagram** | Level 3 | Internal modules/classes | Detailed design |
| **Code Diagram** | Level 4 | Classes, functions | Implementation details |
| **Sequence Diagram** | - | Message flow over time | Protocol understanding |
| **Data Flow Diagram** | - | How data moves through system | Security analysis |

---

## References

- [C4 Model](https://c4model.com/) - Software architecture diagrams
- [Simon Brown's C4 Model](https://www.infoq.com/articles/C4-architecture-model/) - Original author

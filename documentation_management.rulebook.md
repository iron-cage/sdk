---
status: active
version: 1.0
authority: iron_runtime
applicability: all_documentation
precedence: project_specific
---

# documentation_management

**Authority:** This rulebook is the authoritative standard for all documentation in iron_runtime project, covering Design Collections, module specifications, entity models, and reference documentation.

**Alternative names:** docs_management, iron_runtime_documentation

### Vocabulary

**Design Collection:** Directory of focused concept files (~30-50 lines each) covering specific domain (principles, architecture, deployment, etc.)

**Concept File:** Single-topic document in Design Collection, numbered sequentially (001_, 002_), focusing on one concept only

**Specification Philosophy:** Architectural-level focus describing what module does and why, not implementation details or algorithms. Minimal constraints to allow flexibility

**Entity:** Core domain object (User, Agent, Project, Inference Provider, IC Token) with defined relationships and attributes

**Budget Type:** Classification of budgets - Restrictive (blocks requests, agent budget only) or Informative (statistics only, all others)

**Inference Provider (IP):** LLM provider entity (OpenAI, Anthropic, etc.). NOT IP address

### Governing Principles

1. **Design-Level Focus:** Documentation describes what and why, not how. Implementation details archived or omitted
2. **Entity-Driven:** All architecture built on clear entities (User, Agent, Project, IP) with explicit 1:1, 1:N relationships
3. **Agent-Centric Control:** Agents are ONLY way to control budget. All other budgets informative
4. **Minimal Constraints:** Specifications don't over-constrain. Allow implementer flexibility
5. **Flat Structure:** H1→H3 hierarchy only. No H2, no H4 or deeper nesting
6. **Responsibility Tables:** All collections have tables with 5 columns (ID, Entity, Responsibility, Input→Output, Scope, Out of Scope)

### Scope

**Responsibilities:**
Defines documentation standards for iron_runtime project including Design Collection structure and organization (10 collections with numbered concept files), module specification format (simplified design-level specs with philosophy notes), entity model documentation (6 core entities with relationships), vocabulary management (canonical term definitions), architecture documentation (execution models, budget control, roles), and quality standards (H1→H3 structure, Responsibility Tables, cross-references). Codifies the design/principle level focus achieved through comprehensive documentation transformation.

**In Scope:**
- Design Collection structure (10 collections: principles, constraints, capabilities, architecture, deployment, security, integration, technology, protocol, decisions)
- Concept file format (NNN_ numbering, 30-50 lines, single topic)
- Module spec.md format (simplified, design-level, philosophy notes)
- Entity model documentation (User, Agent, Project, Master Project, IP, IC Token)
- Budget type taxonomy (restrictive vs informative)
- Vocabulary.md management (canonical term definitions)
- Responsibility Table requirements (5-column format)
- H1→H3 structure enforcement

**Out of Scope:**
- Implementation-level details (see archived detailed specs in module/*/spec/-archived_*)
- Code-level documentation (see organizational_principles.rulebook.md)
- Test documentation (see test_organization.rulebook.md)
- File system structure (see files_structure.rulebook.md)
- Commit messages and git workflow (never use git per CLAUDE.md)

### Quick Reference Summary

| Group | Rule | Description |
|-------|------|-------------|
| Design Collections | [10 Required Collections](#design-collections--10-required-collections) | Must have exactly 10 Design Collections |
| Design Collections | [Numbered Concept Files](#design-collections--numbered-concept-files) | Concept files use NNN_ format (001_, 002_, etc.) |
| Design Collections | [Collection Readme](#design-collections--collection-readme) | Each collection has readme.md with Responsibility Table |
| Design Collections | [Concept File Length](#design-collections--concept-file-length) | Concept files 30-50 lines targeting design-level |
| Module Specifications | [Philosophy Note Required](#module-specifications--philosophy-note-required) | All spec.md files have specification philosophy note |
| Module Specifications | [Design-Level Only](#module-specifications--design-level-only) | Specs focus on what/why, not how. No FR-x.y, NFR-x.y |
| Module Specifications | [Simplified Structure](#module-specifications--simplified-structure) | Specs have 6 sections: Philosophy, Responsibility, Scope, Dependencies, Core Concepts, Integration Points |
| Module Specifications | [Target Length](#module-specifications--target-length) | Specs 50-100 lines. Implementation details in archived specs |
| Entity Model | [Six Core Entities](#entity-model--six-core-entities) | Document User, Agent, Project, Master Project, IP, IC Token |
| Entity Model | [Relationships Explicit](#entity-model--relationships-explicit) | All 1:1, 1:N, N:M relationships stated explicitly |
| Entity Model | [IP Clarification](#entity-model--ip-clarification) | Always clarify IP means Inference Provider, NOT IP address |
| Budget Types | [Two Types Only](#budget-types--two-types-only) | Restrictive (agent budget) and Informative (all others) |
| Budget Types | [Agent Budget Enforcement](#budget-types--agent-budget-enforcement) | ONLY agent budget blocks requests. State clearly |
| Budget Types | [Informative Budgets](#budget-types--informative-budgets) | Project, IP, Master budgets statistics only, no blocking |
| Vocabulary | [Canonical Definitions](#vocabulary--canonical-definitions) | vocabulary.md is single source of truth for all terms |
| Vocabulary | [Entities Section](#vocabulary--entities-section) | Must have Entities section with all 6 core entities |
| Vocabulary | [Roles Section](#vocabulary--roles-section) | Must have Roles section (Admin, Super User, Developer) |
| Structure | [H1 to H3 Only](#structure--h1-to-h3-only) | Use H1→H3 hierarchy. No H2, no H4 or deeper |
| Structure | [Responsibility Tables](#structure--responsibility-tables) | All collection readmes have 5-column tables (ID, Entity, Responsibility, Input→Output, Scope, Out of Scope) |
| Cross-References | [Protocol References](#cross-references--protocol-references) | Budget control protocol at protocol/005, not architecture/006 |
| Cross-References | [No Two-Repo Mentions](#cross-references--no-two-repo-mentions) | No references to two-repo split, iron_cage repository |
| ADRs | [Active ADRs Only](#adrs--active-adrs-only) | Only ADR-002 through ADR-007. ADR-001 archived |
| ADRs | [Numbered Sequentially](#adrs--numbered-sequentially) | ADRs use adr_NNN_ format with sequential numbers |

### Design Collections : 10 Required Collections

The project must have exactly 10 Design Collections, each in its own directory under docs/:

1. **principles/** - Design philosophy, quality attributes, error handling, testing strategy, development workflow
2. **constraints/** - Technical constraints, business constraints, scope boundaries, trade-offs
3. **capabilities/** - Platform capabilities (8 capabilities)
4. **architecture/** - System architecture (execution models, layers, boundaries, data flow, integration, roles, entity model, runtime modes)
5. **deployment/** - Deployment concepts (package model, actors, distribution, scaling, module mapping)
6. **security/** - Security model (threat model, isolation layers, credential flow, audit)
7. **integration/** - External systems (LLM providers, secrets, identity, observability)
8. **technology/** - Technology choices (why Rust, PyO3, dependencies, infrastructure)
9. **protocol/** - Communication protocols (IronLang, REST API, WebSocket, MCP, budget control)
10. **decisions/** - Architecture Decision Records (ADR-002 through ADR-007)

**Rationale:** These 10 collections comprehensively cover design/principle level documentation without overlap. Each has distinct domain per Unique Responsibility Principle.

### Design Collections : Numbered Concept Files

Concept files within collections must use NNN_ numbering format (001_, 002_, 003_, etc.) for sequential organization.

**Format:** `NNN_descriptive_name.md` where NNN is zero-padded 3-digit number

**Examples:**
- ✅ `001_design_philosophy.md`
- ✅ `002_quality_attributes.md`
- ✅ `006_budget_control_protocol.md`
- ❌ `1_philosophy.md` (not zero-padded)
- ❌ `design_philosophy.md` (not numbered)

**Rationale:** Sequential numbering ensures consistent ordering, enables easy reference, shows collection size at glance.

### Design Collections : Collection Readme

Each Design Collection directory must have readme.md with:
- Purpose statement
- Directory Responsibilities table (5 columns)
- Collection overview table listing all concept files
- Cross-references to related collections

**5-Column Responsibility Table Format:**
```markdown
| ID | Entity | Responsibility | Input → Output | Scope | Out of Scope |
|----|--------|----------------|----------------|-------|--------------|
| 001 | **001_file.md** | [Responsibility] | [Question] → [Answer] | [What's included] | NOT [excluded] (→ cross-refs) |
```

**Rationale:** Responsibility Tables enforce Unique Responsibility Principle and enable One-Second Test for overlap detection.

### Design Collections : Concept File Length

Concept files must be concise, targeting 30-50 lines, not exceeding 70 lines.

**Structure:**
- Purpose statement
- User Need section
- Core Idea section
- 2-4 content sections (tables, lists, brief explanations)
- Related cross-references

**Rationale:** Concise files focus on single concept, easy to read, maintain design-level abstraction. Implementation details belong elsewhere.

### Module Specifications : Philosophy Note Required

All module spec.md files must include specification philosophy note after metadata, before first section.

**Required text:**
```markdown
> **Specification Philosophy:** This specification focuses on architectural-level design and well-established knowledge. It describes what the module does and why, not implementation details or algorithms. Implementation constraints are minimal to allow flexibility. For detailed requirements, see spec/-archived_detailed_spec.md.
```

**Special case for spec-only modules:**
```markdown
> **Specification Philosophy:** This specification focuses on architectural-level design and well-established knowledge. It describes what the module does and why, not implementation details or algorithms. This is a spec-only module - implementation planned for production phase. Schema definitions document intent, not enforce exact structure.
```

**Rationale:** Sets reader expectations, clarifies design vs implementation separation, explains archived detailed specs.

### Module Specifications : Design-Level Only

Specifications must focus on design level (what module does, why it exists) not implementation level.

**Forbidden:**
- FR-x.y functional requirements with code examples
- NFR-x.y non-functional requirements with exact thresholds
- Database schemas with column definitions
- API signatures with parameter lists
- Step-by-step algorithms
- Implementation timelines

**Required:**
- Module responsibility (1 paragraph)
- Scope (3-5 bullets in/out)
- Dependencies (names only)
- Core concepts (2-4 key components)
- Integration points (used by/uses)

**Rationale:** Design specs stay stable. Implementation details in archived specs or code. Prevents over-specification.

### Module Specifications : Simplified Structure

Simplified spec.md must have exactly these 6 sections in order:

1. **Header** - Module name, layer, status, philosophy note
2. **Responsibility** - One paragraph what module does
3. **Scope** - In Scope (3-5 bullets), Out of Scope (3-5 bullets with cross-refs)
4. **Dependencies** - Required Modules, Required External, Optional
5. **Core Concepts** - Key Components (2-4 components, one sentence each)
6. **Integration Points** - Used by, Uses (module names with brief purpose)

**Footer:** Cross-references to archived detailed spec and design concepts

**Rationale:** Consistent structure across all 16 modules. Predictable, scannable, complete at design level.

### Module Specifications : Target Length

Module specifications must be 50-100 lines (excluding archived detailed specs).

**Current average:** ~92 lines across 16 modules

**If exceeding 100 lines:**
- Check for implementation details (move to archived spec)
- Verify design-level focus
- Condense to essential design concepts

**Rationale:** 50-100 lines sufficient for architectural understanding. Longer specs indicate implementation creep.

### Entity Model : Six Core Entities

Entity model documentation must define all 6 core entities:

1. **User** - Person with role (Admin, Super User, Developer)
2. **Agent** - AI agent with IC Token and budget
3. **Project** - Collection of agents and Inference Providers
4. **Master Project** - Special project with ALL resources (admin only, required in Pilot)
5. **IP (Inference Provider)** - LLM provider with IP budget and tokens
6. **IC Token** - Agent authentication token (1:1 with agent)

**Location:** docs/architecture/007_entity_model.md

**Rationale:** Complete entity model shows all domain objects, relationships, ownership chains.

### Entity Model : Relationships Explicit

All entity relationships must be explicitly stated with cardinality.

**Format:** "[Entity] has [relationship] [other entity] ([cardinality])"

**Cardinality notation:**
- 1:1 - One-to-one (Agent ↔ IC Token)
- 1:N - One-to-many (User owns multiple Agents)
- N:M - Many-to-many (Users belong to Projects)

**Examples:**
- ✅ "Has exactly one IC Token (1:1)"
- ✅ "Owns multiple Agents (1:N)"
- ✅ "Belongs to Projects (N:M)"
- ❌ "Has IC Token" (no cardinality)

**Rationale:** Explicit cardinality prevents ambiguity about relationships.

### Entity Model : IP Clarification

Always clarify "IP" means Inference Provider, NOT IP address, on first usage in each document.

**First mention format:**
- "IP (Inference Provider)"
- "Inference Provider (IP)"
- "Inference Providers/IPs"

**Add note at top of docs/architecture/007_entity_model.md:**
```markdown
> **Note:** "IP" in this document means **Inference Provider** (e.g., OpenAI, Anthropic), NOT IP address.
```

**Rationale:** Prevents confusion with Internet Protocol (IP address).

### Budget Types : Two Types Only

Budget taxonomy has exactly two types:

**Restrictive Budget:**
- Agent Budget ONLY
- Blocks requests when exceeded
- Hard limit enforcement

**Informative Budgets:**
- Project Budget (shows project spending)
- IP Budget (shows provider spending)
- Master Budget (shows all spending)
- Statistics only, no blocking

**Rationale:** Clear distinction prevents confusion about which budgets enforce limits.

### Budget Types : Agent Budget Enforcement

Must clearly state "Agents are the ONLY way to control budget" in multiple locations:

Required locations:
- vocabulary.md (Budget Control term)
- capabilities/002_llm_access_control.md
- protocol/005_budget_control_protocol.md
- architecture/007_entity_model.md (Budget Control Principle section)

**Exact phrasing:** "Agents are the ONLY way to control budget"

**Rationale:** Fundamental design principle that must be unmistakable. Prevents expectation of project/IP budget enforcement.

### Budget Types : Informative Budgets

Project, IP, and Master budgets must be consistently described as "informative only" or "statistics only, no blocking."

**Never describe as:**
- ❌ "Per-project budget limits"
- ❌ "IP budget enforcement"
- ❌ "Budget cutoffs" (unless specifically agent budget)

**Always describe as:**
- ✅ "Project budget (informative)"
- ✅ "IP budget (statistics only)"
- ✅ "Master budget (shows spending, no blocking)"

**Rationale:** Prevents false expectations about non-agent budget capabilities.

### Vocabulary : Canonical Definitions

vocabulary.md is single source of truth for all terminology.

**Required sections (13 total):**
1. Platform
2. Architecture
3. Capabilities (8 Total)
4. Deployment Packages (5 Total)
5. Modules (20 Total)
6. Technology
7. Security
8. Entities (6 entities)
9. Roles (3 roles)
10. Tokens
11. Process
12. Deployment
13. Budget Control
14. Token Management

**Format:** Each section uses H3 (###), contains table with Term and Definition columns

**Rationale:** Centralized vocabulary prevents term fragmentation, ensures consistent usage across all documentation.

### Vocabulary : Entities Section

vocabulary.md must have Entities section defining all 6 core entities with relationships.

**Required entities:**
- Agent (with IC Token 1:1, budget 1:1, multiple IPs)
- Project (collection with project budget 1:1)
- Master Project (ALL resources, admin only, Pilot requirement)
- IP/Inference Provider (with IP budget, IP tokens)
- Agent Budget, Project Budget, IP Budget, Master Budget (with type classification)
- Budget Control (agents ONLY way to control)

**Format:** Single-line definition with key relationships and cardinality

**Rationale:** Entities section consolidates domain model in vocabulary for quick reference.

### Vocabulary : Roles Section

vocabulary.md must have Roles section defining exactly 3 roles:

1. **Admin** - Full Control Panel access, allocates budgets, manages all
2. **Super User** - Developer + read-only dashboard (own budgets only)
3. **Developer** - Regular user, runs agents, CLI + Dashboard (read-only own usage)

**Must state for all roles:** "CLI + Dashboard" or "CLI + Dashboard (read-only own)"

**Rationale:** Clarifies access model, eliminates confusion about CLI-only vs dashboard access.

### Structure : H1 to H3 Only

All documentation must use H1→H3 hierarchy only. No H2 (##), no H4 (####) or deeper.

**Allowed:**
- H1 (#) - Document title only
- H3 (###) - All sections

**Forbidden:**
- H2 (##) - Creates unnecessary nesting level
- H4 (####) - Too deep, hurts scannability
- H5, H6 - Excessive nesting

**Rationale:** Flat structure is scannable, avoids deep nesting that hurts readability. Two-level hierarchy (H1, H3) sufficient for all documentation.

### Structure : Responsibility Tables

All Design Collection readme.md files must have Responsibility Table with exactly 5 columns:

**Required columns (in order):**
1. **ID** - Concept file number (001, 002, etc.)
2. **Entity** - Filename in bold (e.g., **001_file.md**)
3. **Responsibility** - What file does (present tense verb)
4. **Input → Output** - Question answered → Answer provided
5. **Scope** - What's included
6. **Out of Scope** - What's excluded (with ≥3 cross-references to other files)

**Format:**
```markdown
| ID | Entity | Responsibility | Input → Output | Scope | Out of Scope |
|----|--------|----------------|----------------|-------|--------------|
| 001 | **001_file.md** | [What it does] | [Question] → [Answer] | [Included] | NOT [excluded] (→ file1), NOT [excluded] (→ file2), NOT [excluded] (→ file3) |
```

**Rationale:** 5-column format enforces uniqueness through Input→Output, prevents overlap via Out of Scope cross-references.

### Cross-References : Protocol References

Budget control protocol is at protocol/005_budget_control_protocol.md, not architecture/006.

**Correct references:**
- protocol/005
- protocol/005_budget_control_protocol.md
- ../protocol/005_budget_control_protocol.md

**Incorrect (former location):**
- ❌ architecture/006
- ❌ architecture/006_budget_control_protocol.md

**Rationale:** Budget control is communication protocol, belongs in protocol/ collection.

### Cross-References : No Two-Repo Mentions

Active documentation must not reference two-repository split or iron_cage repository.

**Forbidden terms:**
- Two-repo split
- Two-repository architecture
- iron_cage repository
- Separate repository

**Exception:** ADR-001 archived as historical context (not in active documentation)

**Rationale:** iron_runtime documentation stands alone. Two-repo split not implemented, creates confusion.

### ADRs : Active ADRs Only

Active ADRs are ADR-002 through ADR-007 only. ADR-001 (two-repo split) archived.

**Active decisions:**
- ADR-002: Rust-Python Boundary (PyO3)
- ADR-003: Client-Side Execution Primary
- ADR-004: Crate Renaming
- ADR-005: CLI Wrapper Architecture
- ADR-006: Package Consolidation
- ADR-007: Testing Philosophy (No Mocking)

**Rationale:** ADR-001 superseded, archived to show evolution but not active guidance.

### ADRs : Numbered Sequentially

ADRs use adr_NNN_name.md format with zero-padded sequential numbers.

**Format:** `adr_NNN_descriptive_name.md`

**Current sequence:** ADR-002, ADR-003, ADR-004, ADR-005, ADR-006, ADR-007

**Note:** Gap at ADR-001 is intentional (archived two-repo split)

**Rationale:** Sequential numbering shows chronology, zero-padding ensures proper sorting.

---

*For organizational governance principles, see $GENAI/code/rules/organizational_principles.rulebook.md*
*For knowledge management principles, see $GENAI/knowledge/knowledge.rulebook.md*

# Iron Cage Pilot Platform

**Version:** 1.0.0
**Status:** In Development (Week 0 of 8)
**Target:** Warsaw EXPO XXI Conference (Dec 16-17, 2025)
**Timeline:** 23 days remaining

---

## Overview

Minimal viable platform for conference demonstrations and early customer pilots. Implements safety (privacy protection) + cost control (budget enforcement) + runtime (Python-Rust gateway) for AI agents.

**NOT a throwaway demo** - this is what you sell to pilot customers as "Iron Cage Pilot Platform" ($10-25K pilot price).

### Scope

**Responsibility:** Warsaw conference demo specifications and execution plans (35+ features with secrets, 8-week build, $10-25K pricing).

**In Scope:**
- Pilot specifications (35+ features with secrets, acceptance criteria)
- Execution plans (8-week, 3-week, Day 1)
- Conference materials (Warsaw Dec 16-17, 2025)
- Technology stack and dependencies (Rust, Python, Vue)

**Out of Scope:**
- Full platform capabilities (see `/spec/`)
- Rust implementation (see `/runtime/`)
- Production architecture (see `/docs/architecture.md`)

---

## Directory Responsibility

**Scope:** Warsaw conference demo (23 days until Dec 16-17, 2025)

**Responsibility:** Complete pilot project documentation, execution plans, demo code, and conference materials

---

## Directory Responsibilities

| Entity | Responsibility | Input → Output | Scope | Out of Scope |
|--------|----------------|----------------|-------|--------------|
| **spec.md** | Define pilot platform product specification | Feature requirements → Complete spec | 35+ features with acceptance criteria, demo triggers (leads #34, #67, #85), pricing ($10-25K) | NOT full platform (→ /spec/), NOT implementation (→ /runtime/), NOT execution plans (→ execution/) |
| **development_references.md** | Provide master development quick reference | Developer onboarding → Essential file links | Quick navigation to specs, implementations, plans, architecture docs (START HERE) | NOT detailed specs (→ spec.md), NOT implementation guide (→ /runtime/PILOT_GUIDE.md), NOT execution tracking (→ execution/status.md) |
| **implementation.md** | Navigate to runtime implementation location | Implementation question → Runtime crate pointer | Points to /runtime/ crate, build commands, source structure | NOT specifications (→ spec.md), NOT implementation details (→ /runtime/PILOT_GUIDE.md), NOT execution plans (→ execution/) |
| **llm_inference_providers_landscape.md** | Analyze LLM provider capabilities and pricing | Provider selection question → Provider comparison | OpenAI, Anthropic, Azure, Google analysis, capabilities, pricing, rate limits, budget controls, fallback strategy | NOT integration (→ docs/integration/001_llm_providers.md), NOT capability (→ docs/capabilities/002_llm_access_control.md) |

---

## Quick Start

### For End Users (SDK Demo - Recommended)

**Most users should start here - no build required:**

**Prerequisites:**
- Python 3.8+ (`python --version`)
- uv package manager (`curl -LsSf https://astral.sh/uv/install.sh | sh`)

```bash
# Install SDK
uv pip install iron-sdk

# Run protected agent
python examples/basic_agent.py
```

See [`../module/iron_sdk/readme.md`](../module/iron_sdk/readme.md) for complete SDK documentation.

### For Platform Contributors (Build from Source)

<details>
<summary>Building Rust runtime from source (click to expand - only needed for contributing to Iron Cage)</summary>

**Prerequisites:**
- Rust 1.75+ (`rustup update`)
- Python 3.11+ (`python --version`)
- uv package manager (`curl -LsSf https://astral.sh/uv/install.sh | sh`)
- Node.js 18+ (`node --version`)

```bash
cd /home/user1/pro/lib/willbe/module/iron_cage

# 1. Build Rust runtime (in runtime/ crate)
cd runtime
cargo build --release
cd ..

# 2. Install Python dependencies
cd pilot/demo/agent
uv pip install -r requirements.txt

# 3. Run demo agent
../../../runtime/target/release/iron_cage_runtime lead_gen_agent.py --budget 50

# 4. Open control panel (separate terminal)
cd ../control_panel
npm install
npm start
# Open http://localhost:3000
```

See [`../runtime/PILOT_GUIDE.md`](../runtime/PILOT_GUIDE.md) for detailed build instructions.
</details>

---

## Documentation

- **[Development References](development_references.md)** - Master reference (START HERE for developers)
- **[Specification](spec.md)** - Complete 35-feature pilot spec (with secrets)
- **[Execution Plans](execution/)** - 8-week, 3-week, Day 1 plans
- **[Warsaw Conference](conferences/warsaw_2025/)** - Presentation + submission
- **[Implementation](implementation.md)** - Points to runtime/ crate
- **[Demo Guide](demo/readme.md)** - Run lead generation demo

### Implementation

The Rust implementation lives in the **runtime crate**, not in pilot/.

**See:** [`../runtime/PILOT_GUIDE.md`](../runtime/PILOT_GUIDE.md) for step-by-step build instructions.

---

## Features (28 total)

Grouped by capability:

| Capability | Features | Status | Demo Usage |
|------------|----------|--------|------------|
| **Runtime (Cap 8)** | #1-4 | ⬜ Not started | CLI + PyO3 bridge |
| **Safety (Cap 2)** | #5-8 | ⬜ Not started | privacy protection at lead #67 |
| **Cost (Cap 3)** | #9-15 | ⬜ Not started | Budget warning at lead #85, circuit breaker at #34 |
| **Control Panel (Cap 6)** | #19-24 | ⬜ Not started | Live monitoring UI |
| **Infrastructure** | #25-26 | ⬜ Not started | WebSocket API |
| **Demo** | #16-18 | ⬜ Not started | Lead gen agent + triggers |

**See:** [spec.md](spec.md) for complete feature details

---

## Timeline

**8-Week Plan (580 hours total with secrets):**
- Week 1-2: Core Runtime (Features #1-4)
- Week 3: Infrastructure (Features #25-26)
- Week 4-5: Safety (Features #5-8)
- Week 6-7: Cost + Safety Cutoff (Features #9-15)
- Week 8: Demo + Control Panel (Features #16-24, #27-28)

**3-Week Compressed (300 hours):**
- Week 1: Runtime + PII + Budget basics (Solo, 60h)
- Week 2: Control Panel + WebSocket (Team of 3, 100h)
- Week 3: Demo integration + rehearsal (Team of 3, 140h)

**Current Status:** Week 0 (preparation phase)

**See:** [execution/](execution/) for detailed plans

---

## Relationship to Full Platform

This pilot is a **SUBSET** of the full Iron Cage platform:

| Aspect | Pilot Platform | Full Platform |
|--------|----------------|---------------|
| **Timeline** | 8 weeks | 12-18 months |
| **Team** | 1-2 engineers | 6-10 engineers |
| **Features** | 28 (demo-critical) | 150+ (enterprise-grade) |
| **Capabilities** | Cap 2 Lite, Cap 3 Lite, Cap 8 Minimal | All 8 capabilities (full) |
| **Pricing** | $10-25K pilot | $100-300K/year platform |
| **Purpose** | Market validation + conference demo | Production enterprise deployment |

**See:** [../spec/](../spec/) for full capability specifications

---

## Success Criteria

### Conference Demo (Critical)
- ✅ Zero crashes during 30-minute demo
- ✅ Control Panel matches presentation screenshots
- ✅ Triggers fire at correct leads (#34, #67, #85)
- ✅ Demo completes in 28-30 minutes

### Pilot Sales (High Priority)
- ✅ 2-3 pilot contracts signed ($30-75K revenue)
- ✅ Customer integration in <8 hours
- ✅ 90-day pilot retention (renew or upgrade)

### Market Validation (Medium Priority)
- ✅ 5+ customer interviews
- ✅ Top 3 feature requests documented
- ✅ Pricing validated ($10-25K pilot acceptable)

---

## Next Steps

**Day 1 (4 hours):**
1. Customize presentation (name, company, conference)
2. Generate slides (HTML + PDF)
3. Practice Act 1 (slides 2-7, 10 minutes)
4. Create lead capture form + QR code

**Week 1 (40 hours):**
1. Core runtime scaffold (CLI + PyO3 "Hello World")
2. Configuration system (--budget flag)
3. Logging infrastructure (structured JSON)
4. Market validation (LinkedIn post, 10 emails)

**See:** [execution/](execution/) for detailed execution plans

---

## Directory Structure

```
pilot/
├── readme.md                      # This file
├── development_references.md      # Master dev reference (START HERE)
├── spec.md                        # Complete 28-feature specification
├── tech_stack.md                  # Complete technology inventory (Rust, Python, Vue)
├── crates.md                      # Rust dependency list (WHY each crate needed)
├── implementation.md              # Points to ../runtime/ for Rust code
├── demo/                          # Python agent + Vue control panel
├── execution/                     # Project management & timelines
└── conferences/                   # Conference-specific materials
    └── warsaw_2025/               # Warsaw EXPO XXI (Dec 16-17, 2025)
```

**Rust implementation:** See [`../runtime/`](../runtime/) crate

---

## Contact & Support

**Questions:** See [spec.md](spec.md) for complete specifications
**Status Updates:** See [execution/status.md](execution/status.md)
**Conference Materials:** See [conferences/warsaw_2025/](conferences/warsaw_2025/)

---

**Last Updated:** 2025-11-23
**Next Review:** Day 1 execution

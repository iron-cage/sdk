# Demo Infrastructure

**Purpose:** Demo components for Warsaw conference presentation (Python agent + Vue control panel)

**Last Updated:** 2025-11-24

### Scope

**Responsibility:** Demo components for Warsaw conference (Python agent + Vue control panel).

**Deployment Mode:** Pilot/Demo Mode architecture (single process, localhost deployment). All components run on presenter laptop. Production Mode architecture documented in [docs/deployment_packages.md](../../docs/deployment_packages.md).

**In Scope:**
- Python lead generation agent (LangChain)
- Vue control panel (real-time metrics visualization, localhost:5173)
- Test data (100 synthetic leads)

**Out of Scope:**
- Production deployment (demo only)
- Rust runtime implementation (see `/runtime/`)
- Pilot specifications (see `/pilot/spec.md`)

---

## Directory Contents & Responsibilities

| Directory/File | Responsibility | In Scope | Out of Scope (See) |
|----------------|----------------|----------|-------------------|
| **agent/** | Python demo agent implementation | lead_gen_agent.py, Python dependencies, test data | Rust runtime (→ /runtime/), React control panel (→ control panel/) |
| **control panel/** | React real-time control panel | TypeScript/React UI, WebSocket client, charts | Rust runtime (→ /runtime/), Python agent (→ agent/) |

---

## Quick Reference

| Component | Technology | Lines of Code | Status |
|-----------|-----------|---------------|--------|
| **agent/** | Python 3.11 + LangChain | ~300 LOC | Spec defined, not implemented |
| **control panel/** | React 18 + TypeScript | ~800 LOC | Spec defined, not implemented |

---

## Implementation Status

**As of 2025-11-24:**
- ❌ Python agent: Not implemented (spec exists in /pilot/spec.md Feature #16-18)
- ❌ React control panel: Not implemented (spec exists in /pilot/spec.md Feature #19-24)

**Priority:** Low (slides-only approach recommended for 23-day timeline)

**See:**
- **Specifications:** `/pilot/spec.md` (Features #16-24)
- **Technology stack:** `/pilot/tech_stack.md` (Python/React versions)
- **Execution plan:** `/pilot/execution/quick_start.md` (slides-only approach)

---

## Demo Flow (When Implemented)

1. **Agent startup:** User runs `iron_cage_runtime lead_gen_agent.py --budget 50`
2. **Control Panel connection:** Control Panel connects via WebSocket to runtime
3. **Lead processing:** Agent processes 100 leads from test_data/
4. **Real-time updates:** Control Panel shows live metrics (cost, safety, performance)
5. **privacy protection:** At lead #67, PII detected and redacted (demo trigger)
6. **Budget warning:** At lead #85, budget warning alert fires (demo trigger)
7. **Completion:** Final summary shown on control panel

**See:** `/conferences/warsaw_2025/presentation/talk_slides.md` for complete demo script

---

## Directory Structure

```
demo/
├── readme.md                    # This file
├── agent/                       # Python demo agent
│   ├── readme.md               # Agent implementation details
│   ├── lead_gen_agent.py       # LangChain agent (NOT IMPLEMENTED)
│   ├── requirements.txt        # Python dependencies
│   └── test_data/              # 100 synthetic leads
│       ├── readme.md           # Test data documentation
│       └── leads.csv           # Synthetic lead data
└── control panel/                   # React real-time control panel
    ├── readme.md               # Control Panel implementation details
    ├── package.json            # npm dependencies
    ├── tsconfig.json           # TypeScript configuration
    ├── vite.config.ts          # Vite build configuration
    ├── public/                 # Static assets
    │   ├── readme.md           # Public assets documentation
    │   └── favicon.ico
    └── src/                    # React source code
        ├── readme.md           # Source code documentation
        ├── main.tsx            # Entry point
        ├── App.tsx             # Main app component
        ├── components/         # React components (Features #19-24)
        ├── hooks/              # Custom hooks (WebSocket connection)
        └── types/              # TypeScript types
```

---

## Usage (When Implemented)

### Python Agent

```bash
cd /home/user1/pro/lib/willbe/module/iron_cage/pilot/demo/agent
python3.11 -m venv .venv
source .venv/bin/activate
uv pip install -r requirements.txt
python lead_gen_agent.py  # Standalone mode (no runtime)
```

### React Control Panel

```bash
cd /home/user1/pro/lib/willbe/module/iron_cage/pilot/demo/control panel
npm install
npm run dev  # Development server (http://localhost:5173)
npm run build  # Production build
```

### Full Demo (with Rust runtime)

```bash
# Terminal 1: Start runtime + agent
cd /home/user1/pro/lib/willbe/module/iron_cage/runtime
cargo run --release -- ../pilot/demo/agent/lead_gen_agent.py --budget 50

# Terminal 2: Start control panel
cd ../pilot/demo/control panel
npm run dev
# Open http://localhost:5173
```

---

## Related Documentation

**Specifications:**
- **Agent spec:** `/pilot/spec.md` lines 300-350 (Features #16-18)
- **Control Panel spec:** `/pilot/spec.md` lines 350-432 (Features #19-24)
- **Demo script:** `/conferences/warsaw_2025/presentation/talk_slides.md` Slide 18

**Implementation guides:**
- **Python dependencies:** `agent/requirements.txt`
- **React dependencies:** `control panel/package.json`
- **Technology stack:** `/pilot/tech_stack.md`

**Current status:**
- **Execution plan:** `/pilot/execution/quick_start.md` (slides-only recommended)
- **Implementation guide:** `/runtime/PILOT_GUIDE.md` (if implementing)

---

**Last Updated:** 2025-11-24
**Status:** Specification complete, implementation not started
**Next Review:** After Warsaw conference (Dec 17, 2025)

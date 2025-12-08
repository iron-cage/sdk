# MOVED: Pilot Platform Specification

**This file has been relocated.**

**New Location:** [`../pilot/spec.md`](..../pilot/spec.md)

**Reason:** Pilot project consolidated into dedicated `../pilot/` directory for better organization.

---

## Quick Navigation

- **Pilot Specification:** [`../pilot/spec.md`](..../pilot/spec.md)
- **Pilot Overview:** [`../pilot/readme.md`](..../pilot/readme.md)
- **Execution Plans:** [`../pilot/execution/`](..../pilot/execution/)
- **Warsaw Conference:** [`/conferences/warsaw_2025/`](../conferences/warsaw_2025/)

---

## What Moved

- ✅ `spec/pilot_platform.md` → `../pilot/spec.md`
- ✅ Execution plans → `../pilot/execution/`
- ✅ Conference materials → `/conferences/warsaw_2025/`
- ✅ All cross-references updated

---

## Directory Structure

```
iron_cage/
├── pilot/                     # ← NEW - Self-contained pilot project
│   ├── spec.md                # ← Pilot specification (28 features)
│   ├── readme.md              # Pilot overview
│   ├── execution/             # 8-week, 3-week, Day 1 plans
│   ├── conferences/           # Conference materials
│   ├── implementation/        # Rust runtime (to be built)
│   └── demo/                  # Python agent + React dashboard
└── spec/                      # Full platform specifications
    ├── capability_1_enterprise_data_access.md
    ├── capability_2_ai_safety_guardrails.md
    ├── capability_3_llm_access_control.md
    └── ...
```

---

**Migration Date:** 2025-11-23
**See:** [Pilot Project README](..../pilot/readme.md) for complete pilot documentation

# Pilot Project Execution Plans

**Current Week:** Week 0 (Preparation)
**Days Until Conference:** 23 days
**Timeline Status:** 62% behind schedule (originally planned Nov 22 start)

---

## Directory Responsibility

**Scope:** Warsaw pilot execution planning and status tracking

**Responsibility:** Define execution plans (8-week, quick start), track status, manage timeline decisions

**Deployment Mode:** Plans assume Pilot Mode implementation (single process, localhost deployment). Production Mode architecture documented separately in [docs/deployment_packages.md](../../docs/deployment_packages.md).

---

## Directory Responsibilities

| Entity | Responsibility | Input → Output | Scope | Out of Scope |
|--------|----------------|----------------|-------|--------------|
| **8_week_plan.md** | Define complete 8-week implementation plan | Implementation need → Detailed timeline | 580 hours, 35+ features with secrets, week-by-week breakdown, 3-person team allocation | NOT minimal approach (→ quick_start.md), NOT status tracking (→ status.md), NOT feature specs (→ ../spec.md) |
| **quick_start.md** | Define minimal viable demo plan | Quick path need → Streamlined approach | 80-120 hours, 4 core features, slides-only option, solo/small team | NOT full implementation (→ 8_week_plan.md), NOT status tracking (→ status.md), NOT presentation materials (→ ../conferences/) |
| **status.md** | Track weekly execution progress | Progress question → Status report | Decision log, risk panel, team availability, timeline tracking, success probability | NOT execution plans (→ 8_week_plan.md, quick_start.md), NOT feature specs (→ ../spec.md), NOT implementation guide (→ /runtime/PILOT_GUIDE.md) |

---

## Available Plans

### 1. 8-Week Execution Plan (Original)
**File:** [8_week_plan.md](8_week_plan.md)
**Timeline:** 8 weeks (56 days)
**Team:** Solo founder (320h) + Dev 1 (120h) + Dev 2 (100h)
**Total:** 580 hours
**Features:** All 35+ features (with secrets) included
**Status:** ⚠️ **INFEASIBLE** - Would need 540h in 23 days (23.5h/day)

### 2. Quick Start (Minimal Viable Demo)
**File:** [quick_start.md](quick_start.md)
**Timeline:** 1 week
**Features:** 4 core features (runtime, basic PII, basic budget, CLI demo)
**Status:** 💡 **RECOMMENDED** - Realistic fallback if team unavailable

---

## Decision Matrix

Choose based on:

| Scenario | Plan | Probability | Effort |
|----------|------|-------------|--------|
| **Full team available immediately** | 8-week (infeasible) | 0% | 580h (25.2h/day) |
| **Solo only (no devs)** | Quick start | 60% | 160h (40h/week) |
| **Slides-only (no demo)** | Presentation only | 90% | 60h (presentation practice) |

**See:** [status.md](status.md) for latest decision

---

## Resource Requirements

### Solo Founder
- **8-week plan:** 320 hours (40h/week)
- **Quick start:** 80 hours (40h/week) ✅ Sustainable

### Dev 1 (Rust/Backend)
- **8-week plan:** 120 hours (Week 3-7, 20h/week)
- **Quick start:** 40 hours (Week 1, 40h/week)

### Dev 2 (Web/Frontend)
- **8-week plan:** 100 hours (Week 5-7, 33h/week)
- **Quick start:** Optional (can skip control panel)

---

## Current Recommendation

**Execute:** Quick Start Plan (160 hours, 4 features, CLI demo)

**Rationale:**
1. Realistic timeline (1-2 weeks at 40h/week)
2. Deliverable for conference (CLI demo is technically credible)
3. Sustainable pace (no burnout)
4. Can upgrade to control panel if time permits

**See:** [quick_start.md](quick_start.md) for execution details

---

## Timeline Comparison

| Plan | Duration | Features | Team | Hours/Week | Success Probability |
|------|----------|----------|------|------------|---------------------|
| 8-week (extended) | 56 days | 35+ | 1-3 | 40h | N/A (timeline expired) |
| Quick start | 7-14 days | 4 | 1-2 | 40h | 60% |
| Presentation only | 1 day | 0 (slides) | 1 | 4h | 90% |

---

## Next Actions

1. **Review execution plans** (30 minutes)
2. **Choose execution path** based on team availability
3. **Update status.md** with chosen plan
4. **Begin implementation** following chosen plan

---

## File Index

- `8_week_plan.md` - Extended detailed plan (580h, all 35+ features with secrets)
- `quick_start.md` - Minimal viable demo (160h, 4 features)
- `status.md` - Current week progress tracking
- `readme.md` - This file

---

**Last Updated:** 2025-11-23
**Owner:** Solo Founder
**Next Review:** After Day 1 execution

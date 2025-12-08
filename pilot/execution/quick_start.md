# Quick Start - Iron Cage Conference Prep

**Current Status:** All materials ready, 8-week plan created
**Your Timeline:** 8 weeks until conference
**Your Team:** You (solo founder) + 2 developers (can join later)

---

### Scope

**Responsibility:** Minimal slides-only conference preparation (1 week, 80-120 hours, no code implementation)

**Deployment Mode:** This guide assumes Pilot Mode implementation (single process, localhost deployment). Production Mode architecture documented in [docs/deployment_packages.md](../../docs/deployment_packages.md).

**In Scope:**
- Immediate next steps for conference preparation (30 minutes to first deliverable)
- Slide customization workflow (name, company, conference details)
- Slide generation and preview process (Marp HTML/PDF output)
- Minimal viable Day 1 checklist (90 minutes total)
- Presentation rehearsal guidance (Act 1 focus, 10-minute timing, 5-10 rehearsals recommended)
- Market validation activities without code (warm contact emails, LinkedIn posts)
- Decision framework for slides-only vs implementation approaches
- Quick reference navigation to other documents (implementation guide, strategy, week plan)

**Out of Scope:**
- Full 8-week implementation plan with code (see `8_week_plan.md` for 580-hour, 3-person, 35-feature plan with secrets)
- Rust runtime implementation (see `/runtime/PILOT_GUIDE.md` for build instructions)
- Feature specifications and acceptance criteria (see `../spec.md` for all 35+ features)
- Rust crate dependencies (see `../crates.md` for complete dependency specifications)
- Technology stack setup (see `../tech_stack.md` for Rust/Python/React installation)
- Weekly status tracking (see `status.md` for decision log and progress updates)
- Demo agent or control panel implementation (see `../demo/` for Python agent and React control panel)

---

## What You Have Right Now

**✓ Complete 30-slide presentation** → `talk_slides.md`
**✓ 8-week execution plan** → `8_week_plan.md`
**✓ Product specification** → `../../spec.md`
**✓ Quick start guide** → `quick_start.md` (THIS IS YOUR NEXT STEP)

---

## What to Do in the Next 30 Minutes

### Step 1: Customize Your Slides (5 min)
```bash
cd /home/user1/pro/lib/willbe/module/iron_cage/business/presentations
nano talk_slides.md
```

**Edit line 40-42:**
- Replace `[Your Name]` with your name
- Replace `[Your Company]` with your company
- Replace `[Conference Name] • [Date]` with conference details

**Save and exit** (Ctrl+O, Enter, Ctrl+X)

### Step 2: Generate Slides (1 min)
```bash
./-generate_slides.sh
```

### Step 3: Preview Slides (2 min)
```bash
xdg-open talk_slides.html
# Press F11 for fullscreen
# Use arrow keys to navigate
```

**Verify:** No [placeholders] visible, all 30 slides render correctly

### Step 4: Review Quick Start Guide (2 min)
```bash
cat quick_start.md | less
```

**Understand:** Implementation path ahead (runtime, demo, rehearsal)

### Step 5: Begin Implementation (20 min)
Start with runtime setup and Feature #2 (PyO3 bridge).

---

## If You're Time-Constrained

**Minimum viable Day 1 (90 min):**
1. Customize slides (5 min)
2. Generate slides (1 min)
3. Practice slides 2-7 out loud ONCE (12 min)
4. Email 3 warm contacts (30 min)
5. Review spec/pilot_platform.md (30 min)
6. List your blockers/questions (12 min)

**Skip for now:**
- Recording yourself
- Creating lead form + QR code
- Repository audit

---

## If You Have Questions

**Presentation questions** → See `-marp_usage_guide.md`
**Implementation questions** → See `../../spec/pilot_platform.md`
**Strategy questions** → See `../strategy/capability_product_strategy.md`
**Week-by-week plan** → See `-8_week_execution_plan.md`

---

## Your Single Most Important Task Today

**Practice slides 2-7 out loud.**

Everything else can wait. If you can't deliver Act 1 confidently in 10 minutes, the rest doesnt matter.

**Do it now:**
1. Open `talk_slides.html` in browser
2. Go to slide 2
3. Set timer for 10 minutes
4. Present to empty room
5. Did you finish? Too fast? Too slow?
6. Do it again tomorrow.

**After 5 rehearsals, you'll be confident. After 10, you'll be great.**

---

## Current Working Directory

You're here:
```
/home/user1/pro/lib/willbe/module/iron_cage/business/-default_topic
```

Navigate to presentations:
```bash
cd ../presentations
```

---

## Day 1 Success Criteria

By end of today, you should have:
- [x] Slides customized
- [x] Slides generated (HTML + PDF)
- [x] Act 1 practiced at least once
- [x] 3-5 warm contacts emailed
- [x] Reviewed pilot platform spec

**If you complete all 5 → You're ahead of schedule.**
**If you complete 3 → You're on track.**
**If you complete 1 → At least practice Act 1.**

---

## What Happens After Day 1

**Day 2-5 (Week 1):**
- Set up Rust project skeleton
- Create module structure
- Add PyO3 dependencies
- Write first integration test

**Week 2:**
- Build core runtime (agent lifecycle)
- Get something runnable (hello world agent)
- Market validation continues

**Week 3-4:**
- Dev 1 joins (privacy protection + budget tracking)
- You focus on integration + discovery calls

**Week 5-6:**
- Dev 2 joins (control panel UI)
- Demo starts looking real

**Week 7:**
- Sprint week (all hands, polish everything)

**Week 8:**
- Final prep + conference

---

## Stop Overthinking, Start Executing

The plan is done. The slides are done. The spec is done.

**Your job now:**
1. Customize 3 lines in talk_slides.md
2. Run ./-generate_slides.sh
3. Practice slides 2-7
4. Report status

**That's it. Go.**

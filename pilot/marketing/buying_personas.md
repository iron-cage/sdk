# Iron Cage Buyer Personas

**Version:** 1.0
**Date:** 2025-12-03
**Purpose:** Decision framework for all content, positioning, and distribution choices

---

## Quick Reference

| Persona | Role | Value | Decision Authority | Content Priority | Distribution |
|---------|------|-------|-------------------|------------------|--------------|
| **Alex** | Enterprise CTO | $50-250K/year | Signs contracts >$100K | 1st (high value) | LinkedIn, Email |
| **Sarah** | SMB Tech Founder | $5-25K/year | Signs all purchases | 1st (volume) | HackerNews, Twitter |
| **Marcus** | VP Engineering | Influences Alex | Recommends, runs pilots | 2nd (gatekeeper) | LinkedIn |
| **Elena** | Senior Developer | Bottom-up demand | Can veto ("this sucks") | 3rd (amplifier) | HackerNews, GitHub |

**Strategic allocation:** 60% content for Alex/Sarah (buyers), 30% for Marcus (influencer), 10% for Elena (amplifier)

---

## Persona 1: Alex Chen - Enterprise CTO

### Core Profile
- **Age:** 38-52 | **Company:** 500-2,000 employees | **Industry:** B2B SaaS, FinTech, HealthTech
- **Budget authority:** $500K-$5M infrastructure spend
- **Reports to:** CEO or Board
- **Background:** 15-25 years tech (developer → VP Eng → CTO)

### The Trigger (What Makes Alex Search for Solutions)
1. **Recent production AI failure** - $47K cost overrun, data breach, or 3am cascade failure
2. **Upcoming compliance audit** - SOC 2, HIPAA renewal in 60-90 days
3. **Board pressure** - "Show AI ROI by Q2" mandate
4. **Competitor announcement** - Rival shipped AI feature, pressure to catch up
5. **New regulation** - EU AI Act, new GDPR enforcement

### Pain Priority (Ranked)
1. **"AI projects keep failing in production"** (3 attempts, 2 shutdowns, millions wasted)
2. **"I don't trust our AI cost projections"** (AWS bills fluctuate $20K-80K/month)
3. **"Compliance is blocking AI deployment"** (auditor rejected last implementation)
4. **"My team is firefighting at 3am"** (burnout, retention risk)
5. **"Board wants results fast"** (pressure to prove AI investment)

### Decision Criteria (In Order)
1. **Risk reduction** - "Can this prevent another $47K incident?"
2. **Time to value** - "Can we deploy in <30 days?"
3. **Vendor stability** - "Will this company exist in 2 years?"
4. **Team adoption** - "Will my engineers actually use this?"
5. **ROI proof** - "Show me 3 customers like me who succeeded"

### What Alex Reads (Content Preferences)
- **Format:** Case studies (3-5 min read), ROI calculators, war stories with numbers
- **Tone:** Direct, data-driven, no buzzwords ("AI is transformative" = instant close)
- **Proof:** Fortune 500 references, specific metrics ($X saved, Y% uptime improvement)
- **Where:** LinkedIn 6-7am daily, email newsletters (scans subject lines)
- **Shares:** Content that makes them look smart to CEO/Board

### Content Topics That Move Alex
- "How to Prevent $2.9M GDPR Fine with AI Agents (Checklist)"
- "SOC 2 Compliance for AI: What Auditors Actually Check"
- "ROI Calculator: AI Infrastructure Cost Savings"
- "War Story: The $47K Agent Loop (and How to Prevent It)"
- "Migration Path: Production AI in 30 Days (No 6-Month Rewrite)"

### Objections to Overcome
| Objection | Counter |
|-----------|---------|
| "We can build this in-house" | Show 6-12 month timeline + $500K cost vs 4-hour integration |
| "Too risky to adopt new tech" | Fortune 500 references, MIT open source (no lock-in) |
| "Our team doesn't know Rust" | Show Python-only integration, zero Rust knowledge required |
| "Budget's locked for 2026" | Position as cost reduction (pays for itself in 2 months) |

### Buying Journey
- **Awareness (Month 1-2):** "We have an AI production problem" → Content: War stories, failure analysis
- **Consideration (Month 3-4):** "What solutions exist?" → Content: Architecture comparisons, ROI calculators
- **Decision (Month 5-6):** "Which vendor?" → Content: Case studies, reference calls, pilot details

---

## Persona 2: Sarah Martinez - SMB Tech Founder

### Core Profile
- **Age:** 28-38 | **Company:** 10-50 employees, Series A/B | **Industry:** AI-first SaaS, MarTech
- **Budget authority:** $10K-$100K (every dollar scrutinized)
- **Role:** Co-Founder & CTO (or is the CEO)
- **Background:** 5-12 years tech (senior engineer → founder)

### The Trigger
1. **Surprise cloud bill** - Unexpected $5K+ OpenAI charge
2. **Near-miss security incident** - Almost leaked customer data, woke up sweating
3. **Investor pressure** - Demo day in 60 days, need working product
4. **Competitor announcement** - Panic ("we're falling behind")
5. **Team burnout** - Someone quit due to 3am on-call load

### Pain Priority
1. **"Burning $15K/month on OpenAI"** (investors asking why costs rising)
2. **"I'm the only one who can fix production"** (no team to delegate, 3am pages)
3. **"We almost leaked customer data"** (one mistake away from startup-killing incident)
4. **"Can't hire fast enough"** (need AI agents to do work of 5 engineers)
5. **"Bigger competitors moving faster"** (need edge to survive)

### Decision Criteria
1. **Price** - "Can we afford this on Series A budget?"
2. **Setup speed** - <1 day to value, no multi-week implementations
3. **Self-service** - Comprehensive docs, no "contact sales" gates
4. **Community** - Active Discord, GitHub issues, Stack Overflow
5. **Open source** - Can self-host if we run out of money

### What Sarah Reads
- **Format:** Technical deep-dives, open source projects, "show me the code"
- **Tone:** Engineering peer, not vendor sales pitch
- **Proof:** GitHub stars, HackerNews points, real code examples
- **Where:** HackerNews (all day), Twitter/X (daily), Discord communities
- **Shares:** Cool tech, novel architectures, things that make them look smart

### Content Topics That Move Sarah
- "How to Cut OpenAI Costs 60% Without Sacrificing Quality"
- "Self-Hosted AI Infrastructure on $500/Month Budget"
- "War Story: I Woke Up to $8,200 AWS Bill (Here's What I Built)"
- "Integration Guide: LangChain + Iron Cage in 4 Hours"
- "Scaling 10 → 10,000 Agents (Architecture Walkthrough)"

### Objections to Overcome
| Objection | Counter |
|-----------|---------|
| "Too expensive" | Show cost savings exceed subscription (ROI in 60 days) |
| "We'll outgrow this" | Show scaling path: self-hosted → cloud, 10 → 10,000 agents |
| "Another dependency" | MIT license, no lock-in, can fork if needed |
| "Our needs are unique" | Show extensibility, plugin architecture, open roadmap |
| "Not enough time to evaluate" | 14-day trial, 4-hour integration, cancel anytime |

### Buying Journey
- **Awareness (Week 1-2):** "Our AI costs are out of control" → HackerNews, Twitter threads
- **Consideration (Week 3-4):** "What's the fastest fix?" → Quick start guides, GitHub examples
- **Decision (Week 5-6):** "Can we afford this?" → Pricing transparency, ROI calculator, open source option

---

## Persona 3: Marcus Johnson - VP Engineering (Influencer)

### Core Profile
- **Age:** 35-48 | **Company:** 200-5,000 employees | **Industry:** E-commerce, Media, Enterprise SaaS
- **Reports to:** CTO or CPO | **Manages:** 30-150 engineers (3-8 teams)
- **Budget influence:** Recommends purchases, doesn't sign >$100K contracts

### Why Marcus Matters
- **Gatekeeper:** Alex (CTO) asks Marcus to evaluate 3-5 solutions and recommend one
- **Pilot runner:** Marcus runs 30-day pilot, reports results to Alex
- **Champion or killer:** Marcus's "this is great" or "don't waste money" determines deal

### Pain Priority
1. **"AI incidents killing team morale"** (3am pages, weekend firefighting, retention risk)
2. **"Can't predict AI infrastructure costs"** (CFO demands accurate forecasts)
3. **"Compliance slowing us down"** (security team blocking deployments)
4. **"Senior engineers leaving"** (on-call load unsustainable)
5. **"CTO wants faster AI deployment"** (pressure to ship, but safely)

### Decision Criteria
1. **Team impact** - "Will this reduce on-call burden?"
2. **Integration complexity** - "Can we adopt without 6-month migration?"
3. **Operational maturity** - "Does this have monitoring/logs/alerts we need?"
4. **Support quality** - "Can we get help at 2am if something breaks?"
5. **References** - "Who else in our industry uses this?"

### What Marcus Reads
- **Format:** Architecture case studies, post-mortems, "how we scaled X"
- **Tone:** Engineering leader peer (not vendor pitch)
- **Proof:** Real incidents, specific metrics, technical depth
- **Where:** LinkedIn 2-3x/week, internal team Slack (shares there)
- **Shares:** Content that helps their teams (posts to internal channels)

### Content Topics That Move Marcus
- "How to Reduce AI On-Call Incidents by 90%"
- "Post-Mortem: $2.4M ARR Lost to 6-Hour AI Outage"
- "Building Reliable AI Infrastructure Teams Love"
- "Observability for AI Agents: Metrics That Actually Matter"
- "How to Present AI Infrastructure ROI to CFO (Template)"

### Marcus's Role in Deal
1. **Awareness:** Brings problem to Alex ("Our AI on-call load is unsustainable")
2. **Consideration:** Evaluates 3-5 solutions, writes technical analysis doc
3. **Decision:** Recommends to Alex, runs pilot, reports results
4. **Adoption:** Champions internally, ensures team actually uses it

---

## Persona 4: Elena Rodriguez - Senior Developer (Amplifier)

### Core Profile
- **Age:** 26-35 | **Role:** Senior/Staff Engineer, AI Platform Team
- **Reports to:** Engineering Manager or Tech Lead
- **Budget influence:** None (but can veto: "this tool sucks, don't buy it")

### Why Elena Matters
- **Bottom-up demand:** "Our devs love this tool" gives Alex/Marcus confidence
- **Community amplifier:** One GitHub star, one HackerNews upvote compounds
- **Integration reality check:** If Elena says "docs are terrible", deal dies
- **Future buyer:** Today's senior dev is tomorrow's CTO (long-term relationship)

### Pain Priority
1. **"Debugging AI agent failures is a nightmare"** (no good tooling, hours wasted)
2. **"Our codebase is a mess"** (tech debt from rushing features)
3. **"I don't understand why costs spiked"** (no visibility into LLM usage)
4. **"Management keeps adding features"** (no time to fix foundations)
5. **"On-call rotation is killing me"** (considering switching teams/companies)

### Decision Criteria
1. **Developer experience** - Good docs? Good API? Good error messages?
2. **Learning value** - "Will this make me better at my craft?"
3. **Time savings** - "Does this reduce my toil?"
4. **Technical credibility** - "Built by people who understand the problem?"
5. **Community** - "Can I get help on Discord/Slack?"

### What Elena Reads
- **Format:** Code examples, architecture deep-dives, "how it works" explanations
- **Tone:** Engineer-to-engineer (no marketing fluff)
- **Proof:** GitHub stars, technical correctness, beautiful code
- **Where:** HackerNews (daily), Dev.to (weekly), Reddit r/rust, r/MachineLearning
- **Shares:** Cool open source, novel tech, things that make them look smart to peers

### Content Topics That Move Elena
- "How Iron Cage Actually Works (Architecture Deep-Dive)"
- "Rust Performance Optimization Techniques for AI"
- "Building Production-Grade AI Agents (Engineering Best Practices)"
- "Contributing to Iron Cage: Your First Pull Request"
- "Career Growth: IC → Staff → Principal Engineering Path"

### Elena's Role in Deal
1. **Awareness:** Sees on HackerNews front page or GitHub trending
2. **Consideration:** Tries it locally, evaluates code quality, reads source
3. **Decision:** Tells Marcus "this is really good" or "don't waste money"
4. **Adoption:** Integrates it, writes internal docs, champions to engineering peers

---

## Content Decision Framework

### Which Persona to Target? (Decision Tree)

**Question 1: What's the goal?**
- **Goal: Close enterprise deals (>$50K)** → Target Alex (70%) + Marcus (30%)
- **Goal: Volume SMB signups (<$25K)** → Target Sarah (80%) + Elena (20%)
- **Goal: Build community/awareness** → Target Elena (60%) + Sarah (40%)

**Question 2: What buying stage?**
- **Awareness (create urgency)** → Alex war stories, Marcus post-mortems
- **Consideration (solution education)** → Marcus architecture, Sarah technical guides
- **Decision (close deal)** → Alex case studies + ROI, Sarah pricing transparency

**Question 3: What format?**
- **Article (1,500-2,500 words)** → Alex (case studies), Marcus (post-mortems), Sarah (how-tos)
- **Video (6-12 min)** → Sarah (code walkthroughs), Marcus (architecture), Elena (deep-dives)
- **Quick post (<500 words)** → All personas (share widely)

### Example Content Mapping

| Content Piece | Primary Persona | Secondary | Goal | Distribution |
|--------------|----------------|-----------|------|--------------|
| "Why 85% of AI Projects Fail" | Alex | Marcus | Awareness (urgency) | LinkedIn, Email |
| "The $47K Agent Loop Breakdown" | Alex | Sarah | Awareness (fear) | LinkedIn, Medium |
| "Rust for AI: The Cloudflare Proof" | Marcus | Elena | Consideration | LinkedIn, HackerNews |
| "LangChain + Iron Cage in 4 Hours" | Sarah | Elena | Decision (remove friction) | HackerNews, Dev.to |
| "SOC 2 Compliance Checklist" | Alex | Marcus | Decision (overcome objection) | LinkedIn, Email, Gated PDF |
| "Iron Cage Architecture Deep-Dive" | Elena | Marcus | Adoption (credibility) | HackerNews, GitHub |

---

## Validation Checklist (Before Publishing)

**Before creating any content, answer:**

1. **Which persona is this for?** (Pick ONE primary)
2. **What pain point does this address?** (Top 3 for that persona?)
3. **What buying stage?** (Awareness, Consideration, Decision, Adoption?)
4. **What action should they take?** (CTA matches persona + stage?)
5. **Where will they see this?** (Distribute where that persona hangs out?)

**If you can't answer all 5 clearly → Don't create the content.**

---

## Distribution by Persona

| Channel | Alex (CTO) | Sarah (Founder) | Marcus (VP) | Elena (Dev) |
|---------|-----------|----------------|-------------|-------------|
| **LinkedIn** | ★★★ (primary) | ★ (low reach) | ★★★ (primary) | ★ (low engagement) |
| **Email Newsletter** | ★★★ (primary) | ★★ (if qualified) | ★★ (forwards to team) | ★ (rarely reads) |
| **HackerNews** | ★ (rarely visits) | ★★★ (primary) | ★★ (occasional) | ★★★ (primary) |
| **Twitter/X** | ★ (rare) | ★★★ (daily) | ★ (occasional) | ★★★ (daily) |
| **Reddit** | - | ★★ (specific subs) | - | ★★★ (primary) |
| **Dev.to** | - | ★★ (learning) | - | ★★★ (primary) |
| **Discord/Slack** | - | ★★ (communities) | ★ (internal) | ★★★ (primary) |
| **YouTube** | ★★ (case studies) | ★★★ (tutorials) | ★★ (architecture) | ★★★ (deep-dives) |
| **Conference Talks** | ★★★ (attends 2-3/year) | ★ (budget limited) | ★★ (speaks sometimes) | ★ (rare, prefers online) |

**Strategy:** Alex/Marcus = LinkedIn + Email | Sarah/Elena = HackerNews + Twitter + GitHub

---

## Monthly Persona Check-In

**First Monday of each month (30 minutes):**

1. **Sales call review:** What new pain points emerged? (Update persona doc)
2. **Content performance:** Which persona content drove conversions? (Double down)
3. **Market shifts:** Any new triggers/objections? (Adjust messaging)
4. **Persona validation:** Talk to 1-2 customers, confirm pain priorities still accurate

**Keep personas living documents, not static profiles.**

---

## Summary: The Decision Framework

**Every content decision flows from:**
1. **Persona** (Who is this for? Alex, Sarah, Marcus, or Elena?)
2. **Pain point** (What top-3 pain does this address?)
3. **Buying stage** (Awareness, Consideration, Decision, or Adoption?)
4. **Distribution** (Where does this persona consume content?)
5. **CTA** (What action matches their authority level?)

**Without personas:** Generic content, high impressions, low conversions
**With personas:** Targeted content, lower impressions, 15-30x better conversions

**File purpose:** Reference this before creating ANY content, positioning, or distribution decision.

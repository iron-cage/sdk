# Test Data - Synthetic Leads

**Purpose:** 100 synthetic leads for demo agent

**Last Updated:** 2025-11-24

### Scope

**Responsibility:** Synthetic lead data with embedded demo triggers (PII at #67, budget warning at #85).

**In Scope:**
- 100 synthetic company leads (leads.csv)
- Lead generator script
- Demo trigger definitions

**Out of Scope:**
- Real customer data (demo only)
- Agent implementation (see `../`)

---

## Directory Contents (When Created)

| File | Responsibility |
|------|----------------|
| **leads.csv** | 100 synthetic company leads |
| **-generator.py** | Script to generate synthetic leads |

**Format:**
```csv
company,industry,website,trigger
Acme Corp,SaaS,acme.com,none
...
MegaCorp LLC,Enterprise,megacorp.com,pii_at_67
DataViz Co,Analytics,dataviz.com,budget_at_85
...
```

**Demo triggers:**
- Lead #67: privacy protection (CEO email in response)
- Lead #85: Budget warning (90% threshold)

**Status:** NOT CREATED (need to implement)

---

**Last Updated:** 2025-11-24

# Control Panel - Source Code

**Purpose:** Vue/TypeScript source code for control panel

**Last Updated:** 2025-11-24

### Scope

**Responsibility:** Vue control panel source code (components, composables, types).

**In Scope:**
- 6 control panel components (Features #19-24)
- Custom composables (useWebSocket)
- TypeScript type definitions

**Out of Scope:**
- Feature specifications (see `/pilot/spec.md`)
- Static assets (see `../public/`)

---

## Directory Contents (When Implemented)

| Directory/File | Responsibility | Features |
|----------------|----------------|----------|
| **main.tsx** | React entry point | App initialization, root render |
| **App.tsx** | Main app component | Layout, component composition |
| **components/** | React UI components | 6 control panel panels (Features #19-24) |
| **hooks/** | Custom React hooks | useWebSocket (WebSocket connection) |
| **types/** | TypeScript types | AgentMetrics, PiiEvent, etc. |

**Components (when implemented):**
- LiveMetrics.tsx (Feature #19)
- BudgetPanel.tsx (Feature #20)
- ProtectionPanel.tsx (Feature #21)
- PerfPanel.tsx (Feature #22)
- ActivityLog.tsx (Feature #23)
- Notification.tsx (Feature #24)

**Status:** NOT IMPLEMENTED (empty directory)

**See:** `/pilot/spec.md` lines 350-432 for feature specifications

---

**Last Updated:** 2025-11-24

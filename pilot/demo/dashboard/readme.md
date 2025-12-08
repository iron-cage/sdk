# Vue Control Panel

**Purpose:** Real-time metrics control panel for Warsaw conference demo

**Last Updated:** 2025-11-24

### Scope

**Responsibility:** Vue control panel implementation for Warsaw demo (real-time metrics visualization).

**In Scope:**
- Vue 3.4 + TypeScript + Vite + Tailwind CSS
- 6 control panel components (Features #19-24)
- WebSocket client for real-time updates

**Out of Scope:**
- Production control panel
- Rust runtime (see `/runtime/`)
- Python agent (see `../agent/`)

---

## Directory Contents & Responsibilities

| Directory/File | Responsibility | In Scope | Out of Scope |
|----------------|----------------|----------|--------------|
| **package.json** | npm dependencies | React, TypeScript, Vite, TailwindCSS, Recharts, Socket.io | Python/Rust dependencies |
| **tsconfig.json** | TypeScript configuration | Compiler options, path aliases | Runtime config |
| **vite.config.ts** | Vite build configuration | Dev server, build settings, plugins | Webpack/Rollup |
| **public/** | Static assets | favicon.ico, images | Source code |
| **src/** | React source code | Components, hooks, types | Build artifacts |
| **src/components/** | React components | 6 control panel panels (Features #19-24) | Business logic |
| **src/hooks/** | Custom React hooks | useWebSocket (WebSocket connection) | API logic |
| **src/types/** | TypeScript type definitions | Agent metrics types | Implementation |

---

## Implementation Status

**As of 2025-11-24:**
- ❌ NOT IMPLEMENTED (empty directory)

**Priority:** Low (not needed for slides-only approach)

**See:** `/pilot/spec.md` lines 350-432 (Features #19-24)

---

## Features (When Implemented)

| Feature # | Component | Description |
|-----------|-----------|-------------|
| #19 | LiveMetrics.tsx | Agent status (running/stopped), current lead #, LLM provider |
| #20 | BudgetPanel.tsx | Real-time cost ($5.29/$50), cost projection, avg/lead |
| #21 | ProtectionPanel.tsx | privacy protections count, redaction mode, last incident |
| #22 | PerfPanel.tsx | Latency (p50/p95/p99), throughput (leads/min) |
| #23 | ActivityLog.tsx | Scrolling event log (LLM calls, PII, budget alerts) |
| #24 | Notification.tsx | Budget warning modal at 90% threshold |

---

## Technology Stack

```json
{
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "socket.io-client": "^4.6.0",
    "recharts": "^2.10.0",
    "axios": "^1.6.0",
    "date-fns": "^2.30.0"
  },
  "devDependencies": {
    "@types/react": "^18.2.0",
    "@vitejs/plugin-react": "^4.2.0",
    "typescript": "^5.0.0",
    "vite": "^5.0.0",
    "tailwindcss": "^3.4.0",
    "eslint": "^8.55.0"
  }
}
```

**See:** `/pilot/tech_stack.md` lines 185-254 for complete stack

---

## Usage (When Implemented)

```bash
cd /home/user1/pro/lib/willbe/module/iron_cage/pilot/demo/control panel

# Install dependencies
npm install

# Development server (hot reload)
npm run dev
# Open http://localhost:5173

# Production build
npm run build
npm run preview
```

---

## WebSocket Integration

**Connection to Rust runtime:**

```typescript
// src/hooks/useWebSocket.ts
import { useEffect, useState } from 'react';
import io from 'socket.io-client';

export function useWebSocket(url: string) {
  const [socket, setSocket] = useState(null);
  const [metrics, setMetrics] = useState(null);

  useEffect(() => {
    const newSocket = io(url);

    newSocket.on('metrics', (data) => {
      setMetrics(data);  // Update control panel in real-time
    });

    setSocket(newSocket);
    return () => newSocket.close();
  }, [url]);

  return { socket, metrics };
}
```

**Runtime sends:** JSON events every 100ms with current metrics

---

## Related Documentation

**Specifications:**
- **Control Panel spec:** `/pilot/spec.md` lines 350-432 (Features #19-24)
- **Technology stack:** `/pilot/tech_stack.md` lines 185-254
- **Demo script:** `/conferences/warsaw_2025/presentation/talk_slides.md` Slide 18

**Implementation:**
- **Rust runtime:** `/runtime/` (Feature #26 - WebSocket server)
- **Python agent:** `../agent/` (generates metrics)

---

**Last Updated:** 2025-11-24
**Status:** Specification complete, implementation not started
**Priority:** Low (optional for pilot, not in quick start)

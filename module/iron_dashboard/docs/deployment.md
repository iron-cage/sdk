# Iron Control Panel Frontend - Deployment Guide

**Responsibility:** Frontend deployment procedures, build configuration, and operational setup for the Iron Control Panel Vue.js dashboard.

**Audience:** DevOps engineers, frontend developers, operations staff

---

## Table of Contents

1. [Overview](#1-overview)
2. [Build Process](#2-build-process)
3. [Docker Deployment](#3-docker-deployment)
4. [nginx Configuration](#4-nginx-configuration)
5. [Environment Variables](#5-environment-variables)
6. [Troubleshooting](#6-troubleshooting)

---

## 1. Overview

The Iron Control Panel Frontend is a Vue 3 single-page application (SPA) served by nginx. The deployment follows a multi-stage Docker build pattern:
1. **Build Stage:** Compile TypeScript, bundle JavaScript, optimize assets (Node 20)
2. **Runtime Stage:** Serve static files and proxy API requests (nginx 1.27-alpine)

**Key Technologies:**
- Vue 3.5.24 + TypeScript 5.9.3
- Vite 7.2.4 (build tool)
- nginx 1.27-alpine (web server)
- TanStack Query + Pinia (state management)

---

## 2. Build Process

### 2.1 Local Development Build

**Prerequisites:**
- Node.js 20+
- npm 10+

**Development Server:**

```bash
cd module/iron_dashboard

# Install dependencies
npm ci

# Start dev server with HMR (http://localhost:5173)
npm run dev

# Check for TypeScript errors
npm run type-check

# Lint code
npm run lint
```

**Development Features:**
- Hot Module Replacement (HMR) - instant updates without page reload
- TypeScript checking - errors shown in terminal and browser
- Source maps - debug original TypeScript in DevTools
- API proxy - `/api` proxied to `http://localhost:3000` (configured in vite.config.ts)

### 2.2 Production Build

**Build for Production:**

```bash
cd module/iron_dashboard

# Install dependencies (clean install)
npm ci --prefer-offline --no-audit

# Build for production
npm run build

# Output: dist/ directory (~2-3MB gzipped)
```

**Build Output Structure:**

```
dist/
├── index.html           # Entry point (SPA shell)
├── assets/
│   ├── index-[hash].js      # Main bundle (~300KB gzipped)
│   ├── vendor-[hash].js     # Third-party deps (~400KB gzipped)
│   └── index-[hash].css     # Styles (~50KB gzipped)
├── favicon.ico
└── vite.svg
```

**Build Optimizations:**
- Code splitting - separate vendor and app bundles
- Tree shaking - remove unused code
- Minification - compress JavaScript and CSS
- Asset optimization - compress images and fonts
- Cache busting - unique hashes for each build

### 2.3 Build Configuration

**vite.config.ts:**

```typescript
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  build: {
    outDir: 'dist',
    sourcemap: false,      // Disable source maps in production
    chunkSizeWarningLimit: 1000,
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['vue', 'vue-router', 'pinia']  // Separate vendor bundle
        }
      }
    }
  },
  server: {
    proxy: {
      '/api': 'http://localhost:3000'  // Proxy API requests in dev
    }
  }
})
```

---

## 3. Docker Deployment

### 3.1 Dockerfile Structure

The frontend Dockerfile uses a two-stage build:

**Stage 1: Builder (node:20-alpine)**
- Install npm dependencies
- Run Vite production build
- Output: `dist/` directory

**Stage 2: Runtime (nginx:1.27-alpine)**
- Copy `dist/` from builder stage
- Copy `nginx.conf` configuration
- Expose port 80
- Run nginx in foreground

### 3.2 Build Docker Image

**Build Image:**

```bash
# From workspace root (iron_runtime/dev/)
docker build -f module/iron_dashboard/Dockerfile -t iron-dashboard:latest .

# Check image size (should be ~25-30MB)
docker images iron-dashboard:latest
```

**Build Arguments:**

```bash
# Override API URL at build time
docker build \
  --build-arg VITE_API_URL=https://api.example.com \
  -f module/iron_dashboard/Dockerfile \
  -t iron-dashboard:latest .
```

### 3.3 Run Frontend Container

**Standalone Deployment:**

```bash
docker run -d \
  -p 8080:80 \
  --name iron-dashboard \
  iron-dashboard:latest

# Access dashboard at http://localhost:8080
```

**With Backend (Docker Compose):**

See [../../docker-compose.yml](../../docker-compose.yml) for full stack deployment.

---

## 4. nginx Configuration

### 4.1 Configuration Overview

The nginx configuration (`nginx.conf`) handles:
- Serving Vue SPA static files
- Reverse proxy to backend API
- WebSocket proxy support
- Security headers
- Static asset caching

**Configuration File:** [../nginx.conf](../nginx.conf)

### 4.2 Key Configuration Sections

#### 4.2.1 SPA Routing

```nginx
location / {
  try_files $uri $uri/ /index.html;
}
```

**Purpose:** Handle Vue Router client-side routing. All unknown routes fallback to `index.html` (Vue Router takes over).

**Example:**
- User navigates to `/users` → nginx serves `index.html` → Vue Router renders Users page
- Browser refresh on `/users` → nginx serves `index.html` (not 404) → Vue Router restores state

#### 4.2.2 API Reverse Proxy

```nginx
location /api/ {
  proxy_pass http://backend:3000/api/;
  proxy_http_version 1.1;
  proxy_set_header Host $host;
  proxy_set_header X-Real-IP $remote_addr;
  proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
  proxy_set_header X-Forwarded-Proto $scheme;

  proxy_connect_timeout 60s;
  proxy_send_timeout 60s;
  proxy_read_timeout 60s;
}
```

**Purpose:** Proxy API requests to backend (eliminates CORS issues).

**Flow:**
1. Browser sends `GET /api/users` → nginx (port 80)
2. nginx proxies to `http://backend:3000/api/users` (internal Docker network)
3. Backend processes request and returns JSON
4. nginx forwards response to browser

#### 4.2.3 WebSocket Proxy

```nginx
location /ws {
  proxy_pass http://backend:3000/ws;
  proxy_http_version 1.1;
  proxy_set_header Upgrade $http_upgrade;
  proxy_set_header Connection "upgrade";

  proxy_connect_timeout 7d;
  proxy_send_timeout 7d;
  proxy_read_timeout 7d;
}
```

**Purpose:** Support WebSocket connections for real-time updates.

**Timeouts:** Long timeouts (7 days) for persistent connections.

#### 4.2.4 Security Headers

```nginx
add_header X-Content-Type-Options "nosniff" always;
add_header X-Frame-Options "DENY" always;
add_header X-XSS-Protection "1; mode=block" always;
add_header Referrer-Policy "strict-origin-when-cross-origin" always;
```

**Purpose:** Protect against common web vulnerabilities.

**Headers Explained:**
- `X-Content-Type-Options`: Prevent MIME type sniffing
- `X-Frame-Options`: Prevent clickjacking attacks
- `X-XSS-Protection`: Enable browser XSS filtering
- `Referrer-Policy`: Control referrer information leakage

#### 4.2.5 Static Asset Caching

```nginx
location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
  expires 1y;
  add_header Cache-Control "public, immutable";
}
```

**Purpose:** Cache static assets for 1 year (immutable files with hashed names).

**Benefit:** Reduces bandwidth usage and improves page load times.

---

## 5. Environment Variables

### 5.1 Build-Time Variables

**VITE_API_URL (required)**

- **Description:** Backend API base URL (used by frontend JavaScript)
- **Default:** `/api` (relative URL, proxied by nginx)
- **Production:** Can be absolute URL (e.g., `https://api.example.com`)
- **Set In:** Dockerfile `ARG VITE_API_URL=/api`

**Usage in Code:**

```typescript
// src/config.ts
export const API_BASE_URL = import.meta.env.VITE_API_URL || '/api'

// API client uses this URL
const response = await fetch(`${API_BASE_URL}/users`)
```

**Override at Build Time:**

```bash
# Build with custom API URL
docker build \
  --build-arg VITE_API_URL=https://api.example.com \
  -f module/iron_dashboard/Dockerfile \
  -t iron-dashboard:latest .
```

### 5.2 Runtime Variables (nginx)

nginx does not support runtime environment variables directly. Configuration is static at build time.

**For Dynamic Configuration:** Use a startup script to generate `nginx.conf` from a template with environment variable substitution.

---

## 6. Troubleshooting

### 6.1 Build Failures

**Symptom:** `npm run build` fails with TypeScript errors

**Diagnosis:**

```bash
npm run type-check
```

**Solutions:**
- Fix TypeScript errors shown in terminal
- Check `tsconfig.json` configuration
- Ensure all dependencies are installed: `npm ci`

---

**Symptom:** Build runs out of memory (OOM killed)

**Solutions:**
- Increase Docker build memory limit (8GB recommended)
- Set Node.js memory limit: `NODE_OPTIONS="--max-old-space-size=4096" npm run build`

---

### 6.2 Container Startup Issues

**Symptom:** Container exits immediately after startup

**Diagnosis:**

```bash
docker logs iron-dashboard
```

**Common Causes:**
- Missing `dist/` directory - rebuild image
- nginx configuration syntax error - validate with `nginx -t`

---

**Symptom:** Container starts but serves 403 Forbidden

**Diagnosis:**

```bash
# Check file permissions in container
docker exec iron-dashboard ls -la /usr/share/nginx/html
```

**Solutions:**
- Ensure `dist/` files were copied correctly
- Check nginx user has read permissions

---

### 6.3 Runtime Issues

**Symptom:** Dashboard loads but API calls fail with 502 Bad Gateway

**Diagnosis:**

```bash
# Check nginx error logs
docker logs iron-dashboard 2>&1 | grep error

# Check if backend is reachable from frontend container
docker exec iron-dashboard wget -O- http://backend:3000/api/health
```

**Solutions:**
- Verify backend container is running: `docker ps | grep backend`
- Check Docker network connectivity
- Verify backend health: `curl http://localhost:8080/api/health`

---

**Symptom:** Vue Router routes return 404 on page refresh

**Diagnosis:**

```bash
# Check nginx access logs
docker logs iron-dashboard | grep "GET /users"
```

**Cause:** Missing `try_files` fallback in nginx config

**Solution:** Verify nginx.conf has:
```nginx
location / {
  try_files $uri $uri/ /index.html;
}
```

---

**Symptom:** Static assets not loading (JS/CSS 404 errors)

**Diagnosis:**

```bash
# Check browser DevTools Network tab
# Look for failed requests to /assets/

# List files in container
docker exec iron-dashboard ls -la /usr/share/nginx/html/assets/
```

**Solutions:**
- Rebuild image (build may have failed silently)
- Check Vite build output in `dist/assets/`
- Verify nginx is serving from correct directory

---

**Symptom:** WebSocket connections fail

**Diagnosis:**

```bash
# Check browser DevTools Console
# Look for WebSocket connection errors

# Test WebSocket endpoint
curl -i -N -H "Connection: Upgrade" -H "Upgrade: websocket" http://localhost:8080/ws
```

**Solutions:**
- Verify nginx WebSocket proxy configuration (see § 4.2.3)
- Check backend WebSocket endpoint is functional
- Ensure `proxy_http_version 1.1` is set

---

## Related Documentation

- [Architecture](architecture.md) - Frontend architecture and design patterns
- [Development Setup](development_setup.md) - Local development environment
- [API Integration](api_integration.md) - Backend API integration guide
- [../../docs/deployment/006_docker_compose_deployment.md](../../docs/deployment/006_docker_compose_deployment.md) - Full stack deployment architecture
- [../../docs/deployment_guide.md](../../docs/deployment_guide.md) § Pilot Deployment - Operational procedures

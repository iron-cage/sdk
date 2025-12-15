# Multi-stage build for Iron Control Panel Frontend

# Stage 1: Build Vue application
FROM node:20-alpine AS builder

WORKDIR /app

# Copy package files
COPY module/iron_dashboard/package.json module/iron_dashboard/package-lock.json ./

# Install dependencies
RUN npm ci --prefer-offline --no-audit

# Copy source code
COPY module/iron_dashboard/ ./

# Build Vue application
ARG VITE_API_URL=/api
ENV VITE_API_URL=${VITE_API_URL}
RUN npm run build

# Stage 2: Serve with nginx
FROM nginx:1.27-alpine

# Copy built files from builder
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy nginx configuration
COPY module/iron_dashboard/nginx.conf /etc/nginx/conf.d/default.conf

# Health check
HEALTHCHECK --interval=30s --timeout=10s --retries=3 \
  CMD wget --quiet --tries=1 --spider http://localhost:80 || exit 1

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]

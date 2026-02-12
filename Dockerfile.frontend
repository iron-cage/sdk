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
ARG SERVER_PORT=3001
ENV SERVER_PORT=${SERVER_PORT}
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy template
COPY module/iron_dashboard/nginx.conf.template /etc/nginx/templates/default.conf.template

# Config generation
CMD envsubst '$SERVER_PORT' \
    < /etc/nginx/templates/default.conf.template \
    > /etc/nginx/conf.d/default.conf \
 && nginx -g 'daemon off;'

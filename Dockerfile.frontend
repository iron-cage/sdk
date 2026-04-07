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

COPY --from=builder /app/dist /usr/share/nginx/html
COPY module/iron_dashboard/nginx.conf.template /etc/nginx/conf.d/default.conf

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]

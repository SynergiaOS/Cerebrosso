# 🥷 Cerberus Phoenix v2.0 - Ultra Dashboard PWA Dockerfile
# Multi-stage build for optimized PWA deployment

# 🏗️ Dependencies stage
FROM node:20-alpine AS deps
RUN apk add --no-cache libc6-compat
WORKDIR /app

# 📋 Copy package files
COPY package.json package-lock.json* ./
RUN npm ci --only=production

# 🏗️ Builder stage
FROM node:20-alpine AS builder
WORKDIR /app

# 📋 Copy dependencies
COPY --from=deps /app/node_modules ./node_modules
COPY . .

# 🔧 Set environment variables
ENV NEXT_TELEMETRY_DISABLED 1
ENV NODE_ENV production

# 🏗️ Build the application
RUN npm run build

# 🚀 Runner stage
FROM node:20-alpine AS runner
WORKDIR /app

# 🔧 Set environment variables
ENV NODE_ENV production
ENV NEXT_TELEMETRY_DISABLED 1

# 👤 Create non-root user
RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs

# 📁 Copy built application
COPY --from=builder /app/public ./public

# 📋 Set correct permissions for prerender cache
RUN mkdir .next
RUN chown nextjs:nodejs .next

# 📁 Copy built files with correct permissions
COPY --from=builder --chown=nextjs:nodejs /app/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/.next/static ./.next/static

# 👤 Switch to non-root user
USER nextjs

# 🌐 Expose port for Ultra Dashboard PWA
EXPOSE 3002

# 🔧 Set port environment variable
ENV PORT 3002
ENV HOSTNAME "0.0.0.0"

# 🏥 Health check for Ultra Dashboard PWA
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3002/ || exit 1

# 🚀 Start the application
CMD ["node", "server.js"]

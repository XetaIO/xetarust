# Image of the Next.js frontend (standalone output). Build context: frontend/
#   docker build --build-arg NEXT_PUBLIC_TURNSTILE_SITE_KEY=... frontend

FROM node:22-alpine AS deps
WORKDIR /app
COPY package.json package-lock.json ./
# `npm install` (like the CI): the lockfile, generated on Windows, lacks some
# Linux-only optional dependencies that `npm ci` would reject.
RUN npm install --no-audit --no-fund

FROM node:22-alpine AS builder
WORKDIR /app
# Inlined by `next build` (and required by next.config.ts): Railway passes the
# service variable of the same name as a build argument.
ARG NEXT_PUBLIC_TURNSTILE_SITE_KEY
ENV NEXT_PUBLIC_TURNSTILE_SITE_KEY=$NEXT_PUBLIC_TURNSTILE_SITE_KEY \
    NEXT_TELEMETRY_DISABLED=1
COPY --from=deps /app/node_modules ./node_modules
COPY . .
RUN npm run build

FROM node:22-alpine AS runner
WORKDIR /app
ENV NODE_ENV=production \
    NEXT_TELEMETRY_DISABLED=1 \
    HOSTNAME=0.0.0.0 \
    PORT=3000
COPY --from=builder --chown=node:node /app/public ./public
COPY --from=builder --chown=node:node /app/.next/standalone ./
COPY --from=builder --chown=node:node /app/.next/static ./.next/static
USER node
EXPOSE 3000
# Listens on $PORT (injected by Railway).
CMD ["node", "server.js"]

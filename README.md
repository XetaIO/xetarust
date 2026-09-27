# Xetaravel

Personal website of Emeric Fevre — portfolio, blog and administration.

- **Backend**: Rust, Axum, SeaORM, PostgreSQL, JWT — domain-first architecture (`backend/`): three
  bounded contexts (`identity`, `publishing`, `discussion`), each hexagonal inside, sharing a small
  `kernel` and wired together by the `app` composition root.
- **Frontend**: Next.js 16, React 19, Tailwind CSS v4, shadcn/ui, Motion (`frontend/`), organized by
  feature (`src/features/{identity,publishing,discussion}`).

## Getting started

```bash
docker compose up -d                  # PostgreSQL (dev :5442, test :5443)
cp .env.example .env                  # set a strong JWT_SECRET
cargo run -p xetaravel-app            # API on http://127.0.0.1:8080 (runs migrations)

cd frontend
cp .env.example .env.local
npm install
npm run dev                           # http://localhost:3000
```

Login and registration are protected by a mandatory Cloudflare Turnstile captcha
(`TURNSTILE_SECRET` in `.env`, `NEXT_PUBLIC_TURNSTILE_SITE_KEY` in `frontend/.env.local`: the API and
Next.js refuse to start without them) and a per-IP rate limit (`AUTH_RATE_LIMIT_*`).

> **Deployment**: the API trusts the `X-Forwarded-For` header sent by Next.js to identify clients.
> Keep it reachable by the Next.js server only (`APP_ADDR=127.0.0.1:8080`, the default), and make the
> reverse proxy in front of Next.js overwrite `X-Real-IP` / `X-Forwarded-For` with the real client
> address — otherwise clients could spoof their IP and bypass the rate limit.

## Deployment (Railway)

One Railway project, three services, deployed from `main` (enable _Wait for CI_):

| Service    | Source                                                               | Networking                            |
| ---------- | -------------------------------------------------------------------- | ------------------------------------- |
| `Postgres` | PostgreSQL template                                                  | private only                          |
| `backend`  | root directory `/`, config file `/backend/railway.toml`              | private only (**no public domain**)   |
| `frontend` | root directory `/frontend`, config file `/frontend/railway.toml`     | public domain                         |

The `backend` service gets a **volume mounted on `/data`**: cover images are stored in
`/data/uploads/covers` and survive redeployments (a service with a volume runs a single replica).
Migrations run at startup; the backend is only reachable by Next.js through the private network, and
the Railway edge sets `X-Real-IP` with the real client address.

`backend` variables:

```bash
DATABASE_URL=${{Postgres.DATABASE_URL}}
JWT_SECRET=...                     # openssl rand -base64 48
TURNSTILE_SECRET=...               # real Cloudflare Turnstile secret
APP_ADDR=[::]:8080                 # the private network is IPv6
UPLOADS_DIR=/data/uploads
CORS_ORIGIN=https://${{frontend.RAILWAY_PUBLIC_DOMAIN}}
RUST_LOG=xetaravel=info,xetaravel_app=info,xetaravel_kernel=info,tower_http=info
```

`frontend` variables (`NEXT_PUBLIC_TURNSTILE_SITE_KEY` is read at build time):

```bash
API_URL=http://${{backend.RAILWAY_PRIVATE_DOMAIN}}:8080
NEXT_PUBLIC_TURNSTILE_SITE_KEY=... # real site key; add the public domain to the Turnstile widget
```

Local check of the images:

```bash
docker build -f backend/Dockerfile -t xetaravel-backend .
docker build --build-arg NEXT_PUBLIC_TURNSTILE_SITE_KEY=1x00000000000000000000AA -t xetaravel-frontend frontend
```

## Quality

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace                # includes architecture guards; regenerates frontend/src/types/api
cd frontend && npm run lint && npm run typecheck && npm run build && npm run test:e2e
```

Architecture and development rules: [`.claude/skills/xetaravel-architecture/SKILL.md`](.claude/skills/xetaravel-architecture/SKILL.md).

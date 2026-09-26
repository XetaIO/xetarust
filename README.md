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

Create an account on `/register`, then make it an admin:

```bash
cargo run -p xetaravel-app -- make-admin you@example.com
```

## Quality

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace                # includes architecture guards; regenerates frontend/src/types/api
cd frontend && npm run lint && npm run typecheck && npm run build && npm run test:e2e
```

Architecture and development rules: [`.claude/skills/xetaravel-architecture/SKILL.md`](.claude/skills/xetaravel-architecture/SKILL.md).

<p align="center">
    <img src="art/logo-brand-light-mode.png#gh-light-mode-only" alt="Xetaravel Logo" height="230"/>
    <img src="art/logo-brand-dark-mode.png#gh-dark-mode-only" alt="Xetaravel Logo" height="230"/>
</p>

<p align="center">

|                                                                                            CI                                                                                             |                                        Rust                                        |                                  Axum                                   |                                           Next.js                                           |                                        React                                        |                                                     PostgreSQL                                                     |                                   License                                   |
| :---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------: | :--------------------------------------------------------------------------------: | :---------------------------------------------------------------------: | :-----------------------------------------------------------------------------------------: | :---------------------------------------------------------------------------------: | :----------------------------------------------------------------------------------------------------------------: | :-------------------------------------------------------------------------: |
| [![CI](https://img.shields.io/github/actions/workflow/status/XetaIO/xetarust/ci.yml?branch=main&style=flat-square&label=CI)](https://github.com/XetaIO/xetarust/actions/workflows/ci.yml) | ![Rust](https://img.shields.io/badge/Rust-2024-dea584?style=flat-square&logo=rust) | ![Axum](https://img.shields.io/badge/Axum-0.8-8a5cf5?style=flat-square) | ![Next.js](https://img.shields.io/badge/Next.js-16-000000?style=flat-square&logo=nextdotjs) | ![React](https://img.shields.io/badge/React-19-61dafb?style=flat-square&logo=react) | ![PostgreSQL](https://img.shields.io/badge/PostgreSQL-18-336791?style=flat-square&logo=postgresql&logoColor=white) | ![License](https://img.shields.io/badge/License-MIT-blue?style=flat-square) |

</p>

## Introduction

Xetaravel is my personal website — portfolio, blog and administration. It started life as a [Laravel
application](https://github.com/XetaIO/Xetaravel) since 2015 and has been rewritten from scratch as a **Rust API** (Axum, SeaORM, PostgreSQL) with a
**Next.js frontend**. The source is released to showcase a domain-first Rust architecture; keep in mind
that some parts are built around my own needs.

## Demo

See it live at [xetaravel.com](https://xetaravel.com).

## Features

```
├── Home
│   ├── Animated presentation (profile, skills, career, projects)
│   └── CV download, protected by a captcha for bots
├── Blog
│   ├── Markdown articles with syntax highlighting (Shiki)
│   ├── Categories and cover images
│   └── Comments
│       ├── Members only
│       ├── Can be disabled per article
│       └── Anti-flood: double-post and cooldown limits (admins exempt)
├── Accounts
│   ├── Register / login
│   ├── JWT kept in an httpOnly cookie (the browser never calls the API)
│   ├── Roles: member / admin
│   ├── Ban system
│   └── Cloudflare Turnstile captcha + per-IP rate limit
└── Dashboard (admin)
    ├── Articles & categories CRUD
    ├── Users: role change, ban with optional comment purge
    └── Settings: open / close registrations
```

## Tech stack

| Layer    | Stack                                                                                                                                 |
| -------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| Backend  | Rust 2024, Tokio, Axum 0.8, SeaORM 2 + migrations, PostgreSQL, `validator`, `serde`, `uuid` v7, Argon2id, JWT HS256, `ts-rs`          |
| Frontend | Next.js 16 (App Router, Turbopack), React 19, TypeScript strict, Tailwind CSS v4, shadcn/ui (Base UI), Motion, react-markdown + Shiki |
| Tests    | `cargo test` (unit + mockall + PostgreSQL integration + HTTP + architecture), Playwright (e2e)                                        |

## Architecture

The backend is split into **four bounded contexts**, one crate each, every one of them hexagonal inside
(`domain` ← `application` ← `infrastructure` / `http`, the domain uses no framework):

```
backend/
├── kernel/       minimal shared kernel (errors, pagination, clock, principal…)
├── identity/     users, authentication, authorization
├── publishing/   articles, categories
├── discussion/   comments
├── resume/       CV (no database)
├── migration/    aggregates the contexts' migrations
└── app/          composition root: config, router, cross-context adapters (src/integration/)

frontend/src/features/
├── identity/
├── publishing/
└── discussion/
```

- Contexts depend **only** on `kernel`, never on each other; only `app` knows them all. This is enforced
  by [`backend/app/tests/architecture.rs`](backend/app/tests/architecture.rs).
- A frontend feature never imports another feature (ESLint rule).
- Full details: [`.claude/skills/xetaravel-architecture/SKILL.md`](.claude/skills/xetaravel-architecture/SKILL.md).

## Getting started

### Requirements

|                                                   Rust                                                    |                                                         Node.js                                                          |                                                       Docker                                                        |
| :-------------------------------------------------------------------------------------------------------: | :----------------------------------------------------------------------------------------------------------------------: | :-----------------------------------------------------------------------------------------------------------------: |
| ![Rust](<https://img.shields.io/badge/Rust-stable%20(2024%20edition)-dea584?style=flat-square&logo=rust>) | ![Node.js](https://img.shields.io/badge/Node.js-LTS%20%2B%20npm-339933?style=flat-square&logo=nodedotjs&logoColor=white) | ![Docker](https://img.shields.io/badge/Docker-PostgreSQL%2017-2496ed?style=flat-square&logo=docker&logoColor=white) |

### Installation

1. **Start PostgreSQL** (dev on `:5442`, test on `:5443`):

   ```bash
   docker compose up -d
   ```

2. **Run the API** on `http://127.0.0.1:8080` — migrations are applied at startup:

   ```bash
   cp .env.example .env              # set a strong JWT_SECRET: openssl rand -base64 48
   cargo run -p xetaravel-app
   ```

   To run the migrations alone: `cargo run -p migration -- up`.

3. **Run the frontend** on `http://localhost:3000`:

   ```bash
   cd frontend
   cp .env.example .env.local
   npm install
   npm run dev
   ```

> [!NOTE]
> Login and registration are protected by a mandatory Cloudflare Turnstile captcha: the API
> (`TURNSTILE_SECRET`) and Next.js (`NEXT_PUBLIC_TURNSTILE_SITE_KEY`) refuse to start without it. Both
> `.env.example` files ship Cloudflare's always-pass test keys; use the `2x…` test secret to simulate a
> failure.

### Become admin

Register an account, then promote it:

```bash
docker exec xetaravel_postgres psql -U xetaravel -d xetaravel \
  -c "UPDATE users SET role = 'admin' WHERE email = 'you@example.com';"
```

### Configuration

Every variable is documented in [`.env.example`](.env.example) and
[`frontend/.env.example`](frontend/.env.example). The most notable ones:

| Variable                                                   | Purpose                                               |
| ---------------------------------------------------------- | ----------------------------------------------------- |
| `JWT_TTL_SECONDS`                                          | JWT lifetime (default 7 days)                         |
| `AUTH_RATE_LIMIT_BURST` / `AUTH_RATE_LIMIT_PERIOD_SECONDS` | Per-IP rate limit of login / register                 |
| `COMMENT_DOUBLE_POST_HOURS` / `COMMENT_COOLDOWN_MINUTES`   | Comment anti-flood                                    |
| `UPLOADS_DIR`                                              | Storage of the uploaded cover images                  |
| `API_URL` (frontend)                                       | URL of the Rust API, used server-side only by Next.js |

## Development & tests

### Backend

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

`cargo test` needs the `postgres_test` container, includes the architecture guards and regenerates the
TypeScript types in `frontend/src/types/api` (ts-rs) — never edit them by hand.

### Coverage

```bash
rustup component add llvm-tools-preview && cargo install cargo-llvm-cov --locked   # once
cargo cov                                          # per-file summary in the terminal
cargo cov --html                                   # target/llvm-cov/html/index.html
cargo cov --lcov --output-path target/lcov.info    # for an editor extension (Coverage Gutters…)
```

The `cov` alias ([`.cargo/config.toml`](.cargo/config.toml)) excludes migrations, binaries, test code and
SeaORM entities. The CI publishes the report as the `backend-coverage` artifact.

### Frontend

```bash
cd frontend
npm run lint
npm run typecheck
npm run build
```

### End-to-end

```bash
cd frontend
npx playwright install chromium   # once
npm run test:e2e                  # the API must be running; the dev server is started automatically
```

### Before opening a PR

- [ ] `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `npm run lint` and `npm run typecheck`
- [ ] `npm run build` (and `npm run test:e2e`) if the UI changed

The [CI](.github/workflows/ci.yml) runs the same checks in four jobs: backend fmt & clippy, backend
tests (with coverage), frontend lint/typecheck/build, and Playwright e2e.

## Contribute

- **TDD**: write the test first, at every layer.
- Every function has a documentation comment, **in English**.
- No imports between bounded contexts. A cross-context need is a **port** in the consumer + a
  **contract** in the provider + an **adapter** in [`backend/app/src/integration/`](backend/app/src/integration/).

See [`.claude/skills/xetaravel-architecture/SKILL.md`](.claude/skills/xetaravel-architecture/SKILL.md)
for the complete conventions.

## License

Xetaravel is open-source software licensed under the [MIT license](https://opensource.org/licenses/MIT).

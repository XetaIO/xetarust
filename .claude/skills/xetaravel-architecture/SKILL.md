---
name: xetaravel-architecture
description: Architecture, conventions et workflow de développement du projet Xetaravel (site perso d'Emeric Fevre) — backend Rust domain-first (3 bounded contexts hexagonaux Identity / Publishing / Discussion, Axum, SeaORM, PostgreSQL, JWT) + frontend Next.js 16 organisé par feature. À charger AVANT toute modification du code de ce dépôt (backend/ ou frontend/), pour ajouter une fonctionnalité, un endpoint, une migration, une page, ou écrire des tests.
---

# Xetaravel — architecture & manière de développer

## 1. Le projet

Site personnel d'Emeric Fevre (Xety) :

- **`/`** — page de présentation très animée (profil, compétences, parcours, projets). Contenu dans `frontend/src/content/profile.ts`.
- **`/blog`** — articles en Markdown, catégories, commentaires. Lecture publique, commentaire réservé aux membres connectés.
- **Utilisateurs** — inscription / connexion. Deux rôles : `member` (par défaut) et `admin`.
- **`/dashboard`** (admin uniquement) — CRUD articles et catégories, changement de rôle et bannissement des utilisateurs (avec purge optionnelle de leurs commentaires), réglages (ouverture des inscriptions).

| Couche                  | Stack                                                                                                                                                    |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Backend (API JSON only) | Rust 2024, Tokio, Axum 0.8, SeaORM 2 + migrations, PostgreSQL uniquement, `validator`, `serde`, `uuid` v7, Argon2id, JWT HS256 (`jsonwebtoken`), `ts-rs` |
| Frontend                | Next.js 16 (App Router, Turbopack), React 19, TypeScript strict, Tailwind CSS v4, shadcn/ui (Base UI), Motion, react-markdown + Shiki                    |
| Tests                   | `cargo test` (unitaires + mockall + intégration Postgres + HTTP + architecture), Playwright (e2e)                                                        |

## 2. Architecture domain-first (backend)

Le backend est découpé en **trois bounded contexts**, un crate chacun ; l'hexagone
(domain → application → infrastructure / http) vit **à l'intérieur** de chaque contexte.

| Contexte   | Crate                                         | Responsabilité                           | Concepts possédés                                                                    |
| ---------- | --------------------------------------------- | ---------------------------------------- | ------------------------------------------------------------------------------------ |
| Identity   | `backend/identity` (`xetaravel-identity`)     | Identité, authentification, autorisation | `User`, `UserId`, `Email`, `Username`, `PasswordHash`, `Role`, `Ban`, `IdentitySettings`  |
| Publishing | `backend/publishing` (`xetaravel-publishing`) | Création et publication du contenu       | `Article`, `ArticleDraft`, `Category`, `ArticleId`, `CategoryId`, `AuthorId`, `Slug` |
| Discussion | `backend/discussion` (`xetaravel-discussion`) | Interactions autour du contenu           | `Comment`, `CommentId`, `CommentableArticle`, `ArticleId` et `AuthorId` **locaux**   |

```
backend/
├── kernel/      xetaravel-kernel      noyau partagé minimal (erreurs, pagination, texte, Clock, Principal, http, persistence)
├── identity/    xetaravel-identity
├── publishing/  xetaravel-publishing
├── discussion/  xetaravel-discussion
├── migration/   migration             agrège les migrations des contextes (identity → publishing → discussion)
└── app/         xetaravel-app         composition root + ACL + router (binaire `xetaravel`)
```

Structure identique dans chaque contexte :

```
<contexte>/src/
├── lib.rs            API publique : Module, router, migrations(), contrat pour les autres contextes
├── domain/           entités, value objects, ids (define_id!), ports repository, read models — AUCUN framework
├── application/      use_cases/, dto.rs (ts-rs), ports.rs (dépendances sortantes), contract.rs (contrat public), test_support.rs
├── infrastructure/   persistence/ (entités SeaORM, mappers, repositories), migrations/, security/ (Identity)
├── http/             handlers + `pub fn router<S>() -> Router<S>`
└── module.rs         `XxxModule::new(db, …)` : construit adapters et use cases du contexte
```

**Règles de dépendance (vérifiées par `backend/app/tests/architecture.rs`)** :

- `identity`, `publishing`, `discussion` dépendent **seulement** de `kernel` — jamais les uns des autres, ni de `app`/`migration` (hors dev-dependencies).
- `kernel` ne dépend d'aucun contexte.
- Dans un contexte : `domain/**` n'importe ni `sea_orm`, `axum`, `serde`, `ts_rs`, ni `crate::application|infrastructure|http` ; `application/**` n'importe ni `sea_orm` ni `axum`.
- Seul `app` dépend de tous les contextes.

### Intégration entre contextes : ports + ACL in-process

Un contexte **déclare ce dont il a besoin** (port sortant dans `application/ports.rs`) et **expose un petit contrat** (`application/contract.rs`). `app/src/integration/` branche l'un sur l'autre en traduisant les identifiants :

| Port (consommateur)                                                           | Contrat (fournisseur)                                             | Adapter ACL (`app`)            |
| ----------------------------------------------------------------------------- | ----------------------------------------------------------------- | ------------------------------ |
| `publishing::AuthorDirectory`, `discussion::AuthorDirectory` (noms d'auteurs) | `identity::IdentityDirectory` (use case `GetPublicProfiles`)      | `IdentityAuthorDirectory`      |
| `discussion::ArticleCatalog` (article publié ?)                               | `publishing::PublishedArticles` (use case `FindPublishedArticle`) | `PublishingArticleCatalog`     |
| `kernel::PrincipalResolver` (token → `Principal`)                             | use case `identity::Authenticate`                                 | `IdentityModule::principals()` |

- Les autres contextes ne voient **jamais** `User` : ils manipulent un `Principal { user_id, is_admin }` (kernel) et leurs propres ids (`AuthorId`, `ArticleId`).
- Les vues composites (article + catégorie + nom d'auteur) sont construites **dans l'application** (`publishing/src/application/views.rs`) : le repository joint uniquement ce que le contexte possède (catégories) ; les auteurs viennent du port, en **un seul appel par lot** (pas de N+1). Auteur inconnu → nom `"unknown"`.
- `AppState` (`app/src/state.rs`) contient `identity`, `publishing`, `discussion`, `principals` avec `#[derive(FromRef)]` ; chaque router de contexte est générique sur l'état (`Arc<XxxModule>: FromRef<S>`).

### Base de données

- Une seule base PostgreSQL ; **chaque contexte possède ses tables et ses migrations** (`<contexte>/src/infrastructure/migrations/`, `pub fn migrations()`), nommées `mYYYYMMDD_<contexte>_NNNNNN_<action>`.
- `backend/migration` concatène `identity::migrations()` + `publishing::migrations()` + `discussion::migrations()` (dans cet ordre).
- **Compromis assumé** : les FKs inter-contextes sont conservées (`articles.author_id → users`, `comments.article_id → articles ON DELETE CASCADE`, `comments.author_id → users`) pour garantir l'intégrité. Dans les migrations, une table d'un autre contexte est référencée par son nom (`Alias::new("users")`), jamais par import.

### « Je veux ajouter… → où ? »

| Besoin                                     | Emplacement                                                                                                                                    |
| ------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| Une règle métier / un invariant            | méthode d'entité ou value object dans `backend/<contexte>/src/domain/`                                                                         |
| Une nouvelle requête de persistance        | méthode du trait dans `domain/*_repository.rs` **puis** implémentation dans `infrastructure/persistence/*_repository.rs`                       |
| Un cas d'utilisation                       | `backend/<contexte>/src/application/use_cases/<verbe_nom>.rs` + champ dans `module.rs`                                                         |
| Un format d'entrée/sortie de l'API         | `application/dto.rs` du contexte (`#[ts(export, export_to = "<contexte>/")]`) ; DTO partagé → `kernel/src/dto.rs` (`shared/`)                  |
| Une dépendance technique (mail, stockage…) | trait dans `application/ports.rs` du contexte + adapter dans son `infrastructure/`, câblé dans `module.rs`                                     |
| Un besoin envers un autre contexte         | port dans `application/ports.rs` du consommateur + contrat dans `application/contract.rs` du fournisseur + adapter dans `app/src/integration/` |
| Un endpoint                                | handler dans `<contexte>/src/http/` + route dans son `router()`                                                                                |
| Une colonne / table                        | nouvelle migration dans `<contexte>/src/infrastructure/migrations/` (ajoutée **à la fin** de `migrations()`) + entité SeaORM + mappers         |
| Un nouveau contexte                        | nouveau crate sur le même modèle + membre du workspace + `migration` + `app` (state, router, ACL) + `CONTEXTS` de `architecture.rs`            |
| Une page                                   | `frontend/src/app/**/page.tsx` qui compose les `features/*` (voir `references/frontend.md`)                                                    |

Détails : [`references/backend.md`](references/backend.md).

## 3. Workflow TDD (obligatoire)

Toujours **red → green → refactor**. Écrire le test qui échoue avant le code.

Ordre pour une nouvelle fonctionnalité (dans le contexte concerné) :

1. **Domain** — tests unitaires de l'entité / du value object (`#[cfg(test)] mod tests` dans le même fichier).
2. **Port** — ajouter la méthode au trait repository (mock `Mock*Repository` généré par `mockall` sous `cfg(test)`).
3. **Use case** — tests avec mocks (`crate::domain::Mock*`, `crate::application::ports::Mock*`, fixtures de `application/test_support.rs`), puis implémentation.
4. **Adapter** — implémentation SeaORM + test d'intégration dans `backend/<contexte>/tests/persistence.rs` (vraie base Postgres de test, `migration::testing`).
5. **ACL** (si inter-contextes) — test unitaire de l'adapter dans `app/src/integration/` avec des fakes.
6. **Route** — handler + test HTTP de bout en bout dans `backend/app/tests/api.rs`.
7. **Bindings** — `cargo test` régénère `frontend/src/types/api/<contexte>/*.ts`.
8. **Frontend** — requête (`features/<contexte>/queries.ts`) ou Server Action (`features/<contexte>/actions.ts`), puis UI ; e2e Playwright si c'est un parcours clé.

Stratégie et exemples : [`references/testing.md`](references/testing.md).

## 4. SOLID & clean code — règles concrètes

- **SRP** : un use case = une struct = une responsabilité. Handlers Axum fins : extraire → appeler **un** use case → sérialiser.
- **OCP / DIP** : les use cases dépendent de traits (`Arc<dyn UserRepository>`, `Arc<dyn AuthorDirectory>`, `Arc<dyn Clock>`…), jamais d'implémentations. Le câblage se fait dans `module.rs` (intra-contexte) et `app/src/state.rs` (inter-contextes).
- **ISP** : ports petits et ciblés (un trait par agrégat, un par service technique, un par besoin inter-contexte).
- **LSP** : tout adapter doit respecter le contrat du trait (mêmes erreurs, mêmes cas `None`/`false`) — vérifié par les tests d'intégration.
- Pas de `unwrap()`/`expect()` hors tests et bootstrap. Erreurs typées (`DomainError` → `AppError` → `ApiError`, toutes dans le kernel).
- Pas de primitive obsession : utiliser les value objects (`Email`, `Slug`, `ArticleId`…) dès la frontière application ; convertir un `Uuid` venant d'un autre contexte en id local (`AuthorId::from(principal.user_id)`).
- Le temps passe par le port `Clock` du kernel (`FixedClock` dans les tests).
- Fonctions courtes, noms explicites, pas de code mort, `cargo clippy -- -D warnings` propre, `cargo fmt`.
- Autorisation en profondeur : l'extracteur `AdminPrincipal` filtre, **et** chaque use case admin appelle `principal.require_admin()?`. Le rôle est relu en base à chaque requête (`Authenticate`) : un changement de rôle est immédiat.

## 5. Commentaires : en anglais, sur CHAQUE fonction

Toute fonction, méthode, handler, composant React, hook ou Server Action porte un commentaire de documentation **en anglais** (`///` en Rust, `/** */` en TypeScript), y compris les fonctions privées et les fonctions de test utilitaires. Les modules ont un `//!` qui décrit leur rôle.

```rust
/// Changes the role of `self` on behalf of `actor`.
///
/// Rules: only an admin can change roles, and an admin can never demote
/// themselves (it guarantees at least one admin always remains).
pub fn change_role(&mut self, role: Role, actor_id: UserId, actor_role: Role, now: DateTime<Utc>) -> DomainResult<()> {
```

```tsx
/** Fades and slides its children in the first time they enter the viewport. */
export function Reveal({ children, delay = 0 }: RevealProps) {
```

Les messages de commit, identifiants et logs sont aussi en anglais.

## 6. Conventions API & authentification

- Préfixe `/api`, JSON uniquement, identifiants UUID v7, dates ISO 8601 UTC.
- Erreurs : `{ "error": ErrorCode, "message": string, "fields"?: { champ: [messages] } }` avec
  `validation_error` 422 · `unauthorized` 401 · `forbidden` 403 · `not_found` 404 · `conflict` 409 · `too_many_requests` 429 · `internal_error` 500.
- Les brouillons et slugs invalides sont renvoyés en **404** côté public (on ne révèle rien).
- Auth : `POST /api/auth/login|register` → `{ token, expires_at, user }`. Le JWT s'envoie en `Authorization: Bearer`.
- **Côté Next.js, le JWT vit uniquement dans le cookie httpOnly `xetaravel_token`** (`secure` en prod, `sameSite=lax`), posé par les Server Actions (`features/identity/actions.ts`). Le navigateur ne voit jamais le token ; tous les appels à l'API passent par le serveur Next (`src/lib/api/client.ts`, `server-only`).

Liste des routes et formats : [`references/conventions.md`](references/conventions.md).

## 7. Frontend organisé par feature

`frontend/src/features/{identity,publishing,discussion}/` reflète les contextes du backend (queries, actions, composants, types `types/api/<contexte>/`). **Une feature n'importe jamais une autre feature** (ni ses types) — règle ESLint `no-restricted-imports` ; les routes `src/app/**` les composent (ex. la page article passe un `CommentViewer` dérivé de l'utilisateur Identity à `CommentSection`). Détails : [`references/frontend.md`](references/frontend.md).

## 8. Commandes

```bash
docker compose up -d                          # PostgreSQL dev (5442) + test (5443)
cp .env.example .env                          # puis changer JWT_SECRET
cargo run -p migration -- up                  # migrations (aussi appliquées au démarrage de l'app)
cargo run -p xetaravel-app                    # API sur 127.0.0.1:8080 (binaire `xetaravel`)

cargo test --workspace                        # tous les tests (dont architecture) + régénération des types TS
cargo cov                                     # couverture (cargo-llvm-cov) : résumé par fichier
cargo cov --html                              # rapport HTML dans target/llvm-cov/html/
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all

cd frontend
cp .env.example .env.local                    # API_URL
npm run dev                                   # http://localhost:3000
npm run lint && npm run typecheck && npm run build
npm run test:e2e                              # Playwright (API démarrée requise)
```

Avant de considérer une tâche terminée : `cargo fmt`, `clippy -D warnings`, `cargo test --workspace`, `npm run lint`, `npm run typecheck` et, si l'UI change, `npm run build` doivent passer.

## Références

- [`references/backend.md`](references/backend.md) — patterns Rust détaillés (contexte, use case, repository, port inter-contexte, migration, endpoint).
- [`references/frontend.md`](references/frontend.md) — organisation Next.js 16 par feature, data fetching, Server Actions, UI/animations.
- [`references/testing.md`](references/testing.md) — pyramide de tests, mocks, base de test, garde-fous d'architecture, Playwright.
- [`references/conventions.md`](references/conventions.md) — routes de l'API par contexte, DTOs, nommage, git.

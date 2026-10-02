# Tests

## Pyramide

| Niveau | Où | Outils | Base de données |
|---|---|---|---|
| Unitaires domaine | `#[cfg(test)] mod tests` dans chaque fichier de `<contexte>/src/domain/` (et du `kernel`) | `cargo test` | non |
| Use cases | `#[cfg(test)]` dans chaque use case | `mockall` (`Mock*Repository`, `MockPasswordHasher`, `MockTokenService`, `MockHumanVerifier` (fixture `human(valid)`), `MockAuthorDirectory`, `MockArticleCatalog`), `FixedClock`, fixtures `application/test_support.rs` | non |
| Adapters | unitaires (Argon2, JWT, Turnstile contre un faux serveur Axum, Config, extracteurs) + `backend/<contexte>/tests/persistence.rs` | `cargo test`, `migration::testing` | Postgres de test |
| ACL inter-contextes | `#[cfg(test)]` dans `backend/app/src/integration/*` | fakes des contrats (`IdentityDirectory`, `PublishedArticles`) | non |
| HTTP bout en bout | `backend/app/tests/api.rs` + `registration.rs`, harnais partagé dans `tests/common/mod.rs` (vrai `Router`, les 3 contextes câblés, `tower::ServiceExt::oneshot` ; faux `siteverify` Axum local (jeton `valid`), chaque `call`/`call_raw`/`call_json_raw` envoie un `X-Forwarded-For` unique pour ne jamais partager un seau de rate limit, `call_from` pour une IP donnée ; `TestApp::start()` règle les deux limites sur `GENEROUS`, `start_with(auth, global)` pour tester un 429) | `cargo test` | Postgres de test |
| Architecture | `backend/app/tests/architecture.rs` (`cargo metadata` + scan des sources) | `cargo test` | non |
| E2E navigateur | `frontend/e2e/*.spec.ts` | Playwright | Postgres dev (API lancée) |

## Base de test

- `docker compose up -d` démarre `postgres_test` sur le port **5443** (tmpfs, jetable).
- `migration::testing::test_database()` (feature `test-support`, en dev-dependency des contextes et de `app`) lit `DATABASE_URL_TEST` depuis `.env`, se connecte et applique les migrations une fois par binaire de test.
- Les tests de persistance d'un contexte ne dépendent pas des autres contextes : les lignes étrangères exigées par les FKs sont insérées en SQL brut via `seed_user`, `seed_published_article`, `delete_article`.
- Les tests tournent en parallèle sur la même base : **chaque test crée ses propres données uniques** (`unique()`) et ne dépend jamais d'un comptage global.
- Exception : un test qui modifie un **état global** (ex. fermer les inscriptions) casserait les tests parallèles. Il vit dans son propre binaire (`backend/app/tests/registration.rs`, un seul test) : `cargo test` exécute les binaires de test l'un après l'autre. Le harnais (`TestApp`, faux `siteverify`, `call`, `register`, `register_admin`…) est partagé via `tests/common/mod.rs` (`mod common;`). Un tel test **rétablit toujours l'état par défaut** (inscriptions ouvertes), comme `settings_repository_round_trip` dans `identity/tests/persistence.rs`.
- Après un changement d'historique de migrations : `DATABASE_URL=<url de test> cargo run -p migration -- fresh`.

## Écrire un test de use case

```rust
#[tokio::test]
async fn rejects_duplicated_slug() {
    let mut categories = MockCategoryRepository::new();
    categories.expect_slug_exists().returning(|_, _| Ok(true));

    let error = CreateCategory::new(Arc::new(categories), clock())
        .execute(admin_principal(), request())
        .await
        .unwrap_err();

    assert_eq!(error, AppError::field("slug", "is already taken"));
}
```

- Un test = un comportement, nommé comme une phrase (`admin_cannot_demote_themselves`).
- Couvrir : cas nominal, validation, autorisation (`member_principal()` → `Forbidden`), ressource absente (`NotFound`), conflit, et pour les ports inter-contextes le cas « inconnu » (auteur absent → `"unknown"`, article non publié → `NotFound`).
- Utiliser `.times(n)` pour vérifier qu'une écriture a lieu (ou n'a pas lieu : `.times(0)`).
- Aucune horloge réelle : `test_support::clock()` / `now()` (`FixedClock` du kernel).

## Garde-fous d'architecture

`backend/app/tests/architecture.rs` échoue si :
- un contexte dépend (hors dev-dependencies) d'un autre contexte, de `xetaravel-app` ou de `migration`, ou n'utilise pas le kernel ;
- le kernel dépend d'un contexte ;
- une ligne de code de `*/src/domain/**` mentionne `sea_orm`, `axum`, `serde`, `ts_rs`, `crate::application`, `crate::infrastructure` ou `crate::http` ;
- une ligne de code de `*/src/application/**` mentionne `sea_orm`, `axum`, `crate::infrastructure` ou `crate::http`.

Un nouveau contexte doit être ajouté à `CONTEXTS` dans ce fichier.

## Couverture

- Outil : [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov) sur la toolchain stable. Prérequis, une fois : `rustup component add llvm-tools-preview` puis `cargo install cargo-llvm-cov --locked` ; `postgres_test` doit tourner (`docker compose up -d`).
- Alias de `.cargo/config.toml` (partagés local / CI), les arguments supplémentaires s'y ajoutent :
  - `cargo cov` — lance tous les tests instrumentés et affiche un résumé par fichier ;
  - `cargo cov --html` — rapport navigable dans `target/llvm-cov/html/index.html` ;
  - `cargo cov --lcov --output-path target/lcov.info` — pour une extension d'éditeur (Coverage Gutters…) ;
  - `cargo cov --no-report` puis `cargo cov-report …` — produire plusieurs rapports sans relancer les tests.
- Exclusions (`--ignore-filename-regex`, séparateurs `[\\/]` pour Windows et Linux) :

| Code exclu | Pourquoi |
|---|---|
| crate `backend/migration/` | agrégateur de schéma, CLI et outillage de test |
| `*/infrastructure/migrations/` | DDL déclaratif, exécuté une fois |
| `main.rs` | bootstrap / CLI, non testable unitairement |
| `*/tests/` | code de test (`api.rs`, `registration.rs`, `common/`, `architecture.rs`, `persistence.rs`) |
| `application/test_support.rs` | fixtures de test |
| `persistence/entity.rs`, `persistence/entities/` | entités SeaORM, code généré par les derives |

- Limite : sur stable, les modules `#[cfg(test)] mod tests` inline ne peuvent pas être exclus ; ils restent comptés et la couverture est donc légèrement surévaluée. Ne pas modifier les sources pour contourner cette limite.
- CI (job `backend-test`) : les tests ne tournent qu'une fois via `cargo cov --no-report` ; le résumé s'affiche dans le récapitulatif du run et l'artefact `backend-coverage` (HTML + `lcov.info`, 14 jours) est publié. Aucun seuil : le job n'échoue que si un test échoue.
- Pour un nouveau type de fichier non pertinent, compléter la regex **dans les deux alias** `cov` et `cov-report`.

## E2E

- Prérequis : API démarrée (`cargo run -p xetaravel-app`) ; Playwright lance `npm run dev` si besoin. Clés de test Turnstile dans `.env` (`TURNSTILE_SECRET=1x0000000000000000000000000000000AA`) et `frontend/.env.local` (`NEXT_PUBLIC_TURNSTILE_SITE_KEY=1x00000000000000000000AA`) : le widget de test se valide seul, les vraies clés bloquent un navigateur headless.
- Promotion admin dans les tests directement en base, comme en production : `UPDATE users SET role = 'admin'` via `docker exec xetaravel_postgres psql` (e2e) ou la connexion de test (`register_admin` dans `tests/common/mod.rs`).
- Les e2e vérifient aussi que le cookie de session est `httpOnly` et absent de `document.cookie`.
- Toutes les requêtes e2e viennent de la même IP (`::1`, via Next) : l'API doit tourner avec des rate limits hors d'atteinte (`GLOBAL_RATE_LIMIT_BURST=100000`, `AUTH_RATE_LIMIT_BURST=1000`), dans le `.env` local ou en variables d'environnement (la CI les pose dans le job `e2e`).

# Tests

## Pyramide

| Niveau | Où | Outils | Base de données |
|---|---|---|---|
| Unitaires domaine | `#[cfg(test)] mod tests` dans chaque fichier de `<contexte>/src/domain/` (et du `kernel`) | `cargo test` | non |
| Use cases | `#[cfg(test)]` dans chaque use case | `mockall` (`Mock*Repository`, `MockPasswordHasher`, `MockTokenService`, `MockAuthorDirectory`, `MockArticleCatalog`), `FixedClock`, fixtures `application/test_support.rs` | non |
| Adapters | unitaires (Argon2, JWT, Config, extracteurs) + `backend/<contexte>/tests/persistence.rs` | `cargo test`, `migration::testing` | Postgres de test |
| ACL inter-contextes | `#[cfg(test)]` dans `backend/app/src/integration/*` | fakes des contrats (`IdentityDirectory`, `PublishedArticles`) | non |
| HTTP bout en bout | `backend/app/tests/api.rs` (vrai `Router`, les 3 contextes câblés, `tower::ServiceExt::oneshot`) | `cargo test` | Postgres de test |
| Architecture | `backend/app/tests/architecture.rs` (`cargo metadata` + scan des sources) | `cargo test` | non |
| E2E navigateur | `frontend/e2e/*.spec.ts` | Playwright | Postgres dev (API lancée) |

## Base de test

- `docker compose up -d` démarre `postgres_test` sur le port **5443** (tmpfs, jetable).
- `migration::testing::test_database()` (feature `test-support`, en dev-dependency des contextes et de `app`) lit `DATABASE_URL_TEST` depuis `.env`, se connecte et applique les migrations une fois par binaire de test.
- Les tests de persistance d'un contexte ne dépendent pas des autres contextes : les lignes étrangères exigées par les FKs sont insérées en SQL brut via `seed_user`, `seed_published_article`, `delete_article`.
- Les tests tournent en parallèle sur la même base : **chaque test crée ses propres données uniques** (`unique()`) et ne dépend jamais d'un comptage global.
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

## E2E

- Prérequis : API démarrée (`cargo run -p xetaravel-app`) ; Playwright lance `npm run dev` si besoin.
- Promotion admin dans les tests via le binaire : `target/debug/xetaravel make-admin <email>`.
- Les e2e vérifient aussi que le cookie de session est `httpOnly` et absent de `document.cookie`.

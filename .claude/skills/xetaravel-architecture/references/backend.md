# Backend — patterns de référence

Tout le code métier vit dans un **bounded context** (`backend/identity`, `backend/publishing`, `backend/discussion`), chacun structuré en `domain/` → `application/` → `infrastructure/` + `http/`, assemblé par `module.rs`. Le `kernel` ne contient que ce que tous partagent ; `app` compose.

## Kernel (`backend/kernel`)

| Module | Contenu |
|---|---|
| `error` | `DomainError`, `DomainResult`, `AppError` (dont `TooManyRequests` → 429), `AppResult`, `FieldErrors`, conversions `DomainError`/`validator` → `AppError` |
| `pagination` | `PageRequest` (bornée 1..=50), `Page<T>` |
| `text` | `validate_text`, `validate_optional_text`, `non_blank` |
| `clock` | port `Clock`, `SystemClock` (précision µs), `FixedClock` (tests) |
| `principal` | `Principal { user_id: Uuid, is_admin }` + `require_admin()`, port `PrincipalResolver` |
| `dto` | `Paginated<T>`, `PageQuery`, `AuthorDto` (ts-rs → `types/api/shared/`) |
| `define_id!` | macro des identifiants UUID v7 typés |
| feature `http` | `ApiError`/`ApiResult`/`ErrorBody`/`ErrorCode`, extracteurs `JsonBody`, `PathParam`, `QueryParams`, `ClientIp` (première IP de `X-Forwarded-For`, sinon `ConnectInfo`, sinon `None`), `CurrentPrincipal`, `AdminPrincipal` |
| feature `persistence` | `connect`, `db_error` (SeaORM → `DomainError`), `corrupted` |

Le kernel reste **petit** : n'y ajouter qu'un concept réellement partagé par plusieurs contextes, jamais un concept métier d'un seul contexte.

## Domain (`<contexte>/src/domain/`)

### Value object

Constructeur `parse` qui valide et normalise ; posséder la valeur prouve qu'elle est valide.

```rust
/// A syntactically valid, lower-cased email address.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

impl Email {
    /// Parses and normalizes (trim + lowercase) an email address.
    pub fn parse(raw: &str) -> DomainResult<Self> { /* ... */ }
}
```

Les identifiants sont déclarés dans `domain/ids.rs` avec `xetaravel_kernel::define_id!` : impossible de passer un `CategoryId` à la place d'un `ArticleId`. Un contexte qui référence un concept d'un autre contexte déclare **son propre** id (`publishing::AuthorId`, `discussion::ArticleId`, `discussion::AuthorId`) : il ne connaît que l'identifiant, jamais l'agrégat.

### Entité

- Champs publics typés par des value objects (valide par construction).
- Constructeurs nommés selon le métier (`User::register`, `Article::write`, `Comment::post`, `Category::create`).
- Les mutations portant une règle passent par des méthodes (`User::change_role`, `Article::revise`, `Article::publish`, `Comment::can_be_deleted_by(author_id, is_admin)`).
- Le temps est **toujours** passé en paramètre (`now: DateTime<Utc>`), jamais `Utc::now()` dans le domaine.

### Port repository

```rust
/// Persistence port of the [`Comment`] aggregate.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait CommentRepository: Send + Sync {
    /// Finds a comment by id.
    async fn find_by_id(&self, id: CommentId) -> DomainResult<Option<Comment>>;
    // ...
}
```

- Retourne `Option` pour « absent », `bool` pour « supprimé ou non ».
- Un repository ne lit **que les tables de son contexte**. Les lectures composites intra-contexte renvoient un read model (`CategorizedArticle { article, category }`).
- Réexporter le mock dans `domain/mod.rs` : `#[cfg(test)] pub use comment_repository::MockCommentRepository;`.

## Application (`<contexte>/src/application/`)

### Use case type

```rust
/// Deletes a comment. Allowed for its author and for admins.
pub struct DeleteComment {
    comments: Arc<dyn CommentRepository>,
}

impl DeleteComment {
    /// Builds the use case with its dependencies.
    pub fn new(comments: Arc<dyn CommentRepository>) -> Self {
        Self { comments }
    }

    /// Checks the permissions of `principal` and deletes the comment.
    pub async fn execute(&self, principal: Principal, comment_id: Uuid) -> AppResult<()> {
        // 1. (admin) principal.require_admin()?;
        // 2. input.validate()?;                 — validator sur le DTO
        // 3. conversion en value objects / ids locaux (AuthorId::from(principal.user_id))
        // 4. chargement / règles métier / persistance
        // 5. conversion en DTO de sortie
    }
}
```

- Entrée : DTO de requête (+ `Principal` si authentifié). Sortie : DTO de réponse. Jamais d'entité domaine exposée.
- Validation à deux niveaux : `validator` sur le DTO (formats, longueurs, plusieurs champs à la fois) puis invariants du domaine.
- Conflits d'unicité attendus (slug, email…) : `AppError::field("slug", "is already taken")` pour un affichage par champ.
- Nouveau use case → l'exporter dans `use_cases/mod.rs` et l'ajouter comme champ de `XxxModule` (`module.rs`).

### DTO

```rust
/// Body used to create or update a category.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate, TS)]
#[ts(export, export_to = "publishing/")]
pub struct UpsertCategoryRequest {
    #[validate(length(min = 2, max = 100, message = "must contain between 2 and 100 characters"))]
    pub name: String,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub slug: Option<String>,
}
```

- Toujours `#[ts(export, export_to = "<contexte>/")]` : `cargo test` écrit le `.ts` dans `frontend/src/types/api/<contexte>/` (configuré dans `.cargo/config.toml`, `u64` exportés en `number`). Les DTOs partagés (`Paginated`, `PageQuery`, `AuthorDto`, `ErrorBody`) sont dans le kernel (`shared/`).
- Conversions `impl From<&Entity/View> for XxxDto` dans `dto.rs`.

### Port vers un autre contexte

Le consommateur exprime son besoin **dans son langage** ; le fournisseur expose un contrat minimal ; `app` traduit.

```rust
// discussion/src/application/ports.rs — besoin de Discussion
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ArticleCatalog: Send + Sync {
    /// Returns the id of the published article using `slug`, if any.
    async fn published_article_id(&self, slug: &str) -> AppResult<Option<ArticleId>>;
}

// publishing/src/application/contract.rs — contrat public de Publishing
#[async_trait]
pub trait PublishedArticles: Send + Sync {
    /// Returns the published article using `slug`, or `None`.
    async fn find_published(&self, slug: &str) -> AppResult<Option<PublishedArticleRef>>;
}

// app/src/integration/articles.rs — ACL
#[async_trait]
impl xetaravel_discussion::ArticleCatalog for PublishingArticleCatalog {
    /// Translates the published article reference into a Discussion id.
    async fn published_article_id(&self, slug: &str) -> AppResult<Option<ArticleId>> {
        Ok(self.articles.find_published(slug).await?.map(|a| ArticleId::from(a.id)))
    }
}
```

- Le contrat est implémenté par un **use case** du fournisseur (`FindPublishedArticle`, `GetPublicProfiles`, `Authenticate`) exposé par le module (`published_articles()`, `directory()`, `principals()`).
- Appels **par lot** (`names(&[AuthorId])`) pour éviter le N+1 ; les vues sont assemblées dans l'application (`publishing/src/application/views.rs`).
- Pas d'événements : l'intégration est synchrone et in-process.

## Infrastructure (`<contexte>/src/infrastructure/`)

- Entités SeaORM au format « compact » dans `persistence/` ; **aucune relation SeaORM** : les jointures intra-contexte sont faites par lots dans les repositories (`load_categories`).
- `persistence/mappers.rs` : `to_xxx(Model) -> DomainResult<Entity>` (re-valide les données, une ligne corrompue devient `DomainError::Repository` via `kernel::persistence::corrupted`) et `from_xxx(&Entity) -> ActiveModel` (tous les champs `Set`).
- Toutes les erreurs SeaORM passent par `xetaravel_kernel::persistence::db_error()` : violation d'unicité / FK → `Conflict`, `RecordNotUpdated` → `NotFound`, le reste → `Repository`.
- Travail CPU bloquant (Argon2) → `tokio::task::spawn_blocking`.
- `security/` d'Identity : `Argon2PasswordHasher`, `JwtTokenService` (`JwtSettings`), et le port `HumanVerifier` implémenté par `TurnstileHumanVerifier` (`siteverify`, timeout 5 s, erreur réseau → `AppError::Internal`) (seule implémentation : le captcha est obligatoire) ; `IdentityModule::new` le construit depuis `CaptchaSettings { turnstile_secret, siteverify_url }` (`TURNSTILE_SITEVERIFY_URL` en production, faux serveur dans les tests).

## Migrations

- Un fichier par changement : `<contexte>/src/infrastructure/migrations/mYYYYMMDD_<contexte>_NNNNNN_<action>.rs`, ajouté **à la fin** de `migrations()` du contexte.
- `backend/migration` concatène les contextes dans l'ordre des dépendances (identity → publishing → discussion) ; un nouveau contexte s'y ajoute.
- Référencer la table d'un autre contexte par son nom (`Alias::new("users")`), jamais en important son `Iden`.
- Ne jamais modifier une migration déjà appliquée en production : en créer une nouvelle.
- Enums PostgreSQL et index fonctionnels via `execute_unprepared` (ex. `user_role`, `LOWER(email)`).
- `down()` doit annuler exactement `up()`.

## HTTP (Axum) — `<contexte>/src/http/`

```rust
/// Routes of the Discussion context.
pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Arc<DiscussionModule>: FromRef<S>,
    Arc<dyn PrincipalResolver>: FromRef<S>,
{
    Router::new().route("/api/comments/{id}", delete(delete_comment))
}

/// `DELETE /api/comments/{id}` — author or admin only.
async fn delete_comment(
    State(discussion): State<Arc<DiscussionModule>>,
    CurrentPrincipal(principal): CurrentPrincipal,
    PathParam(id): PathParam<Uuid>,
) -> ApiResult<StatusCode> {
    discussion.delete_comment.execute(principal, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
```

- Utiliser les extracteurs du kernel `JsonBody`, `PathParam`, `QueryParams` (erreurs JSON homogènes), `CurrentPrincipal` / `AdminPrincipal` pour l'auth.
- Codes : `201` + corps pour une création, `204` pour une suppression, `200` sinon.
- Routes Axum 0.8 : paramètres en `{param}` ; deux routes au même niveau doivent utiliser le même nom de paramètre (y compris entre contextes : `/api/articles/{slug}` et `/api/articles/{slug}/comments`).
- `ApiError` ne divulgue jamais le détail d'une erreur interne (logué via `tracing`, cible `xetaravel_kernel`).

## Composition root (`backend/app`)

- `config.rs` : `Config::from_env()` (exige `TURNSTILE_SECRET` non vide ; inclut `JwtSettings` et `CaptchaSettings` d'Identity, `RateLimitSettings` des routes d'auth).
- `state.rs` : `AppState::build(db, &config)` construit `IdentityModule`, puis `PublishingModule` (avec `IdentityAuthorDirectory`), puis `DiscussionModule` (avec `PublishingArticleCatalog` + `IdentityAuthorDirectory`).
- `router.rs` : `/api/health` + `merge` des routers des contextes ; `identity::auth_router()` (login, register) y reçoit le `GovernorLayer` (rate limit par IP, erreurs au format `ApiError`), `identity::account_router()` le reste.
- `main.rs` : binaire `xetaravel` (serveur HTTP uniquement), applique les migrations au démarrage ; sert avec `into_make_service_with_connect_info::<SocketAddr>()`.

# Conventions

## Routes de l'API

| Méthode | Route | Accès | Contexte | Use case |
|---|---|---|---|---|
| GET | `/api/health` | public | app | — |
| POST | `/api/auth/register` | public, captcha, rate limit IP | Identity | `RegisterUser` → 201 `AuthResponse` (429 `too_many_requests`) |
| POST | `/api/auth/login` | public, captcha, rate limit IP | Identity | `LoginUser` → `AuthResponse` (429 `too_many_requests`) |
| GET | `/api/auth/me` | membre | Identity | `GetCurrentUser` → `UserDto` |
| GET | `/api/admin/users?page&per_page` | admin | Identity | `ListUsers` |
| PATCH | `/api/admin/users/{id}/role` | admin | Identity | `ChangeUserRole` (auto-rétrogradation interdite) |
| GET | `/api/articles?page&per_page&category` | public | Publishing | `ListPublishedArticles` → `Paginated<ArticleSummaryDto>` |
| GET | `/api/articles/{slug}` | public | Publishing | `GetPublishedArticle` → `ArticleDto` |
| GET | `/api/categories` | public | Publishing | `ListCategories` → `CategoryDto[]` |
| GET | `/api/covers/{name}` | public | Publishing | `GetCover` → octets de l'image, `Cache-Control: immutable` (404 si nom invalide/absent) |
| GET / POST | `/api/admin/articles` | admin | Publishing | `ListArticles` / `CreateArticle` |
| GET / PUT / DELETE | `/api/admin/articles/{id}` | admin | Publishing | `GetArticle` / `UpdateArticle` / `DeleteArticle` |
| PUT / DELETE | `/api/admin/articles/{id}/cover` | admin | Publishing | `UploadCover` (corps = octets bruts JPEG/PNG/WebP, 5 Mo max) / `RemoveCover` → `ArticleDto` |
| POST | `/api/admin/categories` | admin | Publishing | `CreateCategory` |
| PUT / DELETE | `/api/admin/categories/{id}` | admin | Publishing | `UpdateCategory` / `DeleteCategory` (409 si non vide) |
| GET | `/api/articles/{slug}/comments` | public | Discussion | `ListComments` → `CommentDto[]` |
| POST | `/api/articles/{slug}/comments` | membre | Discussion | `PostComment` → 201 `CommentDto` |
| DELETE | `/api/comments/{id}` | auteur ou admin | Discussion | `DeleteComment` → 204 |

Pagination : `page` commence à 1, `per_page` ∈ [1, 50] (10 par défaut).

## Règles métier clés

- Nouveau compte = `member`. Email et username uniques (insensible à la casse).
- Mot de passe 8–128 caractères, haché en Argon2id.
- Login et register exigent `captcha_token` (réponse du widget Cloudflare Turnstile), vérifié **avant** tout accès base ou calcul Argon2 (port `HumanVerifier`) ; échec → 422 sur le champ `captcha_token`. Captcha obligatoire (l'API refuse de démarrer sans `TURNSTILE_SECRET`, Next.js sans `NEXT_PUBLIC_TURNSTILE_SITE_KEY`) ; clés de test Cloudflare en dev/e2e.
- Login et register sont limités par IP (`tower_governor`, seau commun aux deux routes) : `AUTH_RATE_LIMIT_BURST` tentatives (5), puis une toutes les `AUTH_RATE_LIMIT_PERIOD_SECONDS` (12 s) → 429 `too_many_requests`. L'IP vient de `X-Forwarded-For` posé par Next.js : l'API ne doit être joignable que par Next (`APP_ADDR=127.0.0.1:8080`).
- Slug d'article/catégorie dérivé du titre/nom si vide, modifiable, unique ; conservé lors d'une édition sans slug.
- Un article republié garde sa première date de publication.
- Une catégorie contenant des articles ne peut pas être supprimée.
- Seuls les articles publiés peuvent être commentés (Discussion interroge Publishing).
- Supprimer un article supprime ses commentaires (cascade SQL `comments.article_id → articles`).
- Image de couverture : format détecté par magic bytes (JPEG/PNG/WebP), 5 Mo max, stockée sur disque (`UPLOADS_DIR/covers`, port `CoverStorage`) sous un nom `<uuid v7>.<ext>` régénéré à chaque upload ; l'ancien fichier est supprimé au remplacement, au retrait et à la suppression de l'article.

## Nommage

- Crates : `xetaravel-<contexte>` ; types de module `<Contexte>Module` ; contrats publics nommés d'après le fournisseur (`IdentityDirectory`, `PublishedArticles`) ; ports nommés d'après le besoin du consommateur (`AuthorDirectory`, `ArticleCatalog`) ; adapters ACL `<Fournisseur><Port>` (`IdentityAuthorDirectory`).
- Rust : use cases au format `VerbeNom` (`CreateArticle`), fichiers en `snake_case` du même nom ; DTOs suffixés `Dto` / `Request` / `Query`.
- Migrations : `mYYYYMMDD_<contexte>_NNNNNN_<action>`.
- Tables SQL au pluriel `snake_case`, colonnes `snake_case`, FK `<table>_id`, index `idx_<table>_<col>`.
- TypeScript : composants en `PascalCase` dans des fichiers `kebab-case.tsx`, Server Actions en verbes (`saveArticle`), un dossier `features/<contexte>/` par contexte.

## Git

- Messages de commit en anglais, à l'impératif (`Add category deletion guard`).
- Ne jamais committer `.env`, `.env.local`, `target/`, `node_modules/`.
- Les fichiers `frontend/src/types/api/**/*.ts` générés **sont** commités (le frontend doit compiler sans Rust).

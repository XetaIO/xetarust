# Frontend — Next.js 16

> Next.js 16 diffère des versions précédentes : lire `frontend/node_modules/next/dist/docs/` avant d'utiliser une API inconnue. Points clés : `middleware.ts` est renommé **`proxy.ts`** ; `params`, `searchParams` et `cookies()` sont **asynchrones** ; types globaux `PageProps<"/route">` / `LayoutProps<"/route">` (générés par `next typegen`).

## Organisation par feature

Les features reflètent les bounded contexts du backend.

```
frontend/src/
├── app/                        routes (App Router) — composent les features, ne bougent pas
│   ├── page.tsx                accueil statique animé
│   ├── (auth)/login|register   formulaires d'auth
│   ├── blog/                   liste, catégories, article + commentaires (dynamique)
│   └── dashboard/              admin (layout = requireAdmin()) ; users/actions.ts = banMember (ban Identity + purge Discussion) ;
│                               settings/ = page de composition des réglages (une carte par contexte, aujourd'hui Identity)
├── features/
│   ├── identity/               session.ts (storeSession, clearSession, getCurrentUser, requireUser, requireAdmin),
│   │                           redirect.ts, actions.ts (login, register, logout, changeUserRole, banUser, unbanUser, updateIdentitySettings),
│   │                           queries.ts (getUsers, getIdentitySettings mémoïsée par requête, ouvert si l'API échoue),
│   │                           components/{auth-form (widget Turnstile, prop canRegister), role-toggle, ban-dialog (reçoit l'action de ban en prop), settings-form}
│   ├── publishing/             queries.ts (articles, catégories, admin), actions.ts (saveArticle, deleteArticle, saveCategory, deleteCategory),
│   │                           cover.ts (coverUrl), components/{article-card, article-cover, article-list, category-nav, markdown, article-form, category-form}
│   ├── discussion/             queries.ts (getComments), actions.ts (postComment, deleteComment, deleteAuthorComments),
│   │                           components/{comment-form, comment-section}
│   └── resume/                 actions.ts (downloadResume : jeton Turnstile → octets du PDF ou FormState),
│                               components/cv-download (bouton « Download my CV » de l'accueil, widget Turnstile, saveBlob)
├── lib/
│   ├── api/client.ts           apiFetch() server-only, ajoute le Bearer depuis le cookie ; apiFetchBytes() (réponse binaire, ex. PDF) ; apiUpload() (PUT octets bruts) ; apiProxy() (relaie une réponse binaire publique GET)
│   ├── api/errors.ts           ApiError (miroir de ErrorBody), isApiError, orNull
│   ├── api/session-cookie.ts   nom du cookie httpOnly (partagé client / proxy / identity)
│   ├── client-ip.ts            clientIp() : IP du visiteur relayée en X-Forwarded-For (Identity, Resume)
│   ├── forms.ts                FormState + helpers de lecture de FormData
│   └── format.ts               dates, pagination
├── components/                 transverses : ui/ (shadcn), forms/, site/ (header, footer, pagination), home/, dashboard/ (nav, page-header)
├── content/profile.ts          contenu de la page d'accueil
├── types/api/{shared,identity,publishing,discussion,resume}/   GÉNÉRÉ par ts-rs — ne jamais éditer à la main
└── proxy.ts                    garde optimiste de /dashboard (présence du cookie)
```

## Règles

- **Frontières** : une feature n'importe jamais une autre feature ni les types `types/api/<autre contexte>/` (ESLint `no-restricted-imports`, `eslint.config.mjs`). Elle peut importer `lib/`, `components/` et `types/api/shared/`. Les routes `app/**` composent : ex. `app/blog/[slug]/page.tsx` lit l'utilisateur via Identity et passe un `CommentViewer { id, isAdmin }` à `CommentSection` (Discussion).
- Une action qui combine plusieurs contextes vit **dans la route** (`app/<route>/actions.ts`, `"use server"`) et appelle les actions des features : ex. `banMember` (`app/dashboard/users/actions.ts`) = `banUser` puis, si demandé, `deleteAuthorComments`. Le composant de la feature reçoit l'action composée en prop (`BanDialog action={banMember.bind(null, id)}`).
- **Lecture** : Server Components qui appellent `features/<contexte>/queries.ts`. **Écriture** : Server Actions dans `features/<contexte>/actions.ts`, jamais de `fetch` vers l'API depuis le navigateur. Toute écriture navigateur passe par une Server Action (protection `Origin`/`Host` native) ; les route handlers sont réservés aux GET publics (médias). Une Server Action peut renvoyer des octets (`Uint8Array`, ex. `downloadResume`).
- Tout module qui touche au cookie ou à l'API importe `"server-only"`.
- Server Action type : lire le `FormData` → construire le DTO typé (`@/types/api/<contexte>/...`) → `apiFetch` dans un `try` → `toFormState(error)` en cas d'erreur 4xx → `revalidatePath` → `redirect` **hors du try**.
- Formulaires : `useActionState` + `FieldError` / `FormMessage` pour afficher `fields` renvoyés par l'API. Les messages de validation viennent du backend (source de vérité).
- Suppressions : composant `ConfirmAction` (AlertDialog + toast) avec une Server Action liée (`action.bind(null, id)`).
- L'autorisation réelle est faite par l'API Rust ; `proxy.ts` et `requireAdmin()` ne sont que du confort UX (404 pour les membres sur `/dashboard`).
- **Médias** : les images servies par l'API (couvertures d'articles) passent par le route handler `app/media/covers/[name]/route.ts` (`apiProxy`) ; utiliser `coverUrl(name)` (`features/publishing/cover.ts`) avec `next/image`, qui optimise (tailles, WebP/AVIF). Les chemins optimisables sont listés dans `images.localPatterns` (`next.config.ts`). L'upload passe par la Server Action `saveArticle` (`bodySizeLimit: "6mb"`).
- La page d'accueil doit rester **statique** (pas de `cookies()` ni d'appel API) ; le header avec session n'est utilisé que dans blog/auth/dashboard.

## UI & animations

- Thème sombre forcé (`<html class="dark">`), tokens dans `app/globals.css` (`--brand-orange`, `--brand-amber`, `--brand-red`), utilitaires `.glass`, `.bg-grid`, animations `animate-aurora`, `animate-marquee`.
- Animations avec `motion/react` dans des composants client : `Reveal` pour les apparitions au scroll, `useScroll`/`useSpring` pour les effets liés au scroll, `useMotionValue` pour les effets curseur. `prefers-reduced-motion` est respecté globalement.
- shadcn/ui version Base UI : pour rendre un lien comme un bouton, utiliser `buttonVariants()` sur `<Link>` ; pour personnaliser un trigger, la prop `render` (pas `asChild`).
- Markdown : `features/publishing/components/markdown.tsx` (`MarkdownAsync` côté serveur + Shiki), HTML brut ignoré.

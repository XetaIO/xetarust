# Xetaravel

Site perso d'Emeric Fevre : API Rust domain-first (`backend/`) + frontend Next.js 16 organisé par feature (`frontend/`).

**Avant toute modification, charger le skill `xetaravel-architecture`** (`.claude/skills/xetaravel-architecture/SKILL.md`) : architecture, workflow TDD, conventions et commandes.

Rappels essentiels :
- Trois bounded contexts, un crate chacun : `identity`, `publishing`, `discussion`. Ils dépendent **uniquement** de `kernel`, jamais les uns des autres ; seul `app` (composition root + ACL) les connaît tous. Vérifié par `backend/app/tests/architecture.rs`.
- Dans chaque contexte : `domain` ← `application` ← `infrastructure` / `http` ; le domaine n'utilise aucun framework.
- Besoin inter-contextes = port dans le consommateur + contrat dans le fournisseur + adapter dans `backend/app/src/integration/`.
- TDD : test d'abord, à chaque couche.
- Chaque fonction a un commentaire de documentation **en anglais**.
- `frontend/src/types/api/` est généré par `cargo test` (ts-rs) — ne pas éditer.
- Frontend : une feature (`src/features/*`) n'importe jamais une autre feature (règle ESLint).
- Le JWT reste dans le cookie httpOnly côté Next.js ; le navigateur n'appelle jamais l'API Rust directement.

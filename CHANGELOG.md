# Changelog — La Ruche

Toutes les modifications notables sont documentées ici.
Format : [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/)
Versioning : [Semantic Versioning](https://semver.org/lang/fr/)

---

## [Unreleased]

### Added
- Squelette backend Rust complet (Axum 0.8 + SQLx + Argon2id + WebAuthn)
- Schéma PostgreSQL — 26 tables, UUID, triggers `updated_at`
- Repositories SQLx : copropriete, lot, utilisateur, budget, contrat, tache, document, signalement, consultation, sinistre, carnet
- Handlers API : auth (login/logout/refresh/reset), documents (S3 pre-signed), budget (génération appels de fonds), contrats, tâches, prestataire
- Domain : calcul quorums AG (art. 24/25/26), répartition charges par tantièmes, rappels contrats
- Auth intégrée : JWT httpOnly, Argon2id, tokens invitation/reset/prestataire
- Flux upload S3 sécurisé (pre-signed URL, jamais via le backend)
- Job de fond rappels contrats (poll horaire)
- Design system "The Modern Registry" — Fraunces + DM Sans + JetBrains Mono
- Composants Vue 3 : 7 atomiques, 6 moléculaires, 2 organiques
- Maquettes interactives HTML : Budget, AG detail, Dashboard (light + dark)
- Vues Vue 3 : LoginView, DashboardView
- CI/CD GitHub Actions (tests unitaires + intégration + typecheck + docker build)
- Docker Compose dev + prod (backend + postgres + minio)
- Makefile, CONTRIBUTING.md, NOTICE AGPL-3.0

### Architecture
- ADR-001 : Rust + Axum + SQLx + REST + OpenAPI (utoipa)
- ADR-002 : JWT httpOnly + Argon2id + webauthn-rs
- ADR-003 : Single crate modulaire
- DDR-001 v3 : Fraunces + DM Sans + JetBrains Mono
- DDR-002 : Palette lavande/charcoal, light+dark
- DDR-003 : Bottom nav mobile, sidebar desktop

### Licence
- AGPL-3.0 (Commons Clause abandonnée — non OSI)

---

## Format des versions futures

```
## [0.2.0] - YYYY-MM-DD
### Added     — nouvelles fonctionnalités
### Changed   — modifications de fonctionnalités existantes
### Deprecated— fonctionnalités qui seront retirées
### Removed   — fonctionnalités retirées
### Fixed     — corrections de bugs
### Security  — corrections de failles
```

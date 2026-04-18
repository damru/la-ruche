# ARCHITECTURE.md — La Ruche

> Document vivant. Dernière mise à jour : 2026-04-19

---

## Stack technique

| Couche | Technologie | Référence |
|---|---|---|
| Runtime | Rust (édition 2024) | ADR-001 |
| Framework HTTP | Axum 0.8 | ADR-001 |
| Base de données | PostgreSQL 16 | ADR-001 |
| ORM / Query | SQLx 0.8 (compile-time) | ADR-001 |
| Migrations | SQLx migrate (embedded) | ADR-001 |
| Auth | JWT + Argon2id + webauthn-rs | ADR-002 |
| Stockage | S3-compatible — MinIO self-hosted | schema.sql |
| API | REST JSON + OpenAPI (utoipa) | ADR-001 |
| Edge runtime | Caddy sur Debian slim | ADR-005 |
| Logs | tracing + tracing-subscriber (JSON) | — |
| Config | dotenvy + config crate (LA_RUCHE__*) | — |
| Frontend | Vue 3 + Vite + TypeScript | DDR-003 |
| État UI | Pinia | — |
| État serveur | TanStack Query | — |
| HTTP client | Axios + intercepteurs refresh | — |
| i18n | vue-i18n (fr uniquement MVP) | — |
| Accessibilité | Radix Vue (Dialog, Dropdown) | — |

---

## Structure du projet

```
laruche/
├── backend/
│   ├── migrations/
│   │   └── 0001_init.sql       ← 26 tables, UUID, triggers updated_at
│   └── src/
│       ├── api/                ← handlers (zero business logic)
│       │   ├── assemblies.rs
│       │   ├── auth.rs
│       │   ├── budget.rs
│       │   ├── contracts.rs
│       │   ├── contractor.rs
│       │   ├── documents.rs
│       │   ├── mod.rs          ← router + OpenAPI
│       │   ├── stubs.rs        ← stubs for routes not yet implemented
│       │   └── tasks.rs
│       ├── auth/               ← jwt, middleware, passkey, password, token
│       │   ├── jwt.rs
│       │   ├── middleware.rs
│       │   ├── mod.rs
│       │   ├── passkey.rs
│       │   ├── password.rs
│       │   └── token.rs
│       ├── config.rs           ← typed config from env
│       ├── db/                 ← SQLx repositories (12 modules)
│       │   ├── assembly.rs
│       │   ├── budget.rs
│       │   ├── building.rs
│       │   ├── claim.rs
│       │   ├── consultation.rs
│       │   ├── contract.rs
│       │   ├── document.rs
│       │   ├── maintenance_log.rs
│       │   ├── mod.rs
│       │   ├── report.rs
│       │   ├── task.rs
│       │   ├── unit.rs
│       │   └── user.rs
│       ├── domain/             ← pure logic, no I/O
│       │   ├── charges.rs      ← levy distribution by share (Decimal)
│       │   ├── mod.rs          ← Role, AuthUser
│       │   ├── processor.rs    ← OCR/NLP trait (NoopProcessor MVP)
│       │   ├── reminders.rs    ← contract reminder date computation
│       │   └── vote.rs         ← quorums art. 24/25/26 (tested)
│       ├── errors.rs           ← unified AppError → JSON HTTP
│       ├── jobs/
│       │   ├── mod.rs
│       │   └── reminders.rs    ← hourly poll, ready for notifications
│       ├── main.rs             ← assembly, auto-migrations, jobs
│       └── storage/            ← client S3, presigned URLs
│           ├── client.rs
│           ├── mod.rs
│           └── presigned.rs
├── CONTRIBUTING.md             ← guide de contribution
├── docker-compose.prod.yml     ← override production
├── docker-compose.yml          ← laruche + postgres + minio
├── docker/
│   ├── Caddyfile               ← config Caddy (SPA + reverse proxy)
│   └── entrypoint.sh           ← backend + Caddy démarrage
├── Dockerfile                  ← multi-stage unique (Node + Rust + Caddy)
├── docs/
│   ├── adr/ (ADR-001, 002, 003)
│   ├── ARCHITECTURE.md
│   ├── DESIGN.md
│   └── PRD.md
├── frontend/src/
│   ├── api/                    ← client Axios + modules API
│   ├── components/
│   │   ├── atoms/              ← CBtn, CInput, CSelect, CTextarea,
│   │   │                          CProgressBar, CAlert, CBadge
│   │   ├── molecules/          ← CCard, CKpiCard, CListItem,
│   │   │                          CDataTable, CModal, CDropzone
│   │   └── organisms/          ← AppNav
│   ├── composables/            ← useRole, useFormatters (Intl natif)
│   ├── features/               ← vues lazy-loaded par domaine
│   │   ├── assemblies/         ← AssemblyListView, AssemblyDetailView
│   │   ├── auth/               ← LoginView ✅, AcceptInviteView, ResetPasswordView
│   │   ├── budget/             ← BudgetView
│   │   ├── claims/             ← ClaimsView
│   │   ├── consultations/      ← ConsultationsView
│   │   ├── contractor/         ← ContractorPage
│   │   ├── contracts/          ← ContractsView
│   │   ├── dashboard/          ← DashboardView ✅
│   │   ├── documents/          ← DocumentsView
│   │   ├── maintenance/        ← MaintenanceView
│   │   ├── reports/            ← ReportsView
│   │   ├── residents/          ← ResidentsView
│   │   ├── settings/           ← SettingsView
│   │   └── tasks/              ← TasksView
│   ├── layouts/                ← AppLayout
│   ├── router/index.ts         ← lazy routes, guards (403 vs 404)
│   ├── stores/                 ← auth.ts, ui.ts (Pinia)
│   ├── styles/                 ← tokens.css, base.css, typography.css
│   ├── types/domain.ts         ← types TS miroir schéma Rust
│   └── views/                  ← ForbiddenView, NotFoundView
└── Makefile                    ← cibles make (dev, test, docker…)
```

---

## Schéma DB — 26 tables

| Domaine | Tables |
|---|---|
| Copropriété | `copropriete`, `lot`, `cle_repartition`, `lot_cle_repartition` |
| Utilisateurs | `utilisateur`, `user_copropriete`, `token_utilisateur`, `lot_proprietaire` |
| AG | `assemblee_generale`, `resolution`, `vote` |
| Budget | `budget`, `ligne_budget`, `appel_de_fonds`, `fonds_travaux` |
| Documents | `document` |
| Contrats | `contrat_fournisseur`, `contrat_document`, `rappel_contrat` |
| Tâches | `tache`, `depot_prestataire`, `depot_prestataire_document` |
| Sinistres | `sinistre`, `sinistre_document` |
| Carnet | `carnet_entretien`, `carnet_document` |
| Signalements | `signalement`, `signalement_document` |
| Consultations | `consultation`, `choix_consultation`, `reponse_consultation` |

Conventions : UUID v4, TIMESTAMPTZ, soft delete via `actif`/`statut`, `updated_at` via triggers, JSONB pour OCR futur.

---

## Flux principaux

### Requête authentifiée
```
HTTP → Tower (CORS, trace, timeout 30s)
     → auth::middleware (JWT cookie → AuthUser)
     → api::handler (valide + vérifie rôle)
     → domain:: (calcul si nécessaire)
     → db::repository (SQLx)
     → AppError → JSON
```

### Upload document
```
1. Client → POST /api/v1/documents/upload-url
2. Backend → pre-signed PUT S3 (TTL 15min)
3. Client → PUT direct S3 (jamais via le backend)
4. Client → POST /api/v1/documents (confirme key + métadonnées)
```

### Token prestataire
```
1. Syndic génère token → UUID + expire_at en DB
2. Lien /prestataire/{token} → accès sans JWT
3. Dépôt → staging S3 → validation → bucket final copropriété
```

---

## Sécurité

| Aspect | Décision |
|---|---|
| JWT | httpOnly + Secure + SameSite=Strict |
| Refresh | Token opaque hashé SHA-256, jamais en clair |
| Mots de passe | Argon2id, 64MB mémoire, 3 itérations |
| Tokens invitation/reset | UUID hashé SHA-256, TTL strict |
| Fichiers | Bucket privé, pas d'URL publique, pre-signed uniquement |
| Isolation tenant | Toutes les requêtes filtrent sur `copropriete_id` |
| Erreurs | 403 (pas 404) sur routes protégées — pas de fuite d'info |
| Chiffrement | SSE at-rest S3, TLS at-transit |

---

## Principes de code

- **Zéro logique métier dans les handlers**
- **Zéro SQL dans domain/**
- **`pub(crate)` par défaut**
- **Pas de `unwrap()` hors tests**
- **`Decimal` pour tous les montants financiers** (pas `f64`)
- **`SQLX_OFFLINE=true` en CI** — `.sqlx/` committé

---

## ADR

| Ref | Sujet | Décision |
|---|---|---|
| ADR-001 | Stack & API | Rust + Axum + SQLx + REST + utoipa |
| ADR-002 | Auth | JWT + Argon2id + webauthn-rs, intégré |
| ADR-003 | Structure | Single crate modulaire |
| ADR-004 | Onboarding | First-run public wizard (status + bootstrap transactionnel) |
| ADR-005 | Runtime edge | Caddy installé dans l'image Debian slim |

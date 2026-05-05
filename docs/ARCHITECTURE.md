# ARCHITECTURE.md — l'Application

> **Dénomination** — Dans le présent document, l'expression **« l'Application »** désigne exclusivement **La Ruche**, plateforme web open source de gestion participative de copropriété. Ces deux termes sont interchangeables et désignent la même entité logicielle.

> Document vivant. Dernière mise à jour : 2026-05-04

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
| Distribution image conteneur | Docker Hub (`docker.io/damru/la-ruche`) | CI GitHub Actions |
| Versioning tantièmes + snapshot levy | `valid_from`/`valid_to` + colonnes snapshot `levy_call` | ADR-006 |
| Clé générale système | `key_type` enum + soft delete sur `allocation_key` | ADR-007 |
| Traçabilité tâches ↔ carnet | Lien informatif `task_id` + multiples clôtures + `completion_cost` courant sur `task` | ADR-008 |
| Immutabilité comptable + audit log | Guards de domaine + `audit_log` append-only + `valid_from`/`valid_to` sur `budget_line` + `predecessor_id` sur `vendor_contract` | ADR-009 |
| Logs | tracing + tracing-subscriber (JSON) | — |
| Config | dotenvy + config crate (LA_RUCHE__*) ; `LA_RUCHE__APP__DEMO_MODE=true` rejoue `backend/seeds/demo.sql` au démarrage | — |
| Frontend | Vue 3 + Vite + TypeScript | DDR-003 |
| État UI | Pinia | — |
| Préférences UI persistées | `localStorage` `laruche-theme` (`light` / `dark`), absence de clé = `auto` | DDR-002, DDR-004 |
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
│       ├── api/                ← handlers (zéro logique métier)
│       │   ├── assemblies.rs
│       │   ├── auth.rs
│       │   ├── budget.rs
│       │   ├── building.rs
│       │   ├── claims.rs
│       │   ├── consultations.rs
│       │   ├── contracts.rs
│       │   ├── contractor.rs
│       │   ├── contractors.rs
│       │   ├── documents.rs
│       │   ├── logbook.rs
│       │   ├── mod.rs          ← router + OpenAPI
│       │   ├── onboarding.rs
│       │   ├── reports.rs
│       │   ├── stubs.rs        ← stubs for routes not yet implemented
│       │   ├── units.rs
│       │   ├── users.rs
│       │   └── tasks.rs
│       ├── auth/               ← jwt, middleware, passkey, password, token
│       │   ├── jwt.rs
│       │   ├── middleware.rs
│       │   ├── mod.rs
│       │   ├── passkey.rs
│       │   ├── password.rs
│       │   └── token.rs
│       ├── config.rs           ← typed config from env
│       ├── db/                 ← repositories SQLx
│       │   ├── allocation_key.rs
│       │   ├── assembly.rs
│       │   ├── budget.rs
│       │   ├── building.rs
│       │   ├── claim.rs
│       │   ├── consultation.rs
│       │   ├── contract.rs
│       │   ├── contractor.rs
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
├── docker-compose.prod.yml     ← override production, image Docker Hub
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
│   │   ├── features/               ← vues lazy-loaded par domaine
│   │   ├── assemblies/         ← AssemblyListView, AssemblyDetailView
│   │   ├── auth/               ← LoginView ✅, AcceptInviteView, ResetPasswordView
│   │   ├── budget/             ← BudgetView
│   │   ├── claims/             ← ClaimsView
│   │   ├── consultations/      ← ConsultationsView
│   │   ├── contractor/         ← ContractorPage
│   │   ├── contractors/        ← ContractorsView, ContractorDetailView
│   │   ├── contracts/          ← ContractsView
│   │   ├── dashboard/          ← DashboardView ✅
│   │   ├── documents/          ← DocumentsView
│   │   ├── maintenance/        ← MaintenanceView
│   │   ├── onboarding/         ← OnboardingView
│   │   ├── profile/            ← ProfileView
│   │   ├── reports/            ← ReportsView
│   │   ├── residence/          ← BuildingView, AllocationKeysView
│   │   ├── residents/          ← ResidentsView
│   │   └── tasks/              ← TasksView
│   ├── layouts/                ← AppLayout
│   ├── router/index.ts         ← lazy routes, guards (403 vs 404, onboarding, redirects legacy)
│   ├── stores/                 ← auth.ts, ui.ts (Pinia + préférence d'apparence via localStorage)
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

Invariants techniques implémentés :

- `report.task_id` porte le lien informatif entre signalement et tâche ; cette relation reste non destructive et réutilisable par les workflows applicatifs.
- Les documents de tâche sont répartis entre `task_document` (gestionnaires) et `vendor_submission_document` (soumissions prestataire) ; les endpoints manager `GET/POST/DELETE /tasks/{id}/documents` et `document_count` agrègent ces deux sources.
- `claim_document` suit le même modèle d'association documentaire que `report_document`, avec nettoyage des liens lors des suppressions d'entité.
- `maintenance_log.task_id` et `maintenance_log.contract_id` restent des liens de traçabilité souples, sans cascade destructive depuis les tâches ou contrats.

Les règles produit détaillées de navigation, d'apparence et de workflows métier restent normatives dans `PRD.md`, `EPICS.md`, `DESIGN.md` et les ADR/DDR concernés ; ce document n'en conserve qu'un résumé d'architecture.

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

### Pièce justificative d’un contrat
```
1. Client → POST /api/v1/documents/upload-url
2. Client → PUT direct S3
3. Client → POST /api/v1/documents (crée le document)
4. Client → POST /api/v1/contracts/{id}/documents (associe la pièce au contrat)
5. Client → GET /api/v1/contracts/{id}/documents (liste les pièces liées)
```

### Token prestataire
```
1. Syndic génère token → UUID + expire_at en DB
2. Lien `/vendor/{token}` → accès sans JWT
3. Dépôt → staging S3 → validation → bucket final copropriété
4. Soumission prestataire → statut `awaiting_validation` + commentaire / coût optionnel
5. Clôture syndic → insertion d'une nouvelle ligne carnet avec snapshot coût/prestataire
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

## CI/CD

- Le workflow `CI` génère désormais `backend/.sqlx/` directement dans le job **backend**, puis partage cet artefact au job **docker** sans relancer une préparation SQLx séparée.
- Le workflow sépare la **validation backend**, la **validation frontend** et un job **`docker`** unique, déclenché seulement après succès de `test-backend` et `test-frontend`.
- Dans le job backend, les migrations PostgreSQL ne sont exécutées qu'une seule fois avant `cargo sqlx prepare`, puis la validation Cargo enchaîne `cargo fmt --check`, `cargo clippy -- -D warnings` et un unique `cargo test --lib --tests` pour mutualiser le profil de compilation des tests sans relancer un `cargo check` redondant.
- Le job backend active aussi `sccache` comme cache compilateur Rust dédié, persistant entre exécutions GitHub Actions via le backend GHA natif de `mozilla-actions/sccache-action`, afin de réutiliser plus finement les artefacts `rustc` entre `cargo install sqlx-cli`, `cargo clippy` et `cargo test --lib --tests`.
- Le job Docker unique construit et publie l’image dans la même exécution, ce qui supprime le préchauffage dédié désormais inutile grâce au cache Buildx réutilisé par branche.
- Les caches CI sont mutualisés par écosystème : `Swatinem/rust-cache` porte une clé stable `backend-ci` entre les exécutions du workflow pour réutiliser le registre Cargo, les artefacts `target/`, les crates de test et `sqlx-cli`, `sccache` conserve séparément les objets compilés `rustc` dans le cache GitHub Actions natif, le cache npm passe par `actions/setup-node`, et le cache GHA Buildx reste isolé par branche pour les builds Docker.

---

## Principes de code

- **Zéro logique métier dans les handlers**
- **Zéro SQL dans domain/**
- **`pub(crate)` par défaut**
- **Pas de `unwrap()` hors tests**
- **`Decimal` pour tous les montants financiers** (pas `f64`)
- **`SQLX_OFFLINE=true` en CI et en build Docker** — `backend/.sqlx/` est généré une seule fois dans `test-backend`, puis téléchargé par les jobs Docker

---

## ADR

| Ref | Sujet | Décision |
|---|---|---|
| ADR-001 | Stack & API | Rust + Axum + SQLx + REST + utoipa |
| ADR-002 | Auth | JWT + Argon2id + webauthn-rs, intégré |
| ADR-003 | Structure | Single crate modulaire |
| ADR-004 | Onboarding | First-run public wizard (status + bootstrap transactionnel) |
| ADR-005 | Runtime edge | Caddy installé dans l'image Debian slim |
| ADR-006 | Versioning tantièmes & snapshot levy | `valid_from`/`valid_to` sur `unit_allocation_key` ; colonnes `shares_snapshot`, `shares_total_snapshot`, `key_id` sur `levy_call` |
| ADR-007 | Clé générale système | `key_type allocation_key_type` enum + `deleted_at` soft delete sur `allocation_key` ; clé Charges Générales créée à l'onboarding, indestructible |
| ADR-008 | Traçabilité tâches / carnet / soumissions prestataire | `maintenance_log.task_id` et `vendor_submission.task_id` deviennent informatifs (`ON DELETE SET NULL`) ; suppression de l'unicité 1 tâche → 1 entrée ; `task.completion_cost` porte le montant courant jusqu'à la clôture validée |
| ADR-009 | Immutabilité des données comptables + audit log | Guards `is_mutable`/`is_deletable` dans `domain/immutability.rs` ; table `audit_log` append-only (diff JSONB `old_values`/`new_values`) ; statut `draft` sur `levy_call` ; `predecessor_id` sur `vendor_contract` (pattern avenant) ; `parent_budget_id` + `valid_from`/`valid_to` sur `budget_line` ; trigger PG défensif sur `levy_call` |

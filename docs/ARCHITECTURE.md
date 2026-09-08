---
document: ARCHITECTURE
version: 2.1.0
owner: Damien
status: validé
last_updated: 2026-05-22
description: >
  Source de vérité de l'architecture technique de La Ruche — stack, structure du projet,
  schéma DB, flux principaux, sécurité, CI/CD et principes de code.
  S'applique à l'intégralité du backend (Rust/Axum/SQLx) et aux points de contact frontend/API.
rules:
  - Ce fichier fait foi pour toute décision d'architecture backend.
  - Toute nouvelle table, handler, module domain/ ou store frontend doit être reflété ici.
  - Les noms de tables, colonnes et types Rust utilisent exclusivement les identifiants anglais du code.
  - Les entités marquées "prévu, non encore implémenté" ne doivent pas être codées sans ADR ou Epic associé.
  - Les invariants listés dans "Schéma DB" ont valeur de contrat : toute modification nécessite une mise à jour synchrone de ce document.
references:
  - ADR-001
  - ADR-002
  - ADR-003
  - ADR-004
  - ADR-005
  - ADR-006
  - ADR-007
  - ADR-008
  - ADR-009
  - ADR-010
  - ADR-011

---

# ARCHITECTURE.md — l'Application

> **Dénomination** — Dans le présent document, l'expression **« l'Application »** désigne exclusivement **La Ruche**, plateforme web open source de gestion participative de copropriété. Ces deux termes sont interchangeables et désignent la même entité logicielle.

> Document vivant. Dernière mise à jour : 2026-05-22

---

## Stack technique

| Couche | Technologie | Référence |
|---|---|---|
| Runtime | Rust (édition 2024) | ADR-001 |
| Framework HTTP | Axum 0.8 | ADR-001 |
| Base de données | PostgreSQL 16 | ADR-001 |
| ORM / Query | SQLx 0.8 (compile-time) | ADR-001 |
| Migrations | SQLx migrate (embedded) | ADR-001 |
| Auth | JWT + Argon2id + webauthn-rs (passkeys **stubs** — `auth/passkey.rs` compilable, 4 handlers retournent `tracing::warn!` + 501 ; pas de roadmap active) | ADR-002 |
| Stockage | S3-compatible — MinIO self-hosted | schema.sql |
| API | REST JSON + OpenAPI (utoipa) | ADR-001 |
| Edge runtime | Caddy sur Debian slim | ADR-005 |
| Distribution image conteneur | Docker Hub (`docker.io/damru/la-ruche`) | CI GitHub Actions |
| Versioning tantièmes + snapshot levy | `valid_from`/`valid_to` + colonnes snapshot `levy_call` | ADR-006 |
| Clé générale système | `key_type` enum + soft delete sur `allocation_key` | ADR-007 |
| Traçabilité tâches ↔ carnet | Lien informatif `task_id` + multiples clôtures + `completion_cost` courant sur `task` | ADR-008 |
| Immutabilité comptable + audit log | Guards de domaine + `audit_log` append-only réservé à la preuve comptable/juridique + `valid_from`/`valid_to` sur `budget_line` + `predecessor_id` sur `vendor_contract` | ADR-009 |
| Timeline métier tâches/signalements | `activity_log` polymorphique distinct de `audit_log` pour la frise UX — **prévu, non encore implémenté** | EP-017 / EP-018 |
| Logs | tracing + tracing-subscriber (JSON) | — |
| Config | dotenvy + config crate (LA_RUCHE__*) ; `LA_RUCHE__APP__DEMO_MODE=true` rejoue `backend/seeds/demo.sql` au démarrage | — |
| Frontend | Vue 3 + Vite + TypeScript + Tailwind CSS v4 (couche utilitaire) | DDR-003 |
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
│   │   └── 0001_init.sql       ← 36 tables, 25 ENUMs, UUID, triggers updated_at
│   ├── seeds/
│   │   └── demo.sql            ← données de démonstration (rejoué au démarrage si DEMO_MODE)
│   └── src/
│       ├── api/                ← handlers (zéro logique métier)
│       │   ├── admin.rs        ← POST /admin/reset-demo (manager only, DEMO_MODE)
│       │   ├── assemblies.rs   ← CRUD AG, résolutions, votes, présence, documents, convocation, clôture
│       │   ├── auth.rs         ← login/logout, refresh JWT, reset password, invitation, profil, passkey (stubs)
│       │   ├── budget.rs       ← CRUD budgets et lignes, appels de charges, marquage paiement, fonds travaux
│       │   ├── building.rs     ← lecture/update immeuble, CRUD clés de répartition, tantièmes, stats, registre
│       │   ├── claims.rs       ← CRUD sinistres + documents attachés
│       │   ├── consultations.rs ← CRUD consultations, réponse, résultats, clôture
│       │   ├── contractor.rs   ← portail prestataire : tâche, URL upload, soumission devis (accès token)
│       │   ├── contractors.rs  ← CRUD référentiel prestataires
│       │   ├── contracts.rs    ← CRUD contrats fournisseurs, résiliation, rappels, documents attachés
│       │   ├── documents.rs    ← upload (pre-signed URL), confirmation, liste, download, suppression
│       │   ├── logbook.rs      ← CRUD carnet d'entretien (maintenance_log)
│       │   ├── mod.rs          ← router principal Axum + OpenAPI
│       │   ├── onboarding.rs   ← statut onboarding + bootstrap immeuble
│       │   ├── reports.rs      ← CRUD signalements, transitions statut, conversion en tâche, documents
│       │   ├── runtime.rs      ← GET /runtime/status (demo_mode flag)
│       │   ├── stubs.rs        ← placeholders pour routes non encore implémentées
│       │   ├── tasks.rs        ← CRUD tâches, documents, génération/révocation token prestataire
│       │   ├── units.rs        ← CRUD lots, archivage, historique propriétaires, transfert, charges
│       │   └── users.rs        ← liste des utilisateurs
│       ├── auth/               ← jwt, middleware, passkey, password, token
│       │   ├── jwt.rs
│       │   ├── middleware.rs
│       │   ├── mod.rs
│       │   ├── passkey.rs
│       │   ├── password.rs
│       │   └── token.rs
│       ├── config.rs           ← typed config from env
│       ├── db/                 ← repositories SQLx (14 repos + mod.rs)
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
│       │   ├── charges.rs              ← répartition des charges par tantième (Decimal)
│       │   ├── convocation_checklist.rs ← checklist légale convocation AG (art. 7-13 décret 1967)
│       │   ├── minutes_draft.rs        ← génération PV draft Markdown (art. 17 décret 67-223)
│       │   ├── mod.rs                  ← Role, AuthUser, re-exports
│       │   ├── processor.rs            ← trait DocumentProcessor + NoopProcessor (OCR/NLP futur)
│       │   ├── reminders.rs            ← calcul dates de rappel contrat fournisseur
│       │   └── vote.rs                 ← quorums art. 24/25/25-1/26/26-1 (testé)
│       ├── errors.rs           ← unified AppError → JSON HTTP
│       ├── jobs/
│       │   ├── mod.rs
│       │   └── reminders.rs    ← hourly poll, prêt pour notifications
│       ├── main.rs             ← assembly, auto-migrations, jobs
│       └── storage/            ← client S3, presigned URLs
│           ├── client.rs
│           ├── mod.rs
│           └── presigned.rs
├── CONTRIBUTING.md
├── docker-compose.prod.yml     ← override production, image Docker Hub
├── docker-compose.yml          ← laruche + postgres + minio
├── docker/
│   ├── Caddyfile               ← config Caddy (SPA + reverse proxy)
│   └── entrypoint.sh           ← démarrage backend + Caddy
├── Dockerfile                  ← multi-stage unique (Node + Rust + Caddy)
├── docs/
│   ├── adr/                    ← ADR-001 à ADR-009
│   ├── product/                ← PRD détaillés par feature
│   ├── ARCHITECTURE.md
│   ├── DESIGN.md
│   ├── EPICS.md
│   ├── LEGAL.md
│   ├── PRD.md
│   └── PROGRESS.md
├── frontend/src/
│   ├── api/
│   │   ├── client.ts           ← instance Axios + intercepteurs refresh + publicApi (token prestataire)
│   │   └── index.ts            ← tous les modules API regroupés
│   ├── components/
│   │   ├── atoms/              ← CAlert, CBadge, CBtn, CEyebrow,
│   │   │                          CFormCheckbox, CFormChipInput, CFormChipSelect,
│   │   │                          CFormFeedback, CFormInput, CFormSelect, CFormTextarea,
│   │   │                          CLegalDescription, CMockBanner, CProgressBar,
│   │   │                          CSeparator, CTooltip + icônes SVG (CIcon*.svg)
│   │   ├── molecules/          ← CCard, CCardNote, CDataTable,
│   │   │                          CFormAddressAutocomplete, CFormDropzone,
│   │   │                          CKpiCard, CLegalCard, CLegalNotice,
│   │   │                          CListItem, CMeta, CMetaItem, CMetaLine,
│   │   │                          CModal, CPageLoader
│   │   └── organisms/          ← AppNav
│   ├── composables/            ← hooks transverses
│   │   ├── useAutoClose.ts
│   │   ├── useConfirm.ts
│   │   ├── useFormatters.ts    ← formatage Intl natif (fr-FR), sans date-fns
│   │   ├── useFormFeedback.ts
│   │   ├── usePageLoader.ts
│   │   └── useRole.ts
│   ├── features/               ← vues lazy-loaded par domaine
│   │   ├── assemblies/
│   │   ├── auth/
│   │   ├── budget/
│   │   ├── claims/
│   │   ├── consultations/
│   │   ├── contractor/         ← portail prestataire (accès token, sans JWT)
│   │   ├── contractors/
│   │   ├── contracts/
│   │   ├── dashboard/
│   │   ├── documents/
│   │   ├── maintenance/
│   │   ├── onboarding/
│   │   ├── profile/
│   │   ├── reports/
│   │   ├── residence/
│   │   ├── residents/
│   │   └── tasks/
│   ├── layouts/                ← AppLayout
│   ├── router/index.ts         ← lazy routes, guards (403 vs 404, onboarding, redirects)
│   ├── stores/
│   │   ├── auth.ts             ← session utilisateur (Pinia)
│   │   ├── documents.ts        ← état GED partagé
│   │   ├── files.ts            ← upload / presigned URLs
│   │   └── ui.ts               ← préférences apparence (localStorage)
│   ├── styles/                 ← tailwind.css, tokens.css, base.css, typography.css
│   ├── types/domain.ts         ← types TS miroir schéma Rust
│   ├── utils/                  ← utilitaires purs (pas de composables)
│   └── views/                  ← ForbiddenView, NotFoundView
└── Makefile                    ← cibles make (dev, test, docker…)
```

Le frontend conserve les feuilles globales historiques (`tokens.css`, `base.css`, `tones.css`, etc.) comme source normative du design system. Tailwind CSS n'est ajouté que comme couche utilitaire optionnelle via `@tailwindcss/vite` (sans `tailwind.config.js`). Les tokens custom sont exposés via `@theme inline` dans `frontend/src/styles/tailwind.css`.

---

## Schéma DB — 38 tables

| Domaine | Tables |
|---|---|
| Copropriété | `building`, `unit`, `allocation_key`, `unit_allocation_key` |
| Utilisateurs | `user_account`, `user_token`, `user_building`, `unit_owner` |
| AG | `general_meeting`, `general_meeting_document`, `general_meeting_attendance`, `resolution`, `vote` |
| Budget | `budget`, `budget_line`, `levy_call`, `works_fund` — champ `category` sur `levy_call` **prévu** ; `budget.parent_budget_id` ; `budget_line.valid_from` / `valid_to` / `rectificatif_version` / `version` |
| Documents | `document` |
| Contrats | `vendor_contract`, `contract_document`, `contract_reminder` |
| Prestataires | `contractor` |
| Tâches | `task` (`contractor_id` FK → `contractor`, `contractor_name` texte libre, `vendor_token` ephemeral), `task_document`, `vendor_submission`, `vendor_submission_document` |
| Sinistres | `claim`, `claim_document` |
| Carnet | `maintenance_log`, `maintenance_log_document` |
| Signalements | `report`, `report_document` — `report.task_id` FK informatif (EP-017, conversion signalement → tâche) |
| Consultations | `consultation`, `consultation_choice`, `consultation_response`, `consultation_response_choice` |
| Registres | `judicial_proceeding` (procédures judiciaires en cours/clôturées — EP-002-S10), `governance_mandate` (registre des mandats syndic/conseil syndical — EP-002-S11) |
| Audit comptable | `audit_log` — journal probant append-only (ADR-009) ; champs : `entity_type`, `entity_id`, `operation`, `changed_by`, `old_values`/`new_values` JSONB (champs modifiés seulement). Aucun UPDATE/DELETE applicatif. `activity_log` (timeline UX EP-017) reste une cible backlog. |

Conventions : UUID v4, TIMESTAMPTZ, soft delete via `active`/`status`, `updated_at` via triggers, JSONB pour OCR futur.

**Règle de gestion transverse — Upload = GED (2026-05-22)**
Tout upload de fichier depuis n'importe quel module de l'application **crée systématiquement un enregistrement dans la table `document`** (GED centrale) avant d'être associé à l'entité source via une table de liaison dédiée (`{entite}_document`). Il n'existe pas de pièce jointe "privée" hors GED. Cette règle s'applique à tous les modules existants et à tout nouveau module qui introduit un mécanisme d'upload.

Invariants techniques implémentés :

- `report.task_id` porte le lien informatif entre signalement et tâche ; cette relation reste non destructive et réutilisable par les workflows applicatifs.
- Les documents de tâche sont répartis entre `task_document` (gestionnaires) et `vendor_submission_document` (soumissions prestataire) ; les endpoints manager `GET/POST/DELETE /tasks/{id}/documents` et `document_count` agrègent ces deux sources.
- `claim_document` suit le même modèle d'association documentaire que `report_document`, avec nettoyage des liens lors des suppressions d'entité.
- `maintenance_log.task_id` et `maintenance_log.contract_id` restent des liens de traçabilité souples, sans cascade destructive depuis les tâches ou contrats.
- `budget.parent_budget_id` (`UUID REFERENCES budget(id)`, nullable) lie un budget rectificatif au budget qu'il modifie ; un budget racine a `parent_budget_id IS NULL`.
- `document.category` (`document_category` : `reglement_copropriete` / `edd` / `autre`, défaut `autre`) type les documents fondateurs (LEGAL §1.2, EP-005-S06). `document.supersedes_id` (`UUID REFERENCES document(id)`, nullable, unique) chaîne une nouvelle version à celle qu'elle remplace ; `document.is_current` (`BOOLEAN`, défaut `TRUE`) marque la version en vigueur. Un index partiel garantit au plus une version courante par `(building_id, category)` pour les catégories fondatrices. Le hard delete d'un document fondateur reste refusé (`is_document_deletable`).
- `document.deleted_at`/`deleted_by`/`deletion_reason` (EP-005-S08, PRD US-DOC-004) : la suppression d'un document devient un soft-delete — la ligne et l'objet S3 sont conservés, seule la lecture filtre `deleted_at IS NULL`. La purge physique différée est portée par le job EP-005-S09, hors périmètre ici. Les cascades de suppression (contrat, tâche, sinistre, rapport → `delete_orphaned_tx`) appliquent le même soft-delete aux documents orphelins au lieu de les détruire.
- `document.replacement_reason` (EP-005-S08, PRD US-DOC-007) : pour les catégories critiques (`reglement_copropriete`, `edd` — `domain::immutability::document_requires_replacement_reason`), la suppression directe est refusée (`errors.documents.replacement_required`) ; seul un remplacement via `POST /documents` avec `supersedes_id` + `reason` (réutilise le workflow de version S06) est permis, qui exige une justification non vide avant de basculer `is_current`. Chaque remplacement écrit une entrée `audit_log` (`entity_type = 'document'`, `entity_id` = nouvelle version, `old_values.id` = version remplacée, `new_values.reason` = justification), dans la même transaction (AC-DOC-007-D).
- L'unicité annuelle du budget est portée par trois index partiels, et non par une contrainte de table : `budget_unique_root_per_year (building_id, fiscal_year) WHERE parent_budget_id IS NULL` (un seul budget racine par exercice), `budget_unique_pending_rectificatif (parent_budget_id) WHERE parent_budget_id IS NOT NULL AND status = 'draft'` (un seul rectificatif en préparation par budget modifié), et `budget_unique_approved_per_year (building_id, fiscal_year) WHERE status = 'approved'` (un seul budget en vigueur par exercice).
- `budget_line` est versionnée temporellement par `valid_from` (`DATE NOT NULL`) et `valid_to` (`DATE`, nullable, borne exclusive) : `valid_to IS NULL` marque la version en vigueur. L'identité logique d'une ligne à travers ses versions est `(budget_id, key_id, label)`, garantie par `budget_line_unique_active … WHERE valid_to IS NULL` — au plus une version active à la fois. `CHECK (valid_to IS NULL OR valid_to >= valid_from)` interdit un intervalle inversé.
- `budget_line.rectificatif_version` (`INTEGER`, nullable, `CHECK … >= 1`) porte le numéro du rectificatif dont la ligne provient ; `NULL` sur les lignes d'un budget racine. La valeur est assignée par le workflow de création du rectificatif et n'est jamais acceptée depuis une requête client.
- `budget_line.version` (`INTEGER NOT NULL DEFAULT 1`) est un jeton de verrouillage optimiste, distinct du versioning temporel : il protège l'édition concurrente d'une ligne de brouillon. Il est incrémenté par le trigger `budget_line_version_trigger`, dont la clause `WHEN (OLD.valid_to IS NOT DISTINCT FROM NEW.valid_to)` exclut la clôture d'une ligne — une opération système ne doit pas invalider le jeton détenu par un client.
- Les lignes d'un `budget` ne sont mutables qu'au statut `draft` : `POST/PUT/DELETE /budgets/{id}/lines[/{lid}]` sont refusés dès que le budget est approuvé, et `PUT`/`DELETE` portent `valid_to IS NULL` dans leur clause `WHERE` — une version historique clôturée n'est jamais réécrite ni supprimée. Les corrections passent par `POST /budgets/{id}/rectificatif`, qui recopie les lignes actives dans un nouveau budget `draft` sans toucher au budget modifié, puis par `POST /budgets/{id}/approve`, qui clôt le budget modifié et date ses lignes. `GET /budgets/{id}/rectificatifs` expose la chaîne, racine d'abord.
- `POST /budgets/{id}/approve` exige un `approval_meeting_id` référençant une `general_meeting` du même immeuble au statut `closed`.
- Les mutations `UPDATE` / `DELETE` sur `budget_line` écrivent une entrée `audit_log` avec un diff ciblé `old_values` / `new_values`.
- La clôture d'un `budget` (`status = closed`) gèle les mutations de l'exercice : création/suppression de lignes et génération de nouveaux appels ; le marquage `paid` d'un appel existant reste autorisé.
- Les appels de fonds portent un `call_label` persisted côté backend pour différencier explicitement la nature de chaque appel dans les vues comptables.
- Les appels de fonds persistés portent une décomposition `charge_amount` / `works_fund_amount` sur `levy_call` pour isoler le calcul du solde à appeler selon l'inclusion du fonds travaux.
- La génération des appels de fonds répartit la part `charges courantes` ligne budgétaire par ligne budgétaire à partir de `budget_line.key_id` et des tantièmes actifs de cette clé ; `levy_call` conserve le snapshot immuable par lot et une table append-only de ventilation relie chaque appel à ses `budget_line` sources pour recalculer le « déjà appelé » par ligne et par lot sans mutation destructive.
- Chaque création de `budget` initialise ou réutilise l'entrée `works_fund` du même exercice, qui porte le taux de cotisation cible utilisé par la génération d'appels et par les contrôles de clôture.
- La génération des appels de fonds côté backend refuse désormais l'opération si au moins une clé de répartition active n'est pas équilibrée sur le total des tantièmes de l'immeuble.
- La clôture d'un `budget` refuse la transition si le taux du `works_fund` est sous le seuil légal configuré ou si les appels émis sur l'exercice n'ont pas atteint la cible calculée sur ce taux.
- `BudgetView` mutualise désormais ses KPI via `CKpiCard`, dérive le récapitulatif de génération d'appels à partir des montants déjà appelés côté frontend, expose une modale de répartition circulaire calculée depuis `budget_line`, une carte dédiée de pilotage du fonds travaux branchée sur l'API réelle et une lecture consolidée des appels par copropriétaire.
- Le chantier `EP-003` validé en mai 2026 cible d'abord le branchement des vues AG sur l'API réelle existante (`meetings`, `resolutions`, `votes`, `results`, transitions `call` / `close`) avant toute extension de conduite de séance.
- La frontière documentaire et technique entre `EP-003` et `EP-013` est la suivante : `EP-003` couvre la préparation juridique de l'AG, la saisie des résolutions, le calcul des résultats, le dépôt du PV signé et une traçabilité simple de notification ; `EP-013` couvre la feuille de présence avancée, les rôles de séance, les notes collaboratives et la validation/signature probante du PV.
- Les besoins documentaires minimaux d'`EP-003` doivent réutiliser le module `Documents` existant ; une refonte typée des pièces réglementaires AG au-delà de `convocation` / annexes / `PV` reste renvoyée aux chantiers documentaires dédiés.
- Pour `EP-003-S02`, les pièces AG non-PV sont rattachées via une liaison dédiée `general_meeting_document`, tandis que le PV signé continue d'être référencé sur `general_meeting.minutes_document_id`.
- Pour `EP-003-S03`, la présence AG est portée par un registre dédié au niveau réunion (présent / procuration / vote par correspondance / absent) ; le vote par résolution reste stocké dans `vote` et ne sert plus de substitut implicite à la feuille de présence.
- `judicial_proceeding` (EP-002-S10) tient le registre des procédures judiciaires de la copropriété : statut `ongoing` / `suspended` / `closed`, cohérence `closed ⟺ closed_at NOT NULL` et `closed_at >= opened_at` garantie par CHECK et revalidée côté domaine ; lecture ouverte à tous les rôles, écriture réservée au gestionnaire. `list_ongoing` alimente l'état daté (EP-002-S06).
- `governance_mandate` (EP-002-S11) tient le registre des mandats (`syndic` / `council_president` / `council_member`) avec `end_date >= start_date`. Invariant **syndic actif unique** : deux mandats `syndic` du même immeuble ne peuvent se chevaucher dans le temps (mandats ouverts = `infinity`), garanti à la fois par un index partiel unique DB (`uniq_active_syndic_per_building`) et par un contrôle de chevauchement applicatif sur création/mise à jour ; écriture réservée au gestionnaire.

Arbitrage documentaire : le nom `audit_log` est réservé à la traçabilité probante d'ADR-009 (mutations comptables, diff ciblé, conservation longue, valeur de preuve). Les besoins de timeline UX et de suivi de cycle de vie de `report` / `task` relèveront d'un flux séparé `activity_log`, afin d'éviter de mélanger journal juridique append-only et événements métier orientés interface.

Les règles produit détaillées de navigation, d'apparence et de workflows métier restent normatives dans `PRD.md`, `DESIGN.md` et les ADR/DDR concernés ; ce document n'en conserve qu'un résumé d'architecture.

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

### Pièce justificative d'un contrat
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
| Isolation tenant | Toutes les requêtes filtrent sur `building_id` (ex-`copropriete_id`) |
| Erreurs | 403 (pas 404) sur routes protégées — pas de fuite d'info |
| Chiffrement | SSE at-rest S3, TLS at-transit |

---

## CI/CD

- Le workflow `CI` génère `backend/.sqlx/` directement dans le job **backend**, puis partage cet artefact au job **docker** sans relancer une préparation SQLx séparée.
- Le workflow sépare la **validation backend**, la **validation frontend** et un job **`docker`** unique, déclenché seulement après succès de `test-backend` et `test-frontend`.
- Dans le job backend, les migrations PostgreSQL ne sont exécutées qu'une seule fois avant `cargo sqlx prepare`, puis la validation Cargo enchaîne `cargo fmt --check`, `cargo clippy -- -D warnings` et un unique `cargo test --lib --tests`.
- Le job backend active `sccache` comme cache compilateur Rust dédié, persistant entre exécutions GitHub Actions via le backend GHA natif de `mozilla-actions/sccache-action`.
- Le job Docker unique construit et publie l'image dans la même exécution grâce au cache Buildx réutilisé par branche.
- Les caches CI sont mutualisés par écosystème : `Swatinem/rust-cache` (clé stable `backend-ci`), `sccache` (objets `rustc` natifs GHA), `actions/setup-node` (npm), cache GHA Buildx (Docker, par branche).

---

## Principes de code

- **Zéro logique métier dans les handlers**
- **Zéro SQL dans domain/**
- **`pub(crate)` par défaut**
- **Pas de `unwrap()` hors tests**
- **`Decimal` pour tous les montants financiers** (pas `f64`)
- **`SQLX_OFFLINE=true` en CI et en build Docker** — `backend/.sqlx/` est généré une seule fois dans `test-backend`, puis téléchargé par les jobs Docker
- **Les erreurs** sont renvoyées sous forme de code i18n (`AppError::validation_code("errors.x.y")`), jamais en strings bruts. ex: `errors.unauthorized`, `errors.budget.exercice_closed`

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
| ADR-009 | Immutabilité des données comptables + audit log | Guards `is_mutable`/`is_deletable` dans `domain/` ; table `audit_log` append-only (diff JSONB `old_values`/`new_values` ; statut `draft` sur `levy_call` ; `predecessor_id` sur `vendor_contract` (pattern avenant) ; `parent_budget_id` + `valid_from`/`valid_to` sur `budget_line` ; trigger PG défensif sur `levy_call` |
| ADR-010 | PV conseil syndical | Extension `meeting_type` ENUM (`annual` / `extraordinary` / `council`) ; réutilisation de `general_meeting` + `general_meeting_document` ; champ `minutes_document_id` référence le PV signé — pas de nouvelle table |
| ADR-011 | Génération PDF réglementaire | Génération serveur via crate Rust pur (`typst` par défaut, `genpdf` repli) ; module `domain/` dédié ; PDF persisté en GED ; rendu déterministe ; syndic uniquement — débloque S06/S07/S13 |

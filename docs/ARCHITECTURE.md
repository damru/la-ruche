---
document: ARCHITECTURE
version: 2.2.0
owner: Damien
status: validé
last_updated: 2026-10-21
description: >
  Source de vérité de l'architecture technique de La Ruche — stack,
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
  - ADR-012

---

# ARCHITECTURE.md — l'Application

> **Dénomination** — Dans le présent document, l'expression **« l'Application »** désigne exclusivement **La Ruche**, plateforme web open source de gestion participative de copropriété. Ces deux termes sont interchangeables et désignent la même entité logicielle.

> Document vivant. Dernière mise à jour : 2026-10-21

---

## Stack technique

| Couche | Technologie | Référence |
|---|---|---|
| Runtime | Rust (édition 2024) | ADR-001 |
| Framework HTTP | Axum 0.8 | ADR-001 |
| Base de données | PostgreSQL 16 | ADR-001 |
| ORM / Query | SQLx 0.9 (compile-time) | ADR-001 |
| Migrations | SQLx migrate (embedded) | ADR-001 |
| Auth | JWT + Argon2id + webauthn-rs (passkeys **stubs** — `auth/passkey.rs` compilable, 4 handlers retournent `tracing::warn!` + 501 ; pas de roadmap active) | ADR-002 |
| Stockage | S3-compatible — self-hosted (RustFS) | ADR-001 |
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

## Schéma DB — 40 tables

| Domaine | Tables |
|---|---|
| Copropriété | `building`, `unit`, `allocation_key`, `unit_allocation_key`, `edd_reference`, `edd_reference_share` |
| Utilisateurs | `user_account`, `user_token`, `user_building`, `unit_owner` |
| AG | `general_meeting`, `general_meeting_document`, `general_meeting_attendance`, `resolution`, `vote` |
| Budget | `budget`, `budget_line`, `levy_call`, `works_fund` — champ `category` sur `levy_call` **prévu** ; `budget.parent_budget_id` ; `budget_line.valid_from` / `valid_to` / `rectificatif_version` / `version` |
| Documents | `document`, `document_event` |
| Contrats | `vendor_contract`, `contract_document`, `contract_reminder` — liste filtrable côté serveur : `GET /contracts?status=&contract_type=&vendor=` (EP-006-S06) ; prestataires distincts : `GET /contracts/vendors` |
| Prestataires | `contractor` |
| Tâches | `task` (`contractor_id` FK → `contractor`, `contractor_name` texte libre, `vendor_token` ephemeral), `task_document`, `vendor_submission`, `vendor_submission_document` |
| Sinistres | `claim`, `claim_document` |
| Carnet | `maintenance_log`, `maintenance_log_document` |
| Signalements | `report`, `report_document` — `report.task_id` FK informatif (EP-017, conversion signalement → tâche) |
| Consultations | `consultation`, `consultation_choice`, `consultation_response`, `consultation_response_choice` |
| Calendrier | `calendar_event`, `calendar_event_document` — événements manuels (réunion informelle, rappel, rendez-vous), EP-014-S03 |
| Registres | `judicial_proceeding` (procédures judiciaires en cours/clôturées — EP-002-S10), `governance_mandate` (registre des mandats syndic/conseil syndical — EP-002-S11) |
| État daté | `etat_date_request` (suivi de la demande + snapshot JSONB du document généré — EP-002-S06) |
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
- `document.category` (`document_category` : `reglement_copropriete` / `edd` / `pv_ag` / `comptabilite` / `appel_fonds` / `budget` / `contrat` / `diagnostic` / `sinistre` / `tache` / `libre` / `signalement` / `carnet_entretien` / `prestataire` / `autre`, défaut `autre`) type l'ensemble des documents de l'immeuble (PRD Documents §4, EP-005-S06/S07). Les documents fondateurs (`reglement_copropriete`, `edd`) restent seuls soumis aux règles de conservation permanente ; `diagnostic` couvre les diagnostics techniques réglementaires (DTG, DPE collectif — LEGAL §6.2-6.3, PPT hors périmètre logiciel) et `pv_ag` couvre les procès-verbaux d'assemblée générale, dont la conservation est sans limitation de durée (LEGAL §3.4). `document.supersedes_id` (`UUID REFERENCES document(id)`, nullable, unique) chaîne une nouvelle version à celle qu'elle remplace ; `document.is_current` (`BOOLEAN`, défaut `TRUE`) marque la version en vigueur. Un index partiel garantit au plus une version courante par `(building_id, category)` pour les catégories fondatrices. Le hard delete d'un document fondateur reste refusé (`is_document_deletable`).
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
- `edd_reference` (EP-001-S09, LEGAL §1.2) porte une référence EDD saisie manuellement (`label`, `edd_date`, `document_id` nullable vers `document` en catégorie `edd` — non encore raccordé côté saisie, EP-005-S06 fournit le typage). `edd_reference_share` porte les tantièmes de référence par `(unit_id, allocation_key_id)`. `GET /building/edd-check` compare la référence la plus récente (`edd_date` puis `created_at` DESC) aux `unit_allocation_key` actifs et retourne un rapport d'écarts (`domain::edd_check::compare`, fonction pure) : purement informatif, ne bloque ni la configuration des clés ni la génération d'appels.
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
- `etat_date_request` (EP-002-S06) suit la demande d'état daté d'un lot : `requested_at`/`due_date` (J+15, calculé en domaine), `status` (`requested` / `generated` / `sent` / `cancelled`), `snapshot` JSONB figeant le document (3 parties réglementaires + annexes) au moment de la génération — valeur probante, les données vivantes continuant d'évoluer ensuite. Régénération autorisée tant que `status != 'sent'`. Assemblage des données (charges/fonds travaux EP-004, procédures `judicial_proceeding` EP-002-S10) en lecture pure, aucune duplication SQL ; l'assemblage est partagé avec le pré-état daté (EP-002-S07).
- Génération PDF réglementaire (ADR-011) : `GET /etat-date-requests/{id}/pdf` rend le snapshot figé et `GET /units/{id}/pre-etat-date/pdf` le pré-état daté en PDF, côté serveur (`domain/legal_documents.rs`, crate `genpdf`, polices Liberation Sans embarquées via `include_bytes!`). Le PDF est un **rendu** du snapshot (qui reste la valeur probante) ; il n'est pas persisté en GED (S07 diffère l'archivage, cf. `docs/specs/EP-002-S07-spec.md`).

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
| ADR-011 | Génération PDF réglementaire | Génération serveur via crate Rust pur (`typst` par défaut, `genpdf` repli) ; module `domain/` dédié ; syndic uniquement — débloque S06/S07/S13. Implémentation S06/S07 : `genpdf` (repli, API directe sans langage de balisage) + polices Liberation Sans embarquées ; rendu direct (non persisté en GED, le snapshot JSONB restant la valeur probante) |
| ADR-012 | Verrouillage optimiste des lignes budgétaires | Colonne `budget_line.version` cantonnée au contrôle de concurrence (409 sur jeton périmé) ; incrément par trigger, clôture système exclue ; distinct du versioning temporel |

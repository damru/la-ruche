# Spécification technique — EP-012-S01

**Story :** Exporter les données de la copropriété au format attendu par le Registre National des Copropriétés (RNC, registre.coproprietes.gouv.fr) pour faciliter la mise à jour annuelle.

**Réfs :** Épic EP-012 (`docs/EPICS.md` §EP-012), LEGAL.md §1.1, PRD `docs/product/PRD - Conformité et reporting.md` (US-CR-006, matrice CRUD §3).

> Cette spec fonde le socle RNC. EP-012-S03 (dossier de dépôt : contrôles de complétude + journal) s'appuie directement sur le module `rnc` et la table `rnc_export` définis ici — voir `EP-012-S03-spec.md`.

## 1. Quoi (résumé fonctionnel)

Le Syndic génère à la demande un export structuré (JSON ou CSV) des données déclaratives annuelles attendues par le RNC. La télétransmission API est hors scope (PRD §9) : l'export sert de source de recopie/dépôt manuel sur la plateforme gouvernementale. Les champs non dérivables des modules existants sont saisis manuellement dans le formulaire d'export et persistés avec le snapshot.

Accès : Syndic uniquement (`Role::Manager`) — matrice CRUD PRD §3 (`Export RNC : C/R` Syndic, `X` pour tous les autres rôles).

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| RNC / fiche synthétique | **Rien n'existe** — zéro occurrence de `rnc`/`fiche_synthetique` dans backend et frontend |
| Identité copropriété (nom, adresse, `manager_siret`, `shares_total`, `bylaws_date`) | `backend/migrations/0001_init.sql` L150-163 (table `building`), handlers `backend/src/api/building.rs` (`get`, `stats`, `registry`) |
| Lots / tantièmes | table `unit` (`0001_init.sql` L169-182), `backend/src/db/unit.rs`, `backend/src/db/building.rs::get_stats` (`nb_units`, `nb_users`) |
| Budget / exercice | tables `budget` (L391-400), `budget_line` ; repo `backend/src/db/budget.rs` |
| Fonds travaux | table `works_fund` (L459+ : `fiscal_year`, `levied_amount`, `available_amount`, `contribution_rate_pct`) ; `db::budget::get_or_create_works_fund` |
| Impayés | table `levy_call_unit` (statut `levy_status`, `paid_at`) — sommes exigibles non payées dérivables |
| Procédures en cours | module `judicial_proceedings` (`backend/src/api/judicial_proceedings.rs`, `backend/src/db/judicial_proceeding.rs`) |
| Rôles | `backend/src/domain/mod.rs` L17-22 (`Manager`, `BoardMember`, `CoOwner`, `Tenant`), guard `auth.require_role(&Role::Manager)` |
| Routes | `backend/src/api/mod.rs` (router `/api/v1`, blocs commentés par domaine) |
| API client frontend | `frontend/src/api/index.ts` ; types `frontend/src/types/domain.ts` |
| Dernière migration observée | `0039_budget_rectificatif_constraints` — prendre le prochain numéro libre (`NNNN` ci-dessous) |

## 3. Modèle de données

Nouvelle table `rnc_export` — chaque export est un snapshot horodaté (sert aussi de journal pour S03). **Mettre à jour `docs/ARCHITECTURE.md` avant d'écrire la migration.**

### 3.1 Migration `backend/migrations/NNNN_rnc_export.sql`

```sql
-- EP-012-S01: RNC annual declaration export (snapshot + journal)
CREATE TABLE rnc_export (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id   UUID NOT NULL REFERENCES building(id),
    fiscal_year   INTEGER NOT NULL,
    format        TEXT NOT NULL CHECK (format IN ('json', 'csv')),
    payload       JSONB NOT NULL,          -- exact snapshot of the exported dataset
    manual_fields JSONB NOT NULL DEFAULT '{}'::jsonb, -- operator-provided values (see §4.2)
    checks        JSONB,                   -- completeness report — filled by EP-012-S03
    exported_by   UUID NOT NULL REFERENCES app_user(id),
    exported_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX rnc_export_building_year_idx ON rnc_export (building_id, fiscal_year);
```

`payload` en `JSONB` : l'export est un document figé (valeur probante du journal S03), jamais requêté relationnellement — pas de colonnes typées. Les montants sont sérialisés en chaînes décimales (`Decimal` → string), jamais en float.

Après migration : `cargo sqlx migrate run` puis `cargo sqlx prepare` (committer `backend/.sqlx/`).

## 4. Backend Rust

### 4.1 `backend/src/domain/rnc.rs` — dataset et mapping (nouveau)

Struct `RncDataset` (serde `Serialize`/`Deserialize`) reflétant le dictionnaire de données RNC pour la mise à jour annuelle. Table de correspondance :

| Champ RNC (rubrique déclarative) | Champ `RncDataset` | Source app | Dispo |
|---|---|---|---|
| N° d'immatriculation RNC | `registration_number` | — | ❌ manuel |
| Nom d'usage de la copropriété | `name` | `building.name` | ✅ |
| Adresse (n°, voie, CP, ville) | `address_line1/2`, `postal_code`, `city` | `building` | ✅ |
| Identité du syndic (nom, type bénévole) | `manager_name`, `manager_kind` | `building` + profil du Manager (`app_user`) | ⚠️ nom manuel si non déductible |
| SIRET du syndic | `manager_siret` | `building.manager_siret` | ✅ (nullable) |
| Date du règlement de copropriété | `bylaws_date` | `building.bylaws_date` | ✅ (nullable) |
| Nombre total de lots | `total_units` | `COUNT(unit WHERE active)` | ✅ |
| Nombre de lots à usage d'habitation | `residential_units` | `COUNT(unit WHERE unit_type = 'apartment')` | ✅ |
| Budget prévisionnel voté (exercice) | `budget_amount` (`Decimal`) | `SUM(budget_line.budgeted_amount)` du budget de l'exercice | ✅ |
| Charges de l'exercice clos | `charges_amount` (`Decimal`) | `SUM(budget_line.actual_amount)` exercice N-1 | ⚠️ manuel si N-1 absent |
| Solde du fonds travaux | `works_fund_balance` (`Decimal`) | `works_fund.available_amount` | ✅ |
| Sommes restant dues par les copropriétaires | `unpaid_amount` (`Decimal`) | `SUM(levy_call_unit.amount WHERE status <> 'paid')` sur l'exercice | ✅ |
| Dettes fournisseurs | `vendor_debt` (`Decimal`) | — (pas de comptabilité fournisseur) | ❌ manuel |
| Nombre de salariés du syndicat | `employee_count` | — | ❌ manuel |
| Procédures en cours (mandat ad hoc, etc.) | `open_proceedings` | `COUNT(judicial_proceeding)` en cours | ⚠️ pré-rempli, corrigeable |

Fonctions pures (testables sans DB) :
- `build_dataset(sources: RncSources, manual: RncManualFields) -> RncDataset` — fusionne données dérivées + saisies manuelles (le manuel ne peut écraser que les champs ⚠️/❌).
- `to_csv(&RncDataset) -> String` — une ligne d'en-têtes (identifiants anglais stables) + une ligne de valeurs, séparateur `;` (convention ANAH), montants avec point décimal.

### 4.2 `backend/src/db/rnc_export.rs` (nouveau)

- `collect_sources(pool, building_id, fiscal_year) -> AppResult<RncSources>` — requêtes d'agrégation SQL listées §4.1 (SQL confiné à `db/`).
- `insert(pool, building_id, input: CreateRncExportInput) -> AppResult<RncExportRow>`.
- `list(pool, building_id) -> AppResult<Vec<RncExportRow>>` (ordre `exported_at DESC`) — consommé par S03.

### 4.3 `backend/src/api/rnc.rs` (nouveau — zéro logique métier)

- `POST /rnc/exports` — body `{ fiscal_year, format, manual_fields }`. Guard `auth.require_role(&Role::Manager)?`. Orchestration : `collect_sources` → `domain::rnc::build_dataset` → `insert` → retourne la ligne créée avec `payload`.
- `GET /rnc/exports/{id}/file` — télécharge le fichier (JSON brut ou `to_csv`), header `Content-Disposition: attachment; filename="rnc-{fiscal_year}.{format}"`.
- Erreurs i18n : `AppError::validation_code("errors.rnc.fiscal_year_invalid")` (année hors plage 2000..=2100), `errors.rnc.no_budget_for_year` (aucun budget pour l'exercice demandé — l'export exige au moins le budget).

### 4.4 `backend/src/api/mod.rs`

```rust
// ── RNC (EP-012) ─────────────────────────────────────────
.route("/rnc/exports", get(rnc::list).post(rnc::create))
.route("/rnc/exports/{id}/file", get(rnc::download))
```
(`rnc::list` est spécifié en S03 mais la route est posée ici pour éviter un second passage dans `mod.rs`.)

## 5. Frontend Vue

- `frontend/src/types/domain.ts` : `RncExport`, `RncDataset`, `RncManualFields`.
- `frontend/src/api/index.ts` : `rncApi = { list, create, downloadUrl }`.
- Nouveau feature dir `frontend/src/features/compliance/` :
  - `RncExportView.vue` — sélection d'exercice, formulaire des champs manuels (`registration_number`, `vendor_debt`, `employee_count`, corrections ⚠️), aperçu du dataset dérivé (lecture seule), boutons « Exporter JSON » / « Exporter CSV ».
- Route `compliance/rnc` (name `rnc-export`), visible Syndic uniquement ; entrée de navigation dans la section administration.
- Composants catalogue `C*` uniquement, Tailwind sans valeurs brutes. Nouveau composant → `docs/DESIGN.md` §2 avant usage.

## 6. Cas limites

- Aucun budget pour l'exercice → 422 `errors.rnc.no_budget_for_year` (pas d'export partiel silencieux).
- `manager_siret` / `bylaws_date` NULL → exportés `null` (JSON) / vide (CSV), signalés dans l'aperçu — le blocage éventuel relève de S03, pas de S01.
- Champs manuels : seuls les champs marqués ⚠️/❌ sont acceptés ; toute autre clé dans `manual_fields` → 422 `errors.rnc.unknown_manual_field`.
- Exercice sans fonds travaux → `works_fund_balance = 0` (comportement `get_or_create_works_fund`).
- Ré-export du même exercice : autorisé sans limite — chaque export est une nouvelle ligne (le journal S03 rend l'historique lisible).

## 7. Critères d'acceptation

- **AC-S01-A** : `POST /rnc/exports` produit un snapshot persistant contenant tous les champs du mapping §4.1, montants en `Decimal` sérialisé chaîne.
- **AC-S01-B** : le fichier JSON et le fichier CSV téléchargés sont dérivés du même `payload` persisté (aucun recalcul au téléchargement).
- **AC-S01-C** : les champs non dérivables sont pré-remplis depuis `manual_fields` et identifiés comme tels dans l'aperçu frontend.
- **AC-S01-D** : un rôle non-Manager reçoit 403 sur toutes les routes `/rnc/*`.

## 8. Plan de tests

- Unitaires `domain/rnc.rs` : `build_dataset` (fusion manuel/dérivé, rejet clé inconnue), `to_csv` (échappement `;`, montants décimaux, en-têtes stables).
- e2e HTTP (`backend/tests/api_e2e_test.rs`) : création export nominal ; 422 sans budget ; 403 pour `co_owner` ; téléchargement CSV = snapshot.
- Frontend : rendu du formulaire manuel, appel `rncApi.create`, lien de téléchargement.

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` (table `rnc_export`) → migration `NNNN` → `cargo sqlx migrate run` + `cargo sqlx prepare`.
2. `domain/rnc.rs` + tests unitaires (vert avant toute I/O).
3. `db/rnc_export.rs` (sources + insert + list).
4. `api/rnc.rs` + routes `mod.rs` + tests e2e.
5. Frontend (`types`, `api`, `RncExportView.vue`, route) + `docs/DESIGN.md` si nouveau composant.
6. `graphify update .` + entrée datée `docs/PROGRESS.md` (EP-012-S01).

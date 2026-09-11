# Spécification technique — EP-012-S02

**Story :** Générer la fiche synthétique annuelle de copropriété (art. 8-2 loi 1965, décret n° 2016-1822) — contenu normé : identification, syndic, composition, données financières, contrats d'assurance, sinistres.

**Réfs :** Épic EP-012 (`docs/EPICS.md` §EP-012), LEGAL.md §1.3, PRD `docs/product/PRD - Conformité et reporting.md` US-CR-001 (AC-CR-001-A à D), §4 (cycle de vie), §5 (contenu normé).

> Spec fondatrice du modèle `fiche_synthetique`. S04 (vérification/correction du brouillon), S05 (publication), S06 (consultation résident) et S07 (export PDF) s'appuient sur la table, le cycle de vie et les routes définis ici — cross-références dans chaque spec.

## 1. Quoi (résumé fonctionnel)

Le Syndic déclenche la génération de la fiche synthétique d'un exercice (`fiscal_year`). Le backend pré-remplit toutes les sections depuis les modules existants (AC-CR-001-A : profil immeuble, registre des lots, propriétaires, budget, fonds travaux, contrats, sinistres) et crée la fiche au statut `draft` (AC-CR-001-C). Aucune ressaisie de donnée financière ou légale (PRD §2).

Cycle de vie (PRD §4) : `draft` → `published` → `archived`. Cette spec couvre la création `draft` et la lecture ; les transitions sont en S05.

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| Fiche synthétique | **Rien n'existe** (zéro occurrence `fiche_synthetique`/`synthetic`) |
| Profil immeuble | table `building` (`0001_init.sql` L150-163) ; `backend/src/db/building.rs` (`get`, `get_stats` : `nb_units`, `nb_users`, `shares_total`, `works_fund_available`, `works_fund_pct`) |
| Lots / copropriétaires | tables `unit`, `app_user` ; `backend/src/db/unit.rs`, `user.rs` |
| Budget / charges | tables `budget` (`fiscal_year`, `status`), `budget_line` (`budgeted_amount`, `actual_amount`) ; `backend/src/db/budget.rs` |
| Fonds travaux | table `works_fund` ; `db::budget::get_or_create_works_fund` |
| Contrats (assurance MRI) | table `vendor_contract` (`contract_type`) ; `backend/src/db/contract.rs` |
| Sinistres | table `claim` ; `backend/src/db/claim.rs` |
| Précédent de cycle de vie draft/published | Pattern A `levy_call` : migrations `0030_levy_call_draft.sql` + `0033_levy_call_draft_status.sql` ; guards `backend/src/domain/immutability.rs` |
| Erreurs i18n | `backend/src/errors.rs` (`AppError::validation_code`) |
| Dernière migration observée | `0039_budget_rectificatif_constraints` — prochain numéro libre = `NNNN` |

## 3. Modèle de données

**Mettre à jour `docs/ARCHITECTURE.md` avant migration.** Décision structurelle (payload JSONB vs colonnes typées) : la fiche est un *document-snapshot* normé, immuable après publication, jamais requêté champ par champ → **JSONB unique + struct Rust typée** (même choix que `rnc_export.payload`, cf. `EP-012-S01-spec.md` §3.1). Si ce trade-off est jugé structurel au moment de l'implémentation, créer l'ADR avant la migration ; sinon la justification ci-dessus suffit.

### 3.1 Migration `backend/migrations/NNNN_fiche_synthetique.sql`

```sql
-- EP-012-S02: fiche synthétique annuelle (art. 8-2 loi 1965, décret 2016-1822)
CREATE TYPE fiche_status AS ENUM ('draft', 'published', 'archived');

CREATE TABLE fiche_synthetique (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id   UUID NOT NULL REFERENCES building(id),
    fiscal_year   INTEGER NOT NULL,
    status        fiche_status NOT NULL DEFAULT 'draft',
    payload       JSONB NOT NULL,             -- FichePayload (typed in domain/, amounts as decimal strings)
    generated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at  TIMESTAMPTZ,                -- set by EP-012-S05
    published_by  UUID REFERENCES app_user(id),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- One draft and one published fiche max per (building, fiscal_year).
CREATE UNIQUE INDEX fiche_one_draft_per_year
    ON fiche_synthetique (building_id, fiscal_year) WHERE status = 'draft';
CREATE UNIQUE INDEX fiche_one_published_per_year
    ON fiche_synthetique (building_id, fiscal_year) WHERE status = 'published';
```

Pas de valeur enum ajoutée après coup ici (type créé d'un bloc) — le split en deux migrations façon `0030`/`0033` n'est pas nécessaire.

Après migration : `cargo sqlx migrate run` + `cargo sqlx prepare` (committer `backend/.sqlx/`).

## 4. Backend Rust

### 4.1 `backend/src/domain/fiche_synthetique.rs` (nouveau)

Struct `FichePayload` (serde) — sections conformes PRD §5 / décret 2016-1822 :

```rust
pub struct FichePayload {
    pub identification: Identification,   // name, address_line1/2, postal_code, city, bylaws_date
    pub administration: Administration,   // manager_name, manager_siret, manager_kind ("volunteer"), mandate info
    pub composition: Composition,         // total_units, residential_units, co_owner_count, shares_total
    pub financial: Financial,             // budget_amount, closed_year_charges, works_fund_balance,
                                          //   works_fund_rate_pct — Decimal everywhere
    pub insurance: Insurance,             // MRI contract: vendor_name, subject, end_date (nullable block)
    pub claims: ClaimsSection,            // open_claims_count
}
```

- `Decimal` pour tout montant (jamais `f64`) ; sérialisation en chaîne dans le JSONB.
- `build_payload(sources: FicheSources) -> FichePayload` — fonction pure de pré-remplissage (AC-CR-001-A), testable sans DB.
- Transitions (consommées par S04/S05) : `can_edit(status) == (status == Draft)`, `can_publish(status) == (status == Draft)` — même esprit que `domain/immutability.rs`.

### 4.2 `backend/src/db/fiche_synthetique.rs` (nouveau)

- `collect_sources(pool, building_id, fiscal_year) -> AppResult<FicheSources>` — agrégations SQL : building, comptages lots/copropriétaires, budget de l'exercice (somme `budgeted_amount` ; `actual_amount` de l'exercice N-1 pour les charges closes), fonds travaux, contrat MRI actif (`contract_type` assurance, statut `active`), sinistres ouverts.
- `create_draft(pool, building_id, fiscal_year, payload) -> AppResult<FicheRow>` — INSERT ; l'index partiel garantit l'unicité du draft → mapper la violation en `AppError::conflict_code("errors.fiche.draft_already_exists")`.
- `get(pool, id, building_id)`, `list(pool, building_id)` (ordre `fiscal_year DESC, status`).

### 4.3 `backend/src/api/fiche_synthetique.rs` (nouveau — zéro logique métier)

- `POST /fiche-synthetique/generate` — body `{ fiscal_year }`. `auth.require_role(&Role::Manager)?`. Orchestration : `collect_sources` → `domain::fiche_synthetique::build_payload` → `create_draft`. 422 `errors.fiche.fiscal_year_invalid` si année hors plage ; 422 `errors.fiche.no_budget_for_year` si aucun budget pour l'exercice (le pré-remplissage financier est le cœur de la story).
- `GET /fiche-synthetique` — liste (Manager ; l'accès élargi lecture arrive en S05/S06).
- `GET /fiche-synthetique/{id}` — détail.

### 4.4 `backend/src/api/mod.rs`

```rust
// ── Fiche synthétique (EP-012) ───────────────────────────
.route("/fiche-synthetique", get(fiche_synthetique::list))
.route("/fiche-synthetique/generate", post(fiche_synthetique::generate))
.route("/fiche-synthetique/{id}", get(fiche_synthetique::get))
```
(S04 ajoutera le `PUT`, S05 le `POST /{id}/publish`, S06 le `GET /fiche-synthetique/published`.)

## 5. Frontend Vue

- `frontend/src/types/domain.ts` : `FicheStatus`, `FicheSynthetique`, `FichePayload` (montants en `string`).
- `frontend/src/api/index.ts` : `ficheApi = { list, get, generate }`.
- `frontend/src/features/compliance/FicheSynthetiqueView.vue` (nouveau, même feature dir que S01) : liste des fiches par exercice avec badge de statut, bouton « Générer la fiche {année} », lecture des sections en panneaux. L'édition (S04), la publication (S05) et l'impression (S07) enrichiront cette vue.
- Route `compliance/fiche` (name `fiche-synthetique`), navigation Syndic.
- Catalogue `C*` uniquement ; nouveau composant → `docs/DESIGN.md` §2.

## 6. Cas limites

- Draft déjà existant pour l'exercice → 409 `errors.fiche.draft_already_exists` (pas de régénération silencieuse — le Syndic supprime/écrase explicitement, hors scope S02).
- Budget de l'exercice au statut `draft` (non voté) → génération permise mais `budget_amount` calculé sur le prévisionnel courant ; signalé dans le payload (`financial.budget_status`).
- Pas d'exercice N-1 → `closed_year_charges = null` (le décret l'exige pour l'exercice clos ; corrigeable manuellement en S04).
- Aucun contrat MRI actif → section `insurance` nulle, la fiche reste générable (complétude bloquante uniquement à la publication, S05).
- `generated_at` fige l'instant de pré-remplissage : une régénération ultérieure (après suppression du draft) recalcule tout.

## 7. Critères d'acceptation

- **AC-CR-001-A** : tous les champs §4.1 pré-remplis depuis les modules, zéro ressaisie.
- **AC-CR-001-B** : l'exercice est identifié par `fiscal_year` (entier, ex. 2025).
- **AC-CR-001-C** : la fiche est créée au statut `draft`.
- **AC-CR-001-D** (perf, NFR-Perf-001) : génération ≤ 2 s.
- **AC-S02-E** : un rôle non-Manager reçoit 403 sur `POST /generate`.

## 8. Plan de tests

- Unitaires `domain/fiche_synthetique.rs` : `build_payload` (mapping complet, sections nulles, montants `Decimal`), transitions `can_edit`/`can_publish`.
- e2e HTTP : génération nominale (payload complet vérifié champ à champ) ; 409 double draft ; 422 sans budget ; 403 `co_owner`.
- Frontend : liste + génération, affichage des sections, badge statut.

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` (table + enum) → décision ADR éventuelle (§3) → migration `NNNN` → `sqlx migrate run` + `sqlx prepare`.
2. `domain/fiche_synthetique.rs` + tests unitaires.
3. `db/fiche_synthetique.rs` (`collect_sources`, `create_draft`, `get`, `list`).
4. `api/fiche_synthetique.rs` + routes + tests e2e.
5. Frontend (types, `ficheApi`, `FicheSynthetiqueView.vue`, route, navigation).
6. Entrée `docs/PROGRESS.md` (EP-012-S02).

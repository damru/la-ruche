# Spécification technique — EP-004-S08

**Story :** Émission trimestrielle paramétrable des appels de fonds + contrôles d'échéances par exercice + catégories d'appels typées (art. 35 décret 1967, version consolidée v6.0) : avances PPT, cotisations fonds travaux (art. 14-2-1 loi 1965), provisions déléguées au conseil syndical.

**Épic :** EP-004 — Budget & comptabilité
**Réf. LEGAL :** §4.3 · **Réf. PRD :** US budget/appels de fonds · **Réf. ADR :** ADR-009 (les appels émis restent immuables — Pattern A `levy_call`)

> Statut : PARTIEL (mince). L'existant se limite à un libellé trimestriel par défaut ; ni catégorie typée, ni calendrier d'émission paramétrable n'existent.

---

## 1. État actuel du code

| Élément | Emplacement |
|---|---|
| Helper libellé trimestriel `quarterly_default_call_label` | `backend/src/api/budget.rs:280-283` (utilisé `:472`) |
| `GenerateLeviesInput = { due_date, include_works_fund, call_label }` | `backend/src/api/budget.rs:269-273` |
| Table `levy_call` (`call_label`, `due_date`, split `charge_amount` / `works_fund_amount`) | `backend/migrations/0001_init.sql:413-422` |
| **Absent** | Colonne de catégorie typée sur `levy_call` ; table de calendrier d'émission ; contrôle d'échéance par exercice |
| Audit infra | `backend/src/db/audit.rs` (insert générique sur `Executor`, append-only), migration `0029_audit_log.sql` |

---

## 2. Quoi (résumé fonctionnel)

1. **Catégorie typée sur chaque appel** : enum PostgreSQL `levy_category` = `budget_provision | works_fund | ppt_advance | council_delegation | exceptional`. Aujourd'hui la nature d'un appel n'est qu'implicite (split montant charges / fonds travaux).
2. **Calendrier d'émission par exercice** : les dates des 4 appels trimestriels sont configurables par exercice budgétaire (défaut : 1er jour de chaque trimestre de l'exercice), éditables tant que l'exercice est ouvert. La génération d'appels lit ce calendrier au lieu d'une `due_date` unique saisie à la volée.
3. **Contrôles d'échéances** : à la génération, refuser une `due_date` hors de l'exercice budgétaire ; avertir/refuser une émission en double pour un même trimestre.
4. **Audit** : toute modification du calendrier d'émission écrit une ligne `audit_log` dans la même transaction.

**Note PPT :** la catégorie `ppt_advance` (avances sur plan pluriannuel de travaux, art. 14-2 loi 1965) est créée dès maintenant dans l'enum pour figer le modèle, mais reste **inutilisable côté génération** tant que le modèle PPT n'existe pas (EP-009-S04, hors périmètre). La génération rejette `ppt_advance` avec `errors.levies.ppt_not_available`.

---

## 3. Modèle de données

Migration `backend/migrations/NNNN_levy_category_and_schedule.sql` (numérotation en cours de renumérotation — la branche porte des 0038/0039 non trackées ; caler au moment de l'implémentation).

```sql
-- EP-004-S08: typed levy categories (art. 35 décret 1967) + per-fiscal-year emission schedule (LEGAL §4.3)
CREATE TYPE levy_category AS ENUM
    ('budget_provision', 'works_fund', 'ppt_advance', 'council_delegation', 'exceptional');

ALTER TABLE levy_call
    ADD COLUMN category levy_category NOT NULL DEFAULT 'budget_provision';

-- Backfill: existing calls with works_fund_amount > 0 and charge_amount = 0 are pure works-fund calls.
UPDATE levy_call SET category = 'works_fund'
    WHERE works_fund_amount > 0 AND charge_amount = 0;

CREATE TABLE levy_schedule (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    budget_id UUID NOT NULL REFERENCES budget(id) ON DELETE CASCADE,
    quarter SMALLINT NOT NULL CHECK (quarter BETWEEN 1 AND 4),
    due_date DATE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (budget_id, quarter)
);
```

Attention à la contrainte enum PostgreSQL : `levy_category` est un **nouveau** type (pas un `ADD VALUE`), donc une seule migration suffit — pas besoin du split 0030/0033.

Étapes associées : mettre à jour `docs/ARCHITECTURE.md` (nouvelle table `levy_schedule` + colonne `levy_call.category`) **avant** la migration ; puis `cargo sqlx migrate run` + `cargo sqlx prepare` (commit `backend/.sqlx/`).

---

## 4. Backend

### 4.1 Domain — `backend/src/domain/` (nouveau module ou extension du module budget)

- Fonction pure `default_quarterly_schedule(fiscal_year_start: NaiveDate) -> [NaiveDate; 4]` : 1er jour de chaque trimestre de l'exercice.
- Validation pure `validate_due_date_in_fiscal_year(due_date, fy_start, fy_end) -> Result<(), AppError>` → `AppError::validation_code("errors.levies.due_date_outside_fiscal_year")`.
- Validation catégorie : `ppt_advance` → `AppError::validation_code("errors.levies.ppt_not_available")` (jusqu'à EP-009-S04).
- Le calendrier n'est éditable que si le budget/exercice est **ouvert** (statut ≠ `closed`) — guard cohérent avec `domain/immutability.rs` (les appels déjà émis, eux, restent immuables — Pattern A, inchangé).

### 4.2 DB — `backend/src/db/budget.rs` (ou nouveau `db/levy_schedule.rs`)

- `LevyScheduleRow { id, budget_id, quarter, due_date }`.
- `list_schedule(pool, budget_id, building_id)`, `upsert_schedule(tx, budget_id, entries)` — l'upsert écrit dans la même transaction une ligne `audit_log` via `db::audit::insert` (entity=`levy_schedule`, operation=`UPDATE`, payload avant/après).
- `generate_levies` : ajouter le bind `category` ; propager `category::text` dans les SELECT/RETURNING de `levy_call`.

### 4.3 API — `backend/src/api/budget.rs` + routes `backend/src/api/mod.rs`

- `GET /budgets/{id}/levy-schedule` (manager + board, lecture) — renvoie le calendrier, initialisé aux défauts si absent.
- `PUT /budgets/{id}/levy-schedule` (manager) — remplace les 4 dates ; `400` si exercice fermé (`errors.levies.schedule_frozen`) ou date hors exercice.
- `GenerateLeviesInput` : ajouter `category: Option<String>` (défaut `budget_provision`) ; si `include_works_fund` reste utilisé, il mappe la part fonds travaux (compat ascendante). `due_date` devient optionnelle : si absente, prise depuis `levy_schedule` pour le trimestre visé.
- Contrôle anti-doublon : refuser la génération si un appel non-annulé de même `(budget_id, quarter/category, call_label)` existe → `errors.levies.already_issued_for_quarter`.

Aucune logique dans les handlers : validations dans `domain/`, SQL dans `db/`.

### 4.4 Erreurs i18n

`errors.levies.due_date_outside_fiscal_year`, `errors.levies.schedule_frozen`, `errors.levies.already_issued_for_quarter`, `errors.levies.ppt_not_available` — via `AppError::validation_code`, jamais de chaînes brutes. Ajouter les clés dans les fichiers i18n frontend.

---

## 5. Frontend

- `frontend/src/types/domain.ts` : `LevyCategory`, champ `category` sur `LevyCall`, type `LevySchedule`.
- `frontend/src/api/index.ts` : `budgetApi.getLevySchedule(budgetId)`, `updateLevySchedule(budgetId, payload)` ; `generateLevies` enrichi (`category`).
- `frontend/src/features/budget/GenerateLeviesModal.vue` : sélecteur de catégorie (libellés i18n ; `ppt_advance` désactivé avec tooltip « disponible avec le PPT ») ; la `due_date` se pré-remplit depuis le calendrier.
- `frontend/src/features/budget/BudgetView.vue` : section « Calendrier d'émission » (4 dates éditables si exercice ouvert) ; badge de catégorie sur chaque appel listé.
- Nouveau composant éventuel `LeveyScheduleCard.vue` → à cataloguer dans `docs/DESIGN.md §2` avant usage.

---

## 6. Cas limites

| Cas | Règle |
|---|---|
| Exercice fermé | Calendrier gelé (`errors.levies.schedule_frozen`) ; les appels émis restent immuables (Pattern A, inchangé). |
| Date hors exercice | Rejet `errors.levies.due_date_outside_fiscal_year`. |
| Double émission trimestre | Rejet `errors.levies.already_issued_for_quarter` (les avoirs/annulations existants ne comptent pas comme émission active). |
| `ppt_advance` demandé | Rejet `errors.levies.ppt_not_available` tant qu'EP-009-S04 n'est pas livré. |
| Budget sans calendrier (données antérieures) | `GET` renvoie les défauts calculés ; le premier `PUT` matérialise les lignes. |
| Appels historiques | Backfill `category` par heuristique montant (cf. migration) ; aucun recalcul de montant. |

---

## 7. Critères d'acceptation

1. Chaque `levy_call` porte une `category` typée ; les appels existants sont backfillés.
2. Le calendrier des 4 échéances est consultable et éditable par exercice ouvert ; toute modification écrit une ligne `audit_log` (même transaction).
3. Génération refusée hors exercice, en double sur un trimestre, ou en catégorie `ppt_advance`.
4. La modale de génération pré-remplit la date depuis le calendrier et expose la catégorie.
5. `cargo clippy -- -D warnings`, `cargo test --lib --tests`, build + lint frontend verts.

---

## 8. Plan de tests

- **Unitaires domaine** : `default_quarterly_schedule` (exercice civil et à cheval), `validate_due_date_in_fiscal_year`, rejet `ppt_advance`.
- **E2E (`backend/tests/api_e2e_test.rs`)** : GET calendrier par défaut → PUT calendrier → ligne audit présente ; PUT sur exercice fermé → 400 ; génération avec catégorie → appel créé avec la bonne catégorie ; double génération même trimestre → 400 ; date hors exercice → 400.
- **Frontend** : couverture backend prioritaire ; smoke test manuel de la modale.

---

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` (table `levy_schedule`, colonne `category`).
2. Migration `NNNN` (caler le numéro après la renumérotation rectificatif) → `sqlx migrate run` + `sqlx prepare`.
3. Domain (fonctions pures + validations) + tests unitaires.
4. DB (`levy_schedule` repo, `category` propagée, audit dans la tx).
5. API (2 routes calendrier, `GenerateLeviesInput` enrichi) + clés i18n.
6. Tests e2e.
7. Frontend (types, api, modale, BudgetView).
8. `graphify update .` + entrée `docs/PROGRESS.md`.

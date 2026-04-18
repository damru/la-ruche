# Spécification technique — EP-009-S10

**Story :** Filtres du carnet d'entretien par catégorie, période et source.

**Épic :** EP-009 — Carnet d'entretien
**Réf. PRD :** US-CE-005
**État :** PARTIEL — cette spec ne couvre que l'écart restant.

---

## 1. État actuel du code (écart)

| Élément | Emplacement | Constat |
|---|---|---|
| Recherche + onglets période (client) | `MaintenanceView.vue:39-41` (état), tabs `:417`, search `:422` | Existants |
| `source` | `MaintenanceView.vue:459,466,475` | Utilisé pour les badges uniquement — **pas de filtre** |
| Colonne `category` | `maintenance_log` (`0001_init.sql:625-639` : title, description, intervention_date, cost, vendor_name, contract_id, task_id, source) | **N'existe pas** |

Écart : (a) colonne `category` + backfill ; (b) filtre UI par source ; (c) filtre UI par catégorie.

## 2. Décision — filtrage côté client, colonne côté base

- La **catégorie** doit exister en base (donnée métier persistée, saisie/éditable par le syndic) → migration.
- Le **filtrage** reste **côté client** : la liste carnet est déjà chargée intégralement et la volumétrie d'une copropriété (quelques dizaines à centaines de lignes) le permet largement. Des query params backend seraient de la flexibilité non demandée. **Plafond noté :** si un immeuble dépasse ~1 000 entrées, basculer vers des query params + pagination (même pattern que EP-007-S09) — hors périmètre.

## 3. Modèle de données

Migration `backend/migrations/NNNN_maintenance_log_category.sql` — nouvelle colonne + enum (pas de nouvelle table ; mettre à jour la description de `maintenance_log` dans `docs/ARCHITECTURE.md`) :

```sql
-- EP-009-S10: logbook category (PRD US-CE-005).
CREATE TYPE maintenance_category AS ENUM ('travaux', 'contrat', 'sinistre', 'autre')

ALTER TABLE maintenance_log
    ADD COLUMN category maintenance_category NOT NULL DEFAULT 'autre'

-- Backfill heuristic: linked contract => 'contrat', linked task => 'travaux', else 'autre'.
UPDATE maintenance_log SET category = 'contrat' WHERE contract_id IS NOT NULL
UPDATE maintenance_log SET category = 'travaux' WHERE task_id IS NOT NULL AND contract_id IS NULL
```

Note : `CREATE TYPE` + usage immédiat sont permis dans la même transaction (la contrainte PostgreSQL ne vise que `ALTER TYPE … ADD VALUE`). Une seule migration suffit. Puis `cargo sqlx migrate run` + `cargo sqlx prepare`.

Valeur `sinistre` : aucun backfill automatique (pas de FK claim sur `maintenance_log`) — posée à la main ou par les flux futurs.

## 4. Backend

- `backend/src/db/maintenance_log.rs` : ajouter `category` à la row struct, aux `SELECT`/`RETURNING`, à l'input create/update (`category::text` / `$n::text::maintenance_category`). L'insert automatique à la clôture de tâche (`:159-187`, appelé depuis `db/task.rs:443`) pose `category = 'travaux'` explicitement.
- Validation : valeur hors enum → l'erreur de cast SQL ne doit pas fuiter ; valider en amont avec `AppError::validation_code("errors.maintenance.invalid_category")`.
- **Aucune nouvelle route, aucun query param** (décision §2). Handlers inchangés hors passage du champ.

## 5. Frontend

- `frontend/src/types/domain.ts` : `type MaintenanceCategory = 'travaux' | 'contrat' | 'sinistre' | 'autre'` ; champ `category` sur l'entrée carnet.
- Formulaire d'entrée carnet (create/edit) : select catégorie (défaut `autre`).
- `MaintenanceView.vue` :
  - **Filtre catégorie** : select (ou `CTabs` secondaires) sur les 4 valeurs + « Toutes »
  - **Filtre source** : select sur les valeurs de `source` existantes (déjà connues pour les badges `:459,466,475`) + « Toutes »
  - Les deux filtres se **combinent** (ET) avec la recherche `:422` et les onglets période `:417` existants — même mécanique computed côté client
  - Badge catégorie sur chaque ligne (couleurs tokens, pas de valeurs brutes).
- Aucun nouveau composant catalogue attendu (selects `C*` existants).

## 6. Cas limites

| Cas | Règle |
|---|---|
| Entrées pré-migration | Backfillées par l'heuristique (`contract_id` → contrat, `task_id` → travaux, sinon autre) |
| Ligne liée à la fois à un contrat et une tâche | Heuristique : `contrat` gagne (ordre des UPDATE du backfill) — arbitraire mais déterministe |
| Filtres combinés sans résultat | État vide explicite (« Aucune entrée ne correspond aux filtres ») |
| Catégorie invalide au POST | `400 errors.maintenance.invalid_category` |
| Entrées créées automatiquement (clôture tâche) | `category = 'travaux'` posée par le flux |
| Volumétrie > ~1 000 entrées | Hors périmètre — bascule server-side documentée §2 |

## 7. Critères d'acceptation

1. `maintenance_log.category` existe, backfillée, documentée dans `docs/ARCHITECTURE.md`.
2. Création/édition d'une entrée permet de choisir la catégorie ; la clôture de tâche pose `travaux`.
3. La vue carnet filtre par catégorie **et** par source, cumulables avec recherche et période existantes.
4. Aucune entrée existante ne se retrouve sans catégorie.

## 8. Plan de tests

- **e2e** (`backend/tests/api_e2e_test.rs`) :
  1. POST entrée avec `category = 'contrat'` → restituée au GET.
  2. POST sans catégorie → `autre`.
  3. Catégorie invalide → `400`.
  4. Clôture d'une tâche → l'entrée carnet générée a `category == "travaux"`.
- **Migration** : vérifier le backfill sur données seedées (contract_id / task_id / ni l'un ni l'autre).
- **Frontend** : vérification manuelle des filtres combinés + état vide.

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` (colonne + enum) puis migration `NNNN` → `sqlx migrate run` + `sqlx prepare`.
2. DB : row/inputs + `category='travaux'` sur l'insert auto.
3. Validation API + i18n.
4. Tests e2e.
5. Frontend : types, formulaire, filtres catégorie + source, badges.

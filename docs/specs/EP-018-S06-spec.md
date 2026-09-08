# Spécification technique — EP-018-S06

**Story :** `EP-018-S06` — Appliquer le Pattern B sur `budget` / `budget_line`
**Réf. :** ADR-009 §Pattern B · ADR-012 · ADR-006 · LEGAL.md §11 · PRD Budget et charges (US-BC-009)
**Statut :** implémenté (backend + frontend)

---

## 1. Quoi (résumé fonctionnel)

Un budget prévisionnel voté en assemblée générale est un acte juridique : il ne
peut plus être modifié. Une correction en cours d'exercice se fait par un
**budget rectificatif**, lui-même voté en AG, qui remplace le précédent sans
l'effacer.

Concrètement :

- le budget approuvé reste consultable et **en vigueur** tant que le rectificatif
  n'est pas voté ;
- ses lignes sont recopiées dans le rectificatif, puis datées à l'approbation de
  celui-ci ;
- l'historique complet des exercices successifs reste interrogeable.

## 2. Modèle de données

### 2.1 `budget`

| Colonne | Type | Rôle |
|---|---|---|
| `parent_budget_id` | `UUID REFERENCES budget(id)` nullable | budget modifié par ce rectificatif ; `NULL` sur un budget racine |

L'ancienne contrainte de table `UNIQUE (building_id, fiscal_year)` interdisait
tout second budget sur un exercice : elle est remplacée par trois index partiels.

| Index | Prédicat | Invariant |
|---|---|---|
| `budget_unique_root_per_year` | `WHERE parent_budget_id IS NULL` | un seul budget racine par exercice |
| `budget_unique_pending_rectificatif` | `WHERE parent_budget_id IS NOT NULL AND status = 'draft'` | un seul rectificatif en préparation par budget modifié (AC-BC-009-C) |
| `budget_unique_approved_per_year` | `WHERE status = 'approved'` | un seul budget en vigueur par exercice (AC-BC-009-C) |

### 2.2 `budget_line`

| Colonne | Type | Rôle |
|---|---|---|
| `valid_from` | `DATE NOT NULL DEFAULT CURRENT_DATE` | date de prise d'effet de la version |
| `valid_to` | `DATE` nullable | borne **exclusive** ; `NULL` = version en vigueur |
| `rectificatif_version` | `INTEGER` nullable | numéro du rectificatif d'origine ; `NULL` sur un budget racine |
| `version` | `INTEGER NOT NULL DEFAULT 1` | jeton de verrouillage optimiste (ADR-012) |

| Contrainte | Rôle |
|---|---|
| `budget_line_unique_active (budget_id, key_id, label) WHERE valid_to IS NULL` | au plus une version active par ligne logique |
| `budget_line_valid_period` | `valid_to IS NULL OR valid_to >= valid_from` |
| `budget_line_rectificatif_version_positive` | `rectificatif_version IS NULL OR >= 1` |

L'identité logique d'une ligne à travers ses versions est `(budget_id, key_id,
label)`, comme `unit_allocation_key` (ADR-006).

### 2.3 Migrations

- `0038_budget_rectificatif_schema` — colonnes, backfill `valid_from`, trigger de
  verrouillage optimiste.
- `0039_budget_rectificatif_constraints` — index partiels et `CHECK`.

Le `down` de `0039` échoue explicitement si un rectificatif existe : restaurer
`UNIQUE (building_id, fiscal_year)` serait alors impossible, et un échec bruyant
vaut mieux qu'un schéma à demi rétabli.

## 3. Backend

### 3.1 Garde d'immuabilité

`domain/immutability.rs` gagne `Entity::Budget`, mutable au seul statut `draft`.

Deux gardes distinctes, volontairement séparées :

| Garde | Refuse | Appelée par |
|---|---|---|
| `ensure_budget_lines_mutable` | tout statut ≠ `draft` | création / édition / suppression de ligne |
| `ensure_budget_is_mutable` | `closed` | génération d'appels, fonds travaux, clôture |

Les actions du second groupe portent **par nature** sur un budget approuvé : les
fusionner gèlerait la génération des appels de fonds.

### 3.2 Workflow

```
draft ──POST /budgets/{id}/approve──▶ approved ──POST /budgets/{id}/rectificatif──▶ (nouveau) draft
                                        │                                              │
                                        └──────── closed ◀── approve du rectificatif ───┘
```

**`POST /budgets/{id}/rectificatif`** — aucun corps de requête.

1. verrouille le budget modifié (`FOR UPDATE`) et exige `status = 'approved'` ;
2. calcule `rectificatif_version = MAX(rectificatif_version des lignes du parent) + 1` ;
3. insère un budget `draft` portant `parent_budget_id` ;
4. recopie les lignes **actives** du parent (`valid_to IS NULL`) avec
   `valid_from = CURRENT_DATE` ;
5. journalise l'opération dans `audit_log`.

Le budget modifié n'est **pas** touché : ni son statut, ni ses lignes. Il reste
l'acte en vigueur jusqu'au vote du rectificatif (AC-BC-009-B).

`actual_amount` n'est pas recopié : un montant réalisé appartient à l'exercice, pas
au prévisionnel nouvellement voté, et le recopier le compterait deux fois dans les
totaux de la chaîne.

**`POST /budgets/{id}/approve`** — corps `{ approval_meeting_id }`.

1. vérifie que l'AG existe, appartient à l'immeuble et est au statut `closed` ;
2. exige `status = 'draft'` sur le budget cible ;
3. s'il s'agit d'un rectificatif : passe le parent à `closed` **puis** date ses
   lignes (`valid_to = CURRENT_DATE`) ;
4. seulement ensuite promeut le budget à `approved`.

L'ordre des étapes 3 et 4 est contraint : `budget_unique_approved_per_year` est un
index non différé, les deux budgets ne doivent donc jamais être `approved` au même
instant de la transaction.

**`GET /budgets/{id}/rectificatifs`** — retourne la chaîne complète, racine
d'abord, via un CTE récursif remontant puis redescendant `parent_budget_id`.

### 3.3 Contrat d'API

| Méthode | Route | Corps | Réponse |
|---|---|---|---|
| `POST` | `/budgets/{id}/approve` | `{ approval_meeting_id }` | `200` `BudgetRow` |
| `POST` | `/budgets/{id}/rectificatif` | — | `201` `BudgetRow` |
| `GET` | `/budgets/{id}/rectificatifs` | — | `200` `BudgetRow[]` |
| `POST` | `/budgets/{id}/lines` | `{ label, key_id, budgeted_amount, account_code? }` | `201` `BudgetLineRow` |
| `PUT` | `/budgets/{id}/lines/{lid}` | idem + `version` | `200` `BudgetLineRow` |

`rectificatif_version` et `parent_budget_id` sont **absents** des payloads
d'entrée : ce sont des métadonnées de chaîne, assignées uniquement par
`create_rectificatif`. Les accepter depuis le client permettrait de forger un
rectificatif et de contourner le Pattern B.

### 3.4 Intégrité

Les invariants d'unicité sont portés **exclusivement** par les index partiels. Un
pré-contrôle applicatif `SELECT COUNT(*)` suivi d'un `INSERT` est une course
check-then-act que deux requêtes concurrentes franchissent toutes les deux ; les
noms de contraintes sont donc mappés vers des codes i18n dans
`map_budget_constraint` (`db/budget.rs`), suivant le motif déjà employé pour
`user_account_email_unique`.

`UPDATE` et `DELETE` sur `budget_line` portent `valid_to IS NULL` dans leur clause
`WHERE` : une version historique clôturée n'est jamais réécrite ni supprimée. Les
deux opérations écrivent une entrée `audit_log` avec un diff ciblé.

### 3.5 Totaux

`total_forecast` et `total_actual` (dans `list`, `get`, `get_by_fiscal_year` et
`rectificatif_chain`) filtrent `valid_to IS NULL`. Sans ce filtre, un exercice
rectifié afficherait un total non nul au-dessus d'un tableau de lignes vide.

## 4. Frontend

| Fichier | Changement |
|---|---|
| `types/domain.ts` | `Budget.parent_budget_id?` ; `BudgetLine.valid_from` (requis), `valid_to?`, `rectificatif_version?`, `version` (requis) |
| `api/index.ts` | `budgetApi.approve` / `createRectificatif` / `rectificatifChain` / `updateLine` |
| `BudgetView.vue` | `canEditBudgetLines` (draft seul) gèle « Nouvelle ligne » et la suppression ; actions « Approuver en AG » et « Créer un rectificatif » ; onglets numérotés par rang dans l'exercice |
| `ApproveBudgetModal.vue` | sélection de l'AG (statut `closed` uniquement) + avertissement art. 24 (AC-BC-009-D) |
| `CreateRectificatifModal.vue` | confirmation + avertissement vote AG + rappel du recalcul du seuil fonds travaux (AC-BC-009-E) |

`canEditSelectedBudget` (≠ `closed`) est conservé pour les actions d'exercice
(clôture, fonds travaux, appels) : les geler sur `approved` casserait la
génération des appels de fonds.

Le seuil légal du fonds travaux (art. 14-2-1) se recalcule sans code
supplémentaire : les `computed` existants dérivent du budget sélectionné, qui
devient le rectificatif dès sa création.

## 5. Plan de tests

`backend/tests/budget_rectificatif_test.rs` :

| Test | Invariant |
|---|---|
| `approve_promotes_draft_and_records_the_meeting` | transition nominale |
| `approve_rejects_a_meeting_that_has_not_been_held` | AG non clôturée refusée |
| `approve_rejects_an_unknown_meeting` | AG inexistante refusée |
| `approved_budget_lines_cannot_be_created_updated_or_deleted` | gel Pattern B |
| `rectificatif_requires_an_approved_parent` | pas de rectificatif sur brouillon |
| `creating_a_rectificatif_leaves_the_amended_budget_in_force` | **AC-BC-009-B** ; couvre aussi la recopie de 2 lignes de même nature |
| `only_one_rectificatif_may_be_in_preparation` | AC-BC-009-C |
| `approving_a_rectificatif_closes_the_amended_budget_and_dates_out_its_lines` | clôture + intervalle non inversé le jour même |
| `rectificatif_numbering_increments_along_the_chain` | numérotation + ordre de la chaîne |
| `update_line_bumps_the_version_and_rejects_a_stale_one` | ADR-012 |
| `duplicate_active_line_is_rejected` | `budget_line_unique_active` |
| `a_closed_historical_line_cannot_be_updated_or_deleted` | non-réécriture de l'historique |
| `a_budget_of_another_building_is_not_reachable` | cloisonnement par immeuble |

## 6. Points ouverts

- **AC-BC-009-F (`resolution_id`)** — le lien vers la résolution précise qui a voté
  le budget n'est pas modélisé ; seul `approval_meeting_id` l'est, désormais
  validé. À trancher avec `EP-003` (résolutions) avant de figer une colonne.
- **`generate_levies`** appelle `ensure_budget_is_mutable`, qui autorise le statut
  `draft`. Générer des appels de fonds sur un budget non voté est probablement
  incorrect au regard de l'art. 14-1, mais le restreindre est un changement de
  comportement hors périmètre de cette story.

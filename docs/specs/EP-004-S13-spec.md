# Spécification technique — EP-004-S13

**Story :** Annexes comptables légales I / II / III du décret du 14 mars 2005 (pris pour l'application du décret de 1967) : état financier, compte de gestion général, compte de gestion fonds travaux.

**Épic :** EP-004 — Budget & comptabilité
**Réf. LEGAL :** §4.2 · **Réf. PRD :** US-BC-012

> Statut : NON IMPLÉMENTÉ. Cible : annexes **structurées générées côté serveur (JSON)** + **vue imprimable** côté client. **PDF hors périmètre** — l'impression passe par le print navigateur.

---

## 1. État actuel du code

| Élément | Emplacement |
|---|---|
| Codes comptables (18, arrêté 14/03/2005) + `budget_line.account_code` | migration `0032_accounting_codes.sql`, `backend/src/db/accounting_code.rs` |
| Lignes budgétaires (budgété/réalisé, versionnées `valid_from`/`valid_to`) | `backend/src/db/budget.rs` |
| Appels de fonds (part charges / part fonds travaux, statuts dont `overdue`) | `levy_call` (`0001_init.sql:413-422`), `db/budget.rs:1088` |
| **Absent** | Tout endpoint/vue « annexe » ; soldes bancaires ; écritures de trésorerie |

---

## 2. Quoi (résumé fonctionnel)

Trois annexes par exercice, servies en JSON structuré et rendues dans une vue imprimable :

| Annexe | Contenu réglementaire | Sources disponibles | Lacunes (saisie manuelle) |
|---|---|---|---|
| **I — État financier** | Trésorerie, créances (copropriétaires débiteurs), dettes, situation fonds travaux | Créances = appels émis non payés (par lot) ; fonds travaux = cumul `works_fund_amount` appelé/payé | **Soldes bancaires** (compte courant, compte fonds travaux) : hors périmètre applicatif → champs de saisie manuelle ; dettes fournisseurs : non suivies → saisie manuelle ou N/A |
| **II — Compte de gestion général** | Charges/produits de l'exercice, budgété vs réalisé, par code comptable | `budget_line` (budgeted/actual) groupées par `account_code` | Charges hors budget non suivies ligne à ligne → couvertes par les lignes existantes uniquement |
| **III — Compte de gestion fonds travaux** | Cotisations appelées, encaissées, dépenses imputées, solde | `levy_call.works_fund_amount` (appelé, payé) | **Dépenses fonds travaux** : pas de rattachement dépense→fonds aujourd'hui → saisie manuelle du total dépensé |

Principe : **tout ce qui est calculable l'est côté serveur** ; les données hors périmètre (soldes bancaires, dépenses fonds travaux) sont des **champs manuels persistés** pour que l'annexe imprimée soit complète.

---

## 3. Modèle de données

Une petite table pour les valeurs manuelles (mise à jour `docs/ARCHITECTURE.md` avant migration ; pas d'ADR — pas d'alternative structurelle).

Migration `backend/migrations/NNNN_annex_manual_values.sql` (numéro à caler — renumérotation rectificatif en cours) :

```sql
-- EP-004-S13: manual inputs for legal accounting annexes (décret 14/03/2005) — values the app cannot derive (bank balances, works-fund spend)
CREATE TABLE annex_manual_value (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    budget_id UUID NOT NULL REFERENCES budget(id) ON DELETE CASCADE,
    key TEXT NOT NULL,            -- 'bank_balance_current' | 'bank_balance_works_fund' | 'supplier_debts' | 'works_fund_spent'
    amount NUMERIC(12,2) NOT NULL,
    updated_by UUID NOT NULL REFERENCES app_user(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (budget_id, key)
);
```

Clés contraintes côté domain (enum Rust), pas côté SQL — liste amenée à évoluer.

---

## 4. Backend

### 4.1 Domain — `backend/src/domain/annexes.rs` (nouveau)

Structures de sortie typées (tout en `Decimal`) + assemblage pur à partir des agrégats DB :

```rust
pub(crate) struct AnnexI { /* receivables_by_unit, works_fund_position, manual: bank balances / supplier debts */ }
pub(crate) struct AnnexII { /* rows by account_code: code, label, budgeted, actual, variance; totals */ }
pub(crate) struct AnnexIII { /* called, collected, spent (manual), balance */ }
```

- `build_annex_i/ii/iii(...)` : fonctions pures prenant les agrégats et les valeurs manuelles.
- Enum `AnnexManualKey` avec parsing strict → `AppError::validation_code("errors.annexes.unknown_key")`.
- Chaque annexe porte un bloc `generated_at` + `fiscal_year` + drapeaux `manual_values_missing: Vec<AnnexManualKey>` (l'UI signale ce qui reste à saisir).

### 4.2 DB

- `backend/src/db/annexes.rs` (nouveau) : agrégats SQL —
  - créances par lot (appels émis non payés, réutilise la logique `overdue`) ;
  - lignes actives groupées par `account_code` (budgété/réalisé) ;
  - fonds travaux : `SUM(works_fund_amount)` appelé et payé sur l'exercice ;
  - CRUD minimal `annex_manual_value` (`upsert`, `list_by_budget`) — upsert audité (`db::audit::insert`, même tx).

### 4.3 API — `backend/src/api/annexes.rs` (nouveau) + routes `api/mod.rs`

| Route | Rôle | Effet |
|---|---|---|
| `GET /budgets/{id}/annexes` | manager, board | Les 3 annexes JSON (I, II, III) en un appel. |
| `PUT /budgets/{id}/annexes/manual-values` | manager | Upsert des valeurs manuelles (map clé→montant), audité. |

Génération à la volée (pas de snapshot persisté au MVP) : les annexes reflètent l'état courant ; l'arrêté officiel est le document imprimé/archivé par le syndic. `404` hors immeuble ; handlers minces.

---

## 5. Frontend

- Types `AnnexI/II/III`, `annexesApi` dans `frontend/src/api/index.ts`.
- Nouvelle vue `frontend/src/features/budget/AnnexesView.vue` (accessible depuis `BudgetView.vue`) : onglets Annexe I / II / III, tableaux réglementaires, bandeau d'avertissement listant les valeurs manuelles manquantes, formulaire de saisie des valeurs manuelles (manager).
- **Impression** : bouton « Imprimer » → `window.print()` avec styles `@media print` via utilitaires Tailwind (`print:`) — pas de `<style>` block, pas de PDF.
- Nouveaux composants → catalogue `docs/DESIGN.md §2` avant usage. Montants formatés fr-FR à l'affichage.

---

## 6. Cas limites

| Cas | Règle |
|---|---|
| Valeurs manuelles absentes | Annexes générées quand même, champs à `null` + `manual_values_missing` renseigné ; l'impression affiche « à compléter ». |
| Lignes sans `account_code` | Regroupées sous une rubrique « non codé » en Annexe II (visibles, pas perdues). |
| Budget rectificatif | Annexes calculées sur les lignes **actives** de l'exercice (`valid_to IS NULL`) — cohérent avec `list_lines`. |
| Exercice sans fonds travaux | Annexe III rendue avec des zéros (pas d'erreur). |
| Montant manuel négatif | Autorisé pour `bank_balance_*` (découvert), refusé pour `works_fund_spent` (`errors.annexes.negative_amount`). |

---

## 7. Critères d'acceptation

1. `GET /budgets/{id}/annexes` renvoie les 3 annexes structurées, totaux `Decimal` exacts, chaque montant calculable traçable à sa source (ligne/appel).
2. Les valeurs manuelles se saisissent, persistent, s'auditent, et apparaissent dans les annexes.
3. La vue imprimable produit un document complet lisible via le print navigateur.
4. Les lacunes de données (soldes bancaires, dépenses fonds travaux) sont explicitement signalées, jamais silencieusement à zéro.
5. Clippy/tests/build/lint verts.

---

## 8. Plan de tests

- **Unitaires domaine** : `build_annex_ii` (groupement par code, totaux, rubrique « non codé »), `build_annex_iii` (solde = appelé − dépensé manuel), parsing `AnnexManualKey`, `manual_values_missing`.
- **E2E** : budget peuplé (lignes codées + appels + paiements partiels) → GET annexes : créances correctes, totaux II corrects, III cohérent ; PUT valeurs manuelles → reflétées au GET suivant + ligne audit ; clé inconnue → 400 ; montant négatif interdit → 400.

---

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` (table `annex_manual_value`).
2. Migration `NNNN` → `sqlx migrate run` + `sqlx prepare`.
3. Domain `annexes.rs` + tests unitaires.
4. DB (agrégats + manual values + audit).
5. API + routes + i18n.
6. Tests e2e.
7. Frontend (`AnnexesView.vue`, print, catalogue DESIGN.md).
8. `graphify update .` + `docs/PROGRESS.md`.

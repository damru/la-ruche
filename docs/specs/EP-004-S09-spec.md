# Spécification technique — EP-004-S09

**Story :** Plan comptable réglementaire (arrêté du 14 mars 2005) + exports comptables CSV.

**Épic :** EP-004 — Budget & comptabilité
**Réf. LEGAL :** §4.2 · **Réf. PRD :** US budget/comptabilité

> Statut : PARTIEL. Le **mapping plan comptable est FAIT** ; il ne reste que les **exports** (aucun handler CSV/export dans le repo).

---

## 1. État actuel du code

### Fait (ne pas retoucher)

| Élément | Emplacement |
|---|---|
| 18 codes comptables (arrêté 14/03/2005) + FK `budget_line.account_code` | `backend/migrations/0032_accounting_codes.sql` |
| Repo | `backend/src/db/accounting_code.rs` |
| `GET /accounting-codes` | route `backend/src/api/mod.rs:208`, handler `backend/src/api/budget.rs:616` |
| Sélection du code à la création de ligne | `frontend/src/features/budget/CreateLineModal.vue` |

### Absent (périmètre de cette spec)

- Aucun endpoint d'export (zéro occurrence `csv`/`export` côté handlers).
- Aucune UI d'export.

---

## 2. Quoi (résumé fonctionnel)

Deux exports CSV téléchargeables, par exercice budgétaire :

1. **Lignes budgétaires** : `GET /budgets/{id}/export/lines` — colonnes : code comptable, libellé du code, libellé de ligne, clé de répartition, montant budgété, montant réalisé. Lignes actives uniquement (`valid_to IS NULL`, cohérent avec `list_lines`).
2. **Appels de fonds** : `GET /budgets/{id}/export/levy-calls` — colonnes : lot, copropriétaire, libellé d'appel, échéance, part charges, part fonds travaux, total, statut.

Accès : **manager + board_member** (lecture comptable, cohérent avec les autres lectures budget).

### Décisions de format (fr-FR)

- **Séparateur `;`** (convention Excel fr-FR — la virgule décimale interdit le séparateur `,`).
- **Décimales avec virgule** : `1234,56` (montants `Decimal`, jamais `f64` ; formatage à la sérialisation, 2 décimales).
- **Dates `JJ/MM/AAAA`**.
- **Encodage UTF-8 avec BOM** (`\u{FEFF}` en tête) pour l'ouverture directe dans Excel.
- Échappement CSV standard (guillemets doublés, champ quoté si `;`, `"` ou saut de ligne).
- `Content-Type: text/csv; charset=utf-8`, `Content-Disposition: attachment; filename="budget-{fiscal_year}-lignes.csv"` (resp. `-appels.csv`).

---

## 3. Modèle de données

Aucun. Pas de migration, pas de changement `docs/ARCHITECTURE.md`.

---

## 4. Backend

### 4.1 Domain — `backend/src/domain/csv_export.rs` (nouveau)

Logique pure de formatage, sans SQL ni HTTP :

```rust
pub(crate) fn fr_decimal(d: Decimal) -> String;          // 1234,56
pub(crate) fn fr_date(d: NaiveDate) -> String;           // 26/08/2026
pub(crate) fn csv_escape(field: &str) -> String;         // RFC 4180, séparateur ';'
pub(crate) fn to_csv(headers: &[&str], rows: Vec<Vec<String>>) -> String; // BOM + lignes
```

Pas de crate `csv` supplémentaire : le besoin (écriture seule, format fixe) tient en ~40 lignes testées.

### 4.2 DB — `backend/src/db/budget.rs`

Réutiliser au maximum l'existant (`list_lines` renvoie déjà les lignes actives avec `account_code`). Ajouter si nécessaire :

- une jointure sur `accounting_code` pour le libellé du code (ou lookup en mémoire via `db::accounting_code::list`) ;
- `list_levy_calls_for_export(pool, budget_id, building_id)` : appels de l'exercice joints aux lots (`unit`) et copropriétaires — s'appuyer sur les requêtes existantes autour de `db/budget.rs:1088` (statut `overdue` inclus tel quel dans la colonne statut).

### 4.3 API — `backend/src/api/budget.rs` + `backend/src/api/mod.rs`

Deux handlers minces (aucune logique : fetch db → mapping domain → réponse) :

```rust
GET /budgets/{id}/export/lines        → manager | board_member
GET /budgets/{id}/export/levy-calls   → manager | board_member
```

Retour `axum::response::Response` avec les en-têtes CSV ci-dessus. `404` si budget inexistant / autre immeuble. Statuts d'appel exportés avec le libellé technique anglais (`issued`, `paid`, `overdue`, …) — la traduction est un souci d'affichage, pas d'export ; à confirmer en implémentation si l'utilisateur préfère les libellés français (décision par défaut : technique, stable pour un tableur).

---

## 5. Frontend

- `frontend/src/api/index.ts` : `budgetApi.exportLinesUrl(budgetId)` / `exportLevyCallsUrl(budgetId)` — l'export est un simple lien de téléchargement (GET authentifié) ; suivre le pattern existant du client pour les téléchargements (blob + `URL.createObjectURL` si l'auth passe par header).
- `frontend/src/features/budget/BudgetView.vue` : bouton « Exporter » (menu à 2 entrées : « Lignes budgétaires (CSV) », « Appels de fonds (CSV) »), visible pour manager et board_member.
- Clés i18n : `budget.export.lines`, `budget.export.levyCalls`.

---

## 6. Cas limites

| Cas | Règle |
|---|---|
| Budget sans lignes / sans appels | CSV valide avec en-têtes seuls (pas d'erreur). |
| `account_code` NULL sur une ligne ancienne | Colonne code vide, ligne exportée quand même. |
| Libellés contenant `;`, `"` ou sauts de ligne | Échappement RFC 4180. |
| Montants négatifs (avoirs) | Exportés tels quels (`-123,45`). |
| Budget rectificatif | Export du budget demandé uniquement (ses lignes actives) — pas de fusion de chaîne parent/rectificatif. |

---

## 7. Critères d'acceptation

1. Les deux endpoints renvoient un CSV `;`-séparé, BOM UTF-8, décimales à virgule, dates `JJ/MM/AAAA`, ouvrable dans Excel fr-FR sans manipulation.
2. Accès manager et board_member ; 403 pour un owner simple ; 404 hors immeuble.
3. Les montants proviennent de `Decimal` sans passage par un flottant.
4. Bouton d'export fonctionnel dans `BudgetView.vue`.
5. `cargo clippy -- -D warnings`, tests verts, build + lint frontend verts.

---

## 8. Plan de tests

- **Unitaires domaine** (`domain/csv_export.rs`) : `fr_decimal` (arrondi 2 déc., négatif), `fr_date`, `csv_escape` (`;`, `"`, `\n`), BOM en tête de `to_csv`.
- **E2E** : export lignes d'un budget peuplé → statut 200, `Content-Type` correct, contenu attendu (spot-check une ligne) ; export appels → colonnes lot/statut correctes ; budget vide → en-têtes seuls ; owner → 403.

---

## 9. Ordre d'implémentation

1. `domain/csv_export.rs` + tests unitaires → `cargo test --lib`.
2. DB : requête export appels (jointures lots/propriétaires).
3. API : 2 handlers + routes → `cargo build` + clippy.
4. Tests e2e.
5. Frontend : api client + bouton export + i18n.
6. `graphify update .` + entrée `docs/PROGRESS.md`.

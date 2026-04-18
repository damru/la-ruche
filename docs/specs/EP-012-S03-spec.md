# Spécification technique — EP-012-S03

**Story :** Générer un dossier de déclaration annuelle RNC prêt à dépôt : contrôles de complétude + export + journal horodaté.

**Réfs :** Épic EP-012 (`docs/EPICS.md` §EP-012), LEGAL.md §1.1, PRD `docs/product/PRD - Conformité et reporting.md` (US-CR-006, hors scope §9 : pas de télétransmission).

> S'appuie intégralement sur EP-012-S01 (`EP-012-S01-spec.md`) : module `domain/rnc.rs`, table `rnc_export` (colonne `checks` déjà prévue), routes `/rnc/exports`. **Prérequis : S01 livré.** Pas de nouvelle table.

## 1. Quoi (résumé fonctionnel)

Avant le dépôt annuel sur registre.coproprietes.gouv.fr, le Syndic demande un « dossier de dépôt » pour un exercice : le backend exécute des contrôles de complétude sur le dataset RNC, retourne un rapport (bloquants / avertissements), et — si aucun bloquant — permet l'export (S01) en attachant le rapport au snapshot. La liste des exports horodatés (`exported_at`, `exported_by`, `checks`) constitue le journal probant de mise à jour annuelle.

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| Dataset + mapping RNC | `backend/src/domain/rnc.rs` (S01 — `RncDataset`, `build_dataset`) |
| Snapshot + journal | table `rnc_export` (S01, colonne `checks JSONB` réservée) ; `backend/src/db/rnc_export.rs` (`insert`, `list`) |
| Handlers / routes | `backend/src/api/rnc.rs` ; `backend/src/api/mod.rs` (routes `/rnc/exports` posées en S01, y compris `get(rnc::list)`) |
| Frontend | `frontend/src/features/compliance/RncExportView.vue` (S01) |

## 3. Modèle de données

Aucune migration : `rnc_export.checks JSONB` (créée en S01) reçoit le rapport de complétude sérialisé. Structure :

```json
{ "passed": true, "blocking": [], "warnings": ["charges_amount.missing_previous_year"] }
```

## 4. Backend Rust

### 4.1 `backend/src/domain/rnc.rs` — contrôles de complétude

```rust
pub struct CompletenessReport {
    pub passed: bool,             // no blocking issue
    pub blocking: Vec<&'static str>
    pub warnings: Vec<&'static str>
}
pub fn check_completeness(dataset: &RncDataset) -> CompletenessReport
```

Règles (fonction pure, testable) :
- **Bloquants** : `registration_number` absent ; `name`/adresse incomplète ; `total_units == 0` ; `budget_amount` absent ; `works_fund_balance` absent.
- **Avertissements** : `manager_siret` NULL ; `charges_amount` NULL (pas d'exercice clos) ; `vendor_debt`/`employee_count` non renseignés ; `bylaws_date` NULL.
Les codes sont des clés i18n stables (`rnc.checks.registration_number_missing`, etc.) traduites côté frontend.

### 4.2 `backend/src/api/rnc.rs` — évolutions

- `GET /rnc/dossier?fiscal_year=YYYY` (nouveau) — `require_role(Manager)`. Orchestration : `collect_sources` + `manual_fields` du **dernier export** de l'exercice (pré-remplissage) → `build_dataset` → `check_completeness` → retourne `{ dataset, report }` sans rien persister (dry-run).
- `POST /rnc/exports` (S01) — enrichi : exécute `check_completeness` ; si `blocking` non vide → 422 `AppError::validation_code("errors.rnc.incomplete")` avec le rapport en détail ; sinon persiste le snapshot avec `checks` rempli. Un paramètre `force: bool` n'est **pas** prévu (le dépôt légal exige un dossier complet).
- `GET /rnc/exports` (route posée en S01) — implémentation : journal ordonné `exported_at DESC`, chaque ligne avec `fiscal_year`, `format`, `exported_by` (nom résolu), `exported_at`, `checks`.

### 4.3 `backend/src/db/rnc_export.rs`

- `latest_for_year(pool, building_id, fiscal_year) -> AppResult<Option<RncExportRow>>` (récupération des `manual_fields` précédents).
- `insert` (S01) étendu pour porter `checks`.

## 5. Frontend Vue

- `RncExportView.vue` (S01) devient l'écran « dossier de dépôt » :
  - étape 1 — bouton « Vérifier le dossier {année} » → `rncApi.dossier(year)` : affichage du rapport (liste bloquants en rouge, avertissements en ambre, clés i18n).
  - étape 2 — export activé seulement si `report.passed` ; les champs manuels sont pré-remplis depuis le dernier export.
  - étape 3 — tableau « Journal des dépôts » : date, exercice, format, auteur, état des contrôles, lien de téléchargement (`/rnc/exports/{id}/file`).
- `frontend/src/api/index.ts` : ajouter `dossier: (year) => api.get('/rnc/dossier?fiscal_year=' + year)` à `rncApi`.
- i18n : traductions des clés `rnc.checks.*`.

## 6. Cas limites

- Aucun export antérieur → `manual_fields` vides, tous les champs ❌ remontent en bloquant/avertissement selon §4.1.
- Contrôles OK au dry-run mais données changées avant `POST /exports` → les contrôles sont ré-exécutés au POST (le dry-run ne fait pas foi).
- Journal jamais purgé : `rnc_export` est append-only (aucune route DELETE) — valeur probante.
- Deux exports valides le même jour : autorisé, deux lignes de journal.

## 7. Critères d'acceptation

- **AC-S03-A** : le dry-run retourne un rapport complet sans créer de ligne `rnc_export`.
- **AC-S03-B** : un export avec bloquant est refusé (422 + rapport) ; sans bloquant il est persisté avec `checks`.
- **AC-S03-C** : le journal restitue tous les exports horodatés avec auteur et rapport, ordre antichronologique.
- **AC-S03-D** : les `manual_fields` du dernier export pré-remplissent le dossier suivant.

## 8. Plan de tests

- Unitaires `domain/rnc.rs` : `check_completeness` — chaque règle bloquante et chaque avertissement, cas tout-vert.
- e2e HTTP : dry-run sans persistance ; POST refusé avec bloquant ; POST accepté avec warnings ; journal ordonné ; pré-remplissage manuel depuis export précédent.
- Frontend : rendu rapport (bloquant vs warning), export désactivé si `!passed`, journal.

## 9. Ordre d'implémentation

1. `check_completeness` + tests unitaires.
2. `db::rnc_export::latest_for_year` + extension `insert`.
3. `api/rnc.rs` : `dossier` (dry-run), durcissement `create`, implémentation `list` + tests e2e.
4. Frontend (workflow 3 étapes + i18n).
5. Entrée.

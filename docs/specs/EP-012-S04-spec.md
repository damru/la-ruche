# Spécification technique — EP-012-S04

**Story :** Permettre au Syndic de vérifier et corriger le brouillon de fiche synthétique auto-rempli avant publication.

**Réfs :** Épic EP-012 (`docs/EPICS.md` §EP-012), LEGAL.md §1.3, PRD `docs/product/PRD - Conformité et reporting.md` US-CR-002 (AC-CR-002-A/B), matrice CRUD §3 (`U` Syndic, statut `draft` uniquement).

> S'appuie sur EP-012-S02 (`EP-012-S02-spec.md`) : table `fiche_synthetique`, `FichePayload`, `domain::fiche_synthetique::can_edit`. **Prérequis : S02 livré.** Pas de nouvelle table — une colonne ajoutée.

## 1. Quoi (résumé fonctionnel)

Le Syndic ouvre le brouillon généré (S02), modifie n'importe quel champ du payload dans un formulaire pré-rempli, et enregistre. Toute modification manuelle est tracée (AC-CR-002-B) : la fiche conserve la liste des champs écrasés par rapport au payload auto-généré. Une fiche non-`draft` est immuable (403/422, jamais de modification silencieuse).

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| Table + payload | `fiche_synthetique` (S02), `backend/src/domain/fiche_synthetique.rs` (`FichePayload`, `can_edit`) |
| Repo | `backend/src/db/fiche_synthetique.rs` (`get`, `create_draft`) |
| Handlers / routes | `backend/src/api/fiche_synthetique.rs` ; `backend/src/api/mod.rs` |
| Précédent guard d'édition draft-only | `backend/src/domain/immutability.rs` + `api/contracts.rs::update` (EP-018-S05) |
| Vue liste | `frontend/src/features/compliance/FicheSynthetiqueView.vue` (S02) |

## 3. Modèle de données

### 3.1 Migration `backend/migrations/NNNN_fiche_manual_overrides.sql`

```sql
-- EP-012-S04: trace manual corrections on draft fiches (AC-CR-002-B)
ALTER TABLE fiche_synthetique
    ADD COLUMN manual_overrides JSONB NOT NULL DEFAULT '[]'::jsonb;
    -- array of { "field": "financial.closed_year_charges",
    --            "previous": <json>, "at": timestamptz, "by": uuid }
```

Colonne sur table existante (pas de nouvelle table → pas d'entrée ARCHITECTURE.md obligatoire, mais mettre à jour la description de `fiche_synthetique` si elle y figure déjà). `cargo sqlx migrate run` + `cargo sqlx prepare`.

## 4. Backend Rust

### 4.1 `backend/src/domain/fiche_synthetique.rs`

- `diff_payload(old: &FichePayload, new: &FichePayload) -> Vec<OverrideEntry>` — fonction pure : compare champ à champ (chemins pointés `section.field`), retourne les entrées d'audit. Testable sans DB.
- Réutiliser `can_edit(status)` (S02) comme unique porte d'entrée de la règle « draft only ».

### 4.2 `backend/src/db/fiche_synthetique.rs`

- `update_payload(pool, id, building_id, payload, new_overrides) -> AppResult<Option<FicheRow>>` — UPDATE `payload`, append des overrides (`manual_overrides = manual_overrides || $new`), `updated_at = NOW()`, clause `WHERE status = 'draft'` en défense (le guard métier reste la première ligne).

### 4.3 `backend/src/api/fiche_synthetique.rs`

```rust
// PUT /fiche-synthetique/{id}
pub async fn update(...) -> AppResult<Json<FicheResponse>> {
    auth.require_role(&Role::Manager)?;
    let current = db::fiche_synthetique::get(...).ok_or(AppError::NotFound)?;
    if !domain::fiche_synthetique::can_edit(&current.status) {
        return Err(AppError::validation_code("errors.fiche.not_editable"));
    }
    let overrides = domain::fiche_synthetique::diff_payload(&current.payload, &input.payload)
        .stamped(auth.user_id); // fills at/by
    // zero business logic beyond orchestration
    ...
}
```

Validation du payload entrant : désérialisation stricte `FichePayload` (`deny_unknown_fields`) → 422 automatique sur champ inconnu ; montants invalides → 422 `errors.fiche.invalid_amount`.

### 4.4 `backend/src/api/mod.rs`

```rust
.route("/fiche-synthetique/{id}", get(fiche_synthetique::get).put(fiche_synthetique::update))
```

## 5. Frontend Vue

- Nouveau `frontend/src/features/compliance/FicheEditForm.vue` : formulaire par sections (identification, administration, composition, financier, assurance, sinistres), pré-rempli depuis `payload`, champs numériques en saisie décimale (string, pas de float).
- Indicateur visuel « corrigé manuellement » sur chaque champ présent dans `manual_overrides` (tooltip : valeur d'origine, date, auteur).
- Bouton « Réinitialiser depuis les modules » optionnel **hors scope** (serait une régénération — S02).
- `ficheApi.update(id, payload)` dans `frontend/src/api/index.ts` ; type `FicheOverride` dans `types/domain.ts`.
- Accessible depuis `FicheSynthetiqueView.vue` uniquement pour une fiche `draft`.

## 6. Cas limites

- PUT sur fiche `published`/`archived` → 422 `errors.fiche.not_editable` (AC matrice CRUD ¹).
- PUT sans aucun changement → 200, aucun override ajouté (diff vide).
- Deux éditions concurrentes : dernière écriture gagne ; les overrides des deux passes sont conservés (append) — pas de verrou optimiste pour le MVP, à signaler si jugé insuffisant.
- Champ remis à sa valeur auto-générée : l'override historique n'est pas effacé (journal, pas état).

## 7. Critères d'acceptation

- **AC-CR-002-A** : tous les champs du payload sont modifiables tant que `status = draft`.
- **AC-CR-002-B** : chaque modification manuelle est tracée (champ, valeur précédente, date, auteur) et restituée par l'API.
- **AC-S04-C** : fiche non-draft → 422 i18n, payload inchangé.
- **AC-S04-D** : rôle non-Manager → 403.

## 8. Plan de tests

- Unitaires : `diff_payload` (aucun changement, changement simple, champs imbriqués, sections nulles).
- e2e HTTP : édition nominale avec overrides vérifiés ; 422 sur `published` ; 422 payload inconnu (`deny_unknown_fields`) ; 403 `co_owner` ; overrides cumulés sur deux PUT.
- Frontend : pré-remplissage, marquage des champs corrigés, soumission.

## 9. Ordre d'implémentation

1. Migration `NNNN` (colonne `manual_overrides`) → `sqlx migrate run` + `sqlx prepare`.
2. `diff_payload` + tests unitaires.
3. `db::update_payload`.
4. `api::update` + route + tests e2e.
5. Frontend (`FicheEditForm.vue`, `ficheApi.update`, marquage overrides) + `docs/DESIGN.md` si nouveau composant catalogue.
6. `graphify update .` + entrée `docs/PROGRESS.md` (EP-012-S04).

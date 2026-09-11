# Spécification technique — EP-001-S09

**Story :** Contrôle de cohérence EDD : comparer les tantièmes configurés avec le dernier EDD déposé et produire un rapport d'écarts (non bloquant) (LEGAL.md §1.2)

**Épic :** EP-001 — Configuration copropriété & clés de répartition
**Réf. LEGAL :** §1.2 · **Réf. PRD :** US-RC-* (référentiel copropriété) · **ADR :** aucun requis (table de saisie simple + endpoint de lecture)
**Dépendance conceptuelle :** EP-005-S06 (typage des documents fondateurs, dont `EDD`) — non bloquante, voir §2.

**Statut initial :** NON IMPLÉMENTÉ — aucun concept `edd` dans le code.

---

## 1. Quoi (résumé fonctionnel)

L'état descriptif de division (EDD) est l'acte notarié de référence des tantièmes. Le parsing d'un PDF notarial est **hors scope** : la story repose sur une **saisie manuelle de valeurs de référence EDD** (par lot et par clé), puis un **rapport d'écarts non bloquant** entre ces références et les tantièmes configurés dans l'application.

- Le syndic saisit une « référence EDD » : libellé, date de l'acte, et les tantièmes de référence (lot × clé → valeur).
- Un endpoint de contrôle compare la dernière référence EDD avec les `share_value` actifs et retourne les écarts (lot manquant, valeur divergente, total divergent).
- Le rapport est purement informatif : il ne bloque ni la configuration ni la génération d'appels (contrairement au contrôle d'équilibre existant EP-001-S07).

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| Contrôle d'équilibre des clés (surface la plus proche) | `backend/src/api/building.rs:332` (`shares_balance`) |
| Erreur existante sur dépassement de total | `errors.allocation_keys.shares_exceed_total` — `backend/src/api/building.rs:368` |
| Repo clés / tantièmes | `backend/src/db/allocation_key.rs` (`create` :65, `update` :84, `set_share`) |
| Typage documents fondateurs (`EDD`) | **inexistant** — EP-005-S06 non livrée. La colonne `document_id` reste nullable et sera raccordée quand EP-005-S06 existera. |

## 3. Modèle de données

Migration `backend/migrations/NNNN_edd_reference.sql` (numérotation en cours de flux sur cette branche) :

```sql
-- EP-001-S09: manual EDD reference totals for coherence check (LEGAL.md §1.2)
CREATE TABLE edd_reference (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    building_id UUID NOT NULL REFERENCES building(id),
    label TEXT NOT NULL,
    edd_date DATE NOT NULL,
    document_id UUID NULL, -- FK to document(id) once EP-005-S06 lands (EDD typing)
    created_by UUID NOT NULL REFERENCES app_user(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE edd_reference_share (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    edd_reference_id UUID NOT NULL REFERENCES edd_reference(id) ON DELETE CASCADE,
    unit_id UUID NOT NULL REFERENCES unit(id),
    allocation_key_id UUID NOT NULL REFERENCES allocation_key(id),
    share_value NUMERIC(12,2) NOT NULL,
    UNIQUE (edd_reference_id, unit_id, allocation_key_id)
);
```

Étape associée : **mettre à jour `docs/ARCHITECTURE.md`** (nouvelles tables) avant la migration. Pas d'ADR (pas d'alternative structurelle réelle : saisie manuelle + comparaison en lecture).

## 4. Backend

### 4.1 Domain (`backend/src/domain/edd_check.rs`, nouveau)

Fonction pure `compare(reference: &[EddRefShare], current: &[CurrentShare]) -> EddReport` :

- `Decimal` partout (jamais `f64`).
- Types d'écart : `MissingInEdd` (lot/clé configuré absent de la référence), `MissingInConfig` (référencé mais non configuré), `ValueMismatch { expected, actual }`, `TotalMismatch` par clé.
- Zéro SQL ici ; zéro logique dans les handlers.

### 4.2 DB (`backend/src/db/edd_reference.rs`, nouveau)

`create` (référence + shares en une transaction), `get_latest(building_id)` (max `edd_date`), `list(building_id)`, `list_shares(edd_reference_id)`.

### 4.3 API (`backend/src/api/building.rs` ou module dédié `api/edd.rs`)

Routes à déclarer dans `backend/src/api/mod.rs` :

- `POST /buildings/{id}/edd-references` — création (manager uniquement). Validation : au moins une share, valeurs ≥ 0, sinon `AppError::validation_code("errors.edd.shares_required")` / `"errors.edd.invalid_share_value"`.
- `GET /buildings/{id}/edd-references` — liste.
- `GET /buildings/{id}/edd-check` — assemble la dernière référence + tantièmes actifs (via `db/allocation_key.rs`), délègue à `domain::edd_check::compare`, retourne le rapport JSON. Si aucune référence : `404` avec `AppError::validation_code("errors.edd.no_reference")`.

## 5. Frontend

- `frontend/src/api/index.ts` : `eddApi` (`createReference`, `listReferences`, `check`).
- `frontend/src/types/domain.ts` : `EddReference`, `EddDiscrepancy`, `EddReport`.
- Vue : section « Contrôle EDD » dans la configuration de l'immeuble (feature `building`) — formulaire de saisie de la référence (grille lot × clé) + affichage du rapport d'écarts (badge par type d'écart, bannière « non bloquant »).
- Composants catalogue existants (`C*`) uniquement ; **si un nouveau composant `C*` est nécessaire, l'ajouter au catalogue `docs/DESIGN.md §3` avant usage** (étape dédiée, ne pas improviser).

## 6. Cas limites

- Aucune référence EDD saisie → `edd-check` retourne 404 explicite, l'UI affiche un empty state incitant à la saisie.
- Lot archivé présent dans la référence → signalé `MissingInConfig` avec mention « lot archivé », pas d'erreur.
- Plusieurs références même `edd_date` → départage par `created_at` le plus récent.
- Clé supprimée depuis la saisie → l'écart est reporté, jamais d'erreur 500.
- Le rapport ne modifie rien : endpoint strictement en lecture, aucune interaction avec les guards `domain/immutability.rs`.

## 7. Critères d'acceptation

1. Le syndic peut saisir une référence EDD (libellé, date, tantièmes par lot/clé).
2. `GET /buildings/{id}/edd-check` retourne les écarts valeur/valeur, lots manquants dans les deux sens, et les écarts de total par clé.
3. Le rapport est non bloquant : aucune mutation existante (clés, appels) n'est affectée.
4. Erreurs i18n (`errors.edd.*`), jamais de chaînes brutes.
5. `docs/ARCHITECTURE.md` documente les deux tables.

## 8. Plan de tests

- **Domaine pur** (`domain/edd_check.rs`) : cas identique → rapport vide ; valeur divergente ; lot manquant côté EDD ; lot manquant côté config ; total divergent ; précision `Decimal`.
- **E2E** (`backend/tests/api_e2e_test.rs`, harnais `common::isolated_test_server()`) :
  - création référence + check → écarts attendus ;
  - check sans référence → 404 `errors.edd.no_reference` ;
  - accès refusé pour un rôle non-manager sur `POST`.

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` (tables) → migration `NNNN_edd_reference.sql` → `cargo sqlx migrate run`.
2. `cargo sqlx prepare` (offline) après écriture des requêtes.
3. `domain/edd_check.rs` + tests purs.
4. `db/edd_reference.rs`.
5. Handlers + routes `api/mod.rs` + erreurs i18n.
6. Frontend (types, `eddApi`, section UI) — cataloguer tout nouveau `C*` dans DESIGN.md §3 au préalable.
7. Tests E2E, puis entrée `docs/PROGRESS.md`.

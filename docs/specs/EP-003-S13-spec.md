# Spécification technique — EP-003-S13

**Story :** CRUD complet des résolutions (création, modification, **suppression**, **réordonnancement**) avec qualification de majorité — **restreint au statut `draft`** de l'AG.

**Épic :** EP-003 — Assemblées générales
**Réf. LEGAL :** ordre du jour figé à la convocation (art. 9/11 décret 67-223) · **Réf. PRD :** US-AG-003 · **Réf. ADR :** aucun

**Statut actuel : IMPLÉMENTÉ (backend + frontend) — livré par la PR #129.**

L'Option A a été validée avec le product owner : l'ordre du jour est figé à la convocation. `Entity::Resolution` est restreint à la mutation structurelle en `draft` ; le chemin « patch vote » (description en séance) est porté par un guard distinct `Entity::ResolutionVote` (`ongoing`).

---

## 1. Ce qui existe (livré)

| Élément | Emplacement |
|---|---|
| Création / update / patch de résolution | `backend/src/api/assemblies.rs` |
| Suppression `DELETE /meetings/{id}/resolutions/{rid}` (draft-only, transaction votes + résolution) | `backend/src/api/assemblies.rs`, `backend/src/db/assembly.rs` |
| Réordonnancement `POST /meetings/{id}/resolutions/reorder` (permutation exacte, renumérotation 1..N en deux phases) | `backend/src/api/assemblies.rs`, `backend/src/db/assembly.rs` |
| Guard `Entity::Resolution => status == "draft"` + `Entity::ResolutionVote => status == "ongoing"` | `backend/src/domain/immutability.rs` |
| Qualification de majorité : `vote_type` + `parse_vote_rule` (solide) | `backend/src/api/assemblies.rs`, `backend/src/domain/vote.rs` |
| Table `resolution` — `number INTEGER` fourni par le client, `UNIQUE(meeting_id, number)` | `backend/migrations/0001_init.sql` L363-373 |
| Frontend : boutons Monter / Descendre / Supprimer (draft-only), `DeleteResolutionModal`, méthodes `deleteResolution` / `reorderResolutions` dans `meetingsApi` | `frontend/src/features/assemblies/AssemblyDetailView.vue`, `frontend/src/features/assemblies/DeleteResolutionModal.vue`, `frontend/src/api/index.ts` |
| Tests d'intégration (flux draft, refus post-convocation, non-régression ongoing) | `backend/tests/resolution_crud_test.rs` |

### Exception délibérée — passerelle art. 25-1 / 26-1 (EP-013-S10)

La création `POST /resolutions` reste ouverte en `ongoing` (filtre SQL `IN ('draft','ongoing')`) car les modales Art24/Art25 inscrivent une re-délibération pendant la séance. Refus en `called` → 409 `errors.meetings.meeting_not_mutable`.

## 3. Modèle de données

Aucune migration : `UNIQUE(meeting_id, number)` existe déjà. Le reorder compose avec cette contrainte par renumérotation en deux phases dans une transaction : tous les `number` passent d'abord en négatif (`number = -number`), puis les numéros finaux 1..N sont posés.

## 4. Backend

### 4.1 Domaine — `backend/src/domain/immutability.rs`

```rust
// Option A: agenda frozen at convocation — structural mutations draft-only.
Entity::Resolution => status == "draft",
// Vote-related patches allowed while the meeting is ongoing.
Entity::ResolutionVote => status == "ongoing",
```

Test `resolution_vote_mutable_only_in_ongoing` : `Resolution` mutable uniquement en `draft` ; `ResolutionVote` uniquement en `ongoing`.

### 4.2 DB — `backend/src/db/assembly.rs`

- `create_resolution` : filtre statut restreint — `IN ('draft','ongoing')` (exception passerelle art. 25-1/26-1, voir §1).
- `delete_resolution(pool, meeting_id, resolution_id, building_id)` : DELETE draft-only (filtre SQL sur le statut du meeting, pattern de `update`) ; suppression des votes rattachés dans la même transaction (en pratique inexistants en `draft`).
- `reorder_resolutions(pool, meeting_id, building_id, ordered_ids: Vec<Uuid>)` :
  - vérifier statut `draft` → 409 `errors.meetings.not_in_draft_status` hors draft ;
  - vérifier que `ordered_ids` est une permutation exacte des résolutions du meeting → 400 `errors.resolutions.reorder_mismatch` ;
  - renuméroter 1..N en transaction (négation puis assignation) pour respecter `UNIQUE(meeting_id, number)` ;
  - retourne la liste mise à jour (même forme que GET).
- `update_resolution` : inchangé sur `number` (le numéro ne change que via reorder).

### 4.3 API

- `backend/src/api/mod.rs` : `.delete(assemblies::delete_resolution)` sur la route résolution, et route dédiée :

```rust
.route("/meetings/{id}/resolutions/reorder", post(assemblies::reorder_resolutions))
```

- Handlers `delete_resolution` / `reorder_resolutions` : rôle `manager`, zéro logique, erreurs i18n `errors.meetings.not_in_draft_status`, `errors.resolutions.reorder_mismatch`.

## 5. Frontend

- `frontend/src/api/index.ts` : `deleteResolution(meetingId, resolutionId)`, `reorderResolutions(meetingId, orderedIds: string[])`.
- `AssemblyDetailView.vue` :
  - action « Supprimer » par résolution, visible en `draft` uniquement ;
  - réordonnancement par flèches monter/descendre (`common.moveUp` / `common.moveDown`) → envoi de la liste complète ordonnée, ordre local optimiste + `reload()` ;
  - masquer création/édition/suppression/reorder dès que le meeting quitte `draft` (aligné Option A).
- `DeleteResolutionModal.vue` : modale de confirmation (modélisée sur `DeleteDocumentModal`, CBtn variant `danger`), libellés dans `meetings.*`.
- Composants C* utilisés : `CModal`, `CFormActions`, `CBtn` (catalogue existant).

## 6. Cas limites

| Cas | Règle |
|---|---|
| Delete/reorder sur meeting `called`/`ongoing`/`closed` | Delete → 404 (pattern SQL-filter, pas de leak d'état) ; reorder → 409 `errors.meetings.not_in_draft_status`. |
| Reorder avec ids manquants/étrangers/dupliqués | 400 `errors.resolutions.reorder_mismatch`. |
| Reorder d'un meeting sans résolutions | 200 no-op (liste vide = permutation vide). |
| Patch vote en `ongoing` | Toujours autorisé (guard `ResolutionVote`). |
| Suppression d'une résolution votée | Impossible par construction (votes seulement en `ongoing`, suppression seulement en `draft`). |

## 7. Critères d'acceptation

1. `DELETE` d'une résolution en `draft` → 204 ; en `called`+ → refus.
2. `POST /reorder` renumérote 1..N selon l'ordre fourni, contrainte UNIQUE respectée.
3. Création/update structurel refusés hors `draft` (comportement Option A).
4. Les patch de vote restent fonctionnels en `ongoing`.
5. Test `immutability.rs` mis à jour et vert.

## 8. Plan de tests

Implémenté dans `backend/tests/resolution_crud_test.rs` (3 tests d'intégration) :

- **Flux draft** : delete 204 + reorder 1..N + mismatch 400.
- **Refus post-convocation** : delete 404, reorder 409, create 409.
- **Non-régression ongoing** : patch description 200 + passerelle art. 25-1/26-1 201 + delete 404.

Tests unitaires (`domain/immutability.rs`) : `Resolution` mutable uniquement en `draft` ; `ResolutionVote` uniquement en `ongoing`.

## 9. Ordre d'implémentation (réalisé)

1. Option A validée avec le product owner.
2. `domain/immutability.rs` : guard `Resolution` draft-only + `ResolutionVote` ongoing-only, tests mis à jour.
3. `db/assembly.rs` : `delete_resolution`, `reorder_resolutions`, restriction `create_resolution`.
4. `api/assemblies.rs` + routes `mod.rs`.
5. Tests e2e `backend/tests/resolution_crud_test.rs` — verts.
6. Frontend (api client, actions draft-only, reorder par flèches, `DeleteResolutionModal`).
7. Statut story mis à jour dans `docs/EPICS.md` (✅) et présente spec.

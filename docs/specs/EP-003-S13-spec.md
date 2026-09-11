# Spécification technique — EP-003-S13

**Story :** CRUD complet des résolutions (création, modification, **suppression**, **réordonnancement**) avec qualification de majorité — **restreint au statut `draft`** de l'AG.

**Épic :** EP-003 — Assemblées générales
**Réf. LEGAL :** ordre du jour figé à la convocation (art. 9/11 décret 67-223) · **Réf. PRD :** US-AG-003 · **Réf. ADR :** aucun

**Statut actuel : PARTIEL.**

---

## 1. Ce qui existe déjà (ne pas retoucher)

| Élément | Emplacement |
|---|---|
| Création / update / patch de résolution | `backend/src/api/assemblies.rs` L291-306, L307-332, L523-548 |
| Qualification de majorité : `vote_type` + `parse_vote_rule` (solide) | `backend/src/api/assemblies.rs` L614-624, `backend/src/domain/vote.rs` L16-24 |
| Table `resolution` — `number INTEGER` fourni par le client, `UNIQUE(meeting_id, number)` | `backend/migrations/0001_init.sql` L363-373 |

## 2. Écarts restants

- **(a)** Pas de route `DELETE /meetings/{mid}/resolutions/{rid}` (`api/mod.rs` L142-145 : put + patch seulement).
- **(b)** Pas de réordonnancement : `update_resolution` (`db/assembly.rs` L1233-1244) ne touche pas `number`, aucun endpoint de reorder.
- **(c)** Le guard autorise la mutation en `draft`/`called`/`ongoing` (`domain/immutability.rs` L43 : `Entity::Resolution => status != "closed"` ; `db/assembly.rs` L1188-1200 : create autorisé en draft/called/ongoing) alors que la story (PRD US-AG-003) dit **draft-only**.

### ⚠️ Décision de spec à trancher — périmètre du guard (écart (c))

Conflit assumé entre la story et le comportement actuel **intentionnel** (le test `immutability.rs` L92-96 affirme la mutabilité jusqu'à `closed`).

- **Option A (story stricte)** : create/update/delete/reorder des résolutions uniquement en `draft`.
- **Option B (comportement actuel)** : amendements possibles jusqu'à `closed`.

**Recommandation : Option A**, alignée sur le droit — l'ordre du jour est figé à la convocation ; on ne vote que sur les questions inscrites. Nuance : les **patch liés au vote** (résultats, statut de vote pendant la séance, route patch L523-548) restent autorisés en `ongoing`. Concrètement : restreindre `Entity::Resolution` à la mutation *structurelle* (`status == "draft"`), et faire porter le chemin « patch vote » sur un guard distinct (`ongoing`). **À valider avec Damien avant implémentation** ; le test `immutability.rs` L92-96 sera mis à jour en conséquence.

## 3. Modèle de données

Aucune migration : `UNIQUE(meeting_id, number)` existe déjà. Le reorder doit composer avec cette contrainte (voir §4.2 — mise à jour en deux temps ou contrainte différée via `SET CONSTRAINTS` non disponible sur UNIQUE simple → renumérotation par transaction avec numéros temporaires négatifs).

## 4. Backend

### 4.1 Domaine — `backend/src/domain/immutability.rs`

```rust
// Option A: agenda frozen at convocation — structural mutations draft-only.
Entity::Resolution => status == "draft",
// New: vote-related patches allowed while the meeting is ongoing.
Entity::ResolutionVote => status == "ongoing",
```

Mettre à jour le test L92-96 (qui affirme aujourd'hui la mutabilité en called/ongoing).

### 4.2 DB — `backend/src/db/assembly.rs`

- `create_resolution` (L1188-1200) : restreindre le filtre statut à `'draft'` seul.
- `delete_resolution(pool, meeting_id, resolution_id, building_id)` : DELETE draft-only (filtre SQL sur le statut du meeting, pattern de `update` L244-252) ; supprimer les votes rattachés dans la même transaction (en pratique inexistants en `draft`).
- `reorder_resolutions(pool, meeting_id, building_id, ordered_ids: Vec<Uuid>)` :
  - vérifier statut `draft` (`errors.meetings.not_draft`) ;
  - vérifier que `ordered_ids` est une permutation exacte des résolutions du meeting (`errors.resolutions.reorder_mismatch`) ;
  - renuméroter 1..N en transaction — passer d'abord tous les `number` en négatif (`number = -number`) pour éviter les collisions `UNIQUE(meeting_id, number)`, puis poser les numéros finaux.
- `update_resolution` (L1233-1244) : inchangé sur `number` (le numéro ne change que via reorder).

### 4.3 API

- `backend/src/api/mod.rs` (bloc L142-145) : ajouter `.delete(assemblies::delete_resolution)` sur la route résolution, et une route dédiée :

```rust
.route("/meetings/{id}/resolutions/reorder", post(assemblies::reorder_resolutions))
```

- Handlers `delete_resolution` / `reorder_resolutions` : rôle `manager`, zéro logique, erreurs i18n `errors.meetings.not_draft`, `errors.resolutions.reorder_mismatch`.

## 5. Frontend

- `frontend/src/api/index.ts` : `deleteResolution(meetingId, resolutionId)`, `reorderResolutions(meetingId, orderedIds: string[])`.
- `AssemblyDetailView.vue` (ou vue résolutions) :
  - action « Supprimer » par résolution, visible en `draft` uniquement ;
  - réordonnancement par flèches monter/descendre (minimal, pas de drag-and-drop) → envoi de la liste complète ordonnée ;
  - masquer création/édition/suppression/reorder dès que le meeting quitte `draft` (aligné Option A).
- Aucun nouveau composant C* attendu ; sinon cataloguer dans `docs/DESIGN.md §3` avant usage.

## 6. Cas limites

| Cas | Règle |
|---|---|
| Delete/reorder/create/update sur meeting `called`/`ongoing`/`closed` | 400 `errors.meetings.not_draft` (ou 404 selon pattern SQL retenu — suivre le pattern `update` existant : 404). |
| Reorder avec ids manquants/étrangers/dupliqués | 400 `errors.resolutions.reorder_mismatch`. |
| Reorder d'un meeting sans résolutions | 200 no-op (liste vide = permutation vide). |
| Patch vote en `ongoing` | Toujours autorisé (guard `ResolutionVote`). |
| Suppression d'une résolution votée | Impossible par construction (votes seulement en `ongoing`, suppression seulement en `draft`). |

## 7. Critères d'acceptation

1. `DELETE` d'une résolution en `draft` → 204 ; en `called`+ → refus.
2. `POST /reorder` renumérote 1..N selon l'ordre fourni, contrainte UNIQUE respectée.
3. Création/update structurel refusés hors `draft` (nouveau comportement Option A, après validation).
4. Les patch de vote restent fonctionnels en `ongoing`.
5. Test `immutability.rs` L92-96 mis à jour et vert.

## 8. Plan de tests

- **Unitaires** (`domain/immutability.rs`) : `Resolution` mutable uniquement en `draft` ; `ResolutionVote` uniquement en `ongoing`.
- **E2E** :
  1. delete en draft → 204 ; delete après convocation → refus ;
  2. reorder valide → GET renvoie les numéros 1..N réordonnés ;
  3. reorder permutation invalide → 400 ;
  4. create resolution en `called` → refus (nouveau) ;
  5. patch vote en `ongoing` → 200 (non-régression).

## 9. Ordre d'implémentation

1. **Valider l'Option A avec Damien** (écart (c)) — bloquant.
2. `domain/immutability.rs` : nouveau guard + mise à jour test L92-96 → `cargo test --lib`.
3. `db/assembly.rs` : restriction `create_resolution`, `delete_resolution`, `reorder_resolutions`.
4. `api/assemblies.rs` + routes `mod.rs` → `cargo clippy -- -D warnings`.
5. Tests e2e, boucler jusqu'au vert.
6. Frontend (api client, actions draft-only, reorder par flèches).
7. `docs/PROGRESS.md`.

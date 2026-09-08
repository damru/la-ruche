# Spécification technique — EP-003-S12

**Story :** Modifier et **supprimer** un brouillon d'AG (statut `draft` uniquement).

**Épic :** EP-003 — Assemblées générales
**Réf. LEGAL :** — (un brouillon n'a pas d'existence légale) · **Réf. PRD :** US-AG-* (gestion des brouillons) · **Réf. ADR :** aucun

**Statut actuel : PARTIEL** — la modification existe et est correcte ; **la suppression n'existe pas**. Cette spec ne couvre que la suppression.

---

## 1. Ce qui existe déjà (ne pas retoucher)

| Élément | Emplacement |
|---|---|
| `update` draft-only en SQL (`AND status='draft'`) | `backend/src/db/assembly.rs` L244-252 |
| 404 si non-draft / inexistant | `backend/src/api/assemblies.rs` L243 |
| Bouton « Modifier » affiché uniquement en `draft` | `frontend/src/features/assemblies/AssemblyDetailView.vue` L133-144, L522-534 |
| Routes `/meetings/{id}` : `get` + `put` seulement | `backend/src/api/mod.rs` L116-118 |

## 2. Écart restant

Aucune route `DELETE /meetings/{id}`, aucun handler, aucune action frontend.

**Décision de spec — cascade vs blocage :** un brouillon n'a jamais été convoqué, ses données rattachées (checklist de convocation, feuilles de présence éventuelles, résolutions) n'ont aucune valeur probante. **Cascade retenue** : la suppression d'un meeting `draft` supprime ses lignes filles dans la même transaction. Le blocage « si non vide » créerait une friction inutile (obliger à vider les résolutions une à une avant suppression) sans bénéfice légal.

## 3. Modèle de données

Aucune migration nécessaire si les FK filles sont déjà `ON DELETE CASCADE` dans `0001_init.sql`. Sinon, deux options :
- **Option retenue :** DELETE explicites en transaction dans le repo (pas de migration, comportement visible dans le code) ;
- alternative : migration `NNNN` ajoutant `ON DELETE CASCADE` — à ne prendre que si d'autres stories en bénéficient.

## 4. Backend

### 4.1 Domaine

`backend/src/domain/immutability.rs` couvre déjà la notion via `is_deletable`/statuts ; si `Entity::Meeting` (ou équivalent) n'exprime pas la suppression, la règle reste locale au repo : **suppression uniquement si `status = 'draft'`**.

### 4.2 DB — `backend/src/db/assembly.rs`

```rust
/// Deletes a draft meeting and its children (checklist, attendance,
/// resolutions + votes) atomically. Returns false if the meeting does not
/// exist, belongs to another building, or is not in 'draft'.
pub async fn delete_meeting(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<bool> {
    let mut tx = pool.begin().await?;
    // SELECT ... FOR UPDATE WHERE id=$1 AND building_id=$2 AND status='draft'
    // -> None => Ok(false)
    // DELETE children (votes -> resolutions -> attendance -> checklist items) then the meeting.
    tx.commit().await?;
    Ok(true)
}
```

Le filtre `AND status = 'draft'` en SQL suit le même pattern que `update` (L244-252) : un meeting non-draft se comporte comme introuvable → 404, cohérent avec `api/assemblies.rs` L243.

### 4.3 API

- Handler `delete` dans `backend/src/api/assemblies.rs` : rôle `manager`, appelle `delete_meeting`, `false` → `AppError::NotFound`, succès → `204 No Content`. Zéro logique dans le handler.
- Route dans `backend/src/api/mod.rs` (bloc meetings L116-118) :

```rust
.route("/meetings/{id}", get(assemblies::get).put(assemblies::update).delete(assemblies::delete))
```

## 5. Frontend

- `frontend/src/api/index.ts` : `deleteMeeting: (id: string) => api.delete(`/meetings/${id}`)`.
- `AssemblyDetailView.vue` : action « Supprimer le brouillon » visible uniquement en `draft` (même garde que le bouton « Modifier », L133-144), avec confirmation (modale de confirmation existante du catalogue si disponible, sinon cataloguer dans `docs/DESIGN.md §3` avant usage). Succès → toast + redirection vers la liste des AG.

## 6. Cas limites

| Cas | Règle |
|---|---|
| DELETE sur un meeting `called`/`ongoing`/`closed` | 404 (même sémantique que `update`). |
| DELETE sur un meeting d'un autre immeuble | 404 (filtre `building_id`). |
| Brouillon avec résolutions/checklist/présences | Supprimées en cascade dans la transaction. |
| Double DELETE | Second appel → 404 (idempotent côté client). |
| Rôle insuffisant | 403. |

## 7. Critères d'acceptation

1. `DELETE /meetings/{id}` sur un brouillon → 204, meeting et données filles absents de la base.
2. `DELETE` sur un meeting non-draft → 404, aucune donnée modifiée.
3. Le bouton « Supprimer » n'apparaît qu'en statut `draft` dans le frontend.
4. La suppression est atomique (échec partiel impossible).

## 8. Plan de tests

- **E2E** (`backend/tests/api_e2e_test.rs`) :
  1. créer un brouillon avec résolutions + items de checklist → DELETE → 204 → GET → 404, résolutions absentes ;
  2. convoquer le meeting (`called`) → DELETE → 404 ;
  3. DELETE cross-building → 404 ;
  4. DELETE sans rôle manager → 403.
- **Frontend** : si vitest configuré pour `assemblies/`, test d'affichage conditionnel du bouton ; sinon couverture backend seule.

## 9. Ordre d'implémentation

1. `db/assembly.rs` : `delete_meeting` (transaction, cascade explicite) → `cargo build`.
2. `api/assemblies.rs` : handler `delete` + route dans `mod.rs` → `cargo clippy -- -D warnings`.
3. Tests e2e, boucler jusqu'au vert.
4. Frontend : api client + action dans `AssemblyDetailView.vue`.
5. `docs/PROGRESS.md` + `graphify update .`.

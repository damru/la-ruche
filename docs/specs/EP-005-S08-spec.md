# Spécification technique — EP-005-S08

**Story :** Suppression de document en soft-delete avec conservation des métadonnées ; workflow de remplacement avec justification horodatée pour les documents critiques.

**Épic :** EP-005 — Documents
**Réf. PRD :** US-DOC-004 (suppression), US-DOC-007 (remplacement justifié) · **Réf. LEGAL :** §6 (conservation)
**Stories liées :** **EP-005-S06/S07 (prérequis — `category`, `supersedes_id`, `retention.rs`)**, EP-005-S09 (la purge physique différée est déléguée au job S09)

---

## 1. Quoi (résumé fonctionnel)

Aujourd'hui la suppression est **physique partout** : `DELETE FROM document` + destruction S3 immédiate (`db/document.rs` L172-206, `api/documents.rs` L206-224), et la suppression d'un parent (contrat, tâche, sinistre, rapport) détruit en cascade les documents orphelins (`delete_orphaned_tx`, `db/document.rs` L208-253, appelé depuis `db/task.rs:381`, `db/claim.rs:475`, `db/report.rs:320`, `db/contract.rs:612`). C'est incompatible avec les durées de conservation légales.

Cible :
- **Soft-delete** : `deleted_at`/`deleted_by`/`deletion_reason` sur `document` (précédent : `allocation_key.deleted_at`, `db/allocation_key.rs` L131-132). Toutes les lectures filtrent `deleted_at IS NULL`.
- **L'objet S3 n'est plus détruit à la suppression** : il reste en place tant que la rétention de la catégorie (S07, `domain/retention.rs`) l'exige ; la purge physique différée est le rôle du job S09.
- `delete_orphaned_tx` et le DELETE utilisateur deviennent des soft-deletes — **la cascade contrat (api/contracts.rs L137-145 → db/contract.rs L597/L612) cesse de détruire des documents**.
- **Remplacement justifié** pour les catégories critiques (fondateurs + techniques S07) : nouveau document + justification obligatoire horodatée + chaînage `supersedes_id` (S06).

---

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| Hard delete + S3 delete | `backend/src/db/document.rs` L172-206 · `backend/src/api/documents.rs` L206-224 |
| `delete_orphaned_tx` (8 tables de liens) | `backend/src/db/document.rs` L208-253 |
| Appelants de la cascade | `db/task.rs:381`, `db/claim.rs:475`, `db/report.rs:320`, `db/contract.rs:612` (via `api/contracts.rs` L137-145) |
| Précédent soft-delete | `backend/src/db/allocation_key.rs` L131-132 (`deleted_at`) |
| Chaîne de versions (`create_version`) | S06 — `db/document.rs` |
| Règles de rétention | S07 — `backend/src/domain/retention.rs` |

---

## 3. Modèle de données

### Migration `backend/migrations/NNNN_document_soft_delete.sql`

```sql
-- EP-005-S08: soft-delete with metadata retention (PRD US-DOC-004/US-DOC-007).
-- The DB row and the S3 object are both kept after deletion; physical purge
-- is deferred to the S3 cleanup job (EP-005-S09) once the category's legal
-- retention (domain/retention.rs) has elapsed.

ALTER TABLE document
    ADD COLUMN deleted_at      TIMESTAMPTZ,
    ADD COLUMN deleted_by      UUID REFERENCES user_account(id),
    ADD COLUMN deletion_reason TEXT;

CREATE INDEX document_deleted_idx ON document (building_id) WHERE deleted_at IS NOT NULL;
```

`docs/ARCHITECTURE.md` avant migration ; `cargo sqlx prepare` après.

---

## 4. Backend Rust

### 4.1 `domain/` — règles

Dans `domain/immutability.rs` (à côté de `is_document_deletable` de S06) :

```rust
/// Critical categories require the justified-replacement workflow
/// (US-DOC-007): plain deletion is refused, replacement demands a reason.
pub(crate) fn document_requires_replacement(category: &str) -> bool {
    matches!(category, "reglement_copropriete" | "edd" | "dpe" | "dtg" | "ppt" | "pv_ag")
}
```

### 4.2 `db/document.rs`

- **Lectures** : `list`, `get`, `version_chain` (S06) → ajouter `AND d.deleted_at IS NULL`. Exception : `version_chain` peut inclure les versions soft-deleted avec leur `deleted_at` visible (historique probant) — retenu : **inclure**, marquées.
- **`delete` → soft-delete** : `UPDATE document SET deleted_at = NOW(), deleted_by = $user, deletion_reason = $reason WHERE id = ... AND deleted_at IS NULL`. Ne retourne plus `(bucket, storage_path)` — plus de destruction S3. Refus si `document_requires_replacement(category)` → `errors.documents.replacement_required` ; refus fondateurs inchangé (S06).
- **`delete_orphaned_tx` → `soft_delete_orphaned_tx`** : même détection d'orphelins (8 `NOT EXISTS`), mais `UPDATE ... SET deleted_at = NOW(), deletion_reason = 'parent_deleted'` au lieu de `DELETE`. Signature : ne retourne plus les clés S3 → adapter les 4 appelants (`db/task.rs:381`, `db/claim.rs:475`, `db/report.rs:320`, `db/contract.rs:612`) pour supprimer le `tokio::spawn` de destruction S3 associé.
- **Nouvelle fn `replace`** (transaction) : pour une catégorie critique — vérifie `deletion_reason`/justification non vide, appelle la logique `create_version` (S06 : nouveau doc `supersedes_id = ancien`, bascule `is_current`), et enregistre la justification sur l'**ancien** document (`deletion_reason = $justification` sans `deleted_at` — le document remplacé n'est pas supprimé, il est remplacé ; le champ est renommé conceptuellement « motif », voir §6). Alternative plus simple retenue : colonne dédiée `replacement_reason TEXT` sur le **nouveau** document ? Non — réutiliser `deletion_reason` sur l'ancien prête à confusion. **Décision : ajouter `replacement_reason TEXT` au nouveau document dans la même migration** et le remplir dans `replace`.

```sql
    ADD COLUMN replacement_reason TEXT   -- justification for US-DOC-007, set on the replacing document
```

### 4.3 `api/documents.rs` + `api/mod.rs`

- `DELETE /documents/{id}` : body optionnel `{ "reason": string }` ; supprime le `tokio::spawn` S3 (L212-219). Réponses : `204` ; `400 errors.documents.replacement_required` (catégorie critique) ; `400 errors.documents.founding_locked`.
- Nouvelle route `POST /documents/{id}/replace` (rôle manager) : body = payload de confirmation d'upload (clé S3 déjà uploadée, comme `confirm_upload`) + `reason` **obligatoire** (`errors.documents.reason_required` si vide). Réponse `201` = nouveau document, `replacement_reason` et horodatage `created_at` faisant foi.

---

## 5. Frontend Vue

1. Types : `deleted_at`, `replacement_reason` sur `Document`.
2. `frontend/src/api/index.ts` : `documentsApi.delete(id, reason?)`, `documentsApi.replace(id, payload)`.
3. Vue Documents : la suppression demande un motif optionnel (modale de confirmation existante enrichie) ; sur catégorie critique, l'action devient « Remplacer » et ouvre `CDocumentReplaceModal` (upload + champ justification obligatoire).
4. **Cataloguer `CDocumentReplaceModal` dans `docs/DESIGN.md` §3 avant usage** (étape, ne pas éditer DESIGN.md ici).
5. Historique de versions (S06) : afficher `replacement_reason` et les versions supprimées (badge « supprimé le … »).

---

## 6. Cas limites

| Cas | Règle |
|---|---|
| DELETE sur catégorie critique | `400 errors.documents.replacement_required` — passer par `/replace`. |
| DELETE sur fondateur | `400 errors.documents.founding_locked` (S06, inchangé — même le remplacement ne supprime pas). |
| Suppression d'un contrat avec documents orphelins | Documents soft-deleted, objets S3 conservés (résout le conflit rétention actuel de `api/contracts.rs` L137-145). |
| Double DELETE | 2ᵉ appel → `404` (le filtre `deleted_at IS NULL` fait disparaître la ligne). |
| Download d'un document soft-deleted | `404` — `get` filtre. |
| `/replace` sans `reason` | `400 errors.documents.reason_required`. |
| `/replace` sur document non courant | `400 errors.documents.not_current_version` (S06). |
| Purge physique | Jamais dans cette story — uniquement via le job S09, après rétention. |

---

## 7. Critères d'acceptation

1. Supprimer un document conserve la ligne (métadonnées + `deleted_at`/`deleted_by`/`deletion_reason`) et l'objet S3.
2. Aucun chemin de code ne fait plus de `DELETE FROM document` ni de destruction S3 synchrone (hors job S09).
3. Les listings, téléchargements et liens ignorent les documents supprimés.
4. Un document critique ne se supprime pas : il se remplace, avec justification horodatée et chaînage de versions.
5. La suppression d'un contrat/tâche/sinistre/rapport ne détruit plus aucun document.

---

## 8. Plan de tests

- **Unitaires** : `document_requires_replacement` par catégorie.
- **e2e** :
  1. DELETE document `autre` → `204` ; absent du listing ; ligne présente en base avec `deleted_at` (vérif SQL) ; download → `404`.
  2. DELETE document `dpe` → `400 replacement_required`.
  3. `POST /replace` sur `dpe` courant avec reason → `201`, chaîne S06 correcte, `replacement_reason` posé.
  4. `POST /replace` sans reason → `400`.
  5. Suppression d'un contrat avec document orphelin → contrat supprimé, document soft-deleted, `storage_path` toujours en base.
  6. Double DELETE → `404`.

---

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` → migration `NNNN` (3 colonnes + `replacement_reason` + index) → `sqlx migrate run` + `prepare`.
2. `domain/immutability.rs` (`document_requires_replacement` + tests).
3. `db/document.rs` : filtres lectures, soft `delete`, `soft_delete_orphaned_tx` + adaptation des 4 appelants, `replace`.
4. `api/documents.rs` : DELETE (reason, plus de S3), route `/replace` dans `api/mod.rs` → `clippy`.
5. Tests e2e → vert.
6. Frontend (types, api, `CDocumentReplaceModal` cataloguée puis créée, historique enrichi).
7. `docs/PROGRESS.md`, `graphify update .`.

# Spécification technique — EP-007-S10

**Story :** Documents de tâche côté gestionnaire — mode **append-only** après clôture : ajout toujours possible, suppression bloquée quand la tâche est close/annulée.

**Épic :** EP-007 — Travaux & prestataires
**Réf. PRD :** US-TP-005 · **Réf. ADR :** ADR-009 (données closes immuables)
**État :** PARTIEL — cette spec ne couvre que l'écart restant.

---

## 1. État actuel du code (écart)

| Élément | Emplacement | Constat |
|---|---|---|
| `attach_document` | `backend/src/api/tasks.rs:61-74` | Rôle `Manager\|BoardMember`, **aucun** appel à `assert_mutable` — l'ajout après clôture fonctionne *par omission* |
| `detach_document` | `backend/src/api/tasks.rs:76-88` | Idem — la suppression après clôture fonctionne alors qu'elle devrait être bloquée |
| Gel immutabilité | `backend/src/domain/immutability.rs:44` | `Entity::Task` gelée si `closed`/`cancelled` |

Écart : (a) bloquer `detach` sur tâche `closed`/`cancelled` ; (b) **entériner explicitement** l'ajout après clôture (décision append-only, plus un accident).

## 2. Décision de conception

- **Attach après clôture : autorisé, explicitement.** Une facture ou un PV de réception arrive souvent après la clôture — l'append-only préserve la piste probante sans rouvrir la tâche. Documenter la décision par un commentaire dans le handler pour que le prochain lecteur ne « corrige » pas l'absence de guard.
- **Detach après clôture : bloqué** (`errors.immutability.frozen`) — retirer une pièce d'un dossier clos casse la valeur probante (ADR-009).

## 3. Modèle de données

Aucune migration.

## 4. Backend

### 4.1 API — `backend/src/api/tasks.rs`

`detach_document` (`:76-88`) : charger la tâche puis appliquer le guard existant avant le detach — la vérification de statut passe par le helper domaine, pas par une comparaison de chaîne dans le handler :

```rust
let task = db::task::get(&state.pool, task_id, auth.building_id)
    .await?
    .ok_or(AppError::NotFound)?;
crate::domain::immutability::assert_mutable(&Entity::Task, &task.status)?; // 400 errors.immutability.frozen
```

`attach_document` (`:61-74`) : inchangé fonctionnellement ; ajouter le commentaire de décision :

```rust
// EP-007-S10 (PRD US-TP-005): attaching documents to a closed/cancelled task
// is intentionally allowed — the task document set is append-only after
// closure (late invoices, acceptance reports). Do NOT add assert_mutable here.
```

Si la vérification d'existence/statut est plus naturelle en couche db (éviter un aller-retour), déplacer le fetch dans `db/task.rs` — mais la règle reste exprimée via `assert_mutable`.

### 4.2 Routes / i18n

Aucune nouvelle route. Code d'erreur réutilisé : `errors.immutability.frozen` (existant).

## 5. Frontend

- Vue détail tâche (composant gérant les documents) : si `status ∈ {closed, cancelled}`, masquer/désactiver le bouton de suppression de document, conserver le bouton d'ajout — avec un libellé du type « Documents en ajout seul après clôture ».
- Gérer proprement le `400 errors.immutability.frozen` (toast i18n) au cas où l'état local serait périmé.
- Aucun nouveau composant catalogue.

## 6. Cas limites

| Cas | Règle |
|---|---|
| Attach sur tâche `closed` | `201` — append-only voulu |
| Attach sur tâche `cancelled` | `201` — même règle (une tâche annulée peut recevoir une pièce justificative tardive) |
| Detach sur tâche `closed`/`cancelled` | `400 errors.immutability.frozen` |
| Detach sur tâche active | Inchangé (autorisé, rôles actuels) |
| Document rattaché puis tâche close puis detach | Bloqué — la date d'attachement n'entre pas en compte, seul le statut courant |
| Rôles | Inchangés : `Manager\|BoardMember` sur les deux endpoints |

## 7. Critères d'acceptation

1. `POST /tasks/{id}/documents` réussit sur une tâche `closed` (et `cancelled`).
2. `DELETE /tasks/{id}/documents/{doc_id}` sur tâche `closed`/`cancelled` → `400 errors.immutability.frozen` ; le lien document subsiste.
3. Detach sur tâche active inchangé.
4. UI : bouton suppression absent/désactivé sur tâche close, bouton ajout présent.

## 8. Plan de tests (e2e — `backend/tests/api_e2e_test.rs`)

1. Tâche close → attach document → `201` ; le document apparaît dans la liste.
2. Tâche close avec document → detach → `400 errors.immutability.frozen` ; document toujours listé.
3. Tâche `cancelled` → mêmes deux assertions.
4. Tâche `in_progress` → attach puis detach → les deux réussissent (non-régression).
5. Rôle `co_owner` → `403` sur attach et detach (non-régression).

## 9. Ordre d'implémentation

1. Guard `assert_mutable` dans `detach_document` + commentaire de décision dans `attach_document`.
2. Tests e2e (§8), boucler au vert.
3. Frontend : conditionner le bouton de suppression au statut.
4. `docs/PROGRESS.md`.

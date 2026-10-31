# Spécification technique — EP-007-S08

**Story :** Supprimer / annuler une tâche — suppression **bloquée** si une entrée du carnet d'entretien ou une soumission prestataire est rattachée ; annulation non destructive comme alternative.

**Épic :** EP-007 — Travaux & prestataires
**Réf. PRD :** US-TP-016 · **Réf. ADR :** ADR-009 (piste probante) · **Réf. LEGAL :** conservation des actes
**État :** IMPLÉMENTÉ (backend + frontend)

> **AC-016-D (entrée timeline immuable avec motif) :** non livrée ici — la table `activity_log` n'existe pas encore ; cette entrée sera produite par EP-017-S02 (`task.status_changed` / événement d'annulation) quand l'epic EP-017 sera livré.

---

## 1. État actuel du code (écart)

| Élément | Emplacement | Constat |
|---|---|---|
| `delete_task` handler | `backend/src/api/tasks.rs:187-210` | Guard immutabilité seul (`closed`/`cancelled`) |
| `delete` DB | `backend/src/db/task.rs:326-384` | **Supprime** les `vendor_submission` rattachées (`:354-369`) au lieu de refuser |
| Check carnet | `backend/src/db/task.rs:171` (`is_in_logbook` calculé) | Utilisé seulement comme avertissement UI (`DeleteTaskModal.vue:39`, `:60-63`) — la suppression aboutit quand même |
| Statut `cancelled` | enum `0001_init.sql:69` | Existe, mais **aucune transition** ne le pose (seulement des lectures `TasksView.vue:106,111`) |
| Gel immutabilité | `backend/src/domain/immutability.rs:44` | `Entity::Task` gelée si `closed`/`cancelled` — inchangé |

Écart : (a) blocage dur de la suppression si soumission prestataire ou entrée carnet liée ; (b) action « Annuler » (endpoint + UI) posant `cancelled`.

## 2. Modèle de données

Aucune migration. Les liens existent déjà (`vendor_submission.task_id`, `maintenance_log.task_id`).

## 3. Backend

### 3.1 DB — `backend/src/db/task.rs`

**`delete` durci** : remplacer le bloc `:354-369` (DELETE des `vendor_submission`) par deux checks bloquants avant tout DELETE :

```rust
let has_submissions = sqlx::query_scalar::<_, bool>(
    "SELECT EXISTS(SELECT 1 FROM vendor_submission WHERE task_id = $1)"
).bind(id).fetch_one(&mut *tx).await?
if has_submissions {
    return Err(AppError::validation_code("errors.tasks.has_submissions"))
}
let in_logbook = sqlx::query_scalar::<_, bool>(
    "SELECT EXISTS(SELECT 1 FROM maintenance_log WHERE task_id = $1)"
).bind(id).fetch_one(&mut *tx).await?
if in_logbook {
    return Err(AppError::validation_code("errors.tasks.has_logbook_entry"))
}
```

**Nouvelle fonction `cancel`** : `SELECT … FOR UPDATE` ; refuse si statut `closed` ou déjà `cancelled` (`errors.immutability.frozen` via `assert_mutable`) ; `UPDATE task SET status = 'cancelled', updated_at = NOW()`. Non destructif : documents, soumissions, token — tout est conservé (le token devient de facto inutilisable si la soumission vérifie le statut ; sinon, le laisser tel quel — le gel `immutability.rs:44` bloque toute mutation ultérieure).

### 3.2 API — `backend/src/api/tasks.rs` + routes

- `delete_task` (`:187-210`) : inchangé côté handler — les nouveaux refus remontent de la couche db (zéro logique en handler).
- Nouveau handler `cancel` (rôle `Manager`), zéro logique, appelle `db::task::cancel`.
- Route `api/mod.rs` : `.route("/tasks/{id}/cancel", post(tasks::cancel))`.

Réponses `cancel` : `200` tâche `cancelled` · `400 errors.immutability.frozen` (déjà `closed`/`cancelled`) · `403` · `404`.
Réponses `delete` (nouvelles) : `400 errors.tasks.has_submissions` · `400 errors.tasks.has_logbook_entry`.

## 4. Frontend

- `frontend/src/api/index.ts` : `cancel: (id: string) => api.post<Task>(`/tasks/${id}/cancel`)`.
- `DeleteTaskModal.vue` : l'avertissement carnet (`:39`, `:60-63`) devient un **blocage** — si `is_in_logbook` (ou soumissions présentes), désactiver le bouton « Supprimer » et proposer « Annuler la tâche » à la place. Gérer aussi le `400` renvoyé par l'API (défense en profondeur).
- `TasksView.vue` : action « Annuler la tâche » (visible sur toute tâche non `closed`/`cancelled`), avec confirmation ; badge `cancelled` déjà géré (`:106,111`).
- i18n frontend : messages pour `errors.tasks.has_submissions` / `errors.tasks.has_logbook_entry`.

## 5. Cas limites

| Cas | Règle |
|---|---|
| Suppression, soumission prestataire liée | `400 errors.tasks.has_submissions` — plus jamais de DELETE cascade sur `vendor_submission` |
| Suppression, entrée carnet liée | `400 errors.tasks.has_logbook_entry` (l'entrée carnet est une pièce probante, ADR-009) |
| Suppression, tâche vierge (ni soumission ni carnet) | Autorisée comme aujourd'hui |
| Annulation d'une tâche `closed` | Refusée (`errors.immutability.frozen`) — le gel existant s'applique |
| Annulation d'une tâche déjà `cancelled` | Refusée (idem) |
| Annulation avec token prestataire actif | Token conservé en base ; vérifier que `POST /vendor/{token}/submission` refuse sur tâche `cancelled` — si ce n'est pas le cas, ajouter le check statut côté `api/contractor.rs` (flag en implémentation) |

## 6. Critères d'acceptation

1. `DELETE /tasks/{id}` échoue `400` avec code dédié si soumission ou entrée carnet rattachée ; aucune ligne `vendor_submission` n'est jamais supprimée par ce chemin.
2. `POST /tasks/{id}/cancel` pose `cancelled` sans rien supprimer.
3. Une tâche `cancelled` est gelée (aucun update/délete ultérieur).
4. UI : suppression grisée + alternative « Annuler » proposée quand la suppression est bloquée.

## 7. Plan de tests

- **e2e** (`backend/tests/api_e2e_test.rs`) :
  1. Tâche + soumission prestataire → `DELETE` → `400 errors.tasks.has_submissions` ; la soumission existe toujours.
  2. Tâche close réouverte non applicable — tâche avec entrée carnet (`maintenance_log.task_id`) → `DELETE` → `400 errors.tasks.has_logbook_entry`.
  3. Tâche vierge → `DELETE` → `204`.
  4. `POST /cancel` sur tâche `open` → `200 cancelled` ; puis `PUT` → `400 errors.immutability.frozen` ; puis `DELETE` → `400`.
  5. `POST /cancel` sur tâche `closed` → `400`.
- **Frontend** : vérification manuelle modale (blocage + alternative).

## 8. Ordre d'implémentation

1. DB : checks bloquants dans `delete`, fonction `cancel`.
2. API : handler + route + i18n.
3. Tests e2e, boucler au vert.
4. Frontend : api client, `DeleteTaskModal`, action annuler dans `TasksView`.

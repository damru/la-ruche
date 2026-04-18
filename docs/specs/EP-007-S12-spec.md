# Spécification technique — EP-007-S12

**Story :** Réouvrir une tâche clôturée ou annulée → elle repasse en `in_progress`.

**Épic :** EP-007 — Travaux & prestataires
**Réf. PRD :** US-TP-013 · **Réf. ADR :** ADR-009 (données closes immuables)
**État :** PARTIEL / **CASSÉ** — la réouverture actuelle échoue systématiquement.

---

## 1. État actuel du code (bug)

| Élément | Emplacement | Constat |
|---|---|---|
| `ReopenTaskModal.vue:39` | frontend | Appelle le `update` générique avec `{status: 'open'}` — mauvais statut cible (la story dit `in_progress`) |
| Handler `update` | `backend/src/api/tasks.rs:148-154` | Exécute `assert_mutable` **avant** le patch |
| Gel | `backend/src/domain/immutability.rs:44` | `Entity::Task` gelée si `closed`/`cancelled` |
| Conséquence | — | La réouverture retourne **toujours** `400 errors.immutability.frozen` |
| Logbook à la clôture | `backend/src/db/task.rs:443` (`create_log_entry` sur transition → `closed`), insert inconditionnel `db/maintenance_log.rs:159-187` | La logique de re-clôture fonctionnerait : chaque nouvelle clôture crée une nouvelle entrée carnet |

Le gel générique est correct (ADR-009) ; la réouverture est une **transition d'état explicite** qui doit avoir son propre endpoint avec sa propre règle domaine, pas passer par le `PUT` générique.

## 2. Décision — données de complétion à la réouverture

ADR-009 : les données closes sont immuables → **l'entrée `maintenance_log` créée à la clôture reste intacte** (elle documente l'intervention passée). En revanche, la tâche rouverte redevient active :

- **Recommandé : réinitialiser les champs de complétion de la tâche** (`completion_cost`, `closed_at`, et champs assimilés) à `NULL` au reopen — une tâche active avec un coût de complétion résiduel produirait une double comptabilisation à la re-clôture (nouvelle entrée carnet, `db/task.rs:443`).
- **Trade-off à flagger au reviewer** : on perd sur la ligne `task` la trace du premier coût — mais elle survit dans l'entrée `maintenance_log` d'origine, qui est la pièce probante. Alternative rejetée : conserver les champs (risque de doublon carnet et d'affichage trompeur d'une tâche « en cours » avec date de clôture).

## 3. Modèle de données

Aucune migration.

## 4. Backend

### 4.1 Domain

Règle pure (dans `domain/immutability.rs` ou module tasks domaine) :

```rust
/// PRD US-TP-013: reopen is the one sanctioned exit from the frozen states.
pub(crate) fn can_reopen(status: &str) -> bool {
    matches!(status, "closed" | "cancelled")
}
```

Le gel générique (`immutability.rs:44`) reste inchangé — le reopen ne passe pas par `assert_mutable`, il est l'exception explicite et documentée.

### 4.2 DB — `backend/src/db/task.rs`

Nouvelle fonction `reopen(pool, id, building_id) -> AppResult<Option<TaskRow>>` :

1. `SELECT … FOR UPDATE` ; `can_reopen(status)` sinon `AppError::validation_code("errors.tasks.not_reopenable")`.
2. `UPDATE task SET status = 'in_progress', completion_cost = NULL, closed_at = NULL, updated_at = NOW()` (adapter la liste aux colonnes de complétion réelles).
3. **Ne touche pas** `maintenance_log` : l'entrée créée à la clôture d'origine reste (décision §2). Une re-clôture ultérieure créera une nouvelle entrée (comportement existant `:443`, insert inconditionnel `maintenance_log.rs:159-187`) — c'est voulu : deux interventions distinctes, deux lignes de carnet.

### 4.3 API — `backend/src/api/tasks.rs` + routes

Handler `reopen` (rôle `Manager`), zéro logique. Route `api/mod.rs` : `.route("/tasks/{id}/reopen", post(tasks::reopen))`.

Réponses : `200` tâche `in_progress` · `400 errors.tasks.not_reopenable` (statut non gelé) · `403` · `404`.

Le handler `update` (`:148-154`) reste inchangé — il continue de refuser toute mutation d'une tâche gelée.

## 5. Frontend

- `frontend/src/api/index.ts` : `reopen: (id: string) => api.post<Task>(`/tasks/${id}/reopen`)`.
- `ReopenTaskModal.vue:39` : remplacer l'appel `update({status:'open'})` par `tasksApi.reopen(id)` — corrige à la fois le mauvais endpoint et le mauvais statut cible (`in_progress`).
- Ajouter dans la modale une mention : « La ligne du carnet d'entretien créée à la clôture est conservée ; le coût de complétion de la tâche est réinitialisé. »
- Rendre l'action « Réouvrir » disponible aussi sur les tâches `cancelled` si ce n'est pas déjà le cas.

## 6. Cas limites

| Cas | Règle |
|---|---|
| Reopen sur tâche `open`/`in_progress`/`awaiting_*` | `400 errors.tasks.not_reopenable` |
| Reopen sur `cancelled` | Autorisé → `in_progress` |
| Reopen puis re-clôture | Nouvelle entrée carnet créée ; l'ancienne subsiste (2 lignes = 2 interventions) |
| Champs de complétion | Remis à `NULL` au reopen (décision §2, trade-off flaggé) |
| Documents / soumissions | Intacts |
| Double reopen concurrent | `FOR UPDATE` + check statut → le second reçoit `400` |

## 7. Critères d'acceptation

1. `POST /tasks/{id}/reopen` sur tâche `closed` ou `cancelled` → `200`, `status == "in_progress"`, champs de complétion nulls.
2. L'entrée `maintenance_log` d'origine est inchangée après reopen.
3. Re-clôture après reopen → deuxième entrée carnet.
4. Reopen sur tâche non gelée → `400 errors.tasks.not_reopenable`.
5. La modale frontend réussit (fin du `errors.immutability.frozen` systématique).

## 8. Plan de tests

- **Unitaire** : `can_reopen` vrai pour `closed`/`cancelled`, faux pour les autres.
- **e2e** (`backend/tests/api_e2e_test.rs`) :
  1. Clore une tâche (entrée carnet créée) → reopen → `200 in_progress`, `completion_cost == null` ; `GET` carnet → l'entrée d'origine existe toujours.
  2. Re-clore → deux entrées carnet.
  3. Reopen sur tâche `open` → `400`.
  4. Reopen sur tâche `cancelled` → `200`.
  5. Rôle `co_owner` → `403`.

## 9. Ordre d'implémentation

1. Domain `can_reopen` + test.
2. DB `reopen` (reset complétion).
3. API handler + route + i18n.
4. Tests e2e, boucler au vert.
5. Frontend : api client + `ReopenTaskModal.vue` (endpoint + message).
6. `docs/PROGRESS.md`.

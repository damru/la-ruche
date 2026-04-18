# Spécification technique — EP-007-S07

**Story :** Reprendre en main une tâche en phase devis (`awaiting_quote` / `quote_received`) → repasser en `in_progress` sans annuler les documents rattachés ni révoquer le lien prestataire.

**Épic :** EP-007 — Travaux & prestataires
**Réf. PRD :** US-TP-012 · **Réf. LEGAL :** — · **Réf. ADR :** —
**État :** NON IMPLÉMENTÉ

---

## 1. Analyse préalable — statuts devis inexistants

La story présuppose des statuts `awaiting_quote` / `quote_received` que l'enum `task_status` ne connaît pas. Valeurs actuelles : `'open','in_progress','awaiting_vendor','awaiting_validation','closed','cancelled'` (`backend/migrations/0001_init.sql:69`, mapping Rust `backend/src/db/task.rs:107-114`).

Deux options :

1. **Mapper sur `awaiting_vendor`** — pas de migration, mais perte de la distinction PRD « devis demandé » vs « devis reçu », et sémantique ambiguë (awaiting_vendor sert aussi à l'intervention).
2. **Ajouter `awaiting_quote` et `quote_received` à l'enum** (recommandé) — le PRD décrit explicitement un workflow devis distinct du workflow intervention ; les statuts deviennent lisibles et filtrables.

**Recommandation retenue : option 2** + endpoint de transition explicite `POST /tasks/{id}/take-back`.

## 2. État actuel du code

| Élément | Emplacement |
|---|---|
| Enum `task_status` (6 valeurs, sans phase devis) | `backend/migrations/0001_init.sql:69` |
| Mapping statut Rust | `backend/src/db/task.rs:107-114` |
| Plus proche existant : `revoke_token` — reset `awaiting_vendor` → `open` **et** annule le token (ce que la story interdit) | `backend/src/db/task.rs:488-507` |
| Guards immutabilité (`Entity::Task` gelée si `closed`/`cancelled`) | `backend/src/domain/immutability.rs:44` |
| Routes tasks | `backend/src/api/mod.rs` (bloc tasks) |

## 3. Modèle de données

Migration `backend/migrations/NNNN_task_quote_statuses.sql` :

```sql
-- EP-007-S07: quote phase statuses (PRD US-TP-012)
ALTER TYPE task_status ADD VALUE IF NOT EXISTS 'awaiting_quote' BEFORE 'awaiting_vendor';
ALTER TYPE task_status ADD VALUE IF NOT EXISTS 'quote_received' BEFORE 'awaiting_vendor';
```

Note PostgreSQL : `ADD VALUE` seul dans sa migration (pas d'usage de la nouvelle valeur dans la même transaction — précédent `0030` + `0033` sur `levy_status`). Ici aucune colonne/backfill ne consomme les valeurs, une seule migration suffit.

Pas de nouvelle table → pas de mise à jour `docs/ARCHITECTURE.md` obligatoire, mais mettre à jour la liste des valeurs d'enum si elle y est documentée.

Après migration : `cargo sqlx migrate run` puis `cargo sqlx prepare` (commit `backend/.sqlx/`).

## 4. Backend

### 4.1 Domain — `backend/src/domain/immutability.rs`

Aucun changement de gel : `awaiting_quote`/`quote_received` sont des statuts actifs (mutables). Vérifier que `is_mutable(Entity::Task, …)` raisonne par liste noire (`closed`/`cancelled`) et non liste blanche ; sinon ajouter les deux valeurs.

Nouvelle règle domaine (fonction pure, testable) :

```rust
/// PRD US-TP-012: a task in quote phase can be taken back by the manager.
pub(crate) fn can_take_back(status: &str) -> bool {
    matches!(status, "awaiting_quote" | "quote_received")
}
```

### 4.2 DB — `backend/src/db/task.rs`

- Étendre le mapping statut (`:107-114`) avec les deux nouvelles valeurs.
- Nouvelle fonction `take_back(pool, id, building_id) -> AppResult<Option<TaskRow>>` :
  - `SELECT … FOR UPDATE`, vérifier `can_take_back(status)` sinon `AppError::validation_code("errors.tasks.not_in_quote_phase")` ;
  - `UPDATE task SET status = 'in_progress', updated_at = NOW()` — **sans toucher** `vendor_token`, `token_expires_at`, ni les documents rattachés (contrairement à `revoke_token` `:488-507` qui annule le token).

### 4.3 API — `backend/src/api/tasks.rs` + `backend/src/api/mod.rs`

Handler `take_back` (zéro logique, rôle `Manager`) :

```rust
pub async fn take_back(auth: AuthUser, State(state): State<Arc<AppState>>, Path(id): Path<Uuid>)
    -> AppResult<Json<serde_json::Value>>
```

Route : `.route("/tasks/{id}/take-back", post(tasks::take_back))` dans le bloc tasks de `api/mod.rs`.

Réponses : `200` tâche mise à jour · `400 errors.tasks.not_in_quote_phase` · `403` rôle · `404` tâche inexistante/autre immeuble.

## 5. Frontend

- `frontend/src/types/domain.ts` : étendre `TaskStatus` avec `'awaiting_quote' | 'quote_received'`.
- `frontend/src/api/index.ts` (`tasksApi`) : `takeBack: (id: string) => api.post<Task>(`/tasks/${id}/take-back`)`.
- `TasksView.vue` : badges + libellés pour les deux nouveaux statuts ; action « Reprendre en main » visible si `status ∈ {awaiting_quote, quote_received}`, avec confirmation légère (toast au succès). Onglets de filtre statut : ajouter les nouvelles valeurs.
- Aucune modale nouvelle nécessaire (confirmation via composant catalogue existant).

## 6. Cas limites

| Cas | Règle |
|---|---|
| `take-back` sur `open`/`in_progress`/`awaiting_vendor`/`awaiting_validation` | `400 errors.tasks.not_in_quote_phase` |
| `take-back` sur `closed`/`cancelled` | `400` (même code — la phase devis n'y est pas) ; le gel immutabilité reste intact |
| Token prestataire actif au moment du take-back | **Préservé** — le prestataire peut encore soumettre ; c'est le comportement voulu par la story (reprise en main ≠ révocation, cf. `revoke_token` pour ça) |
| Documents rattachés | Intacts, aucune suppression |

## 7. Critères d'acceptation

1. L'enum `task_status` contient `awaiting_quote` et `quote_received`.
2. `POST /tasks/{id}/take-back` sur une tâche en phase devis → `200`, `status == "in_progress"`.
3. Après take-back, le token prestataire et les documents de la tâche sont inchangés.
4. Take-back refusé (`400`) hors phase devis, avec code i18n dédié.
5. UI : action visible uniquement en phase devis ; nouveaux statuts affichés avec badge.

## 8. Plan de tests

- **Unitaire domaine** : `can_take_back` vrai pour les 2 statuts devis, faux pour les 4 autres + `closed`/`cancelled`.
- **e2e** (`backend/tests/api_e2e_test.rs`) :
  1. Tâche en `awaiting_quote` → `POST /take-back` → `200`, `in_progress`, `vendor_token` non nul inchangé.
  2. Tâche `quote_received` avec document rattaché → take-back → document toujours listé.
  3. Tâche `open` → `400 errors.tasks.not_in_quote_phase`.
  4. Rôle `co_owner` → `403`.

## 9. Ordre d'implémentation

1. Migration enum → `sqlx migrate run` + `sqlx prepare`.
2. Domain `can_take_back` + test.
3. DB `take_back` + mapping statuts.
4. API handler + route + i18n.
5. Tests e2e.
6. Frontend types/api/vue.
7. `docs/PROGRESS.md`.

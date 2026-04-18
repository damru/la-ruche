# Spécification technique — EP-017-S03

**Story :** Backend : matérialiser la conversion signalement → tâche (`linked_task_id`) et exposer `GET /tasks/{id}/timeline` + `GET /reports/{id}/timeline` remontant la chaîne complète.

**Épic :** EP-017 — Journal d'activité & Suivi de cycle de vie
**Réf. PRD :** Gestion courante (US-JA-*, US-TP-015) · **Réf. EPICS :** `docs/EPICS.md` EP-017-S03 (L511)
**Stories liées :** EP-017-S01/S02 (table + événements, prérequis), EP-017-S04/S05 (consommateurs frontend), EP-017-S06 (filtrage par rôle — branché sur les handlers créés ici)

---

## 1. Quoi

Deux endpoints de lecture renvoyant la timeline **fusionnée** de la chaîne signalement ↔ tâche : les événements du signalement source et ceux de la tâche issue de la conversion, triés chronologiquement. Les deux endpoints renvoient la même chaîne, interrogée depuis l'un ou l'autre maillon (symétrie prescrite par EPICS).

### ⚠️ `linked_task_id` : déjà matérialisé sous le nom `task_id`

Le texte EPICS demande « ajouter `linked_task_id UUID REFERENCES task(id)` sur `report` ». Or **`report.task_id` existe déjà** et joue exactement ce rôle : colonne posée sur `report`, renseignée par `convert_to_task` (`backend/src/db/report.rs` L21 — champ `task_id: Option<Uuid>` ; L166-232 — conversion qui lie `report.task_id = task.id`), lue par le LEFT JOIN des SELECT (L37/L50). **Décision : conserver le nom `task_id`** — un rename n'apporterait rien et toucherait migrations, `.sqlx/`, types frontend. Le libellé de `docs/EPICS.md` L511 est à corriger pour refléter la colonne existante (signaler dans la PR ; ne pas créer de colonne).

Conséquence : **aucune migration dans cette story.**

## 2. État actuel du code

| Élément | Emplacement |
|---|---|
| `report.task_id` + conversion | `backend/src/db/report.rs` L21, L166-232 (`convert_to_task`) |
| Repo activité (`list_for_entity`, `ENTITY_REPORT`/`ENTITY_TASK`) | `backend/src/db/activity_log.rs` (EP-017-S01) |
| Handlers tâches / signalements | `backend/src/api/tasks.rs` (`get` L32), `backend/src/api/reports.rs` (`get` L27) |
| Routes | `backend/src/api/mod.rs` blocs tasks (L253-273) et reports (L310-326) |
| Aucun endpoint timeline | — |

## 3. Modèle de données

Aucune migration (cf. §1). Ajouter une fonction de lecture inverse dans `db/report.rs` :

```rust
/// Report converted into the given task, if any (reverse of report.task_id).
pub async fn find_by_task_id(
    pool: &PgPool, task_id: Uuid, building_id: Uuid,
) -> AppResult<Option<Uuid>>;
// SELECT id FROM report WHERE task_id = $1 AND building_id = $2
```

Et dans `db/activity_log.rs` une lecture multi-entités avec noms d'acteurs :

```rust
#[derive(Debug, serde::Serialize)]
pub struct TimelineEventRow {
    pub id: Uuid,
    pub entity_type: String,   // "report" | "task"
    pub entity_id: Uuid,
    pub action: String,
    pub actor_id: Option<Uuid>,
    pub actor_name: Option<String>, // user_account.full_name, NULL si acteur anonyme
    pub payload: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Merged events for a (report?, task?) chain, chronological ascending.
/// LEFT JOIN user_account for actor_name; WHERE clause over the provided pairs.
pub async fn list_for_chain(
    pool: &PgPool,
    report_id: Option<Uuid>,
    task_id: Option<Uuid>,
) -> AppResult<Vec<TimelineEventRow>>;
```

Le tri fusionné est fait **en SQL** (`ORDER BY created_at ASC, id ASC`) — pas de logique de merge en `domain/`.

## 4. Backend Rust

### 4.1 Résolution de chaîne (dans les handlers, séquence sans logique métier)

- `GET /tasks/{id}/timeline` (`api/tasks.rs::timeline`) :
  1. `db::task::get(id, building_id)` → 404 si absent (scoping immeuble).
  2. `db::report::find_by_task_id(id, building_id)` → `report_id` optionnel.
  3. `db::activity_log::list_for_chain(report_id, Some(id))`.
- `GET /reports/{id}/timeline` (`api/reports.rs::timeline`) :
  1. `db::report::get(id, building_id)` → 404 si absent ; en tirer `report.task_id`.
  2. `db::activity_log::list_for_chain(Some(id), report.task_id)`.

### 4.2 Autorisations

Alignées sur la lecture des entités : `GET /tasks/{id}/timeline` suit `tasks::get` (manager/board_member — le routage frontend `tasks/:id` est déjà restreint à ces rôles) ; `GET /reports/{id}/timeline` suit `reports::get` (tout rôle authentifié de l'immeuble). Le **filtrage des actions sensibles par rôle** est spécifié par EP-017-S06 et se branche dans ces deux handlers — prévoir le passage de `auth` mais ne pas implémenter le filtre ici.

### 4.3 Routes — `backend/src/api/mod.rs`

```rust
.route("/tasks/{id}/timeline", get(tasks::timeline))     // bloc tasks, après /vendor-token
.route("/reports/{id}/timeline", get(reports::timeline)) // bloc reports, après /documents
```

### 4.4 Contrat d'API

```
GET /api/v1/tasks/{id}/timeline    → 200 [TimelineEvent], 404 tâche inexistante / autre immeuble
GET /api/v1/reports/{id}/timeline  → 200 [TimelineEvent], 404 signalement inexistant / autre immeuble
TimelineEvent = { id, entity_type, entity_id, action, actor_id, actor_name, payload, created_at }
```

## 5. Frontend

Client API seulement (les vues arrivent en S04/S05) — `frontend/src/api/index.ts` :

```ts
// tasksApi (L322)
timeline: (id: string) => api.get<TimelineEvent[]>(`/tasks/${id}/timeline`),
// reportsApi (L426)
timeline: (id: string) => api.get<TimelineEvent[]>(`/reports/${id}/timeline`),
```

Type `TimelineEvent` dans `frontend/src/types/domain.ts` (miroir du contrat §4.4, `action` typée en union des actions EP-017-S02).

## 6. Cas limites

| Cas | Règle |
|---|---|
| Tâche créée directement (sans signalement source) | `find_by_task_id` → `None` ; timeline = événements de la tâche seuls. |
| Signalement non converti | `task_id IS NULL` ; timeline = événements du signalement seuls. |
| Entité antérieure à EP-017 (aucun événement rétro-rempli) | `200 []` — pas de backfill (décision : la timeline commence à la mise en production de S02). |
| Événements horodatés identiques | Ordre stable via tie-break `id` (S01 §6). |
| Tâche d'un autre immeuble | 404 dès l'étape 1 — `activity_log` n'est jamais interrogée sans scoping préalable. |

## 7. Critères d'acceptation

1. Aucune colonne ajoutée ; `docs/EPICS.md` L511 corrigé (« `linked_task_id` » → « `task_id` existant »).
2. Les deux endpoints renvoient la **même** liste pour un couple converti, quel que soit le maillon interrogé.
3. `actor_name` résolu pour les acteurs connus, `NULL` pour les événements prestataire.
4. Scoping immeuble étanche (404 cross-building).

## 8. Plan de tests (e2e)

1. Signalement → conversion → mutations de la tâche → `GET /reports/{id}/timeline` et `GET /tasks/{task_id}/timeline` renvoient la même séquence ordonnée (`report.created`, …, `converted_to_task`, `task.created`, …).
2. Tâche sans signalement : timeline = événements tâche uniquement.
3. Signalement non converti : timeline = événements signalement uniquement.
4. Cross-building : 404. Entité inexistante : 404.
5. Événement à `actor_id NULL` (soumission vendeur) : `actor_name` null dans la réponse.

## 9. Ordre d'implémentation

1. `db/activity_log.rs::list_for_chain` + `db/report.rs::find_by_task_id` → `cargo sqlx prepare` si requêtes macro.
2. Handlers `tasks::timeline` / `reports::timeline` + routes → `cargo build` + clippy.
3. Tests e2e §8 → vert.
4. Frontend : type `TimelineEvent` + méthodes `timeline` des deux clients → `npm --prefix frontend run build && lint`.
5. `docs/EPICS.md` (statut + correction libellé S03), `docs/PROGRESS.md`.

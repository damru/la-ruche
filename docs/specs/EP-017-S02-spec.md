# Spécification technique — EP-017-S02

**Story :** Backend : émettre les événements auditables aux points de mutation clés de `report` et `task`, avec `payload` JSONB ancien/nouveau quand pertinent.

**Épic :** EP-017 — Journal d'activité & Suivi de cycle de vie
**Réf. PRD :** Gestion courante (US-JA-*) · **Réf. EPICS :** table des actions auditables, `docs/EPICS.md` L490-505
**Stories liées :** EP-017-S01 (table + repo `db/activity_log.rs`, prérequis), EP-017-S03 (lecture), EP-017-S06 (visibilité par rôle — ne change rien à l'émission)

---

## 1. Quoi

Instrumenter chaque mutation métier de `report` et `task` par un appel à `db::activity_log::insert` (EP-017-S01), **dans la même transaction** que la mutation quand une transaction existe déjà, sinon sur le pool juste après la mutation réussie. Le vocabulaire `action` est celui, fermé, de `docs/EPICS.md`.

## 2. État actuel du code — points d'ancrage des mutations

Aucune émission d'événement n'existe. Sites à instrumenter :

| Entité / action | Site d'émission | Handler API appelant |
|---|---|---|
| `report.created` | `backend/src/db/report.rs` `create` (L112) | `api/reports.rs` `create` (L39 — accessible à tout rôle authentifié, y compris résident) |
| `report.status_changed` | `db/report.rs` `update_status` (L142) **et** `close_linked_to_task` (L235 — auto-résolution quand la tâche liée est clôturée, déjà générique sur l'executor) | `api/reports.rs` `update_status` (L62) ; chemin indirect via clôture de tâche |
| `report.photo_added` | `db/report.rs` `attach_document` (L362) | `api/reports.rs` `attach_document` (L158) |
| `report.converted_to_task` + `task.created` (source) | `db/report.rs` `convert_to_task` (L166 — transaction existante : émettre les **deux** événements dedans) | `api/reports.rs` `convert_to_task` (L79) |
| `task.created` | `db/task.rs` `create` (L216) | `api/tasks.rs` `create` (L90) |
| `task.status_changed` / `assignee_changed` / `contractor_assigned` / `closed` | `db/task.rs` `update_fields` (L391 — dispose déjà de `current_status`, L408, et des anciens champs : diff old/new disponible) et `update_status` (L249) | `api/tasks.rs` `update` (L141) |
| `task.vendor_link_generated` | `db/task.rs` `generate_token` (L277 — passe aussi le statut à `awaiting_vendor` : émettre `vendor_link_generated` **et** `status_changed`) | `api/tasks.rs` `generate_token` (L230) |
| `task.document_uploaded` | `db/task.rs` `attach_document` (L585) | `api/tasks.rs` `attach_document` (L61) |
| `task.vendor_submission_received` | `db/task.rs` `create_submission` (L656) | `api/contractor.rs` `submit` (L132 — endpoint **public** par token : `actor_id = None`) |

## 3. Modèle de données

Aucune migration : table `activity_log` livrée par EP-017-S01 (placeholder `0040`).

## 4. Backend Rust

### 4.1 Vocabulaire — `backend/src/db/activity_log.rs` (extension)

Ajouter les constantes d'action (évite les chaînes libres dispersées) :

```rust
pub mod actions {
    // report
    pub const CREATED: &str = "created";
    pub const STATUS_CHANGED: &str = "status_changed";
    pub const PHOTO_ADDED: &str = "photo_added";
    pub const CONVERTED_TO_TASK: &str = "converted_to_task";
    // task (CREATED/STATUS_CHANGED partagés)
    pub const ASSIGNEE_CHANGED: &str = "assignee_changed";
    pub const CONTRACTOR_ASSIGNED: &str = "contractor_assigned";
    pub const VENDOR_LINK_GENERATED: &str = "vendor_link_generated";
    pub const DOCUMENT_UPLOADED: &str = "document_uploaded";
    pub const VENDOR_SUBMISSION_RECEIVED: &str = "vendor_submission_received";
    pub const CLOSED: &str = "closed";
}
```

### 4.2 Règles d'émission

- **Transaction** : quand la fonction repo ouvre déjà une transaction (`convert_to_task`, `update_fields`, `delete`…), l'insert d'activité y participe — l'événement n'existe que si la mutation aboutit. Les fonctions sans transaction (`create`, `update_status`, `attach_document`) : émettre après le succès de la requête, sur le même pool ; une émission qui échoue **ne doit pas** faire échouer la mutation déjà commitée → logguer (`tracing::warn!`) et avaler l'erreur (helper `emit_best_effort`). Décision assumée : la timeline est un confort UX, pas une piste probante (cf. arbitrage EP-018).
- **`actor_id`** : `Some(auth.user_id)` — propagé du handler vers la fonction repo (nouveaux paramètres `actor_id: Option<Uuid>` là où l'info manque, ex. `update_status`, `attach_document`). Endpoint public prestataire : `None`.
- **Payloads** (JSONB, clés stables) :
  - `status_changed` : `{"old_status", "new_status"}`
  - `converted_to_task` : `{"task_id"}` · `task.created` issu d'une conversion : `{"source_report_id"}`
  - `assignee_changed` : `{"old_assignee_id", "new_assignee_id"}`
  - `contractor_assigned` : `{"contractor_id", "contractor_name"}` (nom dénormalisé : lisible même si le prestataire est supprimé)
  - `vendor_link_generated` : `{"expires_at"}` — **jamais** le token lui-même
  - `document_uploaded` / `photo_added` : `{"document_id", "file_name"}` si disponible, sinon `NULL`
  - `vendor_submission_received` : `{"submission_id"}`
- **`closed` vs `status_changed`** : une transition de `task` vers `closed` émet **`closed`** (payload `{"old_status"}`), pas `status_changed` — un seul événement par transition, le vocabulaire EPICS distingue la clôture. Les autres transitions émettent `status_changed`. Événement no-op (nouveau statut == ancien) : rien n'est émis. Idem pour `assignee_changed`/`contractor_assigned` : émis uniquement si la valeur change réellement (diff dans `update_fields`).
- **Pas de logique dans les handlers** : toute émission vit dans `db/` (au plus près de la mutation) ; les handlers se contentent de transmettre `auth.user_id`.

### 4.3 Fichiers touchés

`backend/src/db/activity_log.rs` (constantes + helper), `db/report.rs`, `db/task.rs`, `api/reports.rs`, `api/tasks.rs`, `api/contractor.rs` (propagation `actor_id` uniquement). Aucun changement de route.

## 5. Frontend

Aucun changement (lecture en EP-017-S03/S04/S05).

## 6. Cas limites

| Cas | Règle |
|---|---|
| Mutation en transaction échoue | Rollback → aucun événement (insert dans la même tx). |
| Émission post-commit échoue | Mutation conservée, warn loggué, pas de 500. |
| `update_fields` change statut **et** prestataire en un appel | Deux événements distincts dans la même transaction. |
| Auto-résolution du signalement à la clôture de la tâche (`close_linked_to_task`) | Émet `report.status_changed` avec l'`actor_id` du clôtureur de la tâche. |
| `generate_token` régénéré (token déjà existant) | Nouveau `vendor_link_generated` à chaque génération ; `status_changed` seulement si le statut change. |

## 7. Critères d'acceptation

1. Chaque action du tableau §2 produit une ligne `activity_log` avec `entity_type`, `action`, `actor_id` et `payload` conformes §4.2.
2. Une conversion signalement→tâche produit exactement 2 événements (`report.converted_to_task`, `task.created`) atomiquement.
3. Aucun événement dupliqué ni no-op ; aucun token vendeur en clair dans `payload`.
4. `cargo clippy -- -D warnings` et suite de tests existante au vert (aucune régression de comportement des endpoints).

## 8. Plan de tests (e2e, `backend/tests/api_e2e_test.rs`)

1. `POST /reports` → 1 événement `report.created`, `actor_id` = créateur.
2. `PUT /reports/{id}/status` → `report.status_changed` avec `old/new`.
3. `POST /reports/{id}/task` → `report.converted_to_task` (`task_id`) + `task.created` (`source_report_id`).
4. `PUT /tasks/{id}` changeant statut + prestataire → 2 événements ; repasser le même body → 0 événement.
5. Passage à `closed` → événement `closed`, pas `status_changed` ; le signalement lié reçoit `report.status_changed`.
6. `POST /tasks/{id}/vendor-token` → `vendor_link_generated` (payload sans token) + `status_changed`.
7. `POST /api/v1/vendor/{token}/submission` → `vendor_submission_received`, `actor_id IS NULL`.
8. Vérification en base directe (pas d'endpoint de lecture avant S03) via le pool du harness de test.

## 9. Ordre d'implémentation

1. Constantes `actions` + helper `emit_best_effort` dans `db/activity_log.rs`.
2. Instrumentation `db/report.rs` (4 actions) → tests 1-3.
3. Instrumentation `db/task.rs` (6 actions) + propagation `actor_id` depuis `api/tasks.rs` / `api/contractor.rs` → tests 4-7.
4. `cargo test --lib --tests`, `cargo clippy -- -D warnings`, `cargo fmt --check`.
5. `docs/EPICS.md`, `docs/PROGRESS.md`, `graphify update .`.

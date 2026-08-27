# Spécification technique — EP-017-S01

**Story :** Migration : créer la table `activity_log` polymorphique + repository d'insertion/lecture, socle des cinq stories suivantes de l'épic.

**Épic :** EP-017 — Journal d'activité & Suivi de cycle de vie
**Réf. PRD :** Gestion courante (US-JA-* / US-TP-015) · **Réf. EPICS :** `docs/EPICS.md` §EP-017 (schéma prescriptif, L509)
**Stories liées :** EP-017-S02 (émission des événements — consomme le repo créé ici), EP-017-S03 (endpoints timeline — consomme la lecture), EP-018-S01 (arbitrage `audit_log` / `activity_log`)

---

## 1. Quoi

Créer la table `activity_log` : journal **métier** polymorphique (`entity_type` / `entity_id`) alimentant les timelines de cycle de vie des signalements et des tâches. À ne pas confondre avec `audit_log` (migration `0029`, EP-018) : celle-ci est la piste probante **comptable** (opérations `UPDATE`/`DELETE`, `old_values`/`new_values`, conservation 10 ans). L'arbitrage est documenté dans `docs/EPICS.md` (L484, L522) — `activity_log` = frise métier lisible, `audit_log` = probatoire comptable. Les deux tables coexistent sans recouvrement.

## 2. État actuel du code

| Élément | Emplacement |
|---|---|
| Table `audit_log` (comptable, à ne PAS réutiliser) | `backend/migrations/0029_*.sql`, repo `backend/src/db/audit.rs` |
| Aucune table `activity_log` | — |
| Prochain n° migration | **0040** (dernière = `0039_budget_rectificatif_constraints`) — les migrations récentes (0038/0039) utilisent le format `up`/`down` |
| Registre des tables | `docs/ARCHITECTURE.md` (à mettre à jour **avant** la migration, règle AGENTS.md) |

## 3. Modèle de données

### 3.1 Migration `backend/migrations/0040_activity_log.up.sql`

Schéma prescrit par `docs/EPICS.md` (EP-017-S01), repris tel quel :

```sql
-- EP-017-S01: polymorphic business activity log (lifecycle timeline).
-- Distinct from audit_log (0029): audit_log is the accounting-grade
-- evidentiary trail (EP-018/ADR-009); activity_log is the user-facing
-- lifecycle feed for reports and tasks. Append-only by convention:
-- no UPDATE/DELETE path is exposed by the application.

CREATE TABLE activity_log (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type TEXT NOT NULL,
    entity_id   UUID NOT NULL,
    action      TEXT NOT NULL,
    actor_id    UUID REFERENCES user_account(id),
    payload     JSONB,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX activity_log_entity_idx
    ON activity_log (entity_type, entity_id, created_at DESC);
```

`0040_activity_log.down.sql` : `DROP TABLE activity_log;`

**Notes de conception :**
- `entity_type` / `entity_id` polymorphiques : pas de FK vers `report`/`task` (décision EPICS « pas de méta-objet »). Valeurs de `entity_type` en v1 : `'report'` et `'task'` uniquement (constantes Rust, §4.1). `action` : vocabulaire fermé défini dans la table des actions auditables de `docs/EPICS.md` (L490-505), porté par EP-017-S02.
- `actor_id` **nullable** : les événements déclenchés hors session authentifiée (soumission prestataire via token public, cf. EP-017-S02 §vendor_submission_received) ont `actor_id IS NULL`. `ON DELETE` par défaut (RESTRICT) : `user_account` n'est jamais supprimé physiquement dans le produit.
- Pas de `building_id` : le scoping immeuble se fait via l'entité référencée (le lecteur charge d'abord `report`/`task` par `id + building_id`) — voir EP-017-S03 §4 et EP-017-S06.
- Append-only **applicatif** : aucun endpoint ni fonction repo `update`/`delete`. Pas de trigger PG de blocage (le caractère probant est du ressort d'`audit_log`) — simplicité assumée.

Après migration : `cargo sqlx migrate run` puis `cargo sqlx prepare` (commit `backend/.sqlx/`).

## 4. Backend Rust

### 4.1 Nouveau repo `backend/src/db/activity_log.rs` (déclaré dans `db/mod.rs`)

```rust
/// Business activity log (EP-017). Append-only: insert + read only.
pub const ENTITY_REPORT: &str = "report";
pub const ENTITY_TASK: &str = "task";

#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct ActivityLogRow {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub action: String,
    pub actor_id: Option<Uuid>,
    pub payload: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Generic over the executor so callers can join an open transaction
/// (mirrors db/report.rs::close_linked_to_task).
pub async fn insert<'e, E>(
    executor: E,
    entity_type: &str,
    entity_id: Uuid,
    action: &str,
    actor_id: Option<Uuid>,
    payload: Option<serde_json::Value>,
) -> AppResult<()>
where E: sqlx::Executor<'e, Database = sqlx::Postgres>;

/// Events of ONE entity, chronological ascending.
pub async fn list_for_entity(
    pool: &PgPool,
    entity_type: &str,
    entity_id: Uuid,
) -> AppResult<Vec<ActivityLogRow>>;
```

- `insert` : simple `INSERT INTO activity_log (entity_type, entity_id, action, actor_id, payload) VALUES ($1,$2,$3,$4,$5)`.
- `list_for_entity` : `SELECT ... WHERE entity_type = $1 AND entity_id = $2 ORDER BY created_at ASC, id ASC` (tie-break déterministe sur `id`).
- Pas de handler API dans cette story (lecture exposée par EP-017-S03).

## 5. Frontend

Aucun changement frontend dans cette story.

## 6. Cas limites

| Cas | Règle |
|---|---|
| Entité supprimée (`DELETE /reports/{id}`, `DELETE /tasks/{id}`) | Les lignes `activity_log` deviennent orphelines et inaccessibles (aucune lecture ne part de `activity_log`). Acceptable en v1 — pas de purge, volumétrie négligeable. |
| `payload` absent | `NULL` autorisé (ex. `photo_added` sans métadonnées). |
| Deux événements dans la même milliseconde | L'ordre est stabilisé par le tie-break `id` en lecture. |

## 7. Critères d'acceptation

1. La migration `0040` s'applique et se rollback proprement (`up`/`down`).
2. `docs/ARCHITECTURE.md` documente `activity_log` (et sa frontière avec `audit_log`) avant la migration.
3. `db::activity_log::insert` fonctionne dans un pool **et** dans une transaction ouverte.
4. `list_for_entity` renvoie les événements en ordre chronologique ascendant stable.
5. Aucune route API exposée ; `cargo clippy -- -D warnings` passe (fonctions marquées utilisées par S02/S03 — si clippy signale du dead code avant S02, annoter `#[allow(dead_code)]` temporaire retiré en S02, ou livrer S01+S02 ensemble).

## 8. Plan de tests

- Test d'intégration (harness `backend/tests/`, cf. `api_e2e_test.rs`/`common`) : insert 3 événements (dont un `actor_id: None`, un `payload: Some(json!({...}))`) → `list_for_entity` renvoie 3 lignes ordonnées, champs fidèles.
- Test : `insert` dans une transaction rollbackée → aucune ligne persistée.

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` : section `activity_log` (+ rappel frontière `audit_log`).
2. Migrations `0040_activity_log.up.sql` / `.down.sql` → `cargo sqlx migrate run` → `cargo sqlx prepare`.
3. `backend/src/db/activity_log.rs` + déclaration `db/mod.rs` → `cargo build`.
4. Tests §8 → vert. `cargo clippy -- -D warnings`, `cargo fmt --check`.
5. `docs/EPICS.md` (statut S01), `docs/PROGRESS.md` (entrée datée), `graphify update .`.

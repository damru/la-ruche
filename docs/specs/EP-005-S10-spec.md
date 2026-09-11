# Spécification technique — EP-005-S10

**Story :** Émettre des événements internes au dépôt d'un document pour permettre des notifications contextuelles.

**Épic :** EP-005 — Documents
**Réf. PRD :** US-DOC-009
**Hors scope (table EPICS) :** la **livraison** réelle des notifications (email, in-app). Cette story pose uniquement l'événement interne consommable.

---

## 1. Quoi (résumé fonctionnel)

Aujourd'hui `confirm_upload` (`backend/src/api/documents.rs` L122-167) insère la ligne `document` et retourne — aucun événement, aucun bus, aucune table d'outbox n'existe dans le repo. Impossible de brancher une notification « un document a été déposé » sans repasser sur ce chemin.

Cible **minimale** (outbox-lite) :
- Table `document_event` écrite **dans la même transaction** que l'insertion du document (garantie : pas de document sans événement, pas d'événement sans document).
- Consommation par **polling** depuis un job (`backend/src/jobs/`), pattern identique à `reminders.rs` : sélectionne les événements non traités, les marque traités. Le consommateur MVP ne fait que loguer (`tracing::info!`) — la livraison réelle viendra avec l'épic notifications.
- Pas de bus, pas de broker, pas de trigger PG, pas de LISTEN/NOTIFY : le polling suffit au volume d'une copropriété.

---

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| `confirm_upload` (insert + return, aucun événement) | `backend/src/api/documents.rs` L122-167 |
| `db::document::create` (INSERT simple, hors transaction) | `backend/src/db/document.rs` |
| Pattern de job à polling | `backend/src/jobs/reminders.rs` (boucle horaire, `SELECT ... WHERE status='pending'`, marquage) |
| Jobs existants | `backend/src/jobs/mod.rs` (seulement `reminders`) |

---

## 3. Modèle de données

### Migration `backend/migrations/NNNN_document_event.sql`

```sql
-- EP-005-S10: outbox-lite for document events (PRD US-DOC-009).
-- Written in the SAME transaction as the document insert; consumers poll
-- (jobs/) and mark rows processed. Real notification delivery is out of
-- scope (EPICS "hors scope") — the MVP consumer only logs.

CREATE TABLE document_event (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id   UUID NOT NULL REFERENCES building(id),
    document_id   UUID NOT NULL REFERENCES document(id),
    event_type    TEXT NOT NULL,                -- 'uploaded' (extensible: 'replaced', 'deleted')
    actor_id      UUID REFERENCES user_account(id),
    payload       JSONB NOT NULL DEFAULT '{}',  -- {original_name, category, visible_roles}
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at  TIMESTAMPTZ
);

CREATE INDEX document_event_pending_idx ON document_event (created_at) WHERE processed_at IS NULL;
```

- `event_type` en TEXT (pas d'enum PG) : extensible sans migration à deux temps ; validé côté Rust.
- Nouvelle table → mettre à jour `docs/ARCHITECTURE.md` **avant** la migration. Pas d'ADR : outbox-lite standard, pas d'alternative structurelle réelle au polling à ce volume.
- `cargo sqlx prepare` après migration.

---

## 4. Backend Rust

### 4.1 `db/document.rs` — insertion transactionnelle

`create` fait aujourd'hui un INSERT direct sur le pool. Le passer en transaction :

```rust
pub async fn create(/* mêmes params */) -> AppResult<DocumentRow> {
    let mut tx = pool.begin().await?;
    let doc = /* INSERT INTO document ... RETURNING ... (inchangé, sur &mut *tx) */;
    sqlx::query!(
        r#"INSERT INTO document_event (building_id, document_id, event_type, actor_id, payload)
           VALUES ($1, $2, 'uploaded', $3, $4)"#,
        building_id, doc.id, uploaded_by,
        serde_json::json!({
            "original_name": doc.original_name,
            "visible_roles": doc.visible_roles,
        }),
    ).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(doc)
}
```

- `confirm_upload` (api) ne change pas : l'événement est un détail du repo — zéro logique en handler.
- Si S06/S08 sont livrées, émettre aussi `'replaced'` dans `replace`/`create_version` et `'deleted'` dans le soft-delete, même pattern (optionnel dans cette story ; `'uploaded'` est le livrable).

### 4.2 Nouveau job `backend/src/jobs/document_events.rs`

Squelette `reminders.rs` : boucle `interval` (5 min — les notifications contextuelles tolèrent cette latence), et par tick :

```rust
async fn process_events(state: &Arc<AppState>) -> anyhow::Result<()> {
    let events = sqlx::query!(
        r#"SELECT id, building_id, document_id, event_type, payload
           FROM document_event
           WHERE processed_at IS NULL
           ORDER BY created_at
           LIMIT 100"#,
    ).fetch_all(&state.pool).await?;

    for ev in events {
        // MVP consumer: structured log only. Real delivery = notifications epic.
        tracing::info!(event_id = %ev.id, building_id = %ev.building_id,
                       document_id = %ev.document_id, event_type = %ev.event_type,
                       "document event");
        sqlx::query!("UPDATE document_event SET processed_at = NOW() WHERE id = $1", ev.id)
            .execute(&state.pool).await?;
    }
    Ok(())
}
```

Déclarer `pub mod document_events;` dans `jobs/mod.rs` et spawner à côté de `reminders::run`.

### 4.3 Pas d'API

Aucune route HTTP : les événements sont internes. (Une éventuelle timeline « activité documents » serait une story UI distincte.)

---

## 5. Frontend

Aucun changement.

---

## 6. Cas limites

| Cas | Règle |
|---|---|
| Échec de l'INSERT événement | La transaction annule aussi le document → `confirm_upload` échoue proprement, l'upload S3 reste et sera rattrapé par le job orphelins (S09). |
| Crash entre traitement et marquage | L'événement est retraité au tick suivant (at-least-once) — acceptable, le consommateur doit être idempotent (un log l'est). |
| Document supprimé avant traitement de l'événement | FK conservée (soft-delete S08 garde la ligne) ; si S08 absent, un hard delete casserait la FK → le soft-delete S08 est la trajectoire ; en attendant, le hard delete existant échouera sur la FK — ajouter `ON DELETE CASCADE` **refusé** (perte d'audit) ; noter la dépendance : livrer S08 avant ou en même temps. |
| Accumulation d'événements traités | Volume négligeable ; purge éventuelle = extension future du job S09. |
| Plusieurs instances backend | Non verrouillé (même hypothèse mono-instance que `reminders.rs`) — commentaire dans le code. |

---

## 7. Critères d'acceptation

1. Chaque `POST /documents` (confirm_upload) réussi crée exactement une ligne `document_event` `'uploaded'`, atomiquement.
2. Un échec d'écriture de l'événement annule l'enregistrement du document.
3. Le job marque `processed_at` et logue chaque événement ; les événements traités ne sont jamais repris.
4. Aucune notification réelle n'est envoyée (hors scope), aucun changement d'API publique.

---

## 8. Plan de tests

- **e2e** (`api_e2e_test.rs`) :
  1. `POST /documents` → `201` ; vérif SQL : une ligne `document_event` avec `event_type='uploaded'`, `document_id` correct, `processed_at IS NULL`.
  2. Deux uploads → deux événements, ordonnés par `created_at`.
- **Intégration job** : extraire `process_events` en fn testable (comme `run_once` de S09) — événement pending traité et marqué ; événement déjà `processed_at` ignoré ; limite 100 respectée.

---

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` (table `document_event`) → migration `NNNN` → `sqlx migrate run` + `prepare`.
2. `db/document.rs::create` transactionnel + événement → `cargo build`, e2e upload toujours vert.
3. `jobs/document_events.rs` + `jobs/mod.rs` + spawn.
4. Tests (§8) → vert ; `clippy -D warnings`.
5. `docs/PROGRESS.md`.

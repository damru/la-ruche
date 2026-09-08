# Spécification technique — EP-013-S06

**Story :** Notes collaboratives par résolution pendant la séance, reprises automatiquement dans le projet de PV.

**Épic :** EP-013 — Procès-verbaux
**Réf. LEGAL :** — (aide à la rédaction, sans valeur probante propre) · **Réf. PRD :** US-PV-* (notes de séance) · **Réf. ADR :** aucun (table append-only simple)

**Statut actuel : NON IMPLÉMENTÉ** — le contexte du projet de PV (`backend/src/domain/minutes_draft.rs` L20-60) contient `blank_proxy_distribution` et un texte libre « incident technique », mais aucune note par résolution.

---

## 1. Quoi (résumé fonctionnel)

- Pendant une AG `ongoing`, les participants habilités ajoutent des **notes horodatées par résolution** (append-only : pas d'édition ni de suppression — la note est un élément de contexte de séance).
- Le **projet de PV** reprend ces notes, groupées par résolution, dans l'ordre chronologique.

## 2. État actuel du code

| Élément | Emplacement |
|---|---|
| Contexte du projet de PV (à étendre) | `backend/src/domain/minutes_draft.rs` L20-60 |
| Table `resolution` | `backend/migrations/0001_init.sql` L363-373 |
| Routes meetings | `backend/src/api/mod.rs` L113-174 |
| Aucune table / route / vue « note » | — |

## 3. Modèle de données

Nouvelle table → **mettre à jour `docs/ARCHITECTURE.md` avant la migration**.

Migration `backend/migrations/NNNN_resolution_note.sql` :

```sql
-- EP-013-S06: collaborative per-resolution session notes, replayed into the
-- minutes draft. Append-only: no UPDATE/DELETE path is exposed.
CREATE TABLE resolution_note (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resolution_id UUID NOT NULL REFERENCES resolution(id) ON DELETE CASCADE,
    author_id UUID NOT NULL REFERENCES app_user(id),
    body TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX resolution_note_resolution_idx ON resolution_note (resolution_id, created_at);
```

Après migration : `cargo sqlx migrate run` + `cargo sqlx prepare` (commit `backend/.sqlx/`).

## 4. Backend

### 4.1 Domaine

- Règle : création de note uniquement quand le meeting est `ongoing` (`errors.meetings.not_ongoing`) ; body non vide (`errors.resolution_notes.body_required`).
- `minutes_draft.rs` : étendre le contexte (L20-60) avec

```rust
pub struct ResolutionNoteCtx {
    pub resolution_number: i32,
    pub author_name: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
}
// added to the draft context:
pub resolution_notes: Vec<ResolutionNoteCtx>,
```

et injecter dans le template du projet de PV une section « Notes de séance » par résolution (notes chronologiques, préfixées de l'auteur et de l'heure).

### 4.2 DB — `backend/src/db/assembly.rs` (ou `db/resolution_note.rs`)

- `create_note(pool, meeting_id, resolution_id, building_id, author_id, body)` — vérifie en SQL que la résolution appartient au meeting et que le meeting est `ongoing` (et du bon `building_id`).
- `list_notes(pool, meeting_id, building_id)` — jointure résolution, tri `(resolution.number, created_at)`.
- Le builder du projet de PV appelle `list_notes` et alimente `resolution_notes`.

### 4.3 API — routes dans `backend/src/api/mod.rs` (bloc meetings)

```rust
.route(
    "/meetings/{id}/resolutions/{rid}/notes",
    get(assemblies::list_resolution_notes).post(assemblies::create_resolution_note),
)
```

- POST : rôles `manager` + `board_member` (les rôles qui animent la séance) ; l'auteur est `auth.user_id`.
- GET : mêmes rôles + `co_owner` en lecture.
- Pas de PUT/DELETE (append-only).
- Handlers sans logique — validations dans domain/db.

## 5. Frontend

- `frontend/src/api/index.ts` : `listResolutionNotes(meetingId, resolutionId)`, `createResolutionNote(meetingId, resolutionId, body)`.
- Vue séance (`frontend/src/features/assemblies/`) : sous chaque résolution en statut `ongoing`, fil de notes (auteur, heure, texte) + champ d'ajout. Lecture seule hors `ongoing`.
- Vue projet de PV : les notes apparaissent dans le rendu généré par le backend — aucun travail frontend supplémentaire sur le template.
- Nouveau composant éventuel (fil de notes) → cataloguer dans `docs/DESIGN.md §3` avant usage.

## 6. Cas limites

| Cas | Règle |
|---|---|
| Note sur meeting `draft`/`called`/`closed` | 400 `errors.meetings.not_ongoing`. |
| Body vide / blanc | 400 `errors.resolution_notes.body_required`. |
| Résolution n'appartenant pas au meeting de l'URL | 404. |
| Résolution supprimée (en draft) | Notes supprimées en cascade (impossibles en pratique : notes seulement en `ongoing`). |
| Meeting clôturé après prise de notes | Notes lisibles, plus d'ajout ; elles restent dans le projet de PV. |
| Résolution sans note | Section « Notes de séance » omise pour cette résolution dans le PV. |

## 7. Critères d'acceptation

1. Un manager/board_member ajoute une note pendant `ongoing` → 201, horodatée, attribuée.
2. Aucun ajout possible hors `ongoing` ; aucune édition/suppression de note n'existe.
3. Le projet de PV contient les notes groupées par résolution, ordre chronologique, avec auteur.
4. Un co_owner lit les notes mais ne peut pas en créer.

## 8. Plan de tests

- **Unitaires domaine** : injection des notes dans le contexte `minutes_draft` (résolution avec/sans notes, ordre).
- **E2E** :
  1. POST note en `ongoing` → 201 ; GET → liste ordonnée ;
  2. POST en `called` puis en `closed` → 400 ;
  3. POST body vide → 400 ;
  4. POST par co_owner → 403 ;
  5. GET minutes-draft après notes → le corps contient les notes de la résolution.

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` : table `resolution_note`.
2. Migration `NNNN` → `cargo sqlx migrate run` + `cargo sqlx prepare`.
3. DB : `create_note` / `list_notes`.
4. Domaine : guard `ongoing` + extension du contexte `minutes_draft.rs` et du template.
5. API : handlers + route → `cargo clippy -- -D warnings`.
6. Tests e2e, boucler jusqu'au vert.
7. Frontend : api client + fil de notes en séance.
8. `docs/PROGRESS.md` + `graphify update .`.

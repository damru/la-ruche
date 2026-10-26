# Spécification technique — EP-008-S07

**Story :** Notes de suivi sinistre — journal **append-only** horodaté, accessible Syndic + Conseil syndical.

**Épic :** EP-008 — Sinistres
**Réf. PRD :** US-SIN-004 · **Réf. ADR :** ADR-009 (esprit piste probante — pas de nouvel ADR : table de jointure simple, aucun trade-off structurel)
**État :** IMPLÉMENTÉ (backend + frontend)

---

## 1. État actuel du code

| Élément | Emplacement | Constat |
|---|---|---|
| Table / endpoint / UI notes | — | Inexistants |
| Bloc routes claims | `backend/src/api/mod.rs:282-293` | Point d'insertion des nouvelles routes |
| Handlers claims | `backend/src/api/claims.rs` | Modèle de style pour les nouveaux handlers |
| Gate lifecycle claim | `backend/src/domain/immutability.rs:45` (`transmitted_at`) | Ne s'applique **pas** aux notes (le suivi continue après transmission) |

## 2. Modèle de données

Nouvelle table → **mettre à jour `docs/ARCHITECTURE.md` avant la migration** (règle AGENTS.md). Table simple, pas d'ADR.

Migration `backend/migrations/NNNN_claim_note.sql` :

```sql
-- EP-008-S07: append-only timestamped follow-up notes on claims (PRD US-SIN-004).
-- No updated_at on purpose: rows are never updated nor deleted.
CREATE TABLE claim_note (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    claim_id    UUID NOT NULL REFERENCES claim(id) ON DELETE CASCADE,
    author_id   UUID NOT NULL REFERENCES app_user(id),
    body        TEXT NOT NULL CHECK (length(trim(body)) > 0),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX claim_note_claim_idx ON claim_note (claim_id, created_at);
```

(Adapter le nom de la table utilisateur — `app_user` — à celui du schéma réel.) Puis `cargo sqlx migrate run` + `cargo sqlx prepare`.

## 3. Backend

### 3.1 Domain

Rien à ajouter : l'append-only est garanti par **l'absence** d'endpoints update/delete. Ne pas ajouter de variante `Entity::ClaimNote` dans `immutability.rs` — il n'existe aucun chemin de mutation à garder.

### 3.2 DB — nouveau `backend/src/db/claim_note.rs` (ou module dans `claim.rs`)

```rust
pub struct ClaimNoteRow { pub id: Uuid, pub claim_id: Uuid, pub author_id: Uuid,
    pub author_name: String, pub body: String, pub created_at: DateTime<Utc> }
```

- `create(pool, claim_id, building_id, author_id, body)` : vérifier via jointure que le claim appartient bien à `building_id` (sinon `None` → `404`) ; `INSERT … RETURNING`.
- `list(pool, claim_id, building_id)` : `SELECT n.*, u.display_name AS author_name FROM claim_note n JOIN … ORDER BY n.created_at ASC`.

Validation : `body` non vide après trim → `AppError::validation_code("errors.claims.note_body_empty")` (check applicatif en plus du CHECK SQL).

### 3.3 API — `backend/src/api/claims.rs` + routes

Deux handlers, zéro logique, rôle `Manager | BoardMember` sur **les deux** (POST et GET — US-SIN-004 : Syndic + Conseil) :

```rust
pub async fn create_note(auth, State, Path(id), Json(input)) -> AppResult<(StatusCode, Json<…>)>
pub async fn list_notes(auth, State, Path(id)) -> AppResult<Json<…>>
```

Route (`api/mod.rs`, bloc claims `:282-293`) :

```rust
.route("/claims/{id}/notes", post(claims::create_note).get(claims::list_notes))
```

Aucune route PUT/DELETE — l'append-only est structurel.

Réponses : `201` note créée · `200` liste chronologique · `400 errors.claims.note_body_empty` · `403` co_owner/tenant · `404` claim inexistant/autre immeuble.

## 4. Frontend

- `frontend/src/types/domain.ts` : `interface ClaimNote { id; claim_id; author_id; author_name; body; created_at }`.
- `frontend/src/api/index.ts` (`claimsApi`) : `listNotes(claimId)`, `createNote(claimId, { body })`.
- `ClaimsView.vue` (détail/modale claim) : section « Suivi » — timeline chronologique (auteur, horodatage formaté, corps), champ texte + bouton « Ajouter une note » visible Manager/BoardMember. Pas de bouton éditer/supprimer, par conception.
- Si un composant timeline réutilisable est créé, l'ajouter à `docs/DESIGN.md §2` avant usage ; sinon composer avec les `C*` existants.

## 5. Cas limites

| Cas | Règle |
|---|---|
| Body vide / espaces | `400 errors.claims.note_body_empty` |
| Claim `draft` (cf. EP-008-S06) | Notes autorisées pour le Manager ; le claim lui-même reste invisible des autres rôles donc ses notes aussi |
| Claim transmis (`transmitted_at` posé) | Notes **toujours autorisées** — le gel `immutability.rs:45` porte sur les champs du claim, pas sur son journal de suivi |
| Claim `closed` | Notes autorisées (suivi post-clôture possible) — décision simple ; à restreindre plus tard si le PRD l'exige |
| Suppression du claim | `ON DELETE CASCADE` — les notes suivent le claim |
| co_owner / tenant | `403` sur GET et POST |

## 6. Critères d'acceptation

1. Table `claim_note` créée, documentée dans `docs/ARCHITECTURE.md`.
2. `POST /claims/{id}/notes` (Manager/BoardMember) crée une note horodatée ; `GET` la restitue en ordre chronologique avec le nom de l'auteur.
3. Aucun endpoint de modification/suppression de note n'existe.
4. `403` pour co_owner/tenant sur les deux endpoints.
5. UI : timeline de suivi + saisie dans la vue claim.

## 7. Plan de tests (e2e — `backend/tests/api_e2e_test.rs`)

1. Manager crée 2 notes → `GET` → 2 notes, ordre `created_at ASC`, `author_name` renseigné.
2. BoardMember crée et liste → `201` / `200`.
3. co_owner → `403` sur POST et GET.
4. Body vide → `400 errors.claims.note_body_empty`.
5. Claim d'un autre immeuble → `404`.
6. Notes toujours acceptées après pose de `transmitted_at`.

## 8. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` (table `claim_note`) puis migration `NNNN` → `sqlx migrate run` + `sqlx prepare`.
2. DB `claim_note.rs` (create/list).
3. API handlers + route + i18n.
4. Tests e2e, boucler au vert.
5. Frontend types/api/UI timeline.
6. `docs/PROGRESS.md`.

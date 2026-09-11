# Spécification technique — EP-003-S11

**Story :** Permettre à un copropriétaire de demander la convocation d'une AG **à ses frais** (art. 17-1 AA loi 1965), avec workflow d'acceptation et trace d'imputation des frais.

**Épic :** EP-003 — Assemblées générales
**Réf. LEGAL :** §3.5 · **Réf. PRD :** US-AG-* (convocation à la demande d'un copropriétaire) · **Réf. ADR :** aucun (table de workflow simple, pas d'alternative structurelle)

**Statut actuel : NON IMPLÉMENTÉ** — rien n'existe (ni table, ni route, ni écran).

---

## 1. Quoi (résumé fonctionnel)

- Un copropriétaire soumet une **demande de convocation d'AG** avec les questions qu'il souhaite porter à l'ordre du jour.
- Le syndic **accepte** (ou refuse) la demande ; l'acceptation trace la **décision d'imputation des frais** au demandeur (montant).
- Lorsque l'AG est créée, elle est **liée** à la demande (statut `meeting_created`).
- Périmètre volontairement minimal : la **facturation réelle est manuelle** (note libre) — pas de génération d'appel de fonds automatique ; un lien optionnel vers un `levy_call` existant peut être renseigné.

## 2. État actuel du code

| Élément | Emplacement |
|---|---|
| Routes meetings (bloc de rattachement des nouvelles routes) | `backend/src/api/mod.rs` L113-174 |
| Cycle de vie AG `draft→called→ongoing→closed` | `backend/src/db/assembly.rs` |
| Pattern repo/handlers de référence | `backend/src/db/assembly.rs`, `backend/src/api/assemblies.rs` |
| Aucune entité « demande de convocation » | — |

## 3. Modèle de données

Nouvelle table → **mettre à jour `docs/ARCHITECTURE.md` avant la migration** (étape obligatoire).

Migration `backend/migrations/NNNN_meeting_request.sql` :

```sql
-- EP-003-S11: co-owner request for a general meeting at their own expense
-- (art. 17-1 AA loi 1965, LEGAL §3.5).
CREATE TYPE meeting_request_status AS ENUM ('pending', 'accepted', 'rejected', 'meeting_created');

CREATE TABLE meeting_request (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    building_id UUID NOT NULL REFERENCES building(id),
    requester_id UUID NOT NULL REFERENCES app_user(id),   -- co-owner making the request
    questions TEXT NOT NULL,                               -- agenda items requested (free text)
    status meeting_request_status NOT NULL DEFAULT 'pending',
    fee_amount NUMERIC(12,2),                              -- decided at acceptance (Decimal in Rust)
    fee_note TEXT,                                         -- manual billing note (levy linkage is manual)
    levy_call_id UUID REFERENCES levy_call(id),            -- optional link to an existing levy call
    meeting_id UUID REFERENCES general_meeting(id),        -- set when the AG is created
    decided_by UUID REFERENCES app_user(id),
    decided_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX meeting_request_building_idx ON meeting_request (building_id, status);
```

> Adapter les noms exacts des tables référencées (`building`, `app_user`, `general_meeting`, `levy_call`) à ceux de `0001_init.sql` lors de l'implémentation.

Après migration : `cargo sqlx migrate run` + `cargo sqlx prepare` (commit `backend/.sqlx/`).

## 4. Backend

### 4.1 Domaine — `backend/src/domain/meeting_request.rs` (nouveau)

Transitions autorisées (fonction pure testable) :

```rust
/// pending -> accepted | rejected ; accepted -> meeting_created. Terminal:
/// rejected, meeting_created.
pub(crate) fn can_transition(from: &str, to: &str) -> bool { ... }
```

Règles :
- acceptation ⇒ `fee_amount` (Decimal) requis, `>= 0` (`errors.meeting_requests.fee_required`) ;
- refus ⇒ `fee_amount` interdit ;
- liaison AG ⇒ statut `accepted` uniquement (`errors.meeting_requests.not_accepted`).

### 4.2 DB — `backend/src/db/meeting_request.rs` (nouveau)

- `create(pool, building_id, requester_id, input)` — questions non vides (`errors.meeting_requests.questions_required`).
- `list(pool, building_id)` / `get(pool, id, building_id)`.
- `decide(pool, id, building_id, decided_by, status, fee_amount, fee_note)` — applique `can_transition`, pose `decided_by`/`decided_at`.
- `link_meeting(pool, id, building_id, meeting_id)` — passe en `meeting_created`.
- `Decimal` pour `fee_amount` — jamais `f64`.

### 4.3 API — `backend/src/api/meeting_requests.rs` (nouveau) + routes dans `backend/src/api/mod.rs`

```rust
.route("/meeting-requests", get(meeting_requests::list).post(meeting_requests::create))
.route("/meeting-requests/{id}", get(meeting_requests::get))
.route("/meeting-requests/{id}/decide", post(meeting_requests::decide))
.route("/meeting-requests/{id}/link-meeting", post(meeting_requests::link_meeting))
```

- `create` : rôle `co_owner` (et plus) — le demandeur est `auth.user_id`.
- `list`/`get` : `manager` + `board_member` ; un `co_owner` voit ses propres demandes (filtre `requester_id`).
- `decide`, `link_meeting` : `manager` uniquement.
- Zéro logique dans les handlers — transitions dans domain, SQL dans db.

## 5. Frontend

- `frontend/src/api/index.ts` : `meetingRequestsApi` (`list`, `create`, `decide`, `linkMeeting`).
- `frontend/src/features/assemblies/` :
  - formulaire de demande côté copropriétaire (questions en texte libre) ;
  - liste des demandes côté syndic avec actions « Accepter » (saisie `fee_amount` + `fee_note`) / « Refuser » ;
  - sur une demande `accepted` : action « Créer l'AG » qui pré-remplit la création d'AG puis appelle `linkMeeting`.
- Nouveaux composants éventuels (modale de décision) → les cataloguer dans `docs/DESIGN.md §3` avant usage (étape d'implémentation, ne pas éditer DESIGN.md ici).

## 6. Cas limites

| Cas | Règle |
|---|---|
| Décision sur une demande déjà décidée | 400 `errors.meeting_requests.invalid_transition`. |
| Acceptation sans `fee_amount` | 400 `errors.meeting_requests.fee_required` (montant 0 autorisé — le syndic peut renoncer aux frais, la décision est tracée). |
| Liaison AG sur demande `pending`/`rejected` | 400 `errors.meeting_requests.not_accepted`. |
| Copropriétaire consultant la demande d'un autre | 404 (filtrage par `requester_id`). |
| Suppression | Non prévue — la demande est une trace probante ; pas de DELETE. |

## 7. Critères d'acceptation

1. Un `co_owner` crée une demande avec questions → 201, statut `pending`.
2. Le `manager` accepte avec un montant → statut `accepted`, `fee_amount`/`decided_by`/`decided_at` renseignés.
3. Le `manager` lie une AG créée → statut `meeting_created`, `meeting_id` renseigné.
4. Les transitions invalides sont refusées avec des erreurs i18n.
5. L'imputation des frais est une **trace** (montant + note) — aucun flux comptable automatique.

## 8. Plan de tests

- **Unitaires domaine** : matrice complète `can_transition` (états terminaux inclus).
- **E2E** (`backend/tests/api_e2e_test.rs`) :
  1. création par co_owner → 201 ;
  2. création questions vides → 400 ;
  3. accept sans fee → 400 ; accept avec fee → 200 ;
  4. double décision → 400 ;
  5. link-meeting sur `accepted` → 200 ; sur `pending` → 400 ;
  6. co_owner ne voit que ses demandes.

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` : décrire la table `meeting_request`.
2. Migration `NNNN` → `cargo sqlx migrate run` + `cargo sqlx prepare`.
3. `domain/meeting_request.rs` + tests unitaires.
4. `db/meeting_request.rs`.
5. `api/meeting_requests.rs` + routes → `cargo clippy -- -D warnings`.
6. Tests e2e.
7. Frontend (api client, formulaire, liste, actions) — cataloguer les composants dans `docs/DESIGN.md §3` avant usage.
8. `docs/PROGRESS.md`.

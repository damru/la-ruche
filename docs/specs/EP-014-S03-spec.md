# Spécification technique — EP-014-S03

**Story :** Créer, modifier et supprimer des événements manuels sur le calendrier (réunion informelle, rappel, rendez-vous) — réservé au Syndic et au Conseil.

**Épic :** EP-014 — Calendrier copropriété
**Réf. PRD :** `docs/product/PRD - Calendrier.md` §3 (entité `Event`), US-CAL-005 (AC-CAL-005-A à E)
**Stories liées :** **EP-014-S01 (prérequis)** — les événements manuels alimentent la branche `manual` du feed (cf. `EP-014-S01-spec.md` §3.2) ; EP-014-S02 (filtre « Événements ») ; EP-014-S04 (export ICS les inclut).

---

## 1. État actuel du code

- Aucune table ni module d'événements manuels. Le feed S01 prévoit la branche `manual` (omise tant que la table n'existe pas).
- Patterns à suivre dans le repo :
  - CRUD simple avec guard de rôle : `backend/src/api/logbook.rs` + `backend/src/db/maintenance_log.rs` (structure la plus proche : entité datée par immeuble, table de liaison documents).
  - Liaison GED : pattern `*_document` (`maintenance_log_document`, `contract_document`, `task_document` — `0001_init.sql` L574-578, L641-645).
- Rôles : `require_any_role(&[Role::Manager, Role::BoardMember])` (`backend/src/domain/mod.rs` L92) — le Conseil a ici les mêmes droits CRUD que le Syndic (PRD §4, différent de la plupart des modules).

## 2. Modèle de données

Nouvelle table → **mettre à jour `docs/ARCHITECTURE.md` avant la migration**. Pas d'ADR : table standard sans alternative structurelle réelle (lookup + liaison documents, patterns établis).

### Migration `backend/migrations/NNNN_calendar_event.sql`

```sql
-- EP-014-S03: manual calendar events (PRD Calendrier §3, US-CAL-005)

CREATE TYPE calendar_event_category AS ENUM ('reunion', 'visite', 'permanence', 'autre');

CREATE TABLE calendar_event (
    id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id  UUID NOT NULL REFERENCES building(id),
    title        TEXT NOT NULL,
    description  TEXT,
    event_date   DATE NOT NULL,
    event_time   TIME,                          -- heure optionnelle (PRD §3)
    category     calendar_event_category,       -- optionnelle (AC-CAL-005-B)
    created_by   UUID NOT NULL REFERENCES user_account(id),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_calendar_event_building_date ON calendar_event (building_id, event_date);

-- AC-CAL-005-C: attachments live in the GED (document) and are linked here,
-- mirroring maintenance_log_document / task_document.
CREATE TABLE calendar_event_document (
    event_id    UUID NOT NULL REFERENCES calendar_event(id) ON DELETE CASCADE,
    document_id UUID NOT NULL REFERENCES document(id) ON DELETE CASCADE,
    PRIMARY KEY (event_id, document_id)
);
```

Notes : `event_date DATE + event_time TIME` plutôt qu'un TIMESTAMPTZ — le PRD définit « Date (+ heure optionnelle) » et la nullabilité de l'heure serait inexprimable sur un timestamp unique. Après migration : `cargo sqlx migrate run` puis `cargo sqlx prepare` (commit `backend/.sqlx/`).

## 3. Backend

### 3.1 `backend/src/db/calendar_event.rs` (nouveau)

`CalendarEventRow`, `CreateCalendarEventInput { title, description, event_date, event_time, category }`, et les fonctions `create` / `update` / `delete` / `get` / `list(building_id)` — toutes bornées par `building_id`. Liaison documents : `attach_document` / `detach_document` / `list_documents`, copie conforme du pattern `maintenance_log.rs`. `delete` supprime les liaisons par CASCADE (les documents GED restent, règle transverse ARCHITECTURE.md).

### 3.2 Branche `manual` du feed (S01)

Dans `backend/src/db/calendar.rs::events`, activer la requête sur `calendar_event` (titre, `event_date`, `event_time`, `link = "/calendar?event={id}"`). Visible par **tous les rôles** (AC-CAL-005-D) — `CalendarSource::Manual` est déjà dans `visible_sources` pour les 4 rôles (cf. `EP-014-S01-spec.md` §3.1).

### 3.3 `backend/src/api/calendar_events.rs` (nouveau) + routes

Handlers sans logique, guard `require_any_role(&[Role::Manager, Role::BoardMember])` sur les écritures ; lecture ouverte à tous les rôles authentifiés :

```rust
.route("/calendar/manual-events", get(calendar_events::list).post(calendar_events::create))
.route("/calendar/manual-events/{id}",
       get(calendar_events::get).put(calendar_events::update).delete(calendar_events::remove))
.route("/calendar/manual-events/{id}/documents",
       get(calendar_events::list_documents).post(calendar_events::attach_document))
.route("/calendar/manual-events/{id}/documents/{document_id}", delete(calendar_events::detach_document))
```

Validation : `title` non vide et `event_date` requis (AC-CAL-005-A) → `400 errors.calendar_event.title_required` / `errors.calendar_event.date_required` ; catégorie inconnue → `400 errors.calendar_event.invalid_category`. `404` si l'événement appartient à un autre immeuble.

## 4. Frontend

- `frontend/src/types/domain.ts` : `CalendarEventCategory`, `ManualCalendarEvent`.
- `frontend/src/api/index.ts` : `calendarEventsApi = { list, get, create, update, remove, documents, attachDocument, detachDocument }`.
- `frontend/src/features/calendar/EventFormModal.vue` (nouveau, catalogue `docs/DESIGN.md §2` d'abord) : création/édition — titre, date, heure optionnelle, description, catégorie (select optionnel), pièces jointes (upload → GED puis liaison, même flux que le carnet d'entretien).
- `frontend/src/features/calendar/EventDetailModal.vue` (nouveau) : détail d'un événement manuel (ouvert via `?event={id}` depuis la grille) — champs + documents joints ; boutons Modifier/Supprimer visibles seulement pour Manager/BoardMember (store auth).
- `CalendarView.vue` : bouton « Nouvel événement » (Manager/BoardMember) ; rafraîchissement du feed après création/édition/suppression.
- i18n : clés `calendar.event.*` ; libellés catégories (Réunion, Visite, Permanence, Autre).

## 5. Cas limites

| Cas | Règle |
|---|---|
| Titre vide / date absente | `400` (AC-CAL-005-A). |
| Catégorie absente | Valide — événement sans catégorie (AC-CAL-005-B). |
| CoOwner/Tenant tente une écriture | `403` (guard serveur) ; boutons masqués côté UI (défense en profondeur). |
| Conseil modifie un événement créé par le Syndic | Autorisé — AC-CAL-005-E : tout `Event` de la copropriété, pas seulement les siens. |
| Suppression avec documents joints | Liaisons supprimées (CASCADE), documents conservés en GED. |
| Événement daté dans le passé | Autorisé (compte-rendu a posteriori) — apparaît dans le feed si la plage le couvre. |
| Événement d'un autre immeuble | `404` (filtrage `building_id` systématique). |

## 6. Critères d'acceptation

- **AC-CAL-005-A** : création refusée sans titre ou sans date.
- **AC-CAL-005-B** : catégorie optionnelle parmi REUNION, VISITE, PERMANENCE, AUTRE.
- **AC-CAL-005-C** : documents joints enregistrés en GED et liés via `calendar_event_document`.
- **AC-CAL-005-D** : événements manuels visibles en lecture par les 4 rôles (feed + détail).
- **AC-CAL-005-E** : Manager et BoardMember peuvent modifier/supprimer tout événement de leur immeuble.

## 7. Plan de tests

- **E2E** (`backend/tests/api_e2e_test.rs`) :
  1. Manager crée (titre+date seuls) → `201` ; sans titre → `400`.
  2. BoardMember crée, modifie et supprime un événement créé par le Manager → `200`.
  3. CoOwner : `GET` liste → `200` ; `POST` → `403`.
  4. Tenant : événement manuel visible dans `GET /calendar/events` (branche `manual`).
  5. Attache un document → listé ; suppression de l'événement → liaison disparue, document GED toujours présent.
  6. `GET`/`PUT`/`DELETE` sur un événement d'un autre immeuble → `404`.
- **Frontend** (si `vitest`) : `EventFormModal` — soumission valide → appel `create` mocké ; validation titre/date.

## 8. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` : documenter `calendar_event` + `calendar_event_document`.
2. Migration `NNNN_calendar_event.sql` → `cargo sqlx migrate run` → `cargo sqlx prepare`.
3. `db/calendar_event.rs` (CRUD + documents) + activation branche `manual` dans `db/calendar.rs`.
4. `api/calendar_events.rs` + routes → `cargo clippy -- -D warnings`.
5. Tests e2e (§7), boucler jusqu'au vert.
6. Frontend : types + `calendarEventsApi` → modales (catalogue d'abord) → intégration `CalendarView` → i18n.
7. `npm --prefix frontend run build && npm --prefix frontend run lint`.
8. Docs : `docs/EPICS.md` (statut S03), `docs/PROGRESS.md`, `graphify update .`.

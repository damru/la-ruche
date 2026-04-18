# Spécification technique — EP-014-S01

**Story :** Afficher la vue calendaire des événements de la copropriété (AG, interventions, réunions) dans les modes mensuel, hebdomadaire et liste, avec lien contextuel vers les entités correspondantes.

**Épic :** EP-014 — Calendrier copropriété
**Réf. PRD :** `docs/product/PRD - Calendrier.md` US-CAL-001 (AC-CAL-001-A à C)
**Stories liées :** EP-014-S02 (filtres, s'appuie sur ce feed), EP-014-S03 (événements manuels, intégrés au feed), EP-014-S04 (export ICS, réutilise le domaine), EP-014-S05 (widget dashboard, réutilise l'endpoint)

> Cette story est **fondatrice** : elle définit le feed agrégé `GET /calendar/events` que S02/S04/S05 réutilisent. Les événements agrégés sont en lecture seule (PRD §2) — aucune écriture ici.

---

## 1. État actuel du code

- **Aucun module calendrier n'existe** : pas de `calendar.rs` dans `backend/src/{api,domain,db}`, pas de route `/calendar` dans `frontend/src/router/index.ts` (routes existantes L51-221), pas de feature `frontend/src/features/calendar/`.
- Sources d'événements déjà en base (`backend/migrations/0001_init.sql`) :

| Source | Table / champ date | Anchor code |
|---|---|---|
| AG | `general_meeting.meeting_date` (TIMESTAMPTZ, L317) | `backend/src/db/assembly.rs` |
| Contrats | `contract_reminder.reminder_date` (DATE, L507, statut `pending`) | `backend/src/jobs/reminders.rs` consomme déjà cette table |
| Tâches | `task.closed_at` (TIMESTAMPTZ, L553) | `backend/src/db/task.rs` |
| Sinistres | `claim.declared_at`, `claim.transmitted_at` (DATE) | `backend/src/db/claim.rs` |
| Carnet | `maintenance_log.intervention_date` (DATE, L630) | `backend/src/db/maintenance_log.rs` |
| Manuels | table `calendar_event` — **créée par S03** | cf. `EP-014-S03-spec.md` |

- Rôles : `Role::{Manager, BoardMember, CoOwner, Tenant}` (`backend/src/domain/mod.rs` L17-22), guards `require_role`/`require_any_role` (L84, L92).

## 2. Modèle de données

**Aucune migration.** Le feed est une agrégation en lecture des tables existantes. La table `calendar_event` (événements manuels) est spécifiée dans `EP-014-S03-spec.md` ; tant qu'elle n'existe pas, la branche `manual` du feed renvoie une liste vide (union tolérante).

## 3. Backend

### 3.1 `backend/src/domain/calendar.rs` (nouveau)

Types purs, zéro SQL :

```rust
pub enum CalendarSource { Meeting, Contract, Task, Claim, Maintenance, Manual }

pub struct CalendarEvent {
    pub id: String,          // composite "meeting:{uuid}", "contract:{uuid}", ...
    pub source: CalendarSource,
    pub title: String,
    pub date: NaiveDate,
    pub time: Option<NaiveTime>, // renseigné pour meeting_date (TIMESTAMPTZ)
    pub link: String,            // chemin SPA : "/meetings/{id}", "/contracts", "/tasks/{id}", "/claims", "/logbook", "/calendar?event={id}"
    pub is_deadline: bool,       // true pour rappels contrat et délais légaux (consommé par S04 pour VALARM)
}

/// AC-CAL-001-C — filtrage par rôle, logique pure testable.
pub fn visible_sources(roles: &[Role]) -> Vec<CalendarSource>
```

Règles de visibilité (PRD §4) : Manager/BoardMember → toutes les sources ; CoOwner → `Meeting` (date de séance uniquement) + `Manual` ; Tenant → `Meeting` (date de séance) + `Manual`. Les délais PV/contestation AG (J+30 / J+60, dérivés de `meeting_date` pour les AG `closed`) sont réservés Manager/BoardMember et calculés dans ce module (fonctions pures).

### 3.2 `backend/src/db/calendar.rs` (nouveau)

Une fonction par source, chacune bornée par `building_id` + `[from, to]` :

```rust
pub async fn events(pool, building_id, from: NaiveDate, to: NaiveDate,
                    sources: &[CalendarSource]) -> AppResult<Vec<CalendarEvent>>
```

Requêtes : `general_meeting.meeting_date` ; `contract_reminder.reminder_date` (statut `pending`, JOIN `vendor_contract` pour le titre — masque montants) ; `task.closed_at` (tâches clôturées) ; `claim.declared_at` + délai limite de déclaration + `transmitted_at` ; `maintenance_log.intervention_date` ; `calendar_event` (S03, `SELECT` conditionnel — omis tant que la table n'existe pas). Tri final par `(date, time)` en Rust.

### 3.3 `backend/src/api/calendar.rs` (nouveau) + routes

```rust
// GET /api/v1/calendar/events?from=YYYY-MM-DD&to=YYYY-MM-DD
pub async fn events(auth: AuthUser, State(state), Query(params)) -> AppResult<Json<...>>
```

Handler sans logique : valide `from <= to` et plage ≤ 366 jours (`AppError::validation_code("errors.calendar.invalid_range")`), calcule `visible_sources(&auth.roles)`, délègue à `db::calendar::events`. Tous les rôles authentifiés y accèdent (le filtrage se fait par sources visibles, pas par 403).

Dans `backend/src/api/mod.rs`, nouveau bloc `// ── Calendar ──` :
```rust
.route("/calendar/events", get(calendar::events))
```
Ajouter le tag OpenAPI `(name = "calendar", ...)`.

## 4. Frontend

### 4.1 Rendu calendaire : décision

**Recommandation : grille mensuelle/hebdo construite en interne** (CSS grid 7 colonnes, composants catalogue existants `CCard`/`CBadge`) — pas de librairie calendrier. Volume d'événements faible (copropriété), pas de drag-and-drop, pas de récurrence : une lib (`FullCalendar`, `vue-cal`) apporterait surtout du poids et des styles à surcharger (interdiction des valeurs brutes, Tailwind v4). **Alternative flaggée** : si le mode hebdo doit à terme afficher des créneaux horaires positionnés, réévaluer `vue-cal` — à confirmer avec Damien avant d'en arriver là.

### 4.2 Fichiers

- `frontend/src/api/index.ts` : nouvel export `calendarApi = { events: (from, to) => api.get<CalendarEvent[]>('/calendar/events', { params: { from, to } }) }`.
- `frontend/src/types/domain.ts` : `CalendarSource`, `CalendarEvent`.
- `frontend/src/features/calendar/CalendarView.vue` : vue principale — toggle mensuel/hebdo/liste, navigation période (précédent/suivant/aujourd'hui), fetch sur changement de plage.
- `frontend/src/features/calendar/CalendarGrid.vue` : grille réutilisée en mensuel (6 semaines) et hebdo (7 jours) ; chaque événement = badge coloré par source + titre tronqué, clic → `router.push(event.link)` (AC-CAL-001-B).
- `frontend/src/features/calendar/CalendarListMode.vue` : liste chronologique groupée par jour.
- `frontend/src/router/index.ts` : route `{ path: 'calendar', name: 'calendar', component: CalendarView, meta: { title: 'page.calendar' } }` sous `AppLayout` ; entrée de navigation dans l'organisme de layout (sidebar).
- i18n : clés `calendar.*` dans `frontend/src/i18n/locales/`.
- Nouveaux composants → cataloguer dans `docs/DESIGN.md §2` avant usage.

## 5. Cas limites

| Cas | Règle |
|---|---|
| Plage sans événements | `200` tableau vide ; le frontend affiche la grille vide (pas d'état d'erreur). |
| `from > to` ou plage > 366 j | `400 errors.calendar.invalid_range`. |
| Table `calendar_event` absente (S03 non livrée) | Branche `manual` omise — le feed reste fonctionnel. |
| CoOwner/Tenant | Ne reçoivent jamais d'événements contrat/tâche/sinistre/carnet, même via manipulation d'URL (filtrage serveur, NFR-Sec-001). |
| AG `draft` sans date confirmée | `meeting_date` est NOT NULL — l'AG apparaît dès sa création ; badge de statut si `status = 'draft'`. |
| Événements le même jour | Empilés dans la cellule ; au-delà de 3 en vue mensuelle, lien « +N » vers le mode liste du jour. |
| Fuseaux horaires | `meeting_date` TIMESTAMPTZ converti en date/heure locale côté frontend ; les DATE restent des dates civiles. |

## 6. Critères d'acceptation

- **AC-CAL-001-A** : trois modes disponibles (mensuel, hebdo, liste) avec navigation de période.
- **AC-CAL-001-B** : chaque événement affiche titre, date, badge coloré par source, et lie vers le module source.
- **AC-CAL-001-C** : filtrage par rôle vérifié côté serveur (un CoOwner ne voit ni contrats ni délais légaux ; un Tenant ne voit que les dates de séance AG et les événements manuels).
- `GET /calendar/events` répond en ≤ 300 ms p95 (NFR-Perf-001) — requêtes indexées par `building_id`.

## 7. Plan de tests

- **Unitaires domaine** (`#[cfg(test)]` dans `domain/calendar.rs`) : `visible_sources` pour les 4 rôles ; calcul des délais PV J+30 / contestation J+60.
- **E2E** (`backend/tests/api_e2e_test.rs`, harness `common::isolated_test_server()`) :
  1. Manager : AG + rappel contrat + tâche clôturée + sinistre + intervention dans la plage → tous présents, triés.
  2. Même requête en CoOwner → seuls AG (date séance) visibles.
  3. Événements hors plage `[from, to]` → absents.
  4. `from > to` → `400 errors.calendar.invalid_range`.
  5. Isolation `building_id` : événements d'un autre immeuble absents.
- **Frontend** : si `vitest` configuré, test de `CalendarGrid` (placement des événements dans les bonnes cellules) ; sinon couverture backend uniquement.

## 8. Ordre d'implémentation

1. `domain/calendar.rs` (types + `visible_sources` + délais AG) + tests unitaires → `cargo test --lib`.
2. `db/calendar.rs` (requêtes par source) → `cargo sqlx prepare` (commit `backend/.sqlx/`).
3. `api/calendar.rs` + route + tag OpenAPI → `cargo clippy -- -D warnings`.
4. Tests e2e (§7), boucler jusqu'au vert.
5. Frontend : types + `calendarApi` → composants (catalogue `docs/DESIGN.md §2` d'abord) → route + navigation → i18n.
6. `npm --prefix frontend run build && npm --prefix frontend run lint`.
7. Docs : `docs/EPICS.md` (statut S01), `docs/PROGRESS.md` (entrée datée).

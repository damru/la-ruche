# Spécification technique — EP-014-S04

**Story :** Exporter le calendrier au format ICS pour intégration dans les agendas personnels (Google Calendar, Outlook, Apple Calendar).

**Épic :** EP-014 — Calendrier copropriété
**Réf. PRD :** `docs/product/PRD - Calendrier.md` §2 (Export), US-CAL-003 (AC-CAL-003-A à C)
**Stories liées :** **EP-014-S01 (prérequis)** — l'export sérialise le même feed `CalendarEvent` (dont le flag `is_deadline`, prévu par S01 précisément pour le VALARM) ; EP-014-S02 (paramètre `types` réutilisé) ; EP-014-S03 (les événements manuels sont exportés).

---

## 1. État actuel du code

- Aucun module ICS, aucune dépendance calendrier dans `backend/Cargo.toml`.
- Le feed agrégé (`domain/calendar.rs`, `db/calendar.rs`, S01) fournit déjà tout le nécessaire : `title`, `date`, `time`, `link`, `source`, `is_deadline`. L'export est une **sérialisation alternative du même flux** — zéro nouvelle requête SQL.
- Générations de fichiers existantes dans le repo (précédent pour le pattern « handler → bytes + headers ») : `backend/src/api/reports.rs`.

## 2. Modèle de données

**Aucune migration.**

## 3. Backend

### 3.1 Sérialisation ICS : décision

**Recommandation : sérialiseur RFC 5545 minimal écrit à la main** (`backend/src/domain/ics.rs`), pas de crate (`icalendar`, `ics`) :

- Le besoin se réduit à `VCALENDAR` + `VEVENT` (+ `VALARM`), tous en événements simples sans récurrence, sans invités, sans fuseaux exotiques — une centaine de lignes de Rust pur.
- Une dépendance apporterait parsing, RRULE, TZ database : du poids pour rien, et une surface d'audit supplémentaire.
- **Alternative flaggée** : si un besoin futur exige récurrences (RRULE) ou import ICS, basculer sur la crate `icalendar` — à confirmer avec Damien le moment venu.

### 3.2 `backend/src/domain/ics.rs` (nouveau)

Fonctions pures, testables sans DB :

```rust
/// RFC 5545 minimal serializer: VCALENDAR wrapping one VEVENT per CalendarEvent.
pub fn to_ics(events: &[CalendarEvent], prodid: &str, base_url: &str) -> String
```

Règles de sérialisation :
- `UID:{event.id}@laruche` (stable → ré-import idempotent), `DTSTAMP` à la génération.
- Événement sans heure → `DTSTART;VALUE=DATE:YYYYMMDD` (journée entière) ; avec heure → `DTSTART:YYYYMMDDTHHMMSSZ` (UTC).
- `SUMMARY` = titre ; `DESCRIPTION` + `URL` = `{base_url}{event.link}` (AC-CAL-003-C) ; `CATEGORIES` = source.
- Échappement RFC 5545 (`\,` `\;` `\\` `\n`) et pliage des lignes > 75 octets (CRLF + espace).
- `is_deadline == true` (rappels contrat, délais légaux sinistres, délai notification PV AG — cf. `EP-014-S01-spec.md` §3.1) → bloc `VALARM` :
  ```
  BEGIN:VALARM
  ACTION:DISPLAY
  TRIGGER:-PT24H
  DESCRIPTION:{title}
  END:VALARM
  ```
  (AC-CAL-003-B : alerte 24 h avant.)

### 3.3 `backend/src/api/calendar.rs` — handler d'export

```rust
// GET /api/v1/calendar/export.ics?from&to&types=...
pub async fn export_ics(auth: AuthUser, State(state), Query(params)) -> AppResult<Response>
```

Handler sans logique : mêmes validations et même pipeline que `events` (S01/S02 — `effective_sources(roles, types)`, AC-CAL-003-A : l'export respecte strictement le filtrage de rôle), puis `domain::ics::to_ics(...)`. Réponse : `Content-Type: text/calendar; charset=utf-8`, `Content-Disposition: attachment; filename="laruche-calendar.ics"`. `base_url` depuis la config existante de l'app.

Route dans le bloc `// ── Calendar ──` de `backend/src/api/mod.rs` :
```rust
.route("/calendar/export.ics", get(calendar::export_ics))
```

Défaut de plage si `from`/`to` absents : aujourd'hui → +365 jours (l'export « abonnement » complet n'a pas de sens borné à l'écran courant).

## 4. Frontend

- `frontend/src/api/index.ts` : `calendarApi.exportIcs(from, to, types?)` — `api.get(..., { responseType: 'blob' })`.
- `CalendarView.vue` : bouton « Exporter (.ics) » dans la barre d'outils — télécharge le blob (URL objet + clic programmatique, pattern existant des téléchargements documents/rapports) avec la plage et les filtres **courants** de la vue.
- Pas de nouveau composant catalogue (un simple `CBtn`).
- i18n : `calendar.export`.

**Hors périmètre (PRD §7)** : URL d'abonnement permanente (tokenisée, sans JWT) pour synchronisation continue — c'est de la « synchronisation calendrier externe », explicitement hors scope. Le livrable est un téléchargement ponctuel authentifié.

## 5. Cas limites

| Cas | Règle |
|---|---|
| Aucun événement dans la plage | Fichier ICS valide avec zéro `VEVENT` (Google/Outlook l'acceptent). |
| Titre avec `,` `;` retours ligne | Échappé RFC 5545 (§3.2) — testé unitairement. |
| Ligne > 75 octets (titres longs) | Pliage CRLF + espace (§3.2). |
| CoOwner/Tenant exporte | Mêmes règles de visibilité que la vue — jamais d'événement contrat/sinistre dans leur fichier (AC-CAL-003-A, NFR-Sec-001). |
| Événement journée entière | `VALUE=DATE`, pas de composant horaire (sinon décalage de fuseau à l'import). |
| `types` invalide / `from > to` | `400`, mêmes codes que S01/S02 (`errors.calendar.invalid_type` / `invalid_range`). |
| Double import du même fichier | `UID` stables → mise à jour, pas de doublons. |

## 6. Critères d'acceptation

- **AC-CAL-003-A** : le fichier exporté ne contient que les événements visibles pour le rôle et les filtres demandés.
- **AC-CAL-003-B** : chaque événement à échéance (`is_deadline`) contient un `VALARM` `TRIGGER:-PT24H`.
- **AC-CAL-003-C** : chaque `VEVENT` contient une propriété `URL` (et le lien en `DESCRIPTION`) pointant vers le module source ou le détail de l'événement manuel.
- Le fichier s'importe sans erreur dans Google Calendar, Outlook et Apple Calendar (vérification manuelle une fois, puis validation structurelle automatisée).

## 7. Plan de tests

- **Unitaires domaine** (`#[cfg(test)]` dans `domain/ics.rs`) :
  1. Un événement daté simple → `VEVENT` avec `UID`, `DTSTART;VALUE=DATE`, `SUMMARY`, `URL`.
  2. Événement avec heure → `DTSTART` UTC.
  3. `is_deadline` → présence du bloc `VALARM` `-PT24H` ; sinon absence.
  4. Échappement (`,` `;` `\n`) et pliage de ligne > 75 octets.
  5. Zéro événement → calendrier vide valide (`BEGIN:VCALENDAR`…`END:VCALENDAR`).
- **E2E** (`backend/tests/api_e2e_test.rs`) :
  1. Manager : export contient AG + rappel contrat ; `Content-Type: text/calendar`.
  2. CoOwner : même plage → aucun `VEVENT` contrat.
  3. `types=meeting` → seuls les `VEVENT` AG (réutilise S02).

## 8. Ordre d'implémentation

1. `domain/ics.rs` + tests unitaires (§7) → `cargo test --lib`.
2. `api/calendar.rs::export_ics` + route → `cargo clippy -- -D warnings`.
3. Tests e2e (§7), boucler jusqu'au vert ; import manuel de vérification dans Google Calendar.
4. Frontend : `calendarApi.exportIcs` + bouton dans `CalendarView` → `npm --prefix frontend run build && npm --prefix frontend run lint`.
5. Docs : `docs/EPICS.md` (statut S04), `docs/PROGRESS.md`.

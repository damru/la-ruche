# Spécification technique — EP-014-S06

**Story :** Afficher une modale de description à la sélection d'un événement du calendrier, avec un lien vers la page cible, avant toute navigation.

**Épic :** EP-014 — Calendrier copropriété
**Réf. PRD :** `docs/product/PRD - Calendrier.md` US-CAL-006 (AC-CAL-006-A à D)
**Stories liées :** **EP-014-S01 (prérequis)** — le feed `GET /calendar/events` et le type `CalendarEvent` (cf. `EP-014-S01-spec.md` §3.1) ; **EP-014-S03 (prérequis)** — `EventDetailModal.vue` reste la fiche complète des événements manuels ; EP-014-S05 (le widget dashboard consomme le même feed — voir §5, point flagué).

**Statut actuel : NON IMPLÉMENTÉ** — le clic sur un événement navigue directement (`router.push(event.link)`), aucune modale de description n'existe.

> La revue de la story (PR #152, F1) a tranché : la description est affichée **si disponible**. Seule la branche `manual` du feed peut en porter une (table `calendar_event.description`, nullable) — les sources agrégées (AG, contrats, tâches, sinistres, carnet) n'en fournissent pas. **Option (a) retenue** : pas d'ajout de champ fallback par source à l'agrégat.

---

## 1. État actuel du code

- **Type agrégé** : `CalendarEvent` (`backend/src/domain/calendar.rs` L71-82, `frontend/src/types/domain.ts` L719-727) — champs `id`, `source`, `title`, `date`, `time`, `link`, `is_deadline`. **Pas de champ `description`.**
- **Branche manual** : `db/calendar.rs::manual_events` (L316-354) — `SELECT id, title, event_date, event_time FROM calendar_event` ; la colonne `description` (nullable, créée par S03) **n'est pas sélectionnée**.
- **Navigation directe au clic** (comportement à remplacer) :
  - `frontend/src/features/calendar/CalendarGrid.vue` L86 — `router.push(event.link)` (modes mensuel + hebdomadaire).
  - `frontend/src/features/calendar/CalendarAgendaMode.vue` L48 — `router.push(event.link)` (mode liste).
  - `frontend/src/features/dashboard/DashboardAgendaCard.vue` L36 — `router.push(event.link)` (widget S05, **hors périmètre AC — cf. §5**).
- **Modale existante** : `EventDetailModal.vue` (S03) — fiche complète d'un événement **manuel** (champs + documents joints + Modifier/Supprimer), ouverte via `?event={id}` (`CalendarView.vue` watcher L190-196). **Conservée telle quelle** : la modale S06 en est le prélude, pas le remplacement.
- **Catalogue** : `CModal` (`frontend/src/components/molecules/CModal.vue`, `docs/DESIGN.md` §3 L551) — Dialog Radix, fermeture par clic hors modale + Échap native (AC-CAL-006-D). `CBadge` pour le badge coloré par source (mêmes couleurs que la grille S01).

## 2. Modèle de données

**Aucune migration.** Un seul ajout de champ sur le DTO agrégé :

| Champ | Type | Règle |
|---|---|---|
| `CalendarEvent.description` | `Option<String>` (Rust) / `string \| null` (TS) | `Some(...)` **uniquement** pour `source = Manual` (lu depuis `calendar_event.description`) ; `None` pour toutes les sources agrégées. Jamais de fallback fabriqué par source. |

Sérialisation : le champ est présent dans le JSON (`Option` → `null`), le frontend l'affiche conditionnellement.

## 3. Backend

### 3.1 `backend/src/domain/calendar.rs`

```rust
pub struct CalendarEvent {
    // … champs existants …
    pub is_deadline: bool,
    /// Populated only for manual events (calendar_event.description).
    pub description: Option<String>,
}
```

### 3.2 `backend/src/db/calendar.rs`

- `manual_events` (L322-328) : `SELECT id, title, description, event_date, event_time FROM calendar_event …` + `description: r.try_get("description")?` dans le mapping.
- **Toutes les autres branches** (`meeting_events`, rappels contrats, tâches, sinistres, carnet, délais légaux) : `description: None` dans le struct literal.

### 3.3 API

**Aucune nouvelle route.** `GET /api/v1/calendar/events` sérialise le champ ajouté. Régénérer `backend/.sqlx/` (`cargo sqlx prepare`) après la modification de la requête.

## 4. Frontend

### 4.1 `frontend/src/features/calendar/EventPreviewModal.vue` (nouveau)

Composant **local de feature** (comme `EventFormModal.vue` / `EventDetailModal.vue`) — pas d'ajout au catalogue `docs/DESIGN.md` §2, le catalogue ne référence que les `C*` et tous les atomes/molécules requis (`CModal`, `CBadge`, `CBtn`, `RouterLink`) y figurent déjà.

Props : `open: boolean`, `event: CalendarEvent | null`. Émet `update:open`.

Contenu (AC-CAL-006-B/C/D) :

- Titre de la modale : `event.title` (prop `title` de `CModal`).
- Meta : date (+ heure si `event.time`) via `useFormatters().formatDate` — `CMetaLine`/`CMetaItem`, cohérent avec le reste de l'app.
- Badge de type : `CBadge` avec les **mêmes couleurs par source** que la grille S01 (`sources.ts`).
- Description : paragraphe affiché **uniquement si `event.description` est non null et non vide** (AC-CAL-006-B « si disponible »).
- Lien vers la page cible : `RouterLink :to="event.link"` stylisé en `CBtn variant="primary"` — libellé i18n `calendar.preview.goToEvent` (« Aller vers l'événement » / « Voir le détail » pour les manuels). **La modale ne navigue jamais d'elle-même** — la navigation passe exclusivement par ce lien.
- Fermeture sans navigation : croix `CModal` + clic hors modale + Échap (natif Radix, AC-CAL-006-D).

### 4.2 Intégration — les trois modes (AC-CAL-006-A)

Le clic sur un événement **ne fait plus `router.push`** ; il ouvre la modale avec l'événement sélectionné :

| Fichier | Changement |
|---|---|
| `CalendarGrid.vue` | `onEventClick(event)` (L86) → émet/ouvre la modale au lieu de `router.push(event.link)`. |
| `CalendarAgendaMode.vue` | idem (L48). |
| `CalendarView.vue` | détient l'état `previewEvent` + rend `<EventPreviewModal>` ; transmet un handler `open-preview` aux deux enfants (même pattern que `detailOpen`/`EventDetailModal`). |

### 4.3 Événements manuels — cible du lien

Pour `source = Manual`, `event.link = /calendar?event={id}` (S01) : le lien de la modale S06 navigue vers la fiche complète `EventDetailModal` (S03), qui affiche déjà la description, les documents et les actions Modifier/Supprimer. La modale S06 reste le passage obligé (AC-CAL-006-A s'applique à **tous** les événements) ; elle affiche la description si renseignée, puis le lien « Voir le détail » mène à la fiche.

### 4.4 i18n

Nouvelles clés `calendar.preview.*` (`goToEvent`, `goToDetail`, `description`) dans `frontend/src/i18n/locales/` (fr + en si le repo tient les deux — suivre le pattern existant).

## 5. Cas limites

| Cas | Règle |
|---|---|
| Événement agrégé sans description | Ligne description omise dans la modale — aucun placeholder vide (AC-CAL-006-B « si disponible »). |
| Événement manuel sans description (`NULL`) | Idem — omission. |
| Fermeture de la modale | `update:open=false` ; **aucune navigation**, aucun changement d'URL. |
| Clic sur le lien cible | Navigation classique `RouterLink` vers `event.link` ; la modale se ferme (déréférencement de `previewEvent`). |
| Événement supprimé entre-temps | Non applicable — la modale consomme l'objet déjà en mémoire (pas de refetch). |
| Filtres par rôle | Inchangés — le filtrage reste serveur (S01, NFR-Sec-001) ; la modale n'affiche que des événements déjà visibles. |
| `?event={id}` (lien profond S03) | Inchangé — ouvre directement `EventDetailModal`, sans passer par la modale S06 (la modale S06 concerne la **sélection dans la vue**, pas la navigation directe). |
| **Widget dashboard (S05)** | **Hors périmètre AC-CAL-006-A** (qui cible la vue calendaire : mensuel, hebdomadaire, liste). Le widget garde sa navigation directe au clic. **Flag** : à discuter avec Damien — aligner le dashboard sur la modale serait cohérent UX mais sort du périmètre de cette story. |
| Plusieurs modales ouvertes | Impossible — un seul `previewEvent` dans `CalendarView`; `EventDetailModal` et `EventPreviewModal` ne s'ouvrent pas simultanément (la preview se ferme avant navigation). |

## 6. Critères d'acceptation

- **AC-CAL-006-A** : dans les modes mensuel, hebdomadaire et liste, le clic sur un événement ouvre `EventPreviewModal` — aucune navigation directe.
- **AC-CAL-006-B** : la modale affiche titre, date, badge coloré par source, et la description **si disponible**.
- **AC-CAL-006-C** : la modale propose un lien explicite vers la page cible (`event.link` — module source pour les agrégés, fiche `Event` S03 pour les manuels).
- **AC-CAL-006-D** : fermeture sans navigation par bouton, clic hors modale ou Échap (comportement natif `CModal`/Radix).
- Le feed `GET /calendar/events` expose `description` (null hors branche manual) — vérifié par test e2e.

## 7. Plan de tests

- **Unitaires domaine** (`domain/calendar.rs`) : ajuster les struct literals existants au nouveau champ ; ajouter un test `manual event carries description` sur le mapping (None → Some, trim non appliqué — valeur brute).
- **E2E** (`backend/tests/api_e2e_test.rs`) :
  1. Créer un `calendar_event` avec `description` + un sans → `GET /calendar/events` renvoie `description` renseigné pour le premier, `null` pour le second et pour un événement AG de la même plage.
  2. Rôle CoOwner : la réponse ne contient toujours que `meeting` + `manual` (pas de régression de visibilité avec le nouveau champ).
- **Frontend** : pas de `vitest` configuré dans le repo — pas de tests composant (cohérent avec S01 §7). Vérification manuelle des 3 modes + fermeture Échap/clic extérieur.

## 8. Ordre d'implémentation

1. Backend : champ `description` sur `CalendarEvent` → `SELECT description` dans `manual_events` + `None` ailleurs → tests unitaires → `cargo test --lib`.
2. `cargo sqlx prepare` (commit `backend/.sqlx/`) → `cargo clippy -- -D warnings` → tests e2e §7 → vert.
3. Frontend : `CalendarEvent.description` dans `types/domain.ts` → `EventPreviewModal.vue` → intégration `CalendarGrid` + `CalendarAgendaMode` + `CalendarView` → i18n.
4. `npm --prefix frontend run build && npm --prefix frontend run lint`.
5. Docs à la livraison : `docs/EPICS.md` (S06 → ⏳ puis ✅), `Statut actuel` de cette spec → `IMPLÉMENTÉ (backend + frontend)`.

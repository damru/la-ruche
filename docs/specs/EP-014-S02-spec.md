# Spécification technique — EP-014-S02

**Story :** Filtrer les événements du calendrier par type (AG, tâche, sinistre, contrat, autre) et par période.

**Épic :** EP-014 — Calendrier copropriété
**Réf. PRD :** `docs/product/PRD - Calendrier.md` US-CAL-002 (AC-CAL-002-A/B)
**Stories liées :** **EP-014-S01 (prérequis)** — cette story enrichit le feed `GET /calendar/events` et la `CalendarView` définis dans `EP-014-S01-spec.md` ; EP-014-S04 (l'export ICS réutilise le paramètre `types`).

---

## 1. État actuel du code

- Rien n'existe : le feed et la vue sont livrés par S01 (cf. `EP-014-S01-spec.md` §3-4). Cette spec suppose S01 terminée.
- Le filtrage **par période** existe déjà de fait via `?from&to` (S01) — la navigation mensuelle/hebdo pilote la plage. Cette story ajoute le filtrage **par type** et la **persistance** de l'état des filtres.
- Le repo n'a pas de mécanisme de préférences utilisateur côté serveur (pas de table `user_preference`) — voir décision §3.3.

## 2. Modèle de données

**Aucune migration** (cf. décision §3.3 : persistance des filtres en `localStorage`, pas en base).

## 3. Backend

### 3.1 Paramètre `types` sur le feed

`GET /api/v1/calendar/events?from&to&types=meeting,contract,task,claim,maintenance,manual`

- `types` optionnel : absent → toutes les sources visibles pour le rôle (comportement S01 inchangé).
- Parsing dans `api/calendar.rs` (désérialisation CSV → `Vec<CalendarSource>`), valeur inconnue → `400 errors.calendar.invalid_type`.
- **L'intersection avec le rôle reste souveraine** : dans `domain/calendar.rs`, fonction pure
  ```rust
  pub fn effective_sources(roles: &[Role], requested: Option<&[CalendarSource]>) -> Vec<CalendarSource>
  ```
  = `visible_sources(roles) ∩ requested`. Un CoOwner demandant `types=contract` reçoit `200` avec une liste vide — jamais d'élargissement de visibilité (NFR-Sec-001).
- `db/calendar.rs::events` prend déjà `sources: &[CalendarSource]` (S01) — aucun changement DB.

### 3.2 Correspondance types EPICS ↔ sources

Les 5 types de l'EPIC (« AG, tâche, sinistre, contrat, autre ») se projettent sur les 6 `CalendarSource` de S01 : `meeting`, `task`, `claim`, `contract`, et « autre » = `maintenance` + `manual`. L'API reste au grain fin (6 sources) ; le regroupement « autre » est une affaire de présentation frontend.

### 3.3 Persistance des filtres — décision

AC-CAL-002-B demande une persistance « dans les préférences utilisateur ». **Recommandation : `localStorage`** (clé `laruche.calendar.filters.<userId>`), pas de table serveur :

- Aucun autre écran du repo ne persiste de préférences serveur — créer une table `user_preference` pour un seul cas d'usage serait spéculatif.
- Perte acceptée : les filtres ne suivent pas l'utilisateur entre appareils. **Flag** : si ce besoin multi-appareils est confirmé par Damien, une story dédiée introduira `user_preference` (avec passage par `docs/ARCHITECTURE.md` + migration) — hors périmètre ici.

## 4. Frontend

- `frontend/src/api/index.ts` : `calendarApi.events(from, to, types?)` — ajoute le paramètre optionnel.
- `frontend/src/features/calendar/CalendarFilterBar.vue` (nouveau, à cataloguer `docs/DESIGN.md §2`) : rangée de toggles (chips) par type — AG, Contrats, Tâches, Sinistres, Carnet, Événements — chaque chip reprenant la couleur de badge de sa source (cohérence AC-CAL-001-B). Seuls les types **visibles pour le rôle courant** sont affichés (un Tenant ne voit que AG / Événements).
- `CalendarView.vue` : intègre la barre de filtres ; l'état (types actifs) est un `ref` initialisé depuis `localStorage`, sauvegardé à chaque changement (`watch`), et injecté dans l'appel `calendarApi.events`.
- Filtrage par période : inchangé — la navigation de S01 (mois/semaine, mode liste borné) reste le mécanisme ; en mode liste, ajouter deux champs date `from`/`to` libres.
- i18n : clés `calendar.filters.*`.

## 5. Cas limites

| Cas | Règle |
|---|---|
| `types` vide (`types=`) | Traité comme absent → toutes les sources visibles. |
| Type inconnu (`types=foo`) | `400 errors.calendar.invalid_type`. |
| Type demandé hors rôle | Silencieusement exclu par `effective_sources` — `200`, jamais `403` (le calendrier reste utilisable). |
| Tous les filtres désactivés côté UI | Grille vide + état explicite « aucun type sélectionné » (pas d'appel réseau inutile : court-circuit frontend). |
| `localStorage` indisponible/corrompu | Fallback silencieux : tous les types actifs. |
| Changement d'utilisateur sur le même navigateur | Clé par `userId` → pas de fuite de préférences entre comptes. |

## 6. Critères d'acceptation

- **AC-CAL-002-A** : filtres disponibles selon rôle — AG, Contrats, Tâches, Sinistres, Carnet d'entretien, Événements de copropriété ; un rôle ne voit que ses filtres autorisés.
- **AC-CAL-002-B** : l'état des filtres survit à un rechargement de page et à une nouvelle session sur le même appareil (limite `localStorage` documentée §3.3).
- Le filtrage serveur ne peut jamais élargir la visibilité au-delà de `visible_sources(roles)`.

## 7. Plan de tests

- **Unitaires domaine** (`domain/calendar.rs`) : `effective_sources` — intersection stricte ; `requested = None` → identité ; type hors rôle → exclu.
- **E2E** (`backend/tests/api_e2e_test.rs`) :
  1. Manager, `types=meeting` → seuls les événements AG.
  2. Manager, `types=meeting,contract` → union des deux.
  3. CoOwner, `types=contract` → `200`, liste vide.
  4. `types=foo` → `400 errors.calendar.invalid_type`.
  5. Sans `types` → comportement S01 inchangé (non-régression).
- **Frontend** (si `vitest`) : `CalendarFilterBar` — toggle → émission ; restauration depuis `localStorage` mocké.

## 8. Ordre d'implémentation

1. `domain/calendar.rs` : `effective_sources` + tests unitaires → `cargo test --lib`.
2. `api/calendar.rs` : parsing `types` + branchement → `cargo clippy -- -D warnings`.
3. Tests e2e (§7), boucler jusqu'au vert.
4. Frontend : `calendarApi` (param `types`) → `CalendarFilterBar.vue` (catalogue d'abord) → intégration `CalendarView` + persistance `localStorage` → i18n.
5. `npm --prefix frontend run build && npm --prefix frontend run lint`.
6. Docs : `docs/EPICS.md` (statut S02), `docs/PROGRESS.md`.

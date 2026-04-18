# Spécification technique — EP-014-S05

**Story :** Afficher une vue agenda (prochains 30 jours) depuis le tableau de bord, avec les 5 prochains événements.

**Épic :** EP-014 — Calendrier copropriété
**Réf. PRD :** `docs/product/PRD - Calendrier.md` US-CAL-004 (Could — vue agenda 30 j ; KPI : identifier une échéance entrante en < 10 s depuis la page d'accueil)
**Stories liées :** **EP-014-S01 (prérequis)** — le widget consomme tel quel le feed `GET /calendar/events` (cf. `EP-014-S01-spec.md` §3.3) ; EP-014-S03 (les événements manuels y apparaissent) ; EP-014-S02/S04 non requis.

---

## 1. État actuel du code

- `frontend/src/features/dashboard/DashboardView.vue` : aucun widget agenda. Le dashboard agrège déjà 8 appels (`buildingApi.stats`, `budgetApi.list`, `meetingsApi`, `tasksApi`, `reportsApi`, `contractsApi`, rappels, consultations) via `Promise.allSettled` avec fallback `valueOr`, et affiche des blocs `CCard` (rappels contrats triés/limités à 4, `priorityTasks` limités à 4, `nextMeeting`…).
- Ce widget **remplace avantageusement** deux calculs partiels existants (`reminders` slice(0,4) et `nextMeeting`) par une vue unifiée — mais on **ne les retire pas** dans cette story (changement de scope UX → décision Damien, cf. §5 point flaggé).
- Backend : rien à créer — `GET /calendar/events?from&to` (S01) filtre déjà par rôle côté serveur ; le tri `(date, time)` est garanti par S01.

## 2. Modèle de données

**Aucune migration.**

## 3. Backend

**Aucun changement.** Le widget appelle `GET /api/v1/calendar/events?from={today}&to={today+30j}` et tronque à 5 côté client. Pas de paramètre `limit` ajouté à l'endpoint : le volume sur 30 jours est trivial pour une copropriété, et un `limit` serveur serait de la configurabilité spéculative. Si un jour la mesure prouve le contraire, l'ajout est rétro-compatible.

## 4. Frontend

### 4.1 `frontend/src/features/dashboard/DashboardAgendaCard.vue` (nouveau)

À cataloguer dans `docs/DESIGN.md §2` avant usage (ou à ranger comme composant local de feature si le catalogue ne référence que les `C*` — suivre la pratique du catalogue existant).

- Fetch au montage : `calendarApi.events(today, today + 30j)` — intégré au `Promise.allSettled` de `loadData()` dans `DashboardView.vue` (même pattern `valueOr`, le dashboard ne casse pas si le calendrier échoue).
- Affichage : `CCard` titré « Agenda — 30 prochains jours », liste des **5 premiers** événements : date (format relatif via `useFormatters().formatRelative` / `daysUntil`, cohérent avec le reste du dashboard), badge coloré par source (mêmes couleurs que la grille S01 — AC-CAL-001-B), titre, clic → `router.push(event.link)`.
- Pied de carte : lien « Voir le calendrier » → `/calendar` (raccourci principal vers S01).
- État vide : « Aucun événement dans les 30 prochains jours » + lien vers le calendrier.
- Le filtrage par rôle est serveur (S01) : le même widget sert les 4 rôles sans branche conditionnelle — un CoOwner y voit ses AG et les événements manuels, un Manager tout.

### 4.2 `DashboardView.vue`

- Ajouter l'appel au `Promise.allSettled` + `ref` d'état ; insérer `DashboardAgendaCard` dans la grille de cartes existante (position : colonne latérale, au-dessus des rappels contrats — l'agenda est la synthèse, les rappels le détail).
- i18n : clés `dashboard.agenda.*`.

## 5. Cas limites

| Cas | Règle |
|---|---|
| Aucun événement sur 30 jours | État vide explicite (§4.1), pas de carte masquée — le lien vers le calendrier reste. |
| Plus de 5 événements | Troncature à 5 + mention « +N autres » liant vers `/calendar`. |
| Échec de l'appel calendrier | `valueOr(..., [])` → carte en état vide, le reste du dashboard vit sa vie (pattern existant). |
| Plusieurs événements aujourd'hui | Inclus (borne `from = today` inclusive) ; les événements d'aujourd'hui déjà passés en heure restent listés (simplicité — granularité jour). |
| CoOwner / Tenant | Même composant, contenu réduit par le serveur (NFR-Sec-001) — jamais de logique de rôle dupliquée côté widget. |
| **Redondance avec les blocs existants** (rappels contrats, prochaine AG) | Conservés en l'état dans cette story. **Flag** : proposer à Damien, après livraison, la fusion/suppression de ces blocs au profit de l'agenda — décision UX séparée (DDR si structurelle). |

## 6. Critères d'acceptation

- Le tableau de bord affiche les 5 prochains événements des 30 jours à venir, triés chronologiquement, avec badge de source et lien contextuel.
- Un clic sur un événement mène au module source (ou au détail de l'événement manuel) ; un lien mène au calendrier complet.
- Le contenu respecte le rôle de l'utilisateur (filtrage serveur S01).
- KPI PRD : depuis la page d'accueil, une échéance entrante est identifiable sans navigation supplémentaire (< 10 s).
- L'échec de l'appel calendrier ne dégrade pas le reste du dashboard.

## 7. Plan de tests

- **Backend** : aucun nouveau test (endpoint couvert par S01 §7) ; vérifier seulement qu'un test e2e S01 couvre la plage « aujourd'hui → +30 j » avec borne inclusive — sinon l'ajouter.
- **Frontend** (si `vitest` configuré) :
  1. `DashboardAgendaCard` avec 7 événements mockés → 5 rendus + « +2 autres ».
  2. Liste vide → état vide + lien `/calendar`.
  3. Clic sur un événement → navigation vers `event.link` (router mocké).
- Sinon : vérification manuelle des trois états (rempli, tronqué, vide) sur les 4 rôles en local.

## 8. Ordre d'implémentation

1. Cataloguer `DashboardAgendaCard` (`docs/DESIGN.md §2`, si applicable §4.1).
2. Créer `DashboardAgendaCard.vue` (fetch délégué au parent, props `events` + `loading` — cohérent avec les cartes existantes).
3. Intégrer dans `DashboardView.vue` (`Promise.allSettled` + grille) + i18n.
4. Tests frontend (§7) ou passe manuelle 4 rôles.
5. `npm --prefix frontend run build && npm --prefix frontend run lint`.
6. Docs : `docs/EPICS.md` (statut S05), `docs/PROGRESS.md` (entrée datée) ; ouvrir la discussion « fusion des blocs redondants » (§5).

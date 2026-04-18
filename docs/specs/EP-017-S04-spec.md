# Spécification technique — EP-017-S04

**Story :** Frontend : vue `TaskTimelineView` remplaçant `TaskDetailView` — données statiques de la tâche + timeline événementielle depuis le signalement source jusqu'à la clôture.

**Épic :** EP-017 — Journal d'activité & Suivi de cycle de vie
**Réf. PRD :** Gestion courante (US-TP-015) · **Réf. EPICS :** `docs/EPICS.md` EP-017-S04 (L512)
**Stories liées :** EP-017-S03 (endpoint + `tasksApi.timeline`, prérequis), EP-017-S05 (pendant côté signalements — mêmes composants de rendu), EP-017-S06 (le backend filtre déjà les actions sensibles ; aucune logique de rôle à dupliquer ici)

---

## 1. Quoi

Remplacer la timeline **dérivée côté client** de `TaskDetailView.vue` (computed reconstituant des étapes depuis les champs de la tâche, plusieurs étapes non datées) par la timeline **événementielle réelle** servie par `GET /tasks/{id}/timeline`. La vue est renommée `TaskTimelineView.vue` (prescription EPICS) ; les blocs statiques (titre, statut, priorité, assignataire, prestataire, documents) sont conservés tels quels.

## 2. État actuel du code

| Élément | Emplacement |
|---|---|
| Vue actuelle | `frontend/src/features/tasks/TaskDetailView.vue` — computed `timelineEvents` L81-175 : reconstruit `created` / assignation (non datée) / `awaiting_vendor` / soumission / `closed` depuis les champs de la tâche ; helpers `dotClass`, interface `TimelineEvent` locale |
| Routage | `frontend/src/router/index.ts` L105-108 : `tasks/:id` → `TaskDetailView`, `roles: ['manager', 'board_member']` |
| Client API | `frontend/src/api/index.ts` `tasksApi` (L322) — méthode `timeline` livrée par EP-017-S03 §5 |
| Type partagé | `TimelineEvent` dans `frontend/src/types/domain.ts` (EP-017-S03) |
| Catalogue composants | `docs/DESIGN.md` §2 — tout nouveau composant y est ajouté avant usage |

## 3. Modèle de données

Aucun (frontend pur).

## 4. Backend

Aucun changement — consommateur de EP-017-S03.

## 5. Frontend Vue

### 5.1 Renommage `TaskDetailView.vue` → `TaskTimelineView.vue`

- `git mv` + mise à jour de `router/index.ts` (import L17, `component` L107). Route, path et guards de rôle inchangés.
- Supprimer : l'interface locale `TimelineEvent`, le computed `timelineEvents` (L81-175) et les imports d'icônes devenus inutiles. Conserver `dotClass`/`statusVariant`/`priorityVariant` si réutilisés par le nouveau rendu.

### 5.2 Chargement de la timeline

Au montage (avec le fetch existant de la tâche) : `tasksApi.timeline(route.params.id)` → `ref<TimelineEvent[]>`. États : chargement (squelette), erreur (message + réessayer), vide (`200 []` — message « Aucun événement enregistré », cas des tâches antérieures à EP-017, cf. S03 §6).

### 5.3 Nouveau composant `TimelineEventList.vue` (`frontend/src/features/tasks/` ou catalogue si promu)

Composant de rendu **partagé avec EP-017-S05** (à cataloguer dans `docs/DESIGN.md` §2 avant usage) :

- Props : `events: TimelineEvent[]`.
- Rendu vertical chronologique : pastille (variant par action), libellé i18n, horodatage formaté, acteur (`actor_name` ou « Prestataire externe » quand null sur les événements vendeur), détail issu du `payload`.
- Mapping action → libellé/pastille (clés i18n `timeline.actions.*`, fichiers `frontend/src/locales/`) :

| `action` | Libellé (fr) | Détail depuis `payload` |
|---|---|---|
| `created` (report) | « Signalement créé » | — |
| `created` (task) | « Tâche créée » | « issue du signalement » si `source_report_id` |
| `status_changed` | « Statut modifié » | `old_status → new_status` (libellés statuts existants) |
| `photo_added` | « Photo ajoutée » | `file_name` |
| `converted_to_task` | « Converti en tâche » | lien `tasks/:task_id` |
| `assignee_changed` | « Assignataire modifié » | — (ids résolus non exigés en v1) |
| `contractor_assigned` | « Prestataire assigné » | `contractor_name` |
| `vendor_link_generated` | « Lien prestataire généré » | expiration |
| `document_uploaded` | « Document joint » | `file_name` |
| `vendor_submission_received` | « Intervention prestataire reçue » | — |
| `closed` | « Clôturé(e) » | — |

- Les événements `entity_type === 'report'` sont visuellement distingués (pastille secondaire + mention « Signalement ») : la frise couvre la chaîne complète depuis le signalement source.
- Action inconnue (évolution future du vocabulaire) : rendu générique avec l'`action` brute — pas de crash.

### 5.4 Contraintes de style

Règles AGENTS.md : pas de valeurs brutes (`px`/hex) — utilitaires Tailwind/tokens ; pas de bloc `<style>` dans la feature view ; pas d'import `radix-vue` direct (wrappers `C*`). Le CSS timeline existant de `TaskDetailView` (classes `timeline-dot--*`) est repris/adapté dans `TimelineEventList.vue` en respectant ces règles.

## 6. Cas limites

| Cas | Règle |
|---|---|
| Tâche antérieure à EP-017 (timeline vide) | Message vide dédié — **ne pas** réafficher l'ancienne timeline dérivée en fallback (elle est supprimée). |
| Timeline en erreur mais tâche chargée | Blocs statiques affichés, section timeline en erreur isolée. |
| Événements sensibles absents (filtrés par S06 selon le rôle) | Aucun traitement : la vue affiche ce que l'API renvoie. |
| Tâche sans signalement source | Frise purement tâche — aucune mention signalement. |

## 7. Critères d'acceptation

1. `TaskDetailView.vue` n'existe plus ; la route `tasks/:id` rend `TaskTimelineView.vue` sans régression des blocs statiques (titre, statut, priorité, assignataire, prestataire, documents, actions).
2. La timeline affichée provient exclusivement de `GET /tasks/{id}/timeline` (aucune étape reconstruite côté client, plus d'étape « non datée »).
3. Une tâche issue d'un signalement affiche les événements du signalement en tête de frise, avec distinction visuelle.
4. Toutes les actions du tableau §5.3 ont un libellé i18n (fr) ; action inconnue → rendu générique.
5. `npm --prefix frontend run build` et `run lint` passent ; `TimelineEventList` catalogué dans `docs/DESIGN.md` §2.

## 8. Plan de tests

Pas de suite de tests composants Vue établie pour `features/tasks/` — vérification manuelle guidée :
1. Tâche créée directement : frise `created` → mutations → `closed`.
2. Signalement converti : frise complète signalement + tâche (parcours croisé avec EP-017-S05).
3. Tâche ancienne sans événements : état vide.
4. Erreur réseau sur la timeline : blocs statiques intacts, erreur localisée.
Si `vitest` est disponible, un test de `TimelineEventList` (mapping action → libellé, événement inconnu) suffit.

## 9. Ordre d'implémentation

1. `TimelineEventList.vue` + entrée `docs/DESIGN.md` §2 + clés i18n.
2. `git mv TaskDetailView.vue TaskTimelineView.vue`, purge du computed L81-175, branchement `tasksApi.timeline`.
3. `router/index.ts` (import/composant).
4. `npm --prefix frontend run build && run lint`, vérification manuelle §8.
5. `docs/EPICS.md` (statut S04), `docs/PROGRESS.md`.

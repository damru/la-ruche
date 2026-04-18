# Spécification technique — EP-017-S05

**Story :** Frontend : vue `ReportDetailView` — événements auditables du signalement + lien contextuel vers la tâche créée si conversion effective.

**Épic :** EP-017 — Journal d'activité & Suivi de cycle de vie
**Réf. PRD :** Gestion courante (US-JA-*) · **Réf. EPICS :** `docs/EPICS.md` EP-017-S05 (L513)
**Stories liées :** EP-017-S03 (`reportsApi.timeline` + champ `task_id`, prérequis), EP-017-S04 (composant partagé `TimelineEventList.vue`, prérequis), EP-017-S06 (filtrage des actions sensibles côté backend — les résidents ne verront jamais `assignee_changed`/`vendor_link_generated`)

---

## 1. Quoi

Créer la page de détail d'un signalement — elle **n'existe pas** aujourd'hui (seule `ReportsView.vue`, la liste, existe). Contenu : fiche statique du signalement (titre, description, statut, auteur, photos/documents) + frise des événements via `GET /reports/{id}/timeline` + lien vers la tâche créée quand `report.task_id` est renseigné.

## 2. État actuel du code

| Élément | Emplacement |
|---|---|
| Liste des signalements (seule vue existante) | `frontend/src/features/reports/ReportsView.vue` — aucune `ReportDetailView.vue` |
| Routage | `frontend/src/router/index.ts` L147-150 : route `reports` (`roles: []` — tous rôles authentifiés) ; pas de route `reports/:id` |
| Client API | `frontend/src/api/index.ts` `reportsApi` (L426) — méthode `timeline` livrée par EP-017-S03 §5 |
| Champ de conversion | `report.task_id` (backend `db/report.rs` L21, exposé dans les réponses `GET /reports*` avec `task_status` joint) — cf. arbitrage de nommage EP-017-S03 §1 |
| Composant timeline partagé | `TimelineEventList.vue` (EP-017-S04 §5.3) + clés i18n `timeline.actions.*` |
| Route détail tâche | `tasks/:id` (`TaskTimelineView`, roles manager/board_member — EP-017-S04) |

## 3. Modèle de données

Aucun.

## 4. Backend

Aucun changement — consommateur de EP-017-S03 (endpoint) et EP-017-S06 (filtrage par rôle, transparent pour cette vue).

## 5. Frontend Vue

### 5.1 Nouvelle vue `frontend/src/features/reports/ReportDetailView.vue`

- Route : `reports/:id`, name `report-detail`, `meta: { title: 'nav.reports', roles: [] }` — mêmes rôles que la liste : un résident consulte le détail de ses signalements (la restriction fine des événements est faite par le backend, EP-017-S06).
- Chargement au montage : `reportsApi.get(id)` (fiche) + `reportsApi.timeline(id)` (frise), en parallèle. 404 → redirection liste avec toast, pattern des autres vues détail du repo.
- Sections :
  1. **Fiche statique** : titre, description, badge statut (réutiliser les variantes de `ReportsView`), auteur, date de création, documents/photos attachés (`reportsApi.listDocuments` si exposé, sinon les données de `get`).
  2. **Lien conversion** : si `report.task_id` renseigné, encart « Converti en tâche » avec badge `task_status` et `RouterLink` vers `tasks/:task_id`. Le lien n'est rendu **que** si le rôle courant peut voir la route cible (manager/board_member — lire le rôle depuis le store auth existant) ; pour un résident, l'encart affiche la mention et le statut de la tâche **sans lien**.
  3. **Timeline** : `<TimelineEventList :events="events" />` (composant partagé EP-017-S04) — affiche la chaîne complète renvoyée par l'endpoint (événements signalement **et** tâche), la distinction visuelle par `entity_type` étant déjà gérée par le composant.
- Contraintes de style AGENTS.md : Tailwind/tokens uniquement, pas de `<style>`, wrappers `C*` — identiques à EP-017-S04 §5.4.

### 5.2 `ReportsView.vue` — navigation vers le détail

Rendre chaque carte/ligne de signalement cliquable vers `report-detail` (RouterLink). Aucune autre modification de la liste.

### 5.3 Catalogue

`ReportDetailView` est une feature view (pas un composant catalogue). Si un sous-composant réutilisable est extrait (ex. encart de conversion), l'ajouter à `docs/DESIGN.md` §2 avant usage ; sinon aucun ajout — `TimelineEventList` est déjà catalogué par EP-017-S04.

## 6. Cas limites

| Cas | Règle |
|---|---|
| Signalement non converti | Pas d'encart conversion ; frise = événements signalement seuls (S03 §6). |
| Signalement antérieur à EP-017 (frise vide) | Fiche statique complète + état vide de la frise (même message que S04 §6). |
| Résident consultant un signalement converti | Encart conversion sans lien (rôle insuffisant pour `tasks/:id`) ; événements sensibles absents de la réponse API (S06). |
| Tâche liée supprimée | Le backend renvoie `task_id` avec `task_status` null (LEFT JOIN) — l'encart se rabat sur la mention sans badge ni lien. |
| Erreur réseau sur la frise seule | Fiche affichée, section frise en erreur isolée (pattern S04 §6). |

## 7. Critères d'acceptation

1. Route `reports/:id` accessible à tout rôle authentifié ; navigation depuis la liste opérationnelle.
2. La frise reflète exactement `GET /reports/{id}/timeline` (aucune donnée reconstruite côté client).
3. Signalement converti : encart tâche avec statut ; lien actif uniquement pour manager/board_member.
4. Toutes les chaînes en i18n (fr) ; `npm --prefix frontend run build` et `run lint` passent.

## 8. Plan de tests

Vérification manuelle guidée (pas de suite composants Vue établie, cf. EP-017-S04 §8) :
1. Résident : création signalement → détail → frise `created` (+ `photo_added` après ajout de photo) ; aucun événement sensible visible.
2. Manager : conversion → l'encart apparaît, lien vers la tâche fonctionnel, frise étendue aux événements tâche.
3. Parcours croisé avec EP-017-S04 : la même chaîne se lit depuis `reports/:id` et `tasks/:id`.
4. URL directe vers un signalement d'un autre immeuble / inexistant : 404 backend → redirection liste.
Si `vitest` est disponible : test de rendu conditionnel de l'encart conversion (avec/sans `task_id`, avec/sans rôle gestionnaire).

## 9. Ordre d'implémentation

1. Route `reports/:id` + squelette `ReportDetailView.vue` (fiche statique via `reportsApi.get`).
2. Branchement `reportsApi.timeline` + `TimelineEventList` + états chargement/vide/erreur.
3. Encart conversion conditionné par `task_id` et le rôle ; clic-through depuis `ReportsView.vue`.
4. `npm --prefix frontend run build && run lint`, vérification manuelle §8.
5. `docs/EPICS.md` (statut S05), `docs/PROGRESS.md`.

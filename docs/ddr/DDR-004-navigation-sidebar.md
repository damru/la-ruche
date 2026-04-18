# DDR-004 — Restructuration de la navigation sidebar

**Date :** 2026-04-21  
**Statut :** Accepté  
**Auteur :** Équipe produit

---

## Contexte

La sidebar desktop comptait 13 entrées à plat, rendant le menu difficile à scanner visuellement à mesure que le produit grossissait. Les items liés au bâtiment et aux résidents étaient éparpillés entre deux entrées distinctes ("Paramètres" et "Résidents"). Les actions personnelles (compte, notifications) étaient enfouies dans un onglet de Settings partagé avec la configuration de la copropriété.

---

## Décision

### 1. Groupement en sections visuelles (Pattern B — labels fixes)

La sidebar est restructurée en 6 sections avec un label texte non-cliquable séparant chaque groupe. Tous les items restent toujours visibles (pas d'accordion, pas de collapse) :

| Groupe | Items |
|---|---|
| *(solo)* | Tableau de bord |
| Gouvernance | Assemblées générales · Consultations · Rapports |
| Finance | Budget & Charges · Contrats |
| Opérations | Tâches · Sinistres · Carnet d'entretien · Documents |
| Résidence | Bâtiment · Clés de répartition · Registre · Copropriétaires |

### 2. Suppression de l'entrée "Paramètres"

L'ancienne page `/settings` (4 onglets) est décomposée en routes distinctes :
- `/residence/building` → **Bâtiment** (infos copropriété, données légales)
- `/residence/keys` → **Clés de répartition** (CRUD des clés + tantièmes)
- `/settings` → redirect permanent vers `/residence/building`

### 3. Profil utilisateur dans le footer sidebar

Les onglets "Notifications" et "Mon compte" — anciennement dans Settings — migrent dans un menu déroulant attaché à la zone `c-sidebar__user` (bas de la sidebar). Clic sur le bloc utilisateur → panel contextuel avec :
- Mon compte → `/profile/account`
- Notifications → `/profile/notifications`
- Se déconnecter (action directe)

---

## Alternatives écartées

| Alternative | Raison |
|---|---|
| **Accordion** (Pattern A) | Cache les items, interrompt le scan visuel, surcharge d'interaction |
| **Page parente par groupe** (Pattern C) | Surcoût navigation pour des groupes qui n'ont pas de vue d'ensemble propre |
| **Garder `/settings`** avec onglets Bâtiment/Clés | URL et label incohérents : le bâtiment n'est pas un "paramètre système" |
| **Profil dans une page dédiée sans raccourci sidebar** | Accès à "Mon compte" trop enfoui pour un usage fréquent |

---

## Conséquences

- `AppNav.vue` : structure de données `NavItem[]` → `NavGroup[]` (label + items)
- `router/index.ts` : 4 nouvelles routes (`/residence/building`, `/residence/keys`, `/profile/account`, `/profile/notifications`), redirect `/settings`
- `features/settings/SettingsView.vue` : dépréciée, conservée en redirect au niveau routeur
- `AppLayout.vue` : slots icônes mis à jour (suppresson `icon-settings`, ajout `icon-residence-building`, `icon-residence-keys`, `icon-registry`)
- Navigation mobile (bottom nav) : inchangée — les 4 items prioritaires restent dérivés du même tableau d'items

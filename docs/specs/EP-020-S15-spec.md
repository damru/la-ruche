# Spécification technique — EP-020-S15

**Story :** Créer la molécule `CCardHeader`, première ligne de l'ossature de carte d'entité (icône métier + titre + chips), et migrer les en-têtes de cartes assemblés à la main dans les vues features.

**Épic :** EP-020 — Design system
**Réf. DESIGN :** `docs/DESIGN.md` §3.2 (ligne `CCardHeader` — 🔲 planifié), §3.2 ligne `CCard` (« Ossature d'une carte d'entité : `CCardHeader` › titre › description › `CMetaLine` › `CCardActions` › `CCardNote` »), §4.1 « Cartes de liste » (« `CCardHeader` — chips de statut / type toujours visibles »), §4.2 règles par feature (carte contrat / signalement / tâche / carnet).

> Frontend uniquement.

---

## 1. État actuel

Chaque vue liste assemble l'en-tête de ses cartes à la main, avec des variantes divergentes :

- `frontend/src/features/contracts/ContractsView.vue:455-470` — `div.flex.items-center.gap-3.min-w-0` › `CIconTile` (tone métier) › `h3.text-lg.font-semibold…` › `div.flex.flex-wrap.gap-x-2.gap-y-1` de `CBadge`.
- `frontend/src/features/claims/ClaimsView.vue:385-414` — `CIconTile` + `CMetaLine.justify-end` de 3 `CBadge` sur une ligne, titre `h3` sur la ligne suivante (ordre inversé par rapport aux contrats).
- `frontend/src/features/maintenance/MaintenanceView.vue:285-320` — détourne `CSectionHeader` avec `span.title-sm` + badge dans `#actions` (usage hors contrat de `CSectionHeader`, qui est un en-tête de *section*, pas de carte).
- Autres occurrences du même pattern : cartes de `TasksView.vue`, `ConsultationsView.vue`, `ContractorsView.vue`, `ReportsView.vue` (icône/chips/titre combinés en flex ad hoc).

Conséquences : ordre icône/titre/chips non uniforme, retours à la ligne des chips gérés différemment, `min-width: 0` oublié par endroits (titres qui poussent les chips hors carte).

## 2. API du composant

`frontend/src/components/molecules/CCardHeader.vue` :

```ts
withDefaults(defineProps<{
  withIcon?: boolean   // réserve la colonne icône (slot #icon) — défaut false
}>(), {
  withIcon: false,
})
```

Slots :

- `#icon` — contenu icône, typiquement `CIconTile` + SVG métier (rendu seulement si `withIcon`).
- défaut — zone titre : le `h3` (ou `CHeading type="card"`) fourni par le parent.
- `#chips` — chips de statut / type (`CBadge`), toujours visibles, alignées à droite.

Aucun emit.

## 3. Comportement / design

- Layout : `display: flex; align-items: center; gap: var(--space-3); min-width: 0`. Zone titre `flex: 1 1 auto; min-width: 0` (troncature propre) ; `#icon` et `#chips` `flex-shrink: 0` — sauf wrap : la zone chips passe en `flex-wrap: wrap; gap: var(--space-1) var(--space-2); justify-content: flex-end`.
- Le composant possède le **layout** uniquement ; le style texte du titre reste au parent via Tailwind (`text-lg font-semibold text-on-surface m-0 leading-snug`, jamais `font-display` — DESIGN §4.1 « titres en font-body text-lg »).
- « Chips toujours visibles » (DESIGN §4.1) : les chips ne tronquent pas et ne disparaissent pas ; en manque de place, elles passent à la ligne sous le titre (media query mobile ou `flex-wrap` naturel).
- Tokens uniquement (`--space-*`) ; pas de `<style>` avec valeurs brutes ; aucun import radix-vue.
- A11y : composant purement structurel — la sémantique (`h3`) reste dans le slot défaut ; les icônes fournies dans `#icon` doivent être `aria-hidden="true"` (responsabilité de l'appelant, à rappeler dans le catalogue).
- Variantes feature (DESIGN §4.2) exprimées par composition, pas par props : contrat = `#icon` + titre + chip statut ; signalement = chips type+statut+délai ; tâche = chips priorité+statut sans icône ; carnet = icône colorée par type + chip type + chip statut.

## 4. Plan de migration

1. Vue pilote : `ContractsView.vue` (carte contrat — cas `withIcon` + chips, DESIGN §4.2 « Carte contrat »).
2. `ClaimsView.vue` (ré-aligne l'ordre chips/titre sur l'ossature canonique — écart visuel à valider avec le user avant merge, la maquette actuelle met les chips au-dessus du titre).
3. `TasksView.vue` (sans icône), `MaintenanceView.vue` (remplace le détournement de `CSectionHeader` dans les cartes), `ConsultationsView.vue`, `ContractorsView.vue`, `ReportsView.vue`.
4. Les cartes qui ne suivent pas l'ossature d'entité (KPI, cartes légales, `CReminderCard`) sont hors périmètre.

## 5. Critères d'acceptation

- [ ] `CCardHeader.vue` existe dans `frontend/src/components/molecules/`, typé TS, tokens uniquement.
- [ ] Titre long : ellipse sans pousser les chips hors de la carte (`min-width: 0` vérifié à 360px de large).
- [ ] Chips toujours visibles — wrap sous le titre en manque de place, jamais tronquées.
- [ ] `withIcon` réserve la colonne icône ; sans `withIcon`, aucun espace fantôme à gauche.
- [ ] `ContractsView`, `ClaimsView`, `TasksView`, `MaintenanceView` (cartes), `ConsultationsView`, `ContractorsView`, `ReportsView` utilisent `CCardHeader` ; plus de flex ad hoc icône/titre/chips dans ces cartes.
- [ ] `MaintenanceView` n'utilise plus `CSectionHeader` à l'intérieur des cartes d'entité.
- [ ] `docs/DESIGN.md` §3.2 : ligne `CCardHeader` passée à « ✅ » avec slots documentés (fait avant première utilisation).

## 6. Plan de tests

- `npm --prefix frontend run build` et `npm --prefix frontend run lint` — passent sans erreur.
- Vérification visuelle : grille contrats (desktop + 360px), cartes signalements (3 chips), cartes tâches (sans icône), thèmes clair + sombre.
- Revue diff : aucun changement hors des blocs d'en-tête de carte (changement d'ordre chips/titre de `ClaimsView` explicitement signalé dans la PR).

## 7. Ordre d'implémentation

1. Mettre à jour `docs/DESIGN.md` §3.2 (statut + slots `#icon`/`#chips`).
2. Créer `CCardHeader.vue`.
3. Migrer `ContractsView` (pilote), valider visuellement.
4. Migrer `ClaimsView` (après validation de l'écart d'ordre), puis les vues restantes (§4).
5. `graphify update .` + entrée datée dans `docs/PROGRESS.md` (réf. EP-020-S15).

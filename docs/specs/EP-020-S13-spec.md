# Spécification technique — EP-020-S13

**Story :** Créer la molécule `CPageHeader` et remplacer les blocs `view-header` répétés dans les vues métier par ce composant unique.

**Épic :** EP-020 — Design system
**Réf. DESIGN :** `docs/DESIGN.md` §3.2 (ligne `CPageHeader` — 🔲 planifié), §4.1 « Eyebrows de page », §4.1 « Entités comptables gelées » (indicateur `CIconLock` + libellé `.tone--muted`)

> Frontend uniquement. Aucun changement backend, aucun changement d'API.

---

## 1. État actuel

Chaque vue métier répète le même bloc d'en-tête à la main :

```html
<header class="view-header">
  <CEyebrow>{{ t('page.eyebrows.contracts') }}</CEyebrow>
  <CHeading type="header">{{ t('contracts.title') }}</CHeading>
  <CMetaItem variant="sub">{{ t('contracts.count', { count }) }}</CMetaItem>
</header>
```

- L'utilitaire `view-header` est défini dans `frontend/src/styles/tailwind.css` L120 (`flex flex-col items-start gap-2`).
- 24 occurrences dans 22 vues : `ContractsView.vue:380`, `TasksView.vue:339`, `ClaimsView.vue:325`, `AssemblyDetailView.vue:586`, `AssemblyListView.vue:127`, `BudgetView.vue:601` + `625`, `ConsultationsView.vue:212`, `ContractorsView.vue:197`, `DashboardView.vue:186`, `DocumentsView.vue:124`, `MaintenanceView.vue:256`, `ProfileView.vue:165`, `GovernanceMandatesView.vue:115`, `JudicialProceedingsView.vue:116`, `ReportsView.vue:160`, `AllocationKeysView.vue:115`, `BuildingView.vue:74`, `MyChargesView.vue:58`, `RegistryView.vue:112`, `ResidentsView.vue:258`, `UnitChargesView.vue:176`, `TaskDetailView.vue:218`.
- Les variantes divergent : présence/absence d'eyebrow, badges dans l'eyebrow (`TaskDetailView.vue:220-230`), description, méta, boutons d'action posés hors header.

## 2. API du composant

`frontend/src/components/molecules/CPageHeader.vue` :

```ts
withDefaults(defineProps<{
  eyebrow?: string       // texte eyebrow (rendu via CEyebrow) — §4.1 : 3 à 6 mots
  title: string          // rendu via CHeading type="header" (h1)
  meta?: string          // rendu via CMetaItem variant="sub"
  icon?: Component       // icône métier optionnelle à gauche du titre (CIconTile)
  frozenLabel?: string   // entité gelée : CIconLock + libellé .tone--muted (§4.1)
}>(), {})
```

Slots :

- `#eyebrow` — remplace la prop `eyebrow` quand l'eyebrow contient des badges (cas `TaskDetailView`).
- `#actions` — boutons alignés à droite (`flex-shrink: 0`), même pattern que `CSectionHeader` (`frontend/src/components/molecules/CSectionHeader.vue`).
- défaut — contenu additionnel sous la ligne titre (description, méta libre).

Aucun emit.

## 3. Comportement / design

- Élément racine `<header>` ; layout interne repris de l'utilitaire `view-header` (`flex flex-col items-start gap-2`, tokens `--space-*` uniquement — pas de valeurs brutes).
- Titre : `CHeading type="header"` (h1, `text-2xl font-serif`) — le composant possède le layout, Tailwind possède le style texte (DESIGN §1.3).
- `icon` : rendue dans un `CIconTile` à gauche du bloc titre ; alignement `flex items-center gap-3 min-w-0`.
- `frozenLabel` : `CIconLock` (SVG existant `frontend/src/components/atoms/CIconLock.svg`) + libellé en `.tone--muted`, affiché sur la ligne méta — conforme à « Entités comptables gelées » (DESIGN §4.1).
- Zone `#actions` : `display: flex; gap: var(--space-2); flex-shrink: 0` — le header passe en `flex-row justify-between` quand le slot est présent, corps en `min-width: 0`.
- A11y : un seul `h1` par page (garanti par `CHeading type="header"`) ; l'icône est `aria-hidden="true"` ; `frozenLabel` reste du texte lisible (pas de tooltip seul).
- i18n : le composant ne traduit rien — il reçoit des chaînes déjà traduites.

## 4. Plan de migration

1. Vues liste simples (eyebrow + titre + méta) : `ContractsView`, `TasksView`, `ClaimsView`, `ConsultationsView`, `ContractorsView`, `DocumentsView`, `ReportsView`, `ResidentsView`, `RegistryView`, `AllocationKeysView`, `BuildingView`, `MyChargesView`, `UnitChargesView`, `GovernanceMandatesView`, `JudicialProceedingsView`, `MaintenanceView`, `AssemblyListView`, `ProfileView`, `DashboardView`.
2. Vues détail avec header enrichi : `TaskDetailView` (badges via `#eyebrow`, description via slot défaut), `AssemblyDetailView`.
3. `BudgetView` (2 occurrences L601/L625 — vérifier si la 2ᵉ est un header de section → `CSectionHeader`, pas `CPageHeader`).
4. Une fois toutes les vues migrées, supprimer l'utilitaire `view-header` de `frontend/src/styles/tailwind.css` L120.

La migration peut être découpée en plusieurs commits (une feature par commit) ; le composant reste utilisable en coexistence avec `view-header` pendant la transition.

## 5. Critères d'acceptation

- [ ] `CPageHeader.vue` existe dans `frontend/src/components/molecules/`, typé TS, sans `<style>` avec valeurs brutes (tokens uniquement).
- [ ] Rendu identique pixel-perfect au bloc `view-header` actuel pour le cas nominal (eyebrow + titre + méta).
- [ ] `frozenLabel` affiche `CIconLock` + texte `.tone--muted`.
- [ ] Slot `#actions` aligne les boutons à droite sans casser la troncature du titre (`min-width: 0`).
- [ ] Toutes les vues listées en §4 utilisent `CPageHeader` ; plus aucune occurrence de `class="view-header"` dans `frontend/src/features/`.
- [ ] L'utilitaire `view-header` est supprimé de `tailwind.css`.
- [ ] Catalogue `docs/DESIGN.md` §3.2 : ligne `CPageHeader` passée de « 🔲 planifié » à « ✅ » (fait par le développeur au moment de l'implémentation — pré-requis avant première utilisation).

## 6. Plan de tests

- `npm --prefix frontend run build` — compile sans erreur TS.
- `npm --prefix frontend run lint` — passe sans warning.
- Vérification visuelle manuelle : `ContractsView` (nominal), `TaskDetailView` (slot `#eyebrow` + description), une vue avec `#actions`, thèmes clair + sombre.
- `grep -r "view-header" frontend/src` ne retourne plus rien après la dernière étape.

## 7. Ordre d'implémentation

1. Mettre à jour `docs/DESIGN.md` §3.2 (statut + contrat final du composant).
2. Créer `CPageHeader.vue` (conventions : `CSectionHeader.vue` pour la structure slots/actions, `CHeading.vue` pour la typographie).
3. Migrer une vue pilote (`ContractsView`) et valider visuellement.
4. Migrer les vues restantes par lots (§4, étapes 1 → 3).
5. Supprimer l'utilitaire `view-header` de `tailwind.css`.
6. `graphify update .` + entrée datée dans `docs/PROGRESS.md` (réf. EP-020-S13).

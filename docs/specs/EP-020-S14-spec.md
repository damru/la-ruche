# Spécification technique — EP-020-S14

**Story :** Créer la molécule `CTopbarBreadcrumb` et remplacer les fils d'Ariane construits à la main (`div.topbar-breadcrumb`) dans le slot `#left` de `CTopbar`.

**Épic :** EP-020 — Design system
**Réf. DESIGN :** `docs/DESIGN.md` §3.2 (ligne `CTopbarBreadcrumb` — 🔲 planifié), §4.1 « Navigation & breadcrumb » (« fluide (`min-width: 0`), libellés tronquent proprement »)

> Frontend uniquement.

---

## 1. État actuel

Chaque vue assemble son breadcrumb à la main dans le slot `#left` de `CTopbar` :

```html
<div class="topbar-breadcrumb">
  {{ buildingName || '…' }}
  <CIconChevronRight />
  <span>{{ t('claims.title') }}</span>
</div>
```

- 22 vues dupliquent ce markup : `AssemblyDetailView.vue:508` (3 niveaux, avec `RouterLink` `.topbar-breadcrumb-link`), `AssemblyListView.vue:100`, `BudgetView.vue:571`, `ClaimsView.vue:300`, `ConsultationsView.vue:186`, `ContractorsView.vue`, `ContractorDetailView.vue`, `ContractsView.vue`, `DocumentsView.vue`, `MaintenanceView.vue`, `ProfileView.vue`, `GovernanceMandatesView.vue`, `JudicialProceedingsView.vue`, `ReportsView.vue`, `AllocationKeysView.vue`, `BuildingView.vue`, `MyChargesView.vue`, `RegistryView.vue`, `ResidentsView.vue`, `UnitChargesView.vue`, `TaskDetailView.vue`, `TasksView.vue`.
- Le style est porté par `CTopbar.vue` via des sélecteurs `:deep(.topbar-breadcrumb)`, `:deep(.topbar-breadcrumb span)`, `:deep(.topbar-breadcrumb svg)`, `:deep(.topbar-breadcrumb-link)` (`frontend/src/components/organisms/CTopbar.vue`, bloc `<style scoped>`) — un couplage fragile : le style vit dans le parent, le markup dans 22 vues.
- Incohérences constatées : placeholder `'...'` vs `'…'`, `aria-hidden` présent ou non sur le chevron, texte parfois nu, parfois dans un `<span>` (donc troncature non uniforme).

## 2. API du composant

`frontend/src/components/molecules/CTopbarBreadcrumb.vue` :

```ts
interface Crumb {
  label: string                      // libellé déjà traduit
  to?: RouteLocationRaw              // si présent → RouterLink, sinon <span>
}

defineProps<{
  crumbs: Crumb[]                    // ordre racine → page courante
}>()
```

Ni slot, ni emit. Le chevron séparateur (`CIconChevronRight`, `aria-hidden="true"`) est rendu par le composant entre chaque crumb — jamais fourni par l'appelant.

## 3. Comportement / design

- Racine `<nav aria-label>` (libellé i18n générique fourni par le composant via `useI18n`, clé `common.breadcrumb` à ajouter si absente) contenant une liste `flex items-center gap-2 min-width: 0`.
- **Fluidité / troncature** (exigence DESIGN §4.1) : chaque libellé dans un `<span>` avec `min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap` ; les chevrons `flex: 0 0 auto`. Le dernier crumb (page courante) garde la priorité de largeur (`flex-shrink` supérieur sur les crumbs intermédiaires).
- Crumb avec `to` → `RouterLink` (couleur héritée, `hover: underline` — reprend `.topbar-breadcrumb-link`). Dernier crumb → `<span aria-current="page">`.
- Couleurs et tailles via tokens : `--color-on-surface-variant` (texte), `--color-outline-variant` (chevrons), `--icon-xs`. Pas de valeurs brutes.
- Le composant embarque son propre `<style scoped>` ; les sélecteurs `:deep(.topbar-breadcrumb*)` de `CTopbar.vue` sont supprimés une fois la migration terminée (les helpers greeting `topbar-greeting-*` restent, hors périmètre).

## 4. Intégration dans `CTopbar`

`CTopbar.vue` n'est pas modifié structurellement (slots `#left`/`#right` inchangés). Usage cible :

```html
<CTopbar>
  <template #left>
    <CTopbarBreadcrumb
      :crumbs="[
        { label: buildingName || '…' }
        { label: t('meetings.title'), to: { name: 'assembly-list' } }
        { label: assembly.title }
      ]"
    />
  </template>
  ...
</CTopbar>
```

## 5. Plan de migration

1. Vues à 2 niveaux (bâtiment › page) : les 20 vues liste (§1).
2. Vues à 3 niveaux avec lien intermédiaire : `AssemblyDetailView`, `TaskDetailView`, `ContractorDetailView`.
3. Nettoyage : supprimer les blocs `:deep(.topbar-breadcrumb)`, `:deep(.topbar-breadcrumb span)`, `:deep(.topbar-breadcrumb svg)` et `:deep(.topbar-breadcrumb-link)` de `CTopbar.vue` ; supprimer les imports `CIconChevronRight` devenus orphelins dans les vues.

## 6. Critères d'acceptation

- [ ] `CTopbarBreadcrumb.vue` existe dans `frontend/src/components/molecules/`, typé TS (interface `Crumb` exportée).
- [ ] Un libellé long tronque avec ellipse sans pousser les actions du topbar (test : titre d'AG très long dans `AssemblyDetailView` à 640px de large).
- [ ] Les crumbs avec `to` naviguent ; le dernier crumb porte `aria-current="page"` ; racine `<nav>` avec `aria-label`.
- [ ] Plus aucune occurrence de `class="topbar-breadcrumb"` dans `frontend/src/features/` ; sélecteurs `:deep` correspondants retirés de `CTopbar.vue`.
- [ ] Aucun import direct `radix-vue` (composant purement HTML/RouterLink).
- [ ] `docs/DESIGN.md` §3.2 : ligne `CTopbarBreadcrumb` passée à « ✅ » avec le contrat final (fait avant première utilisation).

## 7. Plan de tests

- `npm --prefix frontend run build` et `npm --prefix frontend run lint` — passent sans erreur.
- Vérification visuelle : `ClaimsView` (2 niveaux), `AssemblyDetailView` (3 niveaux + lien), viewport 640px (troncature), thèmes clair + sombre.
- `grep -r "topbar-breadcrumb" frontend/src` ne matche plus que `CTopbarBreadcrumb.vue` lui-même.

## 8. Ordre d'implémentation

1. Mettre à jour `docs/DESIGN.md` §3.2 (statut + contrat `crumbs[]`).
2. Créer `CTopbarBreadcrumb.vue` (style repris des sélecteurs `:deep` actuels de `CTopbar.vue`).
3. Migrer une vue pilote 2 niveaux (`ClaimsView`) puis une vue 3 niveaux (`AssemblyDetailView`).
4. Migrer les vues restantes ; nettoyer `CTopbar.vue` et les imports orphelins.
5. Entrée datée dans.

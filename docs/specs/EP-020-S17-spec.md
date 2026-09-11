# Spécification technique — EP-020-S17

**Story :** Créer l'atome `CDataText` (texte tabulaire JetBrains Mono, chiffres alignés) et remplacer les spans `font-mono` ad hoc des tableaux, montants et KPI.

**Épic :** EP-020 — Design system
**Réf. DESIGN :** `docs/DESIGN.md` §2 (« Les styles typographiques nommés sont encapsulés dans les composants atomiques `CHeading`, `CEyebrow` et `CDataText` », L241), §3.1 (ligne `CDataText` — 🔲 planifié — « `font-mono font-medium lining-nums tabular-nums` »), §3.2 ligne `CKpiMetric` (déjà porteur du même style pour les valeurs KPI display).

> Frontend uniquement.

---

## 1. État actuel

Le style « donnée chiffrée » est réappliqué à la main, avec des combinaisons incohérentes (`font-medium` et `tabular-nums` présents ou non) :

Spans Tailwind ad hoc dans les templates :

- `frontend/src/features/budget/BudgetView.vue:801,809,813` (`strong.font-mono`), `:882,939,952` (`font-mono font-medium [text-sm]`), `:994-995` (`b.text-on-surface.font-medium.font-mono`), `:1004,1107` (`span.font-mono`).
- `frontend/src/features/assemblies/AssemblyDetailView.vue:894` (`font-mono font-medium self-center`), `frontend/src/features/assemblies/VoteModal.vue:170` (`td.font-mono`).
- `frontend/src/features/budget/GenerateLeviesModal.vue:129,133` (`font-mono tabular-nums` — seul endroit avec `tabular-nums`).
- `frontend/src/features/consultations/ConsultationsView.vue:318` (`choice-percent font-mono text-xs`), `frontend/src/features/residents/LevyBreakdownModal.vue:87` (`font-mono`).

Déclinaisons CSS `font-family: var(--font-mono)` dans des `<style scoped>` de vues :

- `DashboardView.vue:654,675,680` (`.reminder-date`, `.budget-prog-amounts`, `.budget-total-val`), `AssemblyDetailView.vue:1568,1574,1724,1743`, `AssemblyListView.vue:331`, `MaintenanceView.vue:672`.

Composants catalogue déjà porteurs du style (hors périmètre, restent tels quels) : `CKpiMetric.vue` (valeurs display), `CProgressBar.vue:41`, `CCopyField.vue:85`, `CDataTable.vue:214` (cellules `numeric`).

## 2. API du composant

`frontend/src/components/atoms/CDataText.vue` :

```ts
withDefaults(defineProps<{
  size?: 'sm' | 'md' | 'lg'
  as?: 'span' | 'strong' | 'div'   // 'strong' pour les emphases actuelles (<b>/<strong>)
}>(), {
  size: 'md',
  as: 'span',
})
```

Slot défaut : la valeur (déjà formatée par le parent — `formatCurrency`, pourcentages, dates courtes…). Aucun emit.

## 3. Comportement / design

- Classes constantes : `font-mono font-medium lining-nums tabular-nums` (contrat DESIGN §3.1) — chiffres doublure + chasse fixe, alignement vertical garanti en colonnes.
- `size` mappe sur les tokens texte, mêmes conventions que `CHeading.vue` (record `sizeClass`) :
  - `sm` → `text-xs` (méta, dates compactes — remplace `--text-2xs` ad hoc du dashboard),
  - `md` → `text-sm` (cellules, montants inline — défaut),
  - `lg` → `text-base` (totaux mis en avant).
- La couleur n'est **pas** fixée par le composant — héritée du contexte ; le parent ajoute `text-on-surface` / tone si besoin (le composant possède le style typographique, pas la sémantique de couleur).
- Interdits (déjà actés pour `CKpiMetric`, DESIGN §4.1 KPI) : jamais `font-display` ni `text-[…px]` pour des chiffres. Les valeurs KPI display (40px) restent dans `CKpiMetric` — `CDataText` couvre les tailles texte courantes uniquement.
- Pas de `<style>` : composant 100 % utilitaires Tailwind (tokens). Aucun import radix-vue.
- A11y : purement présentation ; `as="strong"` disponible quand l'emphase sémantique actuelle (`<strong>`/`<b>`) doit être conservée.

## 4. Plan de migration

1. Vue pilote : `BudgetView.vue` — remplacer les 10 occurrences listées en §1 (mélange `span`/`strong`/`b`, tailles sm/md).
2. `GenerateLeviesModal.vue`, `LevyBreakdownModal.vue`, `VoteModal.vue`, `AssemblyDetailView.vue:894`, `ConsultationsView.vue:318`.
3. Déclinaisons CSS scoped : `DashboardView` (`.reminder-date`, `.budget-prog-amounts`, `.budget-total-val`), `AssemblyDetailView` (4 règles), `AssemblyListView`, `MaintenanceView` — remplacer le markup par `CDataText` et supprimer les règles `font-family: var(--font-mono)` devenues orphelines.
4. Hors périmètre : `CKpiMetric`, `CProgressBar`, `CCopyField`, `CDataTable` (styles encapsulés dans le catalogue) ; futur nettoyage possible en les faisant composer `CDataText`, non requis ici.

## 5. Critères d'acceptation

- [ ] `CDataText.vue` existe dans `frontend/src/components/atoms/`, typé TS, sans bloc `<style>`.
- [ ] Rendu : JetBrains Mono (`--font-mono`), `font-medium`, `lining-nums tabular-nums` pour les trois tailles ; chiffres alignés verticalement dans un tableau de montants.
- [ ] Toutes les occurrences template listées en §1 utilisent `CDataText` ; plus aucun `font-mono` inline dans `frontend/src/features/` hors composants catalogue.
- [ ] Les règles `font-family: var(--font-mono)` des `<style scoped>` de vues listées en §4.3 sont supprimées.
- [ ] Aucun changement visuel non voulu : les emphases `<strong>`/`<b>` conservent leur graisse via `as="strong"`.
- [ ] `docs/DESIGN.md` §3.1 : ligne `CDataText` passée à « ✅ » (fait avant première utilisation).

## 6. Plan de tests

- `npm --prefix frontend run build` et `npm --prefix frontend run lint` — passent sans erreur.
- Vérification visuelle : `BudgetView` (colonnes de montants alignées), `DashboardView` (dates de rappel), `GenerateLeviesModal` (totaux), thèmes clair + sombre.
- `grep -rn "font-mono" frontend/src/features --include=*.vue` ne retourne plus rien ; `grep -rn "var(--font-mono)" frontend/src/features` idem.

## 7. Ordre d'implémentation

1. Mettre à jour `docs/DESIGN.md` §3.1 (statut + prop `as`).
2. Créer `CDataText.vue` (convention `sizeClass` de `CHeading.vue`).
3. Migrer `BudgetView` (pilote), vérifier l'alignement des colonnes.
4. Migrer les occurrences restantes (§4, étapes 2-3) et supprimer les règles CSS orphelines.
5. Entrée datée dans `docs/PROGRESS.md` (réf. EP-020-S17).

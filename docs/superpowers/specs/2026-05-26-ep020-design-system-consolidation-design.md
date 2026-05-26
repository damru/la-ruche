# EP-020 — Design System : Consolidation tokens & wrappers Radix
**Date :** 2026-05-26
**Epic :** EP-020 · DESIGN.md
**Statut :** spec validée
**Approche retenue :** Plan linéaire S01→S12, un PR par story, séquencement tokens → wrappers → cleanup CSS.

---

## 1. Contexte & objectifs

`tokens.css` définit toutes les couleurs en hex `#rrggbb`. Tailwind v4 expose ces tokens via `@theme inline`. Cinq fichiers CSS globaux (`tabs.css`, `typography.css`, `forms.css`, `tables.css`, `activity-cards.css`) coexistent avec les composants `C*` et génèrent un double-circuit de styles. Plusieurs wrappers Radix sont manquants (`CDropdownMenu`, `CStatusBanner`, `CTabs`, `CHeading`). Les composants existants `CFormCheckbox`, `CFormSelect`, `CTooltip` ne sont pas basés sur Radix.

**Objectifs :**
1. Migrer tous les tokens couleur vers OKLCH (chroma/luminosité vérifiables en dark mode).
2. Enregistrer les tokens Tailwind manquants (`--text-xl`, `--text-2xl`, `--text-display`, `--space-7`, `--space-9`, `--leading-*`, `--z-*`).
3. Implémenter les wrappers Radix manquants et refaire les wrappers existants non-Radix.
4. Supprimer progressivement les 5 fichiers CSS globaux legacy (migration vers Tailwind inline ou styles scoped).

**Non-objectifs :** corrections visuelles au-delà du dark mode, refonte des layouts existants, migration des vues vers de nouveaux patterns de layout.

---

## 2. Phase 1 — Migration OKLCH (S01-S02)

### 2.1 Périmètre

Fichier : `frontend/src/styles/tokens.css` — `:root` light + `[data-theme="dark"]`.

`tones.css` n'est pas migré (aucun hex direct — référence uniquement des vars).

### 2.2 Règles de conversion

- Espace cible : `oklch(L C H)` natif CSS. Support universel Chrome/Firefox/Safari 2023+.
- Outil de conversion : `culori` (npm, déterministe, précision maximale). Script de conversion one-shot.
- Valeurs arrondies : L à 3 décimales, C à 4 décimales, H à 2 décimales.
- `rgba(r, g, b, a)` dans les ombres → `oklch(L C H / alpha)`.
- `linear-gradient` hex → `linear-gradient(... oklch(...), oklch(...))`.
- `color-mix(in srgb, ...)` (`--color-outline-light`) → `color-mix(in oklch, ...)`.

### 2.3 Corrections dark mode (S02)

Pour chaque token dark, vérifier :
- Surfaces : L(surface dark) < L(surface light) — ordre de luminosité inversé correct.
- Contraste `on-surface` / `surface` ≥ 4.5:1 (WCAG AA texte).
- Contraste `on-surface-variant` / `surface-container` ≥ 3:1 (WCAG AA UI).
- Role badges : fond dark doit avoir L < 0.4, texte dark doit avoir L > 0.7.

Les corrections légères sont acceptées et documentées dans le message de commit (`fix: dark mode contrast for role-board-fg`).

### 2.4 Vérification

Snapshot visuel avant/après : BudgetView, AssemblyDetailView, TasksView (light + dark).
Run `npm --prefix frontend run lint` + `npm --prefix frontend run build` (zéro erreur).

---

## 3. Phase 2 — Tokens Tailwind manquants (S03)

### 3.1 Tokens à ajouter dans `@theme inline` (`tailwind.css`)

Tokens absents du `@theme inline` actuel mais définis dans `tokens.css` :

| Token CSS | Alias Tailwind | Valeur |
|---|---|---|
| `--text-lg` (16px) | `--text-lg` | `var(--text-lg)` |
| `--text-xl` (20px) à ajouter | `--text-xl` | `20px` |
| `--text-2xl` (24px) à ajouter | `--text-2xl` | `24px` |
| `--text-display` (32px+) | `--text-display` | `32px` |
| `--space-7` | `--spacing-7` | `28px` |
| `--space-9` | `--spacing-9` | `36px` |
| `--leading-tight` | `--leading-tight` | `1.25` |
| `--leading-snug` | `--leading-snug` | `1.375` |
| `--leading-normal` | `--leading-normal` | `1.5` |
| `--leading-relaxed` | `--leading-relaxed` | `1.625` |
| `--z-base` | `--z-base` | `var(--z-base)` |
| `--z-raised` | `--z-raised` | `var(--z-raised)` |
| `--z-nav` | `--z-nav` | `var(--z-nav)` |
| `--z-overlay` | `--z-overlay` | `var(--z-overlay)` |
| `--z-modal` | `--z-modal` | `var(--z-modal)` |
| `--z-toast` | `--z-toast` | `var(--z-toast)` |

Les tokens `--text-xl` et `--text-2xl` sont ajoutés à `tokens.css` avant d'être référencés dans `@theme inline` (ils sont déjà implicites dans DESIGN.md §2.1 pour `CHeading`).

---

## 4. Phase 3 — Wrappers Radix (S04-S06)

### 4.1 Règles générales

- Tout composant `C*` basé sur Radix doit re-émettre les events Radix pertinents via `defineEmits`.
- `v-model` via `defineModel()` (Vue 3.4+).
- Zéro style scoped si les tokens Tailwind suffisent.
- Ajout au catalogue DESIGN.md §2.1 / §2.3 / §3.4 avant implémentation.

### 4.2 CDropdownMenu (S04)

**Radix primitive :** `DropdownMenu` (Root, Trigger, Content, Item, Separator).
**Interface :**
```ts
interface DropdownItem {
  label: string
  icon?: Component
  action: () => void
  destructive?: boolean
  disabled?: boolean
  separator?: boolean  // item suivant est précédé d'un séparateur
}
props: { items: DropdownItem[], align?: 'start' | 'end' | 'center' }
slot: 'trigger' — contenu du déclencheur
```
**Style :** surface-container-lowest, radius-md, shadow-md, items `hover:bg-surface-container`, items destructive `text-error hover:bg-error-container`.
**Accessibilité :** keyboard navigation, `aria-label` sur le trigger slot.

### 4.3 CStatusBanner (S05)

**Usage :** bandeau en-tête des vues AG.
**Interface :**
```ts
type MeetingStatus = 'draft' | 'called' | 'ongoing' | 'closed'
props: { status: MeetingStatus, label?: string }
```
**Mapping status → tone :**
- `draft` → `.tone--muted`
- `called` → `.tone--info`
- `ongoing` → `.tone--accent`
- `closed` → `.tone--valid`

Pas de primitive Radix (composant de display pur). Affiche icône + libellé + badge statut.

### 4.4 CFormCheckbox — refonte Radix (S06)

**Remplacement de l'implémentation actuelle** (non-Radix) par wrapper `Checkbox` Radix.
**Interface :** `v-model` boolean, `label`, `disabled`, `error`.
**Indicateur visuel :** `CheckIcon` Radix dans le slot checked state.
API publique inchangée pour les vues consommatrices (rétrocompatible).

### 4.5 CFormSelect — refonte Radix (S06)

**Remplacement** : l'implémentation actuelle utilise déjà `SelectRoot` de radix-vue (selon DESIGN.md ✅). **À vérifier à l'implémentation** — si déjà correct, story se limite à validation + tests. Si non-Radix, même approche que `CFormCheckbox`.

### 4.6 CTooltip — refonte Radix (S06)

Idem `CFormSelect` — marqué ✅ dans DESIGN.md §3.4. Vérification à l'implémentation.

---

## 5. Phase 4 — Nouveaux composants (S07-S08)

### 5.1 CTabs (S07)

**Remplace** `tabs.css` (`.tabs`, `.tab`, `.tab--active`, `.tab-count`).
**Interface :**
```ts
interface Tab { value: string; label: string; count?: number }
props: { tabs: Tab[] }
v-model: string  // tab active
```
**Style :** flex row, overflow-x auto, scroll touch, indicateur soulignement (border-bottom) sur actif, badge `.tab-count` en `text-primary`. Pas d'icône seule.
**Migration :** 6 vues consommatrices identifiées (grep en phase implémentation). Remplacer `<div class="tabs">` par `<CTabs>`.
Supprimer `tabs.css` et son import dans `tailwind.css` à la fin de cette story.

### 5.2 CHeading (S08)

**Atom Fraunces** semibold italic.
**Interface :**
```ts
props: { size: 'lg' | 'md' | 'sm', as?: 'h1' | 'h2' | 'h3' | 'p' }
// lg = text-2xl (24px), md = text-xl (20px), sm = text-lg (18px)
// as défaut: h2
```
**Utilise** les tokens `--text-2xl`, `--text-xl`, `--text-lg` ajoutés en S03.
**Migration `typography.css` :** les classes `.display-md` (utilisées sur `<h1>` dans ~20 vues) sont remplacées par `<CHeading size="lg" as="h1">`. Les classes `.headline-sm` (AssemblyDetailView, BudgetView) par `<CHeading size="sm">`.
Supprimer `typography.css` et son import une fois toutes les usages migrées.

---

## 6. Phase 5 — Cleanup CSS (S09-S12)

Approche : migration progressive (Choix B — les fichiers restent mais se vident). Suppression finale quand un fichier est à zéro classes actives.

### 6.1 CFormRow, CFormGroup, CFormLabel (S09) — `forms.css`

**Nouveaux composants molecules :**
- `CFormRow` : remplace `.form-grid` (flex column, gap-4) et `.form-grid-2col` (grid 2 colonnes responsive)
- `CFormGroup` : remplace `.field` / `.field--full` (wrapper field avec label + input + hint + error)
- `CFormLabel` : remplace `.form-label` / `.field-label` (label typé avec required indicator)

`.search-bar` → Tailwind inline dans les vues (pas de composant dédié — usage trop contextuel).
`.icon-btn` → `CBtn variant="ghost" size="sm"` avec `class="text-error"` pour la variante danger.
`.dropzone` → déjà encapsulé dans `CFormDropzone` — retirer les classes de `forms.css`.
`.checkbox-label` → absorbé dans `CFormCheckbox` refait (S06).

Consommateurs identifiés : ~15 vues + modales pour `.form-grid`, ~10 vues pour `.field*`.

### 6.2 CDataTable scoped + Tailwind inline (S10) — `tables.css`

`.data-table` : déplacer les styles dans `CDataTable` en `<style scoped>`. 3 vues consommatrices directes (DocumentsView, RegistryView, ResidentsView) migrées vers `<CDataTable>`.
`.cards-grid` : remplacer par `class="grid grid-cols-[repeat(auto-fill,minmax(320px,1fr))] gap-4"` inline dans AllocationKeysView et MyChargesView.

### 6.3 base.css (S11)

**Garder :** browser resets uniquement (`html`, `body`, `:focus-visible`, scrollbar, `a`, `img`, `button`, `input`).
**Migrer :**
- `.view-container` → layout Tailwind inline dans chaque vue (21 vues). Pattern : `class="flex flex-col p-10 max-w-[1200px] gap-6 max-sm:p-5"`.
- `.view-header` → Tailwind inline. Pattern : `class="flex flex-col items-start gap-2"`.
- `.empty-state` → composant `CEmptyState` (nouveau atom simple : slot + style centré muted).
- `.loading-skeleton` → déjà non utilisé dans les vues — supprimer la règle.
- `.spinner` → `CPageLoader` existant.
- `.card--terminated` → Tailwind inline `class="outline outline-1 outline-outline-variant"`.

### 6.4 activity-cards.css (S12)

Déplacer tous les styles `.activity-card` et sous-éléments dans le composant feature consommateur (scoped). Consommateurs : AssemblyListView, ClaimsView, ContractorsView, ContractsView, TasksView.
Si plusieurs vues partagent le même markup d'activity-card → extraire un composant `CActivityCard` dans le catalogue atoms avant de migrer.

---

## 7. Dépendances entre stories

```
S01 (OKLCH light) → S02 (OKLCH dark + corrections)
S02 → S03 (tokens Tailwind : --text-xl/2xl nouveaux)
S03 → S08 (CHeading utilise --text-xl/2xl)
S04 (CDropdownMenu) — indépendant
S05 (CStatusBanner) — indépendant
S06 (CFormCheckbox/Select/Tooltip refonte Radix) — indépendant
S06 → S09 (CFormGroup absorbe checkbox-label)
S07 (CTabs) → suppression tabs.css
S08 (CHeading) → migration de toutes les classes typography.css utilisées dans les vues ; suppression du fichier quand aucune classe n'est plus référencée dans les vues (les classes `.body-*`, `.label-*`, `.data-md` non utilisées dans les vues sont supprimées du fichier sans remplacement)
S09 (CFormRow/Group/Label) → suppression forms.css (progressive)
S10 (CDataTable scoped) → suppression tables.css
S11 (base.css cleanup) — peut commencer après S07/S08 (CEmptyState)
S12 (activity-cards) — indépendant, peut commencer à tout moment
```

---

## 8. Conventions d'implémentation

- Chaque wrapper Radix : `defineOptions({ inheritAttrs: false })` + `v-bind="$attrs"` sur la racine Radix pour pass-through des attributs HTML.
- Nouveaux composants ajoutés à DESIGN.md §2 avant le PR.
- Suppression d'un fichier CSS global : le commit inclut la suppression de l'import dans `tailwind.css`.
- Tests : snapshot Vitest pour `CTabs`, `CHeading`, `CDropdownMenu` (states ouverts/fermés).
- `cargo sqlx prepare` non requis (pas de changement backend).

---

## 9. Critères de succès (Definition of Done)

- [ ] `tokens.css` : zéro hex `#rrggbb`, 100% OKLCH
- [ ] `tailwind.css @theme inline` : tous les tokens listés en §3.1 présents
- [ ] DESIGN.md §3.4 : tous les wrappers Radix marqués ✅
- [ ] `tabs.css` supprimé
- [ ] `typography.css` supprimé (toutes les classes `.display-*`, `.headline-*` migrées vers `CHeading` ; classes `.body-*`, `.label-*`, `.data-md` supprimées si non utilisées dans les vues)
- [ ] `forms.css` : en cours de vidage progressif (toutes les classes `.form-grid`, `.field*`, `.icon-btn` migrées)
- [ ] `tables.css` : `.data-table` et `.cards-grid` migrées
- [ ] `base.css` : seuls les resets navigateur subsistent
- [ ] `activity-cards.css` : zéro classe active, supprimé ou vide
- [ ] `npm run build` et `npm run lint` : zéro erreur

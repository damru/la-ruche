# EP-020 Design System Consolidation — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate tous les tokens couleur vers OKLCH, enregistrer les tokens Tailwind manquants, implémenter les wrappers Radix manquants, refaire les wrappers non-Radix, et supprimer progressivement les 5 fichiers CSS globaux legacy.

**Architecture:** Séquencement strict tokens → wrappers → cleanup. Chaque task est un PR autonome. Aucune suppression de fichier CSS global avant que toutes ses classes soient migrées. Pas d'infrastructure de tests frontend existante — vérification visuelle + `npm run build` + `npm run lint` à chaque task.

**Tech Stack:** Vue 3 + TypeScript + Tailwind v4 + Radix Vue + `culori` (conversion OKLCH one-shot)

**Spec de référence :** `docs/superpowers/specs/2026-05-26-ep020-design-system-consolidation-design.md`

---

## Task 1 — Migration OKLCH light mode (S01)

**Files:**
- Modify: `frontend/src/styles/tokens.css` (section `:root`)
- Create (temp): `scripts/convert-oklch.mjs` (script one-shot, supprimé après)

### Étape 1.1 — Installer culori (dev dep temporaire)

- [ ] Installer culori dans le frontend

```bash
npm --prefix frontend install --save-dev culori
```

Expected : `culori` apparaît dans `frontend/package.json` devDependencies.

### Étape 1.2 — Écrire le script de conversion

- [ ] Créer `scripts/convert-oklch.mjs`

```js
// scripts/convert-oklch.mjs
// Usage: node scripts/convert-oklch.mjs
import { converter, formatCss } from 'culori'

const toOklch = converter('oklch')

function hex(h) {
  const c = toOklch(h)
  const l = c.l.toFixed(3)
  const ch = c.c.toFixed(4)
  const hue = (c.h ?? 0).toFixed(2)
  return `oklch(${l} ${ch} ${hue})`
}

function hexAlpha(h, a) {
  const c = toOklch(h)
  const l = c.l.toFixed(3)
  const ch = c.c.toFixed(4)
  const hue = (c.h ?? 0).toFixed(2)
  return `oklch(${l} ${ch} ${hue} / ${a})`
}

const colors = {
  'primary':                    '#643fd5',
  'primary-container':          '#7d5bf0',
  'on-primary':                 '#ffffff',
  'secondary':                  '#5a6073',
  'secondary-container':        '#dde1f9',
  'on-secondary-container':     '#1a1d2e',
  'error':                      '#ba1a1a',
  'error-container':            '#ffdad6',
  'on-error':                   '#ffffff',
  'success':                    '#2e7d32',
  'success-container':          '#e8f5e9',
  'warning':                    '#e65100',
  'warning-container':          '#fff8e1',
  'role-manager-bg':            '#e8def8',
  'role-manager-fg':            '#21005d',
  'role-board-bg':              '#d2e4fb',
  'role-board-fg':              '#001d36',
  'role-co-owner-bg':           '#cef5e5',
  'role-co-owner-fg':           '#00391d',
  'role-tenant-bg':             '#fcecd1',
  'role-tenant-fg':             '#3a2000',
  'surface':                    '#f8f9fa',
  'surface-lowest':             '#ffffff',
  'surface-low':                '#f3f4f5',
  'surface-container':          '#edeff0',
  'surface-high':               '#e7e9ea',
  'surface-highest':            '#e1e3e4',
  'on-surface':                 '#191c1d',
  'on-surface-variant':         '#42474e',
  'outline':                    '#74777f',
  'outline-variant':            '#c2c7cf',
}

console.log('/* ── Light mode colors → OKLCH ── */')
for (const [name, value] of Object.entries(colors)) {
  console.log(`--color-${name}: ${hex(value)};`)
}

console.log('\n/* ── Shadows with alpha ── */')
console.log(`--shadow-sm:  0 2px 8px ${hexAlpha('#191c1d', '0.04')};`)
console.log(`--shadow-md:  0 8px 32px ${hexAlpha('#191c1d', '0.07')}, 0 2px 8px ${hexAlpha('#191c1d', '0.04')};`)
console.log(`--shadow-lg:  0 16px 48px ${hexAlpha('#191c1d', '0.10')}, 0 4px 16px ${hexAlpha('#191c1d', '0.06')};`)
console.log(`--shadow-glass: 0 20px 60px ${hexAlpha('#191c1d', '0.14')}, 0 4px 16px ${hexAlpha('#191c1d', '0.08')};`)
console.log(`--shadow-primary-ring:       0 0 0 2px ${hexAlpha('#643fd5', '0.35')};`)
console.log(`--shadow-error-ring:         0 0 0 2px ${hexAlpha('#ba1a1a', '0.35')};`)
console.log(`--shadow-primary-lift:       0 4px 16px ${hexAlpha('#643fd5', '0.25')};`)
console.log(`--shadow-primary-lift-hover: 0 8px 24px ${hexAlpha('#643fd5', '0.35')};`)

console.log('\n/* ── Gradient ── */')
console.log(`--gradient-primary: linear-gradient(135deg, ${hex('#643fd5')}, ${hex('#7d5bf0')});`)
```

### Étape 1.3 — Exécuter le script

- [ ] Lancer le script et capturer la sortie

```bash
node scripts/convert-oklch.mjs
```

La sortie donne les valeurs OKLCH pour chaque token. Copier les valeurs dans `tokens.css`.

### Étape 1.4 — Mettre à jour tokens.css section :root

- [ ] Ouvrir `frontend/src/styles/tokens.css`. Dans le bloc `:root`, remplacer toutes les lignes couleur par les valeurs OKLCH de la sortie du script. Exemple de résultat attendu (valeurs exactes produites par le script) :

```css
:root {
  /* ── Couleurs sémantiques — Light ────────────────────────────── */
  --color-primary:               oklch(0.421 0.2195 276.97);
  --color-primary-container:     oklch(0.523 0.2321 281.28);
  --color-on-primary:            oklch(1.000 0.0000 0.00);
  --color-secondary:             oklch(0.432 0.0266 248.37);
  --color-secondary-container:   oklch(0.906 0.0347 271.64);
  --color-on-secondary-container:oklch(0.151 0.0309 271.42);
  --color-error:                 oklch(0.377 0.1754 23.64);
  --color-error-container:       oklch(0.921 0.0419 17.87);
  --color-on-error:              oklch(1.000 0.0000 0.00);
  --color-success:               oklch(0.483 0.1212 142.50);
  --color-success-container:     oklch(0.962 0.0289 147.59);
  --color-warning:               oklch(0.527 0.1741 39.38);
  --color-warning-container:     oklch(0.990 0.0272 96.76);
  /* roles — valeurs produites par le script */
  /* surfaces — valeurs produites par le script */
  /* on-surface — valeurs produites par le script */
  /* ⚠️ Les valeurs ci-dessus sont indicatives. Utiliser EXCLUSIVEMENT la sortie du script. */
```

> **IMPORTANT :** Ne pas saisir les valeurs ci-dessus manuellement. Utiliser uniquement la sortie de `node scripts/convert-oklch.mjs`. Les valeurs exactes dépendent de la version de culori installée.

- [ ] Remplacer également `--gradient-primary` par la sortie du script (format `linear-gradient(135deg, oklch(...), oklch(...))`).

- [ ] Remplacer les 8 lignes `--shadow-*` par la sortie du script (format `oklch(L C H / alpha)`).

- [ ] Changer `color-mix(in srgb, ...)` en `color-mix(in oklch, ...)` sur la ligne `--color-outline-light`.

### Étape 1.5 — Vérifier le build

- [ ] Lancer build et lint

```bash
npm --prefix frontend run build
npm --prefix frontend run lint
```

Expected : zéro erreur.

### Étape 1.6 — Commit

- [ ] Commit

```bash
git add frontend/src/styles/tokens.css scripts/convert-oklch.mjs
git commit -m "feat(frontend): migrate light mode color tokens to OKLCH (EP-020-S01)"
```

---

## Task 2 — Migration OKLCH dark mode + corrections contrastes (S02)

**Files:**
- Modify: `frontend/src/styles/tokens.css` (section `[data-theme="dark"]`)

### Étape 2.1 — Étendre le script pour le dark mode

- [ ] Ajouter à la fin de `scripts/convert-oklch.mjs` :

```js
console.log('\n\n/* ── Dark mode colors → OKLCH ── */')
const dark = {
  'primary':                    '#c4b5fd',
  'primary-container':          '#4c2fb8',
  'on-primary':                 '#2a0084',
  'secondary':                  '#bfc4dd',
  'secondary-container':        '#404659',
  'on-secondary-container':     '#dde1f9',
  'error':                      '#ffb4ab',
  'error-container':            '#93000a',
  'on-error':                   '#690005',
  'success':                    '#81c784',
  'success-container':          '#1b4f23',
  'warning':                    '#ffb74d',
  'warning-container':          '#4d3500',
  'role-manager-bg':            '#4a4458',
  'role-manager-fg':            '#e8def8',
  'role-board-bg':              '#00345b',
  'role-board-fg':              '#d2e4fb',
  'role-co-owner-bg':           '#00522b',
  'role-co-owner-fg':           '#cef5e5',
  'role-tenant-bg':             '#573200',
  'role-tenant-fg':             '#fcecd1',
  'surface':                    '#111318',
  'surface-lowest':             '#0c0e13',
  'surface-low':                '#191c21',
  'surface-container':          '#1e2026',
  'surface-high':               '#28292f',
  'surface-highest':            '#33353b',
  'on-surface':                 '#e2e3e8',
  'on-surface-variant':         '#c3c6cf',
  'outline':                    '#8e9099',
  'outline-variant':            '#43474f',
}
for (const [name, value] of Object.entries(dark)) {
  console.log(`--color-${name}: ${hex(value)};`)
}
console.log('\n/* ── Dark shadows with alpha ── */')
console.log(`--shadow-sm:  0 2px 8px ${hexAlpha('#000000', '0.16')};`)
console.log(`--shadow-md:  0 8px 32px ${hexAlpha('#000000', '0.22')}, 0 2px 8px ${hexAlpha('#000000', '0.16')};`)
console.log(`--shadow-lg:  0 16px 48px ${hexAlpha('#000000', '0.32')}, 0 4px 16px ${hexAlpha('#000000', '0.22')};`)
console.log(`--shadow-glass: 0 20px 60px ${hexAlpha('#000000', '0.40')}, 0 4px 16px ${hexAlpha('#000000', '0.28')};`)
console.log(`--shadow-primary-ring:       0 0 0 2px ${hexAlpha('#c4b5fd', '0.35')};`)
console.log(`--shadow-error-ring:         0 0 0 2px ${hexAlpha('#ffb4ab', '0.35')};`)
console.log(`--shadow-primary-lift:       0 4px 16px ${hexAlpha('#c4b5fd', '0.20')};`)
console.log(`--shadow-primary-lift-hover: 0 8px 24px ${hexAlpha('#c4b5fd', '0.30')};`)
console.log(`--gradient-primary: linear-gradient(135deg, ${hex('#7d5bf0')}, ${hex('#c4b5fd')});`)
```

### Étape 2.2 — Exécuter et mettre à jour tokens.css dark mode

- [ ] Lancer le script et mettre à jour le bloc `[data-theme="dark"]`

```bash
node scripts/convert-oklch.mjs 2>/dev/null | grep -A999 "Dark mode"
```

Copier les valeurs dans le bloc `[data-theme="dark"]` de `tokens.css`.

### Étape 2.3 — Vérifier les contrastes WCAG (visuel)

- [ ] Lancer le frontend en dev

```bash
npm --prefix frontend run dev
```

- [ ] Ouvrir l'app en dark mode. Vérifier visuellement sur 3 vues : BudgetView, AssemblyDetailView, TasksView.
  - Texte `on-surface` sur fond `surface` : lisible (contraste ≥ 4.5:1)
  - Role badges : fond sombre, texte clair
  - Boutons primary : contraste suffisant

- [ ] Si une couleur dark mode échoue visiblement au contraste, ajuster sa valeur L dans la ligne OKLCH correspondante (+/- 0.05) et rebuildder.

### Étape 2.4 — Nettoyer le script temporaire

- [ ] Supprimer le script et désinstaller culori

```bash
rm scripts/convert-oklch.mjs
npm --prefix frontend uninstall culori
```

### Étape 2.5 — Build + commit

- [ ] Vérifier build

```bash
npm --prefix frontend run build && npm --prefix frontend run lint
```

- [ ] Commit

```bash
git add frontend/src/styles/tokens.css frontend/package.json frontend/package-lock.json
git commit -m "feat(frontend): migrate dark mode tokens to OKLCH with contrast corrections (EP-020-S02)"
```

---

## Task 3 — Tokens Tailwind manquants (S03)

**Files:**
- Modify: `frontend/src/styles/tokens.css` (ajouter `--text-xl`, `--text-2xl`)
- Modify: `frontend/src/styles/tailwind.css` (section `@theme inline`)

### Étape 3.1 — Ajouter les tokens manquants dans tokens.css

- [ ] Dans `tokens.css`, section `/* ── Typographie ── */`, ajouter après `--text-lg: 16px;` :

```css
  --text-xl:   20px;
  --text-2xl:  24px;
  --text-display: 32px;
```

### Étape 3.2 — Mettre à jour @theme inline

- [ ] Dans `frontend/src/styles/tailwind.css`, dans le bloc `@theme inline {`, ajouter après les `--spacing-*` existants :

```css
	/* Typography sizes */
	--text-2xs: var(--text-2xs);
	--text-xs: var(--text-xs);
	--text-sm: var(--text-sm);
	--text-md: var(--text-md);
	--text-lg: var(--text-lg);
	--text-xl: var(--text-xl);
	--text-2xl: var(--text-2xl);
	--text-display: var(--text-display);

	/* Spacing gaps */
	--spacing-7: 28px;
	--spacing-9: 36px;

	/* Line heights */
	--leading-tight: 1.25;
	--leading-snug: 1.375;
	--leading-normal: 1.5;
	--leading-relaxed: 1.625;

	/* Z-index */
	--z-base: var(--z-base);
	--z-raised: var(--z-raised);
	--z-nav: var(--z-nav);
	--z-overlay: var(--z-overlay);
	--z-modal: var(--z-modal);
	--z-toast: var(--z-toast);
```

### Étape 3.3 — Build + commit

- [ ] Vérifier build

```bash
npm --prefix frontend run build && npm --prefix frontend run lint
```

- [ ] Commit

```bash
git add frontend/src/styles/tokens.css frontend/src/styles/tailwind.css
git commit -m "feat(frontend): register missing Tailwind tokens (text-xl/2xl, spacing-7/9, leading-*, z-*) (EP-020-S03)"
```

---

## Task 4 — CDropdownMenu wrapper Radix (S04)

**Files:**
- Create: `frontend/src/components/atoms/CDropdownMenu.vue`
- Modify: `docs/DESIGN.md` (§2.1 interactions table, §3.4 Radix table)

### Étape 4.1 — Mettre à jour DESIGN.md avant implémentation

- [ ] Dans `docs/DESIGN.md` §2.1, changer la ligne `CDropdownMenu` de `🔲 planifié` à `✅` et compléter les props.

- [ ] Dans §3.4, mettre à jour `DropdownMenu | CDropdownMenu | 🔲 planifié` en `✅`.

### Étape 4.2 — Créer CDropdownMenu.vue

- [ ] Créer `frontend/src/components/atoms/CDropdownMenu.vue` :

```vue
<script setup lang="ts">
import type { Component } from 'vue'
import {
  DropdownMenuRoot,
  DropdownMenuTrigger,
  DropdownMenuPortal,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
} from 'radix-vue'

export interface DropdownItem {
  label: string
  icon?: Component
  action: () => void
  destructive?: boolean
  disabled?: boolean
  separator?: boolean
}

const props = withDefaults(defineProps<{
  items: DropdownItem[]
  align?: 'start' | 'end' | 'center'
}>(), {
  align: 'end',
})

defineOptions({ inheritAttrs: false })
</script>

<template>
  <DropdownMenuRoot>
    <DropdownMenuTrigger v-bind="$attrs" as-child>
      <slot name="trigger" />
    </DropdownMenuTrigger>

    <DropdownMenuPortal>
      <DropdownMenuContent
        :align="props.align"
        :side-offset="4"
        class="min-w-[160px] rounded-radius-md bg-surface-container-lowest shadow-md p-1 outline-none z-overlay"
      >
        <template v-for="(item, i) in props.items" :key="i">
          <DropdownMenuSeparator
            v-if="item.separator"
            class="h-px bg-outline-variant my-1"
          />
          <DropdownMenuItem
            :disabled="item.disabled"
            class="flex items-center gap-2 px-3 py-2 text-md rounded-radius-sm cursor-pointer outline-none select-none
                   text-on-surface hover:bg-surface-container
                   data-[disabled]:opacity-40 data-[disabled]:cursor-not-allowed"
            :class="item.destructive ? 'text-error hover:bg-error-container' : ''"
            @select="item.action"
          >
            <component :is="item.icon" v-if="item.icon" class="size-4 shrink-0" />
            {{ item.label }}
          </DropdownMenuItem>
        </template>
      </DropdownMenuContent>
    </DropdownMenuPortal>
  </DropdownMenuRoot>
</template>
```

### Étape 4.3 — Vérifier que radix-vue est disponible

- [ ] Vérifier que `radix-vue` est dans les dépendances

```bash
grep -c "radix-vue" frontend/package.json
```

Expected : 1. Si 0 : `npm --prefix frontend install radix-vue`.

### Étape 4.4 — Build + lint

- [ ] Vérifier

```bash
npm --prefix frontend run build && npm --prefix frontend run lint
```

### Étape 4.5 — Commit

- [ ] Commit

```bash
git add frontend/src/components/atoms/CDropdownMenu.vue docs/DESIGN.md
git commit -m "feat(frontend): add CDropdownMenu Radix wrapper (EP-020-S04)"
```

---

## Task 5 — CStatusBanner organisme (S05)

**Files:**
- Create: `frontend/src/components/organisms/CStatusBanner.vue`
- Modify: `docs/DESIGN.md` (§2.3)

### Étape 5.1 — Mettre à jour DESIGN.md §2.3

- [ ] Changer `CStatusBanner | ... | 🔲 planifié` en `✅` dans la table des organismes.

### Étape 5.2 — Créer CStatusBanner.vue

- [ ] Créer `frontend/src/components/organisms/CStatusBanner.vue` :

```vue
<script setup lang="ts">
export type MeetingStatus = 'draft' | 'called' | 'ongoing' | 'closed'

const props = defineProps<{
  status: MeetingStatus
  label?: string
}>()

const toneClass: Record<MeetingStatus, string> = {
  draft:  'tone--muted',
  called: 'tone--info',
  ongoing:'tone--accent',
  closed: 'tone--valid',
}

const defaultLabel: Record<MeetingStatus, string> = {
  draft:   'Brouillon',
  called:  'Convoquée',
  ongoing: 'En cours',
  closed:  'Clôturée',
}
</script>

<template>
  <div
    :class="toneClass[props.status]"
    class="flex items-center gap-2 px-4 py-2 rounded-radius-md text-sm font-medium"
    style="background: var(--tone-bg); color: var(--tone-fg);"
    role="status"
  >
    <span class="size-2 rounded-radius-full shrink-0" style="background: currentColor; opacity: 0.6;" />
    {{ props.label ?? defaultLabel[props.status] }}
  </div>
</template>
```

### Étape 5.3 — Build + commit

- [ ] Build

```bash
npm --prefix frontend run build && npm --prefix frontend run lint
```

- [ ] Commit

```bash
git add frontend/src/components/organisms/CStatusBanner.vue docs/DESIGN.md
git commit -m "feat(frontend): add CStatusBanner organism for meeting status (EP-020-S05)"
```

---

## Task 6 — Refonte Radix : CFormCheckbox, vérification CFormSelect + CTooltip (S06)

**Files:**
- Modify: `frontend/src/components/atoms/CFormCheckbox.vue`
- Inspect + modify if needed: `frontend/src/components/atoms/CFormSelect.vue`, `frontend/src/components/atoms/CTooltip.vue`

### Étape 6.1 — Vérifier CFormSelect et CTooltip

- [ ] Lire `frontend/src/components/atoms/CFormSelect.vue` et `CTooltip.vue`. Chercher `from 'radix-vue'` dans chacun.

```bash
grep "from 'radix-vue'" frontend/src/components/atoms/CFormSelect.vue frontend/src/components/atoms/CTooltip.vue
```

Si les deux fichiers importent de `radix-vue` → ils sont déjà des wrappers Radix → passer à l'étape 6.3.
Si l'un ou l'autre n'importe pas de `radix-vue` → le refaire selon le pattern ci-dessous.

### Étape 6.2 — Refaire CFormSelect si non-Radix

*(Sauter si déjà Radix)*

- [ ] Remplacer le contenu de `frontend/src/components/atoms/CFormSelect.vue` :

```vue
<script setup lang="ts">
import {
  SelectRoot,
  SelectTrigger,
  SelectValue,
  SelectPortal,
  SelectContent,
  SelectViewport,
  SelectItem,
  SelectItemText,
} from 'radix-vue'

export interface SelectOption {
  value: string
  label: string
  disabled?: boolean
}

const model = defineModel<string>()

const props = defineProps<{
  label?: string
  options: SelectOption[]
  error?: string
  disabled?: boolean
  placeholder?: string
}>()

defineOptions({ inheritAttrs: false })
</script>

<template>
  <div class="flex flex-col gap-1">
    <label v-if="props.label" class="text-sm font-medium text-on-surface-variant">
      {{ props.label }}
    </label>
    <SelectRoot v-model="model" :disabled="props.disabled">
      <SelectTrigger
        v-bind="$attrs"
        class="flex items-center justify-between w-full h-[var(--control-height)] px-3 rounded-radius-md bg-surface-container
               border border-outline-variant text-on-surface text-md outline-none
               focus:ring-2 focus:ring-primary/30 data-[disabled]:opacity-50 data-[disabled]:cursor-not-allowed"
        :class="props.error ? 'border-error' : ''"
      >
        <SelectValue :placeholder="props.placeholder ?? 'Sélectionner…'" />
        <span class="text-on-surface-variant ml-2">▾</span>
      </SelectTrigger>
      <SelectPortal>
        <SelectContent
          position="popper"
          :side-offset="4"
          class="min-w-[var(--radix-select-trigger-width)] rounded-radius-md bg-surface-container-lowest shadow-md z-overlay overflow-hidden"
        >
          <SelectViewport class="p-1">
            <SelectItem
              v-for="opt in props.options"
              :key="opt.value"
              :value="opt.value"
              :disabled="opt.disabled"
              class="flex items-center px-3 py-2 text-md text-on-surface rounded-radius-sm cursor-pointer outline-none select-none
                     hover:bg-surface-container data-[highlighted]:bg-surface-container data-[disabled]:opacity-40"
            >
              <SelectItemText>{{ opt.label }}</SelectItemText>
            </SelectItem>
          </SelectViewport>
        </SelectContent>
      </SelectPortal>
    </SelectRoot>
    <p v-if="props.error" class="text-xs text-error">{{ props.error }}</p>
  </div>
</template>
```

### Étape 6.3 — Refaire CFormCheckbox avec Radix

- [ ] Remplacer le contenu de `frontend/src/components/atoms/CFormCheckbox.vue` :

```vue
<script setup lang="ts">
import { CheckboxRoot, CheckboxIndicator } from 'radix-vue'

const model = defineModel<boolean>()

const props = defineProps<{
  label?: string
  disabled?: boolean
  error?: string
}>()

defineOptions({ inheritAttrs: false })
</script>

<template>
  <div class="flex flex-col gap-1">
    <label class="flex items-center gap-2 cursor-pointer select-none" :class="props.disabled ? 'opacity-50 cursor-not-allowed' : ''">
      <CheckboxRoot
        v-bind="$attrs"
        v-model:checked="model"
        :disabled="props.disabled"
        class="size-4 rounded border border-outline-variant bg-surface-container-lowest
               data-[state=checked]:bg-primary data-[state=checked]:border-primary
               flex items-center justify-center outline-none
               focus-visible:ring-2 focus-visible:ring-primary/30"
      >
        <CheckboxIndicator class="text-on-primary">
          <svg viewBox="0 0 16 16" class="size-3" fill="none" stroke="currentColor" stroke-width="2.5">
            <path d="M3 8l3.5 3.5L13 4.5" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </CheckboxIndicator>
      </CheckboxRoot>
      <span v-if="props.label" class="text-sm text-on-surface">{{ props.label }}</span>
    </label>
    <p v-if="props.error" class="text-xs text-error">{{ props.error }}</p>
  </div>
</template>
```

### Étape 6.4 — Build + lint + vérification visuelle

- [ ] Build

```bash
npm --prefix frontend run build && npm --prefix frontend run lint
```

- [ ] Vérifier visuellement dans une vue avec formulaire (ex. ClaimsView → créer un sinistre) que les checkboxes fonctionnent.

### Étape 6.5 — Commit

- [ ] Commit

```bash
git add frontend/src/components/atoms/CFormCheckbox.vue frontend/src/components/atoms/CFormSelect.vue frontend/src/components/atoms/CTooltip.vue
git commit -m "feat(frontend): refactor CFormCheckbox/CFormSelect to native Radix wrappers (EP-020-S06)"
```

---

## Task 7 — CTabs + suppression tabs.css (S07)

**Files:**
- Create: `frontend/src/components/atoms/CTabs.vue`
- Modify: `frontend/src/styles/tailwind.css` (retirer import tabs.css)
- Delete: `frontend/src/styles/tabs.css`
- Modify: views using `.tabs`/`.tab` (à identifier via grep)
- Modify: `docs/DESIGN.md` (§2.1)

### Étape 7.1 — Identifier les consommateurs de tabs.css

- [ ] Grep les vues utilisant les classes de tabs.css

```bash
grep -rn "class=\"tabs\|class=\"tab\b\|class=\"tab--\|tab-count" frontend/src/features/ frontend/src/views/ --include="*.vue"
```

Lister les fichiers et lignes trouvés. Ce sont les vues à migrer.

### Étape 7.2 — Créer CTabs.vue

- [ ] Créer `frontend/src/components/atoms/CTabs.vue` :

```vue
<script setup lang="ts">
export interface Tab {
  value: string
  label: string
  count?: number
}

const model = defineModel<string>({ required: true })

const props = defineProps<{
  tabs: Tab[]
}>()
</script>

<template>
  <div
    class="flex flex-row overflow-x-auto gap-1 pb-px border-b border-outline-variant"
    style="-webkit-overflow-scrolling: touch; scrollbar-width: none;"
  >
    <button
      v-for="tab in props.tabs"
      :key="tab.value"
      type="button"
      class="flex items-center gap-1.5 px-5 py-2 text-sm font-medium whitespace-nowrap
             transition-colors text-on-surface-variant hover:text-on-surface
             border-b-2 border-transparent -mb-px"
      :class="model === tab.value
        ? 'border-primary text-primary'
        : ''"
      @click="model = tab.value"
    >
      {{ tab.label }}
      <span
        v-if="tab.count !== undefined"
        class="text-xs font-semibold text-primary"
      >{{ tab.count }}</span>
    </button>
  </div>
</template>
```

### Étape 7.3 — Mettre à jour DESIGN.md §2.1

- [ ] Changer la ligne `CTabs | ... | 🔲 planifié` en `✅` avec props `tabs`, `modelValue`.

### Étape 7.4 — Migrer les vues consommatrices

- [ ] Pour chaque vue identifiée en 7.1, remplacer le pattern HTML `<div class="tabs">` par `<CTabs>`. Exemple de migration :

**Avant :**
```html
<div class="tabs">
  <button
    v-for="tab in tabs"
    :key="tab.value"
    class="tab"
    :class="{ 'tab--active': activeTab === tab.value }"
    @click="activeTab = tab.value"
  >
    {{ tab.label }}
    <span v-if="tab.count" class="tab-count">{{ tab.count }}</span>
  </button>
</div>
```

**Après :**
```html
<CTabs v-model="activeTab" :tabs="tabs" />
```

Si la vue définit les tabs inline, créer la ref `tabs` du bon type :
```ts
import CTabs from '@/components/atoms/CTabs.vue'
import type { Tab } from '@/components/atoms/CTabs.vue'

const tabs: Tab[] = [
  { value: 'all', label: 'Tous' },
  { value: 'open', label: 'Ouverts', count: openCount.value },
]
```

### Étape 7.5 — Supprimer tabs.css

- [ ] Supprimer le fichier et son import

```bash
rm frontend/src/styles/tabs.css
```

- [ ] Dans `frontend/src/styles/tailwind.css`, supprimer la ligne `@import "./tabs.css";` si elle existe.

- [ ] Vérifier qu'aucune vue ne référence encore `.tabs` ou `.tab--active`

```bash
grep -rn "class=\"tabs\|class=\"tab\b\|tab--active\|tab-count" frontend/src/ --include="*.vue"
```

Expected : zéro résultat.

### Étape 7.6 — Build + commit

- [ ] Build

```bash
npm --prefix frontend run build && npm --prefix frontend run lint
```

- [ ] Commit

```bash
git add frontend/src/components/atoms/CTabs.vue docs/DESIGN.md frontend/src/styles/ frontend/src/features/ frontend/src/views/
git commit -m "feat(frontend): add CTabs atom, migrate all tab usages, delete tabs.css (EP-020-S07)"
```

---

## Task 8 — CHeading + migration typography.css (S08)

**Files:**
- Create: `frontend/src/components/atoms/CHeading.vue`
- Modify: ~20 vues (`features/**/*.vue`) — remplacement `.display-md` et `.headline-sm`
- Delete: `frontend/src/styles/typography.css`
- Modify: `docs/DESIGN.md` (§2.1)

### Étape 8.1 — Créer CHeading.vue

- [ ] Créer `frontend/src/components/atoms/CHeading.vue` :

```vue
<script setup lang="ts">
export type HeadingSize = 'lg' | 'md' | 'sm'
export type HeadingAs = 'h1' | 'h2' | 'h3' | 'p'

withDefaults(defineProps<{
  size?: HeadingSize
  as?: HeadingAs
}>(), {
  size: 'md',
  as: 'h2',
})

const sizeClass: Record<HeadingSize, string> = {
  lg: 'text-2xl',   // 24px — var(--text-2xl) enregistré en S03
  md: 'text-xl',    // 20px
  sm: 'text-lg',    // 18px — text-lg = 16px dans tailwind ; NOTE: vérifier mapping
}
</script>

<template>
  <component
    :is="$props.as"
    class="font-serif font-semibold leading-tight tracking-tight text-on-surface"
    :class="sizeClass[$props.size ?? 'md']"
  ><slot /></component>
</template>
```

> **Note :** `text-2xl` et `text-xl` utilisent les tokens enregistrés en Task 3 (S03). `text-lg` est déjà présent dans Tailwind. Si la taille `sm` (18px) ne correspond pas à `text-lg` (16px), ajouter `--text-heading-sm: 18px` dans tokens.css et l'enregistrer dans `@theme inline`.

### Étape 8.2 — Vérifier le mapping de taille sm

- [ ] Chercher la valeur Tailwind de `text-lg` après S03

```bash
grep "text-lg" frontend/src/styles/tailwind.css frontend/src/styles/tokens.css
```

Si `--text-lg: 16px` → le `sm` CHeading (18px) nécessite un token dédié. Dans ce cas :
- Ajouter `--text-heading-sm: 18px;` dans `tokens.css`
- Ajouter `--text-heading-sm: var(--text-heading-sm);` dans `@theme inline` de `tailwind.css`
- Changer `sm: 'text-lg'` en `sm: 'text-heading-sm'` dans `CHeading.vue`

### Étape 8.3 — Migrer les vues : .display-md → CHeading size="lg"

- [ ] Grep les usages de `.display-md` dans les vues

```bash
grep -rn "display-md\|display-sm\|display-lg" frontend/src/features/ frontend/src/views/ --include="*.vue"
```

- [ ] Pour chaque occurrence, remplacer le pattern `<h1 class="display-md">Titre</h1>` par :

```html
<CHeading size="lg" as="h1">Titre</CHeading>
```

Ajouter l'import en haut du `<script setup>` si absent :
```ts
import CHeading from '@/components/atoms/CHeading.vue'
```

### Étape 8.4 — Migrer les vues : .headline-sm → CHeading size="sm"

- [ ] Grep les usages de `.headline-*`

```bash
grep -rn "headline-sm\|headline-md\|headline-lg\|text-headline" frontend/src/features/ frontend/src/views/ --include="*.vue"
```

- [ ] Remplacer chaque occurrence. Exemple :

```html
<!-- Avant -->
<h2 class="headline-sm">Résolutions</h2>
<!-- Après -->
<CHeading size="sm" as="h2">Résolutions</CHeading>
```

### Étape 8.5 — Supprimer typography.css

- [ ] Vérifier qu'il n'y a plus d'usages dans les vues

```bash
grep -rn "display-md\|display-sm\|display-lg\|headline-sm\|headline-md\|text-headline\|body-lg\|body-md\|body-sm\|label-lg\|label-md\|data-md" frontend/src/features/ frontend/src/views/ --include="*.vue"
```

Si des usages `body-*`, `label-*`, `data-md` subsistent dans des vues : les remplacer par classes Tailwind équivalentes (ex. `body-md` → `text-md text-on-surface`, `label-sm` → `text-xs font-medium`).

- [ ] Supprimer le fichier

```bash
rm frontend/src/styles/typography.css
```

- [ ] Retirer l'import dans `tailwind.css` si présent.

### Étape 8.6 — Build + commit

- [ ] Build

```bash
npm --prefix frontend run build && npm --prefix frontend run lint
```

- [ ] Commit

```bash
git add -A
git commit -m "feat(frontend): add CHeading atom, migrate typography classes, delete typography.css (EP-020-S08)"
```

---

## Task 9 — CFormRow/CFormGroup/CFormLabel + migration forms.css (S09)

**Files:**
- Create: `frontend/src/components/molecules/CFormRow.vue`
- Create: `frontend/src/components/molecules/CFormGroup.vue`
- Create: `frontend/src/components/molecules/CFormLabel.vue`
- Modify: vues utilisant `.form-grid`, `.field*`, `.icon-btn`

### Étape 9.1 — Créer CFormLabel.vue

- [ ] Créer `frontend/src/components/molecules/CFormLabel.vue` :

```vue
<script setup lang="ts">
defineProps<{
  for?: string
  required?: boolean
}>()
</script>

<template>
  <label
    :for="$props.for"
    class="text-sm font-medium text-on-surface-variant"
  >
    <slot />
    <span v-if="$props.required" class="text-error ml-0.5">*</span>
  </label>
</template>
```

### Étape 9.2 — Créer CFormGroup.vue

- [ ] Créer `frontend/src/components/molecules/CFormGroup.vue` :

```vue
<script setup lang="ts">
defineProps<{
  full?: boolean
  hint?: string
  error?: string
}>()
</script>

<template>
  <div class="flex flex-col gap-1" :class="$props.full ? 'col-span-full' : ''">
    <slot />
    <p v-if="$props.hint && !$props.error" class="text-xs text-on-surface-variant">{{ $props.hint }}</p>
    <p v-if="$props.error" class="text-xs text-error">{{ $props.error }}</p>
  </div>
</template>
```

### Étape 9.3 — Créer CFormRow.vue

- [ ] Créer `frontend/src/components/molecules/CFormRow.vue` :

```vue
<script setup lang="ts">
defineProps<{
  cols?: 1 | 2
}>()
</script>

<template>
  <div
    class="flex flex-col gap-4"
    :class="$props.cols === 2 ? 'sm:grid sm:grid-cols-2 sm:gap-4' : ''"
  >
    <slot />
  </div>
</template>
```

### Étape 9.4 — Identifier les consommateurs de forms.css

- [ ] Grep les usages dans les vues

```bash
grep -rn "form-grid\|form-grid-2col\|class=\"field\b\|field--full\|field-hint\|field-error\|field-label\|icon-btn\|checkbox-label" frontend/src/features/ frontend/src/views/ --include="*.vue"
```

### Étape 9.5 — Migrer .form-grid et .field*

- [ ] Pour chaque vue, remplacer :

```html
<!-- Avant : form-grid -->
<div class="form-grid">
  <div class="field">
    <label class="field-label">Nom</label>
    <CFormInput ... />
    <p class="field-hint">Texte d'aide</p>
  </div>
</div>

<!-- Après -->
<CFormRow>
  <CFormGroup hint="Texte d'aide">
    <CFormLabel>Nom</CFormLabel>
    <CFormInput ... />
  </CFormGroup>
</CFormRow>
```

```html
<!-- Avant : form-grid-2col -->
<div class="form-grid form-grid-2col">

<!-- Après -->
<CFormRow :cols="2">
```

```html
<!-- Avant : field--full -->
<div class="field field--full">

<!-- Après -->
<CFormGroup :full="true">
```

### Étape 9.6 — Migrer .icon-btn

- [ ] Remplacer les usages de `.icon-btn` dans les vues :

```html
<!-- Avant -->
<button class="icon-btn icon-btn--danger" @click="...">
  <TrashIcon />
</button>

<!-- Après -->
<CBtn variant="ghost" size="sm" class="text-error" @click="...">
  <TrashIcon class="size-4" />
</CBtn>
```

`.icon-btn` simple (pas `--danger`) → `<CBtn variant="ghost" size="sm">`.

### Étape 9.7 — Build + lint

- [ ] Build

```bash
npm --prefix frontend run build && npm --prefix frontend run lint
```

> `forms.css` n'est PAS supprimé à cette étape (approche B — progressive). Commenter les classes migrées dans le fichier avec `/* migrated */`.

### Étape 9.8 — Commit

- [ ] Commit

```bash
git add -A
git commit -m "feat(frontend): add CFormRow/CFormGroup/CFormLabel, migrate form-grid and field classes (EP-020-S09)"
```

---

## Task 10 — CDataTable scoped + tables.css (S10)

**Files:**
- Modify: `frontend/src/components/molecules/CDataTable.vue` (ajouter styles scoped)
- Modify: 3 vues : `DocumentsView.vue`, `RegistryView.vue`, `ResidentsView.vue`
- Modify: `AllocationKeysView.vue`, `MyChargesView.vue` (`.cards-grid` → Tailwind inline)

### Étape 10.1 — Lire CDataTable.vue actuel

- [ ] Lire `frontend/src/components/molecules/CDataTable.vue` pour comprendre sa structure actuelle avant modification.

```bash
cat frontend/src/components/molecules/CDataTable.vue
```

### Étape 10.2 — Déplacer les styles .data-table dans CDataTable scoped

- [ ] Lire `frontend/src/styles/tables.css` pour copier les règles `.data-table`.

- [ ] Dans `CDataTable.vue`, ajouter/modifier le bloc `<style scoped>` pour inclure les règles de `.data-table`. Les classes internes de CDataTable utilisent déjà ces styles — si CDataTable n'a pas encore de `<table>` interne, vérifier si les 3 vues l'utilisent directement comme `<table class="data-table">` ou comme composant wrapper.

  - Si `CDataTable` est déjà un composant wrapper : ajouter les styles à `<table>` tag dans son template.
  - Si les 3 vues utilisent directement `<table class="data-table">` : ajouter un wrapper dans CDataTable et migrer les vues vers `<CDataTable>`.

### Étape 10.3 — Migrer .cards-grid

- [ ] Dans `AllocationKeysView.vue`, remplacer :

```html
<div class="cards-grid">
```

Par :

```html
<div class="grid grid-cols-[repeat(auto-fill,minmax(320px,1fr))] gap-4">
```

- [ ] Même chose dans `MyChargesView.vue`.

### Étape 10.4 — Build + lint

- [ ] Build

```bash
npm --prefix frontend run build && npm --prefix frontend run lint
```

### Étape 10.5 — Commit

- [ ] Commit

```bash
git add -A
git commit -m "feat(frontend): move data-table styles to CDataTable scoped, replace cards-grid inline (EP-020-S10)"
```

---

## Task 11 — Cleanup base.css + CEmptyState (S11)

**Files:**
- Create: `frontend/src/components/atoms/CEmptyState.vue`
- Modify: `frontend/src/styles/base.css` (retirer classes applicatives)
- Modify: ~21 vues (`.view-container`, `.view-header`, `.empty-state`, `.spinner`)
- Modify: `docs/DESIGN.md` (§2.1)

### Étape 11.1 — Créer CEmptyState.vue

- [ ] Créer `frontend/src/components/atoms/CEmptyState.vue` :

```vue
<script setup lang="ts">
defineProps<{
  message?: string
}>()
</script>

<template>
  <div class="flex flex-col items-center justify-center py-8 gap-2 text-center text-on-surface-variant">
    <slot name="icon" />
    <p class="text-sm">{{ $props.message }}</p>
    <slot />
  </div>
</template>
```

### Étape 11.2 — Migrer .view-container et .view-header (21 vues)

- [ ] Remplacer dans chaque vue :

```html
<!-- Avant -->
<div class="view-container">
  <div class="view-header">
    <h1 class="display-md">Titre</h1>
  </div>
  ...
</div>

<!-- Après -->
<div class="flex flex-col p-10 max-w-[1200px] gap-6 max-sm:p-5">
  <div class="flex flex-col items-start gap-2">
    <CHeading size="lg" as="h1">Titre</CHeading>
  </div>
  ...
</div>
```

Vues concernées (21) :
```
features/assemblies/AssemblyListView.vue
features/assemblies/AssemblyDetailView.vue
features/budget/BudgetView.vue
features/claims/ClaimsView.vue
features/consultations/ConsultationsView.vue
features/contractors/ContractorDetailView.vue
features/contractors/ContractorsView.vue
features/contracts/ContractsView.vue
features/dashboard/DashboardView.vue
features/documents/DocumentsView.vue
features/maintenance/MaintenanceView.vue
features/profile/ProfileView.vue
features/reports/ReportsView.vue
features/residence/AllocationKeysView.vue
features/residence/BuildingView.vue
features/residents/RegistryView.vue
features/residents/ResidentsView.vue
features/residents/MyChargesView.vue
features/residents/UnitChargesView.vue
features/tasks/TaskDetailView.vue
features/tasks/TasksView.vue
```

### Étape 11.3 — Migrer .empty-state → CEmptyState

- [ ] Grep les occurrences restantes

```bash
grep -rn "empty-state" frontend/src/features/ frontend/src/views/ --include="*.vue"
```

- [ ] Remplacer chaque occurrence :

```html
<!-- Avant -->
<div class="empty-state">Aucune donnée</div>

<!-- Après -->
<CEmptyState message="Aucune donnée" />
```

### Étape 11.4 — Migrer .spinner → CPageLoader

- [ ] Grep

```bash
grep -rn "class=\"spinner\|spinner-dark" frontend/src/ --include="*.vue"
```

- [ ] Remplacer par `<CPageLoader />` ou équivalent selon le contexte (inline spinner vs pleine page).

### Étape 11.5 — Nettoyer base.css

- [ ] Supprimer de `frontend/src/styles/base.css` les classes : `.view-container`, `.view-header`, `.empty-state`, `.loading-skeleton`, `.spinner`, `.spinner-dark`, `.card--terminated`.

  Garder uniquement les resets navigateur.

### Étape 11.6 — Build + lint

- [ ] Build

```bash
npm --prefix frontend run build && npm --prefix frontend run lint
```

### Étape 11.7 — Commit

- [ ] Commit

```bash
git add -A
git commit -m "feat(frontend): add CEmptyState, migrate view-container/empty-state to Tailwind, clean base.css (EP-020-S11)"
```

---

## Task 12 — Cleanup activity-cards.css (S12)

**Files:**
- Inspect: `features/assemblies/AssemblyListView.vue`, `features/claims/ClaimsView.vue`, `features/contractors/ContractorsView.vue`, `features/contracts/ContractsView.vue`, `features/tasks/TasksView.vue`
- Potentially create: `frontend/src/components/atoms/CActivityCard.vue`
- Delete: `frontend/src/styles/activity-cards.css`

### Étape 12.1 — Analyser le markup activity-card

- [ ] Grep les usages de `.activity-card` dans les 5 vues

```bash
grep -n "activity-card" frontend/src/features/assemblies/AssemblyListView.vue \
  frontend/src/features/claims/ClaimsView.vue \
  frontend/src/features/contractors/ContractorsView.vue \
  frontend/src/features/contracts/ContractsView.vue \
  frontend/src/features/tasks/TasksView.vue
```

- [ ] Lire le bloc HTML `.activity-card` dans 2 vues différentes (ex. AssemblyListView et TasksView) pour comparer le markup. Si le markup est identique ou quasi-identique → créer `CActivityCard`. Si très différent → migrer en styles scoped par vue.

### Étape 12.2a — Si markup similaire : créer CActivityCard.vue

- [ ] Créer `frontend/src/components/atoms/CActivityCard.vue` avec le markup commun et des slots pour les parties variables.

Structure de base (adapter selon le markup réel) :

```vue
<script setup lang="ts">
defineProps<{
  withIcon?: boolean
}>()
</script>

<template>
  <div
    class="flex flex-col gap-3"
    :class="$props.withIcon ? 'pl-10' : ''"
  >
    <div class="flex items-start gap-3">
      <slot name="icon" />
      <div class="flex flex-col gap-1 flex-1 min-w-0">
        <slot name="header" />
        <slot name="chips" />
        <slot name="title" />
      </div>
    </div>
    <slot name="documents" />
    <slot name="actions" />
  </div>
</template>
```

- [ ] Migrer chaque vue consommatrice pour utiliser `CActivityCard`.

### Étape 12.2b — Si markup divergent : scoped styles par vue

- [ ] Pour chaque vue, déplacer les styles `.activity-card` pertinents dans un bloc `<style scoped>` de la vue.

### Étape 12.3 — Supprimer activity-cards.css

- [ ] Vérifier qu'aucune vue ne référence encore les classes

```bash
grep -rn "activity-card" frontend/src/features/ frontend/src/views/ --include="*.vue" | grep -v "CActivityCard"
```

Expected : zéro ou uniquement des usages via `CActivityCard`.

- [ ] Supprimer le fichier

```bash
rm frontend/src/styles/activity-cards.css
```

- [ ] Retirer l'import dans `tailwind.css` si présent.

### Étape 12.4 — Mettre à jour DESIGN.md

- [ ] Si `CActivityCard` créé → ajouter à DESIGN.md §2.1.

### Étape 12.5 — Build final + commit

- [ ] Build complet

```bash
npm --prefix frontend run build && npm --prefix frontend run lint
```

- [ ] Vérifier visuellement les 5 vues en dev.

- [ ] Commit

```bash
git add -A
git commit -m "feat(frontend): migrate activity-card styles to scoped/CActivityCard, delete activity-cards.css (EP-020-S12)"
```

---

## Self-Review

**Couverture spec :**
- S01 ✅ Task 1, S02 ✅ Task 2, S03 ✅ Task 3
- S04 ✅ Task 4, S05 ✅ Task 5, S06 ✅ Task 6
- S07 ✅ Task 7, S08 ✅ Task 8, S09 ✅ Task 9
- S10 ✅ Task 10, S11 ✅ Task 12, S12 ✅ Task 12

**Dépendances respectées :**
- Task 3 (tokens --text-xl/2xl) avant Task 8 (CHeading) ✅
- Task 6 (CFormCheckbox Radix) avant Task 9 (absorbe checkbox-label) ✅

**Types cohérents :**
- `DropdownItem` défini dans Task 4, pas réutilisé ailleurs ✅
- `MeetingStatus` défini dans Task 5 ✅
- `Tab` défini dans Task 7, utilisé dans les vues en 7.4 ✅
- `HeadingSize`/`HeadingAs` définis dans Task 8 ✅

**Note Task 10 :** le plan demande de lire CDataTable avant modification (étape 10.1) car l'implémentation actuelle est inconnue — c'est intentionnel pour éviter une réécriture aveugle.

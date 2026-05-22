---
document: DESIGN
version: 1.0.0
owner: Damien
status: validé
last_updated: 2026-05-22
references:
  - DDR-001
  - DDR-002
  - DDR-003
  - DDR-004
  - DDR-005
  - DDR-006
  - DDR-007
description: >
  Source de vérité du design system "The Modern Registry" de La Ruche — tokens, catalogue de composants
  et guidelines UX. S'applique à l'intégralité du frontend Vue 3.
  Tout composant, style ou token appliqué en frontend doit être conforme à ce document.
rules:
  - Ce fichier fait foi. Aucune valeur brute (px, hex, rem) dans un attribut class si un token existe.
  - Tout nouveau composant atom/molecule/organism doit être ajouté au catalogue (§2) avant usage.
  - Toute nouvelle primitive interactive passe par un wrapper C* — jamais d'import direct radix-vue dans une vue.
  - Les classes .tone--* sont des utilitaires CSS purs (non Tailwind) — jamais redéfinis localement.
  - Zéro bloc <style> dans les vues de feature — mise en page locale = utilitaire Tailwind ou composant du catalogue.
  - CFormFeedback toujours dans #footer de CModal, jamais dans le body.
llm-directives:
  modification_policy: >
    Ne modifier ce fichier que si une évolution du design system global ou d'une guideline est
    ABSOLUMENT nécessaire, et EXCLUSIVEMENT après validation explicite d'un DDR additionnel.
    Toute modification : inline, succincte, dans une section existante.
    Si aucune section ne convient, requérir une validation utilisateur.
    Le corps du document ne contient aucune référence technique externe
    (ADR, numéro de feature ou de ticket) — uniquement les DDR.
  scope: frontend, composants Vue, tokens CSS, guidelines UX, accessibilité
  do_not_touch: >
    Les règles d'architecture backend, les contraintes DB et les obligations légales
    restent dans ARCHITECTURE.md et LEGAL.md. Ne pas les dupliquer ici.
    Les règles produit métier (workflows, permissions) restent dans PRD.md.
---

# DESIGN.md — Design System "The Modern Registry"

> **Dénomination** — « l'Application » désigne exclusivement **La Ruche**, plateforme de gestion participative de copropriété.  
> Dernière mise à jour : 2026-05-21

---

## 1. Tokens

### 1.1 Typographie

Trois polices, trois rôles. Chargées depuis Google Fonts.

```html
<link href="https://fonts.googleapis.com/css2?family=Fraunces:ital,opsz,wght@0,9..144,100..900;1,9..144,100..900&family=DM+Sans:ital,opsz,wght@0,9..40,300..500;1,9..40,300..400&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
```

**Familles** — tokens nécessaires, non couverts nativement par Tailwind :

| Token | Valeur | Classe Tailwind | Usage |
|---|---|---|---|
| `--font-display` | `'Fraunces', Georgia, serif` | `font-display` | Titres de page, KPI ≥ 28px |
| `--font-body` | `'DM Sans', system-ui, sans-serif` | `font-body` | Corps, labels, navigation |
| `--font-mono` | `'JetBrains Mono', ui-monospace` | `font-mono` | Montants tableaux, codes, numéros lot |

> **Fraunces** produit des chiffres proportionnels non tabulaires → interdit en colonne. Utiliser `font-mono` avec `lnum tnum` pour les montants.

**Échelle de taille** (`--text-*`) — exposée via `@theme inline`, remplace l'échelle Tailwind native :

| Token | Valeur | Classe Tailwind | Usage |
|---|---|---|---|
| `--text-2xs` | `11px` | `text-2xs` | Eyebrows, footnotes, chips de statut |
| `--text-xs`  | `12px` | `text-xs`  | Dates, captions, labels secondaires |
| `--text-sm`  | `13px` | `text-sm`  | Lignes de tableau, list items, texte UI par défaut |
| `--text-md`  | `14px` | `text-md`  | Corps de texte |
| `--text-lg`  | `16px` | `text-lg`  | Titres de section, en-têtes de carte |

> `text-sm` (13px) et `text-lg` (16px) divergent de Tailwind natif (14px et 18px). Toujours utiliser nos tokens.

**Graisses** — utiliser les utilitaires Tailwind natifs : `font-normal` (400), `font-medium` (500), `font-semibold` (600), `font-bold` (700), `font-[800]` pour Fraunces display.

> Les styles typographiques nommés sont encapsulés dans les composants atomiques `CHeading`, `CEyebrow` et `CDataText` (voir §2.1).

---

### 1.2 Palette de couleurs

Source : `frontend/src/styles/tokens.css`. Les valeurs Dark sont dans `[data-theme="dark"]`.

| Token | Light | Dark |
|---|---|---|
| `--color-primary` | `#643fd5` | `#c4b5fd` |
| `--color-primary-container` | `#7d5bf0` | `#4c2fb8` |
| `--color-on-primary` | `#ffffff` | `#2a0084` |
| `--color-secondary` | `#5a6073` | `#bfc4dd` |
| `--color-secondary-container` | `#dde1f9` | `#404659` |
| `--color-on-secondary-container` | `#1a1d2e` | `#dde1f9` |
| `--color-error` | `#ba1a1a` | `#ffb4ab` |
| `--color-error-container` | `#ffdad6` | `#93000a` |
| `--color-success` | `#2e7d32` | `#81c784` |
| `--color-success-container` | `#e8f5e9` | `#1b4f23` |
| `--color-warning` | `#e65100` | `#ffb74d` |
| `--color-warning-container` | `#fff8e1` | `#4d3500` |
| `--gradient-primary` | `linear-gradient(135deg, #643fd5, #7d5bf0)` | — |

**Surfaces** (hiérarchie : Page → Section → Carte → Champ) :

| Token | Light | Dark |
|---|---|---|
| `--color-surface` | `#f8f9fa` | `#111318` |
| `--color-surface-container-lowest` | `#ffffff` | `#0c0e13` |
| `--color-surface-container-low` | `#f3f4f5` | `#191c21` |
| `--color-surface-container` | `#edeff0` | `#1e2026` |
| `--color-surface-container-high` | `#e7e9ea` | `#28292f` |
| `--color-surface-container-highest` | `#e1e3e4` | `#33353b` |
| `--color-on-surface` | `#191c1d` | `#e2e3e8` |
| `--color-on-surface-variant` | `#42474e` | `#c3c6cf` |
| `--color-outline` | `#74777f` | `#8e9099` |
| `--color-outline-variant` | `#c2c7cf` | `#43474f` |

**Rôles** :

| Token (`-bg` / `-fg`) | Background light | Foreground light |
|---|---|---|
| `--color-role-manager-*` | `#e8def8` | `#21005d` |
| `--color-role-board-*` | `#d2e4fb` | `#001d36` |
| `--color-role-co-owner-*` | `#cef5e5` | `#00391d` |
| `--color-role-tenant-*` | `#fcecd1` | `#3a2000` |

---

### 1.3 Espacement

| Token | Valeur | | Token | Valeur |
|---|---|---|---|---|
| `--space-1`  | `4px`  | | `--space-8`  | `32px` |
| `--space-2`  | `8px`  | | `--space-10` | `40px` |
| `--space-3`  | `12px` | | `--space-12` | `48px` |
| `--space-4`  | `16px` | | `--space-16` | `64px` |
| `--space-5`  | `20px` | | `--space-20` | `80px` |
| `--space-6`  | `24px` | | | |

---

### 1.4 Rayons & Ombres

| Token | Valeur |
|---|---|
| `--radius-sm`   | `8px` |
| `--radius-md`   | `12px` |
| `--radius-lg`   | `16px` |
| `--radius-xl`   | `24px` |
| `--radius-full` | `9999px` |

> Jamais `0px` sur un élément interactif.

| Token | Valeur |
|---|---|
| `--shadow-sm`    | `0 2px 8px rgba(25,28,29,.04)` |
| `--shadow-md`    | `0 8px 32px rgba(25,28,29,.07), 0 2px 8px rgba(25,28,29,.04)` |
| `--shadow-lg`    | `0 16px 48px rgba(25,28,29,.10), 0 4px 16px rgba(25,28,29,.06)` |
| `--shadow-glass` | `0 20px 60px rgba(25,28,29,.14), 0 4px 16px rgba(25,28,29,.08)` |

**Ombres de focus et d'interaction** — utilisées par les composants `CForm*` :

| Token | Usage |
|---|---|
| `--shadow-primary-ring` | Focus ring sur les champs et boutons primaires |
| `--shadow-error-ring` | Focus ring sur un champ en état d'erreur |
| `--shadow-primary-lift` | Élévation hover d'un bouton primaire |
| `--shadow-primary-lift-hover` | Élévation pressed/active d'un bouton primaire |

> **Glassmorphism modales :** `background: rgba(255,255,255,.85)` + `backdrop-filter: blur(20px)`.

---

### 1.5 Icônes

| Token | Valeur | Usage |
|---|---|---|
| `--icon-2xs` | `12px` | Meta footer |
| `--icon-xs`  | `14px` | Contenu, sous-titres, chips |
| `--icon-sm`  | `16px` | Inline, boutons, chevrons |
| `--icon-md`  | `18px` | Cartes, headers |
| `--icon-lg`  | `24px` | Icônes proéminentes |

> Toutes les icônes sont des composants SVG Vue (`CIcon*.vue` dans `components/atoms/`). Jamais de font d'icônes ni de `<img>`.

---

### 1.6 Tonalités (`tones.css`)

Source : `frontend/src/styles/tones.css`. Les classes `.tone--*` définissent **uniquement** deux variables CSS — aucune règle de forme, taille ou layout.

Les composants consomment :
```css
background: var(--tone-bg, <fallback>);
color:      var(--tone-fg, <fallback>);
```

| Classe | `--tone-bg` | `--tone-fg` |
|---|---|---|
| `.tone--accent`       | `--gradient-primary`                | `--color-on-primary` |
| `.tone--board-member` | `--color-role-board-bg`             | `--color-role-board-fg` |
| `.tone--co-owner`     | `--color-role-co-owner-bg`          | `--color-role-co-owner-fg` |
| `.tone--default`      | `--color-secondary-container`       | `--color-primary` |
| `.tone--disabled`     | `--color-surface-container-highest` | `--color-on-surface-variant` |
| `.tone--err`          | `--color-error-container`           | `--color-error` |
| `.tone--info`         | `--color-secondary-container`       | `--color-on-secondary-container` |
| `.tone--manager`      | `--color-role-manager-bg`           | `--color-role-manager-fg` |
| `.tone--muted`        | `--color-surface-container`         | `--color-on-surface-variant` |
| `.tone--tenant`       | `--color-role-tenant-bg`            | `--color-role-tenant-fg` |
| `.tone--valid`        | `--color-success-container`         | `--color-success` |
| `.tone--warn`         | `--color-warning-container`         | `--color-warning` |

> Une vue ne doit jamais redéclarer localement une classe `.tone--*` dans son CSS scoped.

---

### 1.7 Transparence

Tokens de mélange pour bordures et overlays légers. Valeur adaptée au thème pour maintenir un contraste cohérent.

| Token | Light | Dark | Usage |
|---|---|---|---|
| `--tint` | `15%` | `20%` | Opacité du composant source dans un `color-mix` |

**Tokens dérivés** :

| Token | Valeur | Usage |
|---|---|---|
| `--color-outline-light` | `color-mix(in srgb, var(--color-outline) var(--tint), transparent)` | Séparateurs discrets dans les tableaux |

```css
/* Exemple d'utilisation dans CDataTable */
border-color: var(--color-outline-light);
```

> Toujours utiliser `--color-outline-light` pour les séparateurs de tableau plutôt qu'une valeur `rgba` codée en dur.

---

## 2. Catalogue de composants

### 2.1 Atomes (`components/atoms/`)

| Composant | Props clés | Description |
|---|---|---|
| `CAlert` | `variant`, `title` | Bandeau d'alerte contextuelle |
| `CBadge` | `variant` | Chip de statut ou de catégorie — consomme `.tone--*` |
| `CBtn` | `variant`, `size`, `loading`, `disabled` | Bouton — variantes `primary`, `secondary`, `ghost`, `destructive` |
| `CDataText` | `size` | Texte tabulaire JetBrains Mono — `font-mono font-medium lining-nums tabular-nums`. `size`: `sm` (défaut) \| `md` \| `lg` |
| `CEyebrow` | — | Libellé eyebrow — `font-body text-2xs font-medium uppercase tracking-[0.06em]`. Rendu en `<div>`. |
| `CFormCheckbox` | `label`, `checked` | Case à cocher — wrapper Radix |
| `CFormChipInput` | `label`, `value` | Saisie d'un chip unitaire (la boucle `v-for` est dans le parent) |
| `CFormChipSelect` | `label`, `options` | Sélection de chips |
| `CFormDropzone` | `accept`, `multiple` | Zone de dépôt de fichiers |
| `CFormFeedback` | `status`, `message` | Retour de formulaire — **toujours dans `#footer` de `CModal`** |
| `CFormInput` | `type`, `label`, `error`, `hint` | Champ texte — variante `password` avec toggle de visibilité |
| `CFormSelect` | `label`, `options`, `error` | Sélecteur — wrapper Radix `SelectRoot` |
| `CFormTextarea` | `label`, `rows`, `error` | Zone de texte |
| `CHeading` | `size`, `as` | Titre typographique Fraunces — semibold, italic. `size`: `lg` (24px) \| `md` (20px) \| `sm` (18px). `as`: `h1`…`h3` \| `p` |
| `CIconWrapper` | `size` (2xs/xs/sm/md/lg) | Wrapper dimensionnel pour icône SVG |
| `CProgressBar` | `value`, `max` | Barre de progression |
| `CSeparator` | `orientation` | Séparateur — wrapper Radix `Separator` |
| `CTooltip` | `content`, `placement` | Info-bulle — wrapper Radix Tooltip |

---

### 2.2 Molécules (`components/molecules/`)

| Composant | Props clés | Description |
|---|---|---|
| `CCard` | `shadow?` | Conteneur générique — définit background, border-radius, padding et flux interne. Pour les listes avec séparateurs : `CFlexGroup`. Pour les listes sans séparateurs : wrapper Tailwind inline. Ossature d'une carte d'entité : `CCardHeader` › titre › description › `CMeta` › actions › `CCardNote`. |
| `CCardHeader` | `withIcon?` | En-tête de carte — slots `#icon`, `#chips` |
| `CCardNote` | `note` | Encart d'information secondaire dans une `CCard` |
| `CDataTable` | `columns`, `rows` | Tableau de données |
| `CDocumentList` | `documents`, `readonly?` | Liste documents avec download/retrait — `emit('remove', doc)` au parent |
| `CEmptyState` | `icon`, `title`, `description?` | État vide — slot `#actions` |
| `CFormAddressAutocomplete` | `label`, `value` | Champ adresse avec autocomplétion |
| `CFormGroup` | `label`, `hint?` | Wrapper label + champ + hint |
| `CFormLabel` | `for`, `required?` | Label de formulaire |
| `CFormRow` | `cols` (1/2) | Grille formulaire 2 colonnes responsive |
| `CKpiCard` | `title`, `value`, `tone` | Carte indicateur KPI |
| `CLegalCard` | — | Carte de note légale (wraps `CLegalNotice`) |
| `CLegalNotice` | `descriptions`, `links` | Bloc notices légales structurées |
| `CListItem` | — | Ligne de liste générique |
| `CMeta` | — | Conteneur de métadonnées |
| `CMetaItem` | `variant`, `icon` | Ligne icône + contenu — variantes `footer` / `sub` |
| `CMetaLine` | — | Rangée horizontale de `CMetaItem` avec `CSeparator` |
| `CModal` | `open`, `title`, `saving?`, `feedback?` | Dialogue Radix `Dialog` — émettre `@submit` pour soumission |
| `CPageHeader` | `eyebrow?`, `title`, `meta?`, `icon?`, `frozenLabel?` | En-tête de vue — slot `#actions` |
| `CPageLoader` | — | Indicateur de chargement |
| `CSectionHeader` | `title`, `subtitle?`, `level?` | En-tête de section dans une `CCard` — slot `#actions` |
| `CTopbarBreadcrumb` | `crumbs[]` | Fil d'Ariane topbar |

---

### 2.3 Organismes (`components/organisms/`)

| Composant | Description |
|---|---|
| `AppNav` | Navigation principale (sidebar desktop + bottom nav mobile) |
| `AuthBrandPanel` | Panneau éditorial des écrans d'authentification |
| `CConfirmDialog` | Dialogue de confirmation destructive |
| `CDocumentLinkActionModal` | Modale gestion des documents liés (retrait / suppression) |
| `CFlexGroup` | Conteneur flex groupé |
| `CLinkedDocumentsModal` | Modale liste des documents liés à une entité |
| `CPageError` | Affichage d'erreur pleine page |
| `CReminderCard` | Carte de rappels synthétiques partagée entre vues |
| `CTopbar` | Barre supérieure avec breadcrumb et actions globales |

---

## 3. Règles globales

### 3.1 Accessibilité

- Contraste corps ≥ 4.5:1 (WCAG AA) — critique ≥ 7:1
- Focus visible : `outline: 2px solid var(--color-primary); outline-offset: 2px`
- Touch targets : min **44×44px** imposé dans `base.css` sur tous les `button`  
  Exception : boutons-icônes embarqués dans une zone interactive plus large → `min-width: unset; min-height: unset`
- Jamais la couleur seule pour transmettre une information
- Toute primitive interactive (Select, Checkbox, Tooltip, DropdownMenu) passe par un wrapper Radix Vue

---

### 3.2 Internationalisation

```ts
Intl.NumberFormat('fr-FR', { style: 'currency', currency: 'EUR' })
Intl.DateTimeFormat('fr-FR')
```

Tous les libellés passent par `vue-i18n` — `t('...')`. Fichiers de traduction dans `src/locales/`.

---

### 3.3 Tailwind v4

Configuration via `@tailwindcss/vite` (pas de `tailwind.config.js`). Tokens exposés via `@theme inline` dans `frontend/src/styles/tailwind.css`.

Tokens custom exposés via `@theme inline` dans `frontend/src/styles/tailwind.css` :

| Utilitaire Tailwind | Token source | Remarque |
|---|---|---|
| `text-2xs` · `text-xs` · `text-sm` · `text-md` · `text-lg` | `--text-*` | Remplace l'échelle Tailwind native — `text-sm` 13px, `text-lg` 16px |
| `font-display` · `font-body` · `font-mono` | `--font-*` | Familles custom |
| `space-1` … `space-20` | `--space-*` | — |
| `radius-sm` … `radius-full` | `--radius-*` | — |
| `shadow-md` · `shadow-lg` · `shadow-glass` | `--shadow-*` | — |

**Règles :**
- Jamais de valeur brute (`px`, `hex`, `rem`) dans un attribut `class` si un token Tailwind existe.
- Les classes `.tone--*` restent des utilitaires CSS purs (non Tailwind) — utiliser directement.
- **Zéro bloc `<style>` dans les vues de feature.** Toute mise en page locale = utilitaire Tailwind **ou** composant du catalogue. Les composants `atoms/molecules/organisms` peuvent conserver des `<style scoped>`.

---

### 3.4 Radix Vue

Toute primitive Radix Vue est enveloppée dans un composant `C*` avant usage dans une vue. Jamais d'import direct de `radix-vue` dans un fichier de vue ou de feature.

| Primitive Radix | Wrapper | Statut |
|---|---|---|
| `Checkbox` | `CFormCheckbox` | 🔲 planifié |
| `Dialog` | `CModal` | ✅ |
| `DropdownMenu` | `CDropdownMenu` | 🔲 planifié |
| `Select` | `CFormSelect` | 🔲 planifié |
| `Separator` | `CSeparator` | ✅ |
| `Tooltip` | `CTooltip` | 🔲 planifié |

---

### 3.5 Composants flottants (tooltips, dropdowns, popovers)

Tout élément flottant : `Teleport to="body"` + `position: fixed` + position via `getBoundingClientRect()`. Jamais `position: absolute` dans un parent scrollable ou `overflow: hidden`.

Comportement : flip vertical si espace insuffisant, clampage horizontal viewport avec marge 10px. Référence : `CTooltip.vue`.

---

### 3.6 Règle No-Line

Jamais de `1px solid` pour séparer du contenu hors structures tabulaires — décalage tonal (`surface-container-*`), espacement ou micro-ombre.

**Exception tableaux :** utiliser `--color-outline-light` (voir §1.7) pour les séparateurs de cellule. Ce token garantit une bordure adaptée au thème sans valeur `rgba` codée en dur.

---

### 3.7 États interactifs

Les états interactifs sont gérés localement dans chaque composant — pas de token système pour hover/active. Conventions à respecter :

| État | Convention |
|---|---|
| **Hover carte cliquable** | Surface : `--color-surface-container-low` → `--color-surface-container` |
| **Hover bouton primary** | `box-shadow: var(--shadow-primary-lift)` |
| **Active / pressed bouton** | `box-shadow: var(--shadow-primary-lift-hover)` |
| **Focus champ** | `box-shadow: var(--shadow-primary-ring)` · `outline: 2px solid var(--color-primary)` |
| **Focus champ en erreur** | `box-shadow: var(--shadow-error-ring)` |
| **Disabled** | `.tone--disabled` — jamais `opacity: 0.5` (contraste insuffisant) |

---

## 4. Guidelines

### 4.1 Patterns transversaux

#### Eyebrows de page

- Toute page métier avec un `h1` visible expose un eyebrow (`CEyebrow` ou prop `eyebrow` de `CPageHeader`).
- 3 à 6 mots — décrit ce que l'utilisateur trouve sur la page, ne répète pas le breadcrumb.
- Jamais redéfini localement.

#### Navigation & breadcrumb

- **≤ 1024px** : bottom nav 4 items primaires filtrés par rôle + tiroir « Plus » + item Profil.
- **> 1024px** : sidebar fixe 240px structurée en 5 sections non-cliquables.

  | Section | Destinations |
  |---|---|
  | *(solo)* | Tableau de bord |
  | Gouvernance | AG · Consultations · Signalements |
  | Finance | Budget · Contrats |
  | Opérations | Tâches · Sinistres · Carnet · Documents |
  | Résidence | Bâtiment · Clés de répartition · Registre · Copropriétaires |

- Bloc utilisateur en pied de sidebar → panel contextuel (Mon compte · Notifications · Déconnexion).
- Changement de thème dans `Profil > Mon compte` uniquement (`Clair` / `Sombre` / `Auto`).
- `CTopbarBreadcrumb` : fluide (`min-width: 0`), libellés tronquent proprement.

#### Écrans d'authentification

Sur mobile, les écrans publics (`/login`, `/onboarding`, invitation, reset) conservent un en-tête compact de marque sans sortir le formulaire du viewport initial. Le formulaire reste la zone scrollable principale.

#### États de chargement & d'erreur

- Toute vue qui charge ses données au montage passe par `usePageLoader`.
- État d'erreur → `CPageError`. État de chargement → `CPageLoader`.
- Les chargements secondaires non bloquants restent isolés.

#### Cartes de liste

`CCard` est un conteneur générique (background, border-radius, padding, flux). Chaque entité de liste est une `CCard`. La mise en liste est de la responsabilité de la vue :

- **Avec séparateurs** → `CFlexGroup` comme wrapper de liste
- **Sans séparateurs** → div Tailwind inline (`flex flex-col gap-*`)

**Ossature commune d'une carte d'entité :**

```
CCard (conteneur)
├── CCardHeader    — chips de statut / type toujours visibles
├── Titre principal
├── Description courte (optionnel)
├── CMeta / CMetaItem  — métadonnées compactes
├── Ligne d'actions    — « Modifier » + action contextuelle accentuée
└── CCardNote          — bloc informatif secondaire (délai, remboursement, lien…)
```

**Règles :** titres en `font-body text-lg`, jamais `font-display` · métadonnées via `CMetaItem` · encarts via `CCardNote` · pas de séparateur entre cartes.

Chaque vue définit ses propres variantes dans ses **règles feature** (§4.2).

#### KPI

Les indicateurs et filtres KPI utilisent `CKpiCard` (molécule) et `CBtn` avec `.tone--*` pour les pills de filtre. La valeur numérique (`font-display text-[56px] font-[800] tracking-[-0.04em]`) est rendue directement dans `CKpiCard` — ce style display 56px est exclusif à `CKpiCard`, pas à `CHeading`.

#### Gestion documentaire liée

- Toute entité qui accepte des documents expose la gestion depuis sa modale d'édition (et depuis la vue détail si elle existe).
- Le retrait d'un document ouvre toujours `CDocumentLinkActionModal` avec deux actions : **Supprimer le lien seulement** / **Supprimer le lien et le document**.
- `CDocumentList` émet `remove(doc)` — le parent gère l'appel API (endpoints différents par entité).
- Un document encore rattaché à d'autres entités n'est pas supprimé.
- Dropzones → `CFormDropzone` : bordure pointillée, état hover/drag.

#### Obligations légales

Les notes légales de bas de page : `CCard` + `CLegalNotice` avec `CLegalDescription` (`⚖️ + titre + résumé`) + liens Légifrance / service-public.fr. Les obligations ponctuelles sur un libellé peuvent utiliser `CTooltip`.

#### Entités comptables gelées

Toute entité en état terminal comptable (levy call `issued`, tâche `closed`, sinistre avec `transmitted_at`…) :

- **Indicateur** : `CIconLock` + libellé en `.tone--muted` dans le header.
- **Actions** : Modifier et Supprimer supprimés via `v-if` (jamais `disabled`).
- **Champs immuables** : `<span>` (jamais `<input readonly>`) + mention « Donnée comptable — non modifiable » en `label-sm .tone--muted`.
- **Pattern B (avenant)** : « Créer un avenant » remplace « Modifier » (`ghost`, non destructif).
- **Historique** : entités avec `predecessor_id` / `parent_budget_id` → section « Historique » en lecture seule.

#### CFormFeedback dans les modales

Tout `CModal` avec formulaire ou action destructive inclut `CFormFeedback` dans `#footer`, avant les boutons.

```vue
<CModal :open="open" title="…" @close="close">
  <form @submit.prevent="save"><!-- champs --></form>
  <template #footer>
    <CFormFeedback :status="feedback.status.value" :message="feedback.message.value" />
    <CBtn variant="ghost" @click="close">Annuler</CBtn>
    <CBtn variant="primary" :loading="saving" @click="save">Enregistrer</CBtn>
  </template>
</CModal>
```

`CFormFeedback` ne doit jamais apparaître dans `<template #default>`. Composable : `useFormFeedback.ts` — appeler `feedback.reset()` à l'ouverture.

#### Reminder cards

Les rappels synthétiques partagés entre vues passent par `CReminderCard`. Ne pas recopier localement : `section-head`, `reminder-list`, `reminder-row`, `reminder-dot`.

---

### 4.2 Règles par feature

#### Vues AG

- Fiche AG : statut, date AG, date limite convocation (J-21), résolutions, état documentaire (convocation / annexes / PV) lisibles immédiatement.
- AG `closed` → lecture seule complète (actions mutantes supprimées via `v-if`).
- Rôles non gestionnaires : lecture seule sur la préparation. Locataire : résultats et PV publié uniquement.
- Bandeau d'état AG (`draft` / `called` / `ongoing` / `closed`) implémenté inline dans la vue avec des helpers Tailwind — pas de composant dédié. Les autres entités utilisent `CBadge` dans `CEyebrow`.
- Carte de présence latérale : mode de participation par lot + indicateur de tantièmes, indépendante des votes saisis.

#### Vue Contrats

- **Carte contrat** : icône métier + titre + chip statut dans `CCardHeader` · chip domaine/type sur ligne dédiée sous le header.
- **Carte prestataire** : même structure, sans chip de statut.
- Rappels à traiter en tête avec action d'acquittement directe.
- Tabs : `Tous` · `Actifs` · `Expirés` avec volume.
- Contrat `active` → `amount`, `start_date`, `end_date` gelés → « Créer un avenant » remplace « Modifier ».
- Fiche contrat expose la chaîne des avenants via `predecessor_id`.
- Suppression uniquement depuis la modale d'édition.

#### Vue Tâches

- **Carte** : titre + chips priorité et statut dans `CCardHeader`, sans icône.
- Champ prestataire accepte une saisie libre — autocomplete si correspondance, suggestion d'ajout au référentiel si non (optionnel).
- Métadonnée prestataire : `CIconWrench` (externe) ou `CIconUser` (auto-assignation) — visible uniquement si assignation réelle.
- Bloc lien prestataire (token actif) : `Générer` · `Afficher` · `Révoquer` au-dessus de la ligne d'actions.

#### Vue Signalements

- **Carte** : icône + chip statut + date de mise à jour relative dans `CCardHeader`.
- Conversion → tâche : choisir la priorité dans la même modale (`Normal` par défaut) → redirection vers la tâche créée.
- `Supprimer` uniquement dans la modale d'édition.

#### Vue Sinistres

- **Carte** : icône obligatoire (couleur selon type) dans `CCardHeader` · chip de type à gauche du chip statut.
- Délai légal restant avant déclaration à l'assurance visible directement sur la carte.
- Tabs : `Tous` · `Ouverts` · `En cours` · `Clos` avec volume.
- Actions contextuelles strictes : `Ouvert → Transmettre` · `En cours → Clore` · `Clos → Supprimer`.
- `Supprimer` uniquement dans la modale d'édition. Note légale délais via `CCard + CLegalNotice`.

#### Vue Carnet d'entretien

- Filtres calendaires : `Mois` · `Trimestre` (défaut) · `Semestre` · `Année` · `Tout`.
- Navigation `précédent / suivant` déplace d'une période complète.
- Bloc réglementaire en tête : adresse immeuble + syndic en exercice.

---

## Appendice — DDR

| Réf | Titre | Résumé |
|---|---|---|
| DDR-001 v3 | [Typographie](ddr/DDR-001-typographie.md) | Fraunces (display) + DM Sans (corps) + JetBrains Mono (data tabulaire) |
| DDR-002 | [Couleurs](ddr/DDR-002-couleurs.md) | Palette lavande/charcoal, light + dark, tokens CSS vars uniquement |
| DDR-003 | [Navigation](ddr/DDR-003-navigation.md) | Bottom nav mobile, sidebar desktop, filtrage par rôle |
| DDR-004 | [Navigation sidebar](ddr/DDR-004-navigation-sidebar.md) | Groupes sidebar (pattern B — labels fixes), dropdown profil, routes `/residence/*` et `/profile/*` |
| DDR-005 | [États de chargement](ddr/DDR-005-page-loading-states.md) | `usePageLoader` + `CPageError` + `CPageLoader` — chargements non bloquants isolés |
| DDR-006 | CFormFeedback modales | `CFormFeedback` dans le slot `#footer` de toute `CModal` avec formulaire ou action destructive |
| DDR-007 | [Spécification UX composants](./../outputs/bmad/planning-artifacts/ux-design-specification.md) | Catalogue Tailwind v4 + Radix Vue + nouveaux composants headers/molécules (`CPageHeader`, `CSectionHeader`, `CCardHeader`, `CTopbarBreadcrumb`, `CDocumentList`, `CEmptyState`, `CStatusBanner`…) |

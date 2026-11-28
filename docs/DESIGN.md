---
document: DESIGN
version: 2.1.0
owner: Damien
status: validé
last_updated: 2026-10-22
references:
  - DDR-001
  - DDR-002
  - DDR-003
  - DDR-004
  - DDR-005
  - DDR-006
description: >
  Source de vérité du design system "The Modern Registry" de La Ruche — tokens, catalogue de composants
  et guidelines UX. S'applique à l'intégralité du frontend Vue 3.
  Tout composant, style ou token appliqué en frontend doit être conforme à ce document.
rules:
  - Ce fichier fait foi. Aucune valeur brute (px, hex, rem) dans un attribut class si un token existe.
  - Tout nouveau composant atom/molecule/organism doit être ajouté au catalogue (§3) avant usage.
  - Toute nouvelle primitive interactive passe par un wrapper C* — jamais d'import direct radix-vue dans une vue.
  - Les classes .tone--* sont des utilitaires CSS purs (non Tailwind) — jamais redéfinis localement.
  - Zéro bloc <style> dans les vues de feature — mise en page locale = utilitaire Tailwind ou composant du catalogue.
  - `CFormFeedback` jamais directement dans les templates — toujours via `CFormActions` (footer modal) ou `CForm` (inline). Cf. DDR-006.
  - Composant = conteneur de mise en page. Utilitaire Tailwind = style de texte. Voir §1.3.

---

# DESIGN.md — Design System "The Modern Registry"

> **Dénomination** — « l'Application » désigne exclusivement **La Ruche**, plateforme de gestion participative de copropriété.  

---

## 0. Identité produit & principes de design

La Ruche est une webapp open source de gestion participative de copropriété, conçue pour les syndics bénévoles français. Elle rend la gestion d'immeuble transparente, collaborative et accessible sans les coûts ni la complexité des logiciels professionnels. Le produit distingue clairement les actes légaux contraignants (votes AG, appels de fonds) des décisions informelles (consultations, sondages).

### Personnalité de marque

Sobre, collaboratif, civique, éducatif.

Ton : direct et humain. Pas de jargon juridique brut, mais pas de condescendance non plus. Le produit parle à des adultes compétents qui ont besoin d'être guidés, pas infantilisés.

Émotion cible : confiance et sérénité. L'utilisateur doit sentir qu'il est bien organisé, que rien ne lui échappe, que la complexité légale est gérée pour lui.

### Principes de design

1. **La complexité légale s'explique, elle ne s'impose pas.** Quand la loi contraint une action (délai, format, majorité requise), l'interface l'indique avec le contexte suffisant pour comprendre — jamais un champ vide avec une obligation cachée dans les docs.

2. **Confiance avant action irréversible.** Appels de fonds émis, PV signés, données anonymisées : ces actes sont irréversibles. Le design marque la différence entre préparer et valider, entre brouillon et acte légal. La confirmation n'est pas de la friction, c'est du soin.

3. **Un rôle, une lecture.** Les mêmes données s'affichent différemment selon le rôle. Les restrictions de permission ne se vivent jamais comme une punition : un locataire qui n'a pas accès à la comptabilité voit un état vide clair, pas une erreur 403.

4. **Sobre sur la forme, riche sur le fond.** Pas de décoration gratuite. L'espace blanc, la typographie et la couleur servent l'information, pas l'inverse. La richesse est dans la donnée bien présentée, pas dans l'UI.

5. **Le guidage est une fonctionnalité.** Wizards, états vides actifs, tooltips légaux et rappels contextuels ne sont pas des détails UX — ils sont le produit pour un syndic bénévole non-technique. Les ignorer revient à abandonner l'utilisateur primaire.

### Références produit

- **Matera** — clarté du parcours syndic, densité assumée de l'information métier
- **Qonto** — précision des états financiers, hiérarchie des données comptables, feedback instantané sur les actions
- **Goodvest** — pédagogie intégrée à l'interface (tooltips contextuels, wizards progressifs), ton rassurant sans être condescendant
- **Luko** — wizards d'onboarding et de déclaration : progression claire, une seule question par écran, abandon facile

### Anti-références

- Logiciels syndic traditionnels (Gestimmo, Syndic One, ICS Copro) — interfaces tabulaires grises, formulaires à 40 champs, aucune guidance
- Outils enterprise lourds (Jira, Monday, SAP) — navigation complexe, tout-sur-tout, sentiment d'usine
- Portails administratifs français de 2010 — zéro feedback, arborescences profondes
- Dashboards SaaS génériques — grille de KPI cards identiques, gradient text, purple-on-dark sans motif

---

## 1. Règles globales

> **Règles critiques à mémoriser :** (1) Jamais de `1px solid` coloré comme accent — cf. §1.7. (2) `CFormFeedback` uniquement dans `#footer` de `CModal` — cf. DDR-006. (3) Pas de `z-index` codé en dur — utiliser `var(--z-*)` cf. §2.9. (4) Pas d'import direct `radix-vue` dans une vue — cf. §1.5. (5) Composant = layout. Utilitaire = texte — cf. §1.3.

### 1.1 Accessibilité

- Contraste corps ≥ 4.5:1 (WCAG AA) — critique ≥ 7:1
- Focus visible : `outline: 2px solid var(--color-primary); outline-offset: 2px`
- Touch targets : min **44×44px** imposé dans `base.css` sur tous les `button`  
  Exception : boutons-icônes embarqués dans une zone interactive plus large → `min-width: unset; min-height: unset`
- Jamais la couleur seule pour transmettre une information
- Toute primitive interactive (Select, Checkbox, Tooltip, DropdownMenu) passe par un wrapper Radix Vue
- Jamais d'icône seule pour une action destructive — libellé textuel explicite
- Messages d'erreur en langage naturel, jamais de code technique brut
- Usage mobile prioritaire pour les rôles résidents (copropriétaire, locataire, prestataire) ; desktop prioritaire pour les rôles gestionnaires (syndic, conseil syndical)

---

### 1.2 Internationalisation

```ts
Intl.NumberFormat('fr-FR', { style: 'currency', currency: 'EUR' })
Intl.DateTimeFormat('fr-FR')
```

Tous les libellés passent par `vue-i18n` — `t('...')`. Fichiers de traduction dans `src/locales/`.

---

### 1.3 Philosophie Composants / Utilitaires

La règle fondamentale du système de design :

**Les composants définissent la mise en page.** Un composant `C*` est responsable de :
- Mode d'affichage (`flex`, `grid`, `block`)
- Espacement interne (`padding`, `gap`)
- Dimensions (`width`, `max-width`, `height`)
- Bordures, rayons, ombres
- Couleur de fond / dégradé
- Alignement (`items-*`, `justify-*`, `self-*`)

**Les utilitaires Tailwind définissent le style de texte.** Un `@utility` ou une classe Tailwind est réservé(e) à :
- Famille de police, taille, graisse
- Interlignage, espacement de lettres
- Couleur du texte
- Décoration, transformation de texte

**Règle :** tout groupement de classes de mise en page qui se répète 3 fois ou plus dans les vues → créer un composant. Tout groupement de propriétés texte qui se répète → créer un `@utility` dans `tailwind.css`.

**Frontière :** un composant ne définit jamais de style textuel inline. Un utilitaire ne définit jamais de propriété de mise en page.

```vue
<!-- ✅ Correct -->
<CCard gap="sm">             <!-- layout : le composant -->
  <h3 class="title-md">…</h3>  <!-- texte : l'utilitaire -->
</CCard>

<!-- ❌ Incorrect -->
<div class="flex flex-col gap-3 rounded-lg bg-surface p-4">  <!-- layout inline interdit en vue -->
  <h3 style="font-size: 16px; font-weight: 600">…</h3>       <!-- style textuel inline interdit -->
</div>
```

---

### 1.4 Tailwind v4

Configuration via `@tailwindcss/vite` (pas de `tailwind.config.js`). Tokens exposés via `@theme inline` dans `frontend/src/styles/tailwind.css`.

**Tokens custom exposés via `@theme inline` :**

| Utilitaire Tailwind | Token source | Remarque |
|---|---|---|
| `text-2xs` · `text-xs` · `text-sm` · `text-md` · `text-lg` · `text-xl` · `text-2xl` · `text-display` | `--text-*` | Remplace l'échelle Tailwind native — `text-sm` 13px, `text-lg` 16px, `text-display` 40px |
| `font-serif` · `font-body` · `font-mono` | `--font-*` | Familles custom |
| `space-1` … `space-20` (dont `space-7`, `space-9`) | `--space-*` | — |
| `radius-full` | `--radius-full` | Border-radius complet — `9999px` (seul token custom conservé) |
| `shadow-md` · `shadow-lg` · `shadow-glass` | `--shadow-*` | — |
| `leading-tight` · `leading-snug` · `leading-normal` · `leading-relaxed` | `--leading-*` | Interlignage — jamais de valeur `em` ou `px` codée en dur |

**Utilitaires `@utility` (styles nommés réutilisables) :**

| Utilitaire | Propriétés | Usage |
|---|---|---|
| `body-xs` / `body-sm` / `body-md` / `body-lg` | `font-size` + `font-weight` + `line-height` | Corps de texte hiérarchisé |
| `title-sm` / `title-md` / `title-lg` | `font-size` + `font-weight` + `letter-spacing` | Titres dans les composants |
| `label-sm` / `label-md` / `label-lg` | `font-size` + `font-weight` + `letter-spacing` + `text-transform` | Labels, eyebrows |
| `data-md` | `font-mono` + `tabular-nums` + `font-medium` | Données numériques tabulaires |
| `view-container` | `flex flex-col` + `p-10` + `max-w-[1200px]` + `gap-6` + responsive `p-5` | Conteneur racine de chaque vue |
| `view-header` | `flex flex-col items-start gap-2` | En-tête de vue (eyebrow + heading + meta) |
| `form-hint` | `text-xs` + `color: on-surface-variant` | Texte d'aide sous un champ |
| `form-label` | `text-sm font-medium` + `color: on-surface-variant` | Label de champ de formulaire |

**Règles :**
- Jamais de valeur brute (`px`, `hex`, `rem`) dans un attribut `class` si un token Tailwind existe.
- Les classes `.tone--*` restent des utilitaires CSS purs (non Tailwind) — utiliser directement.
- **Zéro bloc `<style>` dans les vues de feature.** Toute mise en page locale = utilitaire Tailwind **ou** composant du catalogue. Les composants `atoms/molecules/organisms` peuvent conserver des `<style scoped>`.
  - **Exception** faite si **2 éléments ou plus** ayant **une fonction identique** (layout, style de texte, même niveau hiérarchique) définissent **3 utilitaires identiques ou plus**.

---

### 1.5 Radix Vue

Toute primitive Radix Vue est enveloppée dans un composant `C*` avant usage dans une vue. Jamais d'import direct de `radix-vue` dans un fichier de vue ou de feature.

| Primitive Radix | Wrapper | Statut |
|---|---|---|
| `Checkbox` | `CFormCheckbox` | ✅ |
| `Dialog` | `CModal` | ✅ |
| `DropdownMenu` | `CDropdownMenu` | ✅ |
| `Select` | `CFormSelect` | ✅ |
| `Listbox` | `CFormMultiSelect` | ✅ |
| `Popover` | `CFormMultiSelect` | ✅ |
| `Separator` | `CSeparator` | ✅ |
| `Tooltip` | `CTooltip` | ✅ |

---

### 1.6 Composants flottants (tooltips, dropdowns, popovers)

Tout élément flottant : `Teleport to="body"` + `position: fixed` + position via `getBoundingClientRect()`. Jamais `position: absolute` dans un parent scrollable ou `overflow: hidden`.

Comportement : flip vertical si espace insuffisant, clampage horizontal viewport avec marge 10px. Référence : `CTooltip.vue`.

---

### 1.7 Règle No-Line

Jamais de `1px solid` pour séparer du contenu hors structures tabulaires — décalage tonal (`surface-container-*`), espacement ou micro-ombre.

**Exception tableaux :** utiliser `--color-outline-light` (voir §2.7) pour les séparateurs de cellule. Ce token garantit une bordure adaptée au thème sans valeur `rgba` codée en dur.

---

### 1.8 États interactifs

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

### 1.9 Architecture CSS

**Sources de vérité — immuables, jamais redéclarées :**

| Fichier | Rôle |
|---|---|
| `styles/tokens.css` | Variables CSS de design (couleurs, espace, typographie, ombres, z-index) |
| `styles/tones.css` | Utilitaires de tonalité sémantique — seules classes CSS pures autorisées hors composants |
| `styles/tailwind.css` | Exposition des tokens via `@theme inline` + `@utility` nommés + import Tailwind |

**Règle unique :** tout nouveau pattern d'interface devient soit un composant atomique `C*` avec `<style scoped>`, soit une combinaison d'utilitaires Tailwind. Aucun nouveau fichier CSS global n'est créé. Les fichiers legacy ci-dessous ne doivent plus être étendus.

**Fichiers legacy — à migrer, ne plus étendre :**

| Fichier | Patterns concernés | Migration cible |
|---|---|---|
| `tabs.css` | `.tabs` · `.tab` · `.tab--active` · `.tab-count` | ✅ migré vers `CTabs` (EP-020-S07) — fichier supprimé |
| `typography.css` | `.display-*` · `.headline-*` · `.title-*` · `.body-*` | ✅ migré vers `CHeading` + `@utility` (EP-020-S08) — fichier supprimé |
| `forms.css` | `.form-grid` · `.field` · `.search-bar` · `.dropzone` · `.icon-btn` | `CFormRow`, `CFormGroup` molecules + atoms `CFormInput`, `CFormDropzone` existants + Tailwind inline (EP-020-S09) — fichier encore présent, reliquats à migrer |
| `tables.css` | `.cards-grid` | ✅ `.cards-grid` → Tailwind `grid grid-cols-[…]` inline (EP-020-S10) — fichier supprimé |
| `base.css` | `.view-container` · `.view-header` · `.card--terminated` · `.loading-skeleton` · `.spinner` | `view-container` / `view-header` utilitaires `@utility` ; `CEmptyState` (✅) ; `CPageLoader` (✅) ; resets navigateur conservés (EP-020-S11) |

---

## 2. Tokens

### 2.1 Typographie

Trois polices, trois rôles. Chargées depuis Google Fonts.

```html
<link href="https://fonts.googleapis.com/css2?family=Fraunces:ital,opsz,wght@0,9..144,100..900;1,9..144,100..900&family=DM+Sans:ital,opsz,wght@0,9..40,300..500;1,9..40,300..400&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
```

**Familles** — tokens nécessaires, non couverts nativement par Tailwind :

| Token | Valeur | Classe Tailwind | Usage |
|---|---|---|---|
| `--font-display` | `'Fraunces', Georgia, serif` | `font-display` | Titres de page, grands libellés éditoriaux — **jamais pour des chiffres** |
| `--font-body` | `'DM Sans', system-ui, sans-serif` | `font-body` | Corps, labels, navigation |
| `--font-mono` | `'JetBrains Mono', ui-monospace` | `font-mono` | Montants tableaux, codes, numéros lot, KPI numériques |

> **Fraunces** produit des chiffres proportionnels non tabulaires → interdit en colonne. Utiliser `font-mono` avec `lnum tnum` pour les montants.

**Échelle de taille** (`--text-*`) — exposée via `@theme inline`, remplace l'échelle Tailwind native :

| Token | Valeur | Classe Tailwind | Usage |
|---|---|---|---|
| `--text-2xs` | `11px` | `text-2xs` | Eyebrows, footnotes, chips de statut |
| `--text-xs`  | `12px` | `text-xs`  | Dates, captions, labels secondaires |
| `--text-sm`      | `13px` | `text-sm`      | Lignes de tableau, list items, texte UI par défaut |
| `--text-md`      | `14px` | `text-md`      | Corps de texte |
| `--text-lg`      | `16px` | `text-lg`      | Titres de section, en-têtes de carte |
| `--text-xl`      | `20px` | `text-xl`      | Titres de carte — correspond à `CHeading sm/md` |
| `--text-2xl`     | `24px` | `text-2xl`     | Titres de section majeurs — correspond à `CHeading lg` |
| `--text-display` | `40px` | `text-display` | KPI numériques, valeurs comptables proéminentes — `font-mono lnum tnum` |

> `text-sm` (13px) et `text-lg` (16px) divergent de Tailwind natif (14px et 18px). `text-xl` et `text-2xl` couvrent les titres de `CHeading`; `text-display` remplace tout usage arbitraire `text-[56px]`. Jamais de valeur `text-[…px]` codée en dur.

**Graisses** — utiliser les utilitaires Tailwind natifs : `font-normal` (400), `font-medium` (500), `font-semibold` (600), `font-bold` (700), `font-[800]` pour Fraunces display.

> Les styles typographiques nommés sont encapsulés dans les composants atomiques `CHeading`, `CEyebrow` et `CDataText` (voir §3.1).

---

### 2.2 Palette de couleurs

Source : `frontend/src/styles/tokens.css`. Les valeurs Dark sont dans `[data-theme="dark"]`.

> **Espace colorimétrique :** toute couleur est exprimée en `oklch` (tokens, `color-mix(in oklch, …)`, tones). Les littéraux sRGB/hex sont interdits hors `styles/tokens.css`, les composants verrouillés (`CMockBanner`, `AuthBrandPanel`, voile de progression de `CBtn` — blanc obligatoire sur un fond saturé dans les deux thèmes) et les exceptions documentées (data-URI du check de case, `@media print`). Mélanger un token et un alpha se fait par `color-mix(in oklch, var(--color-x) n%, transparent)` dans la classe consommatrice — jamais via un token dérivé déclaré une seule fois dans `:root`.

| Token | Light | Dark |
|---|---|---|
| `--color-primary` | `oklch(0.503 0.2154 287.24)` | `oklch(0.811 0.1013 293.57)` |
| `--color-on-success` | `oklch(1.000 0.0000 0.00)` | `color-mix(in oklch, var(--color-success), black 70%)` |
| `--color-scrim` | `oklch(0.224 0.0049 219.69 / 0.4)` | `oklch(0.000 0.0000 0.00 / 0.6)` |
| `--color-primary-container` | `oklch(0.587 0.2131 288.56)` | `oklch(0.434 0.2006 283.98)` |
| `--color-on-primary` | `oklch(1.000 0.0000 0.00)` | `oklch(0.302 0.1811 281.25)` |
| `--color-secondary` | `oklch(0.491 0.0313 271.42)` | `oklch(0.824 0.0354 276.63)` |
| `--color-secondary-container` | `oklch(0.914 0.0329 278.20)` | `oklch(0.396 0.0331 271.15)` |
| `--color-on-secondary-container` | `oklch(0.236 0.0330 275.89)` | `oklch(0.914 0.0329 278.20)` |
| `--color-error` | `oklch(0.506 0.1927 27.70)` | `oklch(0.838 0.0891 26.76)` |
| `--color-error-container` | `oklch(0.918 0.0418 25.23)` | `oklch(0.417 0.1702 27.38)` |
| `--color-success` | `oklch(0.523 0.1347 144.17)` | `oklch(0.766 0.1179 145.30)` |
| `--color-success-container` | `oklch(0.957 0.0210 147.64)` | `oklch(0.381 0.0909 146.40)` |
| `--color-warning` | `oklch(0.631 0.1970 40.25)` | `oklch(0.829 0.1452 73.54)` |
| `--color-warning-container` | `oklch(0.979 0.0310 92.94)` | `oklch(0.348 0.0719 80.56)` |
| `--gradient-primary` | `linear-gradient(135deg, oklch(0.503 0.2154 287.24), oklch(0.587 0.2131 288.56))` | `linear-gradient(135deg, oklch(0.587 0.2131 288.56), oklch(0.811 0.1013 293.57))` |
| `--gradient-brand` | `linear-gradient(160deg, oklch(0.413 0.1755 286.61), oklch(0.503 0.2154 287.24) 60%, oklch(0.606 0.2189 292.72))` | idem (thème verrouillé) |
| `--gradient-success` | dérivé de `--color-success` (foncé → base) | auto |
| `--gradient-error` | dérivé de `--color-error` (foncé → base) | auto |
| `--gradient-warning` | dérivé de `--color-warning` (foncé → base) | auto |
| `--color-primary-tint` | `--color-primary` @ 10% (color-mix) | auto |

**Surfaces** (hiérarchie : Page → Section → Carte → Champ) :

| Token | Light | Dark |
|---|---|---|
| `--color-surface` | `oklch(0.982 0.0017 247.84)` | `oklch(0.187 0.0107 268.11)` |
| `--color-surface-container-lowest` | `oklch(1.000 0.0000 0.00)` | `oklch(0.164 0.0111 268.01)` |
| `--color-surface-container-low` | `oklch(0.967 0.0017 247.84)` | `oklch(0.225 0.0107 260.68)` |
| `--color-surface-container` | `oklch(0.951 0.0025 228.79)` | `oklch(0.244 0.0118 270.90)` |
| `--color-surface-container-high` | `oklch(0.933 0.0025 228.79)` | `oklch(0.282 0.0110 278.15)` |
| `--color-surface-container-highest` | `oklch(0.915 0.0026 228.79)` | `oklch(0.329 0.0109 271.07)` |
| `--color-on-surface` | `oklch(0.224 0.0049 219.69)` | `oklch(0.917 0.0068 277.16)` |
| `--color-on-surface-variant` | `oklch(0.396 0.0135 256.76)` | `oklch(0.827 0.0130 271.26)` |
| `--color-outline` | `oklch(0.569 0.0128 269.45)` | `oklch(0.655 0.0135 275.99)` |
| `--color-outline-variant` | `oklch(0.828 0.0125 259.82)` | `oklch(0.397 0.0145 264.40)` |

**Rôles** :

| Token (`-bg` / `-fg`) | Background light | Foreground light | Background dark | Foreground dark |
|---|---|---|---|---|
| `--color-role-manager-*` | `oklch(0.916 0.0365 303.11)` | `oklch(0.242 0.1404 286.09)` | `oklch(0.401 0.0340 298.59)` | `oklch(0.916 0.0365 303.11)` |
| `--color-role-board-*` | `oklch(0.913 0.0369 254.57)` | `oklch(0.225 0.0601 247.68)` | `oklch(0.317 0.0856 248.05)` | `oklch(0.913 0.0369 254.57)` |
| `--color-role-co-owner-*` | `oklch(0.938 0.0457 168.57)` | `oklch(0.303 0.0747 155.07)` | `oklch(0.385 0.0963 154.41)` | `oklch(0.938 0.0457 168.57)` |
| `--color-role-tenant-*` | `oklch(0.949 0.0395 80.90)` | `oklch(0.272 0.0601 67.02)` | `oklch(0.355 0.0789 66.30)` | `oklch(0.949 0.0395 80.90)` |

---

### 2.3 Espacement

| Token | Valeur | | Token | Valeur |
|---|---|---|---|---|
| `--space-1`  | `4px`  | | `--space-7`  | `28px` |
| `--space-2`  | `8px`  | | `--space-8`  | `32px` |
| `--space-3`  | `12px` | | `--space-9`  | `36px` |
| `--space-4`  | `16px` | | `--space-10` | `40px` |
| `--space-5`  | `20px` | | `--space-12` | `48px` |
| `--space-6`  | `24px` | | `--space-16` | `64px` |
| | | | `--space-20` | `80px` |

---

### 2.4 Rayons & Ombres

| Token | Valeur |
|---|---|
| `--radius-full` | `9999px` |

> Jamais `0px` sur un élément interactif.

| Token | Valeur |
|---|---|
| `--shadow-sm`    | `0 2px 8px oklch(0.224 0.0049 219.69 / .04)` |
| `--shadow-md`    | `0 8px 32px oklch(0.224 0.0049 219.69 / .07), 0 2px 8px oklch(0.224 0.0049 219.69 / .04)` |
| `--shadow-lg`    | `0 16px 48px oklch(0.224 0.0049 219.69 / .10), 0 4px 16px oklch(0.224 0.0049 219.69 / .06)` |
| `--shadow-glass` | `0 20px 60px oklch(0.224 0.0049 219.69 / .14), 0 4px 16px oklch(0.224 0.0049 219.69 / .08)` |

**Ombres de focus et d'interaction** — utilisées par les composants `CForm*` :

| Token | Usage |
|---|---|
| `--shadow-primary-ring` | Focus ring sur les champs et boutons primaires |
| `--shadow-error-ring` | Focus ring sur un champ en état d'erreur |
| `--shadow-primary-lift` | Élévation hover d'un bouton primaire |
| `--shadow-primary-lift-hover` | Élévation pressed/active d'un bouton primaire |

> **Surface des modales :** `background: var(--color-surface-container-lowest)` + `box-shadow: var(--shadow-glass)`. Le glassmorphisme (`.c-modal--frosted` : `color-mix(in oklch, var(--color-surface-container-lowest) 85%, transparent)` + `backdrop-filter: blur(20px)`) est un modificateur opt-in réservé aux surfaces qui l'exigent explicitement — jamais la surface par défaut.

---

### 2.5 Icônes

| Token | Valeur | Usage |
|---|---|---|
| `--icon-2xs` | `12px` | Meta footer |
| `--icon-xs`  | `14px` | Contenu, sous-titres, chips |
| `--icon-sm`  | `16px` | Inline, boutons, chevrons |
| `--icon-md`  | `18px` | Cartes, headers |
| `--icon-lg`  | `24px` | Icônes proéminentes |

> Toutes les icônes sont des composants SVG Vue (`CIcon*.vue` dans `components/atoms/`). Jamais de font d'icônes ni de `<img>`.

---

### 2.6 Tonalités (`tones.css`)

Source : `frontend/src/styles/tones.css`. Les classes `.tone--*` définissent **uniquement** deux variables CSS — aucune règle de forme, taille ou layout. 

La classe `.toned` définit le schéma d'utilisation des classes `.tone--*`:
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
| `.tone--iris`         | `--color-role-manager-bg`           | `--color-role-manager-fg` |
| `.tone--apricot`      | `oklch(0.942 0.0334 65.96)` | `oklch(0.418 0.1105 49.96)` |
| `.tone--manager`      | `--color-role-manager-bg`           | `--color-role-manager-fg` |
| `.tone--muted`        | `--color-surface-container`         | `--color-on-surface-variant` |
| `.tone--primary`      | `color-mix(primary 10%)`            | `--color-on-surface` |
| `.tone--tenant`       | `--color-role-tenant-bg`            | `--color-role-tenant-fg` |
| `.tone--valid`        | `--color-success-container`         | `--color-success` |
| `.tone--warn`         | `--color-warning-container`         | `--color-warning` |

> Une vue ne doit jamais redéclarer localement une classe `.tone--*` dans son CSS scoped.
> `.tone--apricot` a une déclinaison dark (`oklch(0.414 0.1000 48.00)` / `oklch(0.942 0.0334 65.96)`) ; `.tone--iris` suit la palette manager, donc son dark vient des tokens de rôle.

---

### 2.7 Transparence

Tokens de mélange pour bordures et overlays légers. Valeur adaptée au thème pour maintenir un contraste cohérent.

| Token | Light | Dark | Usage |
|---|---|---|---|
| `--tint` | `15%` | `20%` | Opacité du composant source dans un `color-mix` |

**Tokens dérivés** :

| Token | Valeur | Usage |
|---|---|---|
| `--color-outline-light` | `color-mix(in srgb, var(--color-outline) var(--tint), transparent)` | Séparateurs discrets dans les tableaux |
| `--color-primary-tint` | `color-mix(in oklch, var(--color-primary) 10%, transparent)` | Tokens dérivés résolus à leur élément déclarant (`:root`) — light-only ; pour un fond tonal theme-correct, composer le `color-mix` dans la classe consommatrice (cf. `.tone--primary`) |

```css
/* Exemple d'utilisation dans CDataTable */
border-color: var(--color-outline-light);
```

> Toujours utiliser `--color-outline-light` pour les séparateurs de tableau plutôt qu'une valeur `rgba` codée en dur.

---

### 2.8 Interlignage

| Token | Valeur | Usage |
|---|---|---|
| `--leading-tight`   | `1.25`  | Titres, KPI numériques (`CHeading`, `CKpiMetric`) |
| `--leading-snug`    | `1.375` | Labels, chips, eyebrows |
| `--leading-normal`  | `1.5`   | Corps de texte, paragraphes (`--text-md`) |
| `--leading-relaxed` | `1.625` | Blocs légaux, notices, texte long |

> Toujours spécifier `line-height` via ces tokens dans les composants — jamais de valeur `em` ou `px` codée en dur.

---

### 2.9 Index de calques (z-index)

| Token | Valeur | Usage |
|---|---|---|
| `--z-base`     | `0`   | Flux normal |
| `--z-raised`   | `10`  | Cartes flottantes, `position: sticky` |
| `--z-nav`      | `100` | Navigation : topbar, sidebar, menus déroulants |
| `--z-overlay`  | `200` | Overlays sous-modaux : autocomplete, popovers |
| `--z-modal`    | `300` | `CModal` et tous les `Dialog` Radix |
| `--z-toast`    | `400` | Toasts, notifications, tooltips |

> Ne jamais utiliser de valeurs z-index codées en dur (`z-10`, `z-50`…) — toujours `var(--z-*)`.

---

## 3. Catalogue de composants

### 3.1 Atomes (`components/atoms/`)

#### Affichage

| Composant | Props clés | Description |
|---|---|---|
| `CAlert` | `variant` (`error` \| `warning` \| `info` \| `success`), `title` | Bandeau d'alerte contextuelle — consomme `.tone--err/warn/info/valid` |
| `CAvatar` | `name` (`string \| null \| undefined`), `size?` (`sm` \| `md` \| `lg`, défaut `md`) | Avatar initiales — hue oklch dérivé du nom par hash djb2, adapté au thème via `[data-theme="dark"]`. `sm`=28px, `md`=36px, `lg`=42px. 1 mot→2 premiers chars, multi-mots→initiales premier+dernier. |
| `CBadge` | `variant`, `truncate` | Chip de statut ou de catégorie — consomme `.tone--*`. `truncate` contraint un libellé long dans la largeur du parent (ellipsis, badge texte sans `dot`) |
| `CDataText` | `size` (`sm` \| `md` \| `lg`), `as` (`span` \| `strong` \| `div`) | ✅ — Texte tabulaire JetBrains Mono — `font-mono font-medium lining-nums tabular-nums`. **La graisse et les chiffres tabulaires appartiennent au composant** : un parent ne les surcharge pas. Échelle de `size`, volontairement décalée d'un cran sous les noms Tailwind : `sm` = `text-xs` (11 px), `md` = `text-sm` (13 px, défaut), `lg` = `text-base` (15 px) ; `lg` n'est pas `text-lg`, qui reste réservé aux titres |
| `CEyebrow` | — | Libellé eyebrow — `font-body text-2xs font-medium uppercase tracking-[0.06em]`. Rendu en `<div>`. |
| `CHeading` | `type` (`header` \| `section` \| `card`), `variant` (`heading` \| `title`) | ✅ Titre Fraunces semibold. `header` = `text-2xl` h1, `section` = `text-xl` h2 (défaut), `card` = `text-lg` h3. Variante `title` = `font-body font-semibold`. |
| `CIconTile` | `size` (`sm` \| `md` \| `lg`) | Tuile icône avec fond tonal — consomme `.tone--*`. `sm` = 32px, `md` = 40px (défaut), `lg` = 56px. `border-radius` ajusté par taille. Slot défaut pour l'icône SVG — icône dimensionnée via `:deep(svg)`. |
| `CLegalDescription` | `title`, `summary` | Ligne de note légale — `⚖️ titre — résumé` en `.body-xs .text-on-surface-variant`. Utilisé comme item dans `CLegalNotice`. |
| `CMockBanner` | `variant` (`mock` \| `demo`) | Bandeau de mode mock/démo — `role="status" aria-live="polite"`. Réservé aux environnements de développement. |
| `CProgressBar` | `value`, `max` | Barre de progression |
| `CIconGrid` | — | ✅ Icône grille (4 carrés) — toggle vue en grille |
| `CIconList` | — | ✅ Icône liste (3 lignes) — toggle vue en liste |
| `CSeparator` | `orientation` | Séparateur — wrapper Radix `Separator` |
| `CTooltip` | `content`, `placement` | Info-bulle — wrapper Radix Tooltip |

#### Formulaires

| Composant | Props clés | Description |
|---|---|---|
| `CFormCheckbox` | `label`, `checked`, `required` | Case à cocher — wrapper Radix. Erreur `required` affichée après interaction/perte de focus |
| `CFormChipInput` | `label`, `value` | Saisie d'un chip unitaire (la boucle `v-for` est dans le parent) |
| `CFormChipSelect` | `label`, `options`, `required` | Sélection de chips. Erreur `required` affichée après interaction/perte de focus |
| `CFormDropzone` | `accept`, `multiple` | Zone de dépôt de fichiers |
| `CFormFeedback` | `status`, `message` | Retour de formulaire — utiliser via `CFormActions` ou `CForm`, jamais directement dans le markup. Animation push (grid-template-rows). |
| `CFormFieldProxyFeedback` | `name`, `required`, `proxyType`, `proxyValue?`, `proxyChecked?`, `displayError?`, `errorId?`, `hint?`, `hintId?`, `showProxy?` | Atome interne de mutualisation pour les `CForm*` : rend l'input proxy caché (optionnel) pour la validité native du `<form>` et le bloc de feedback (`error`/`hint`) |
| `CFormInput` | `type`, `label`, `error`, `hint`, `required` | Champ texte — variante `password` avec toggle de visibilité. Si `required`/contrainte native invalide, erreur affichée après premier `blur` (priorité à `error` externe) |
| `CFormSelect` | `label`, `options`, `error`, `required` | Sélecteur — wrapper Radix `SelectRoot`. Erreur `required` affichée après `blur` + proxy natif caché pour la validité du `<form>` |
| `CFormMultiSelect` | `options`, `placeholder` | Sélecteur multiple — wrapper Radix `PopoverRoot` + `ListboxRoot multiple`. Affiche les 2 premiers labels sélectionnés, puis « {count} sélectionnés » (`common.selectedCount`). Pas de proxy natif (pas de contrainte `required` native) |
| `CFormTextarea` | `label`, `rows`, `error`, `required` | Zone de texte. Si `required`/contrainte native invalide, erreur affichée après premier `blur` (priorité à `error` externe) |

#### Interactions

| Composant | Props clés | Description |
|---|---|---|
| `CBtn` | `variant` (`primary` \| `secondary` \| `ghost` \| `danger` \| `negative` \| `icon`), `size` (`sm` \| `md` \| `lg` \| `icon`), `type` (`button` \| `submit` \| `reset`), `loading`, `disabled`, `closing`, `disableWhenFormInvalid?`, `title?`, `icon?`, `shrink?` (`sm` \| `md` \| `lg` \| `xl` \| `2xl`) | Bouton atomique. Pendant `loading`, seul le spinner est affiché. Usage recommandé dans les vues : props `title` + `icon` (au lieu du slot), surtout pour les actions standardisées. Si `type="submit"`, le bouton est désactivé tant que le `<form>` est invalide (`disableWhenFormInvalid=true` par défaut). Si `shrink` est défini, le `title` est masqué sous le breakpoint choisi et le bouton devient carré (taille dérivée de `size`). |
| `CToggle` | `modelValue` (v-model `boolean`), `disabled?`, `ariaLabel?` | Interrupteur on/off — wrapper Radix `SwitchRoot`. 44×24px, thumb blanc, fond `surface-container-high` → `primary` quand actif. Focus visible. Toujours fournir `ariaLabel` ou un `<label>` associé. |
| `CDropdownMenu` | `items: DropdownItem[]`, `align?: 'start' \| 'end' \| 'center'`, slot: `trigger` | ✅ wrapper Radix `DropdownMenu` |
| `CTabs` | `tabs` (`{ value, label, count? }[]`), `modelValue` (v-model), `variant?` (`'tab'` \| `'filter'`, défaut `'tab'`) | ✅ Bandeau d'onglets ou de filtres. `tab` (défaut) : indicateur souligné primary sur l'actif. `filter` : segment pill avec fond `surface-container-low`, active = `surface-container-lowest` + ombre, badge numérique circulaire primary. Jamais d'icône seule. |

---

### 3.2 Molécules (`components/molecules/`)

| Composant | Props clés | Description |
|---|---|---|
| `CCard` | `level?` (`1`\|`2`\|`3`\|`4`, défaut `2`), `shadow?`, `gap?` (`none` \| `sm` \| `md`) | Conteneur générique — définit background, border-radius, padding et flux interne. `level` couple padding et radius : 1 = padding lg + radius 3xl (carte racine), 2 = padding md + radius 2xl (défaut), 3 = padding sm + radius xl (carte imbriquée), 4 = padding xs + radius lg (carte innermost). `gap` : `sm` = 12px, `md` = 16px (défaut), `none` = 0. Pour les listes avec séparateurs : `CFlexGroup`. Ossature d'une carte d'entité : `CCardHeader` › titre › description › `CMetaLine` › `CCardActions` › `CCardNote`. |
| `CCardActions` | — | ✅ Zone d'actions d'une carte — `flex flex-wrap gap-x-2 gap-y-1`. |
| `CCardHeader` | `withIcon?` | ✅ — En-tête de carte d'entité — slots `#icon`, `#chips` (chips de statut/type toujours visibles, wrap autorisé) |
| `CCardNote` | `note` | Encart d'information secondaire dans une `CCard` |
| `CCopyField` | `value`, `label?`, `hint?`, `copyTitle?`, `copiedTitle?` | ✅ Champ lecture seule + bouton copier (presse-papiers). Gère son propre état « copié ». Remplace les blocs `invite-link-box`. |
| `CDataTable` | `columns`, `rows` | Tableau de données |
| `CDocumentList` | `documents`, `readonly?`, `loadingDocId?`, `removeTitle?` | ✅ — Liste documents avec téléchargement/retrait — `emit('download', doc)` et `emit('remove', doc)` au parent, qui garde l'appel API et la modale de retrait |
| `CEmptyState` | `icon`, `title`, `description?`, `size?` (`page` \| `compact`) | ✅ État vide — slot `#actions`. `page` = centré + `py-8`, `compact` = plus petit pour les sections. |
| `CFileDropzone` | `files` (v-model:files `File[]`), `hint?`, `removeLabel?`, `multiple?`, `accept?` | ✅ Zone de dépôt + sélection de fichiers locaux avant envoi. Prévisualisation par puces (nom + retrait) par défaut ; slot `#preview="{ files, remove }"` pour un rendu custom (ex. vignettes images). À distinguer de `CFormDropzone` (upload direct serveur). |
| `CForm` | `status?`, `message?` | Formulaire structuré (`<form>`) — flex-col gap-4 + slot `#actions` → `CFormActions`. `@submit` émis sur soumission. Remplace `<form>` brut dans les vues. |
| `CFormActions` | `status`, `message?` | Rangée de boutons (droite) + `CFormFeedback` push en dessous. Utiliser dans le slot `#actions` de `CForm` ou dans `#footer` de `CModal`. |
| `CFormAddressAutocomplete` | `label`, `value` | Champ adresse avec autocomplétion |
| `CFormGroup` | `label`, `hint?` | ✅ Wrapper label + champ + hint (EP-020-S09) |
| `CFormRow` | `cols` (1/2) | ✅ Grille formulaire 2 colonnes responsive (EP-020-S09) |
| `CKpiCard` | `title`, `tone` | Conteneur indicateur KPI — compose `CKpiMetric` pour la valeur |
| `CKpiMetric` | `value`, `unit?`, `size?` | Valeur numérique KPI — `text-display font-mono lnum tnum`. Ne jamais utiliser `font-display` ni `text-[…px]` ici. |
| `CLegalCard` | — | Carte de note légale (wraps `CLegalNotice`) |
| `CLegalNotice` | `descriptions`, `links` | Bloc notices légales structurées |
| `CListItem` | — | Ligne de liste générique |
| `CMetaItem` | `variant`, `icon` | Ligne icône + contenu — variantes `footer` / `sub` |
| `CMetaLine` | — | Rangée horizontale de `CMetaItem` avec `CSeparator` |
| `CModal` | `open`, `title`, `saving?`, `feedback?` | Dialogue Radix `Dialog` — émettre `@submit` pour soumission. `CFormFeedback` obligatoire dans `#footer` (DDR-006). |
| `CPageHeader` | `eyebrow?`, `title?`, `meta?` | ✅ — En-tête de vue — slots `#eyebrow` (eyebrow avec badges), `#title` (titre composé), `#actions` (boutons à droite), défaut (description, méta libre) |
| `CPageLoader` | — | Indicateur de chargement |
| `CSectionHeader` | — | ✅ En-tête de section — `flex items-start justify-between gap-3`. Slot default (zone titre : heading + meta). Slot `#actions` pour les boutons droite. Remplace `.section-head` / `.card-head`. |
| `CSectionLink` | `to` | ✅ Lien « voir tout » d'en-tête de section — `RouterLink` + chevron. À utiliser dans le slot `#actions` de `CSectionHeader`. |
| `CTimelineSection` | `events`, `loading`, `error`, `linkTasks?`, `currentTaskId?` | ✅ Section de frise complète — états chargement / erreur (bouton réessayer, `emit('retry')`) / vide + `CTimelineList`. Évite de dupliquer la machine à états dans chaque vue détail. |
| `CTimelineList` | `events` (`TimelineEvent[]`), `linkTasks?` (défaut `true`), `currentTaskId?`, `currentReportId?` | ✅ Frise d'événements de cycle de vie (EP-017) — pastille par action (`created`, `status_changed`, `closed`, …), ordre antéchronologique (plus récent en premier), carte teintée `.tone--primary` (fond `--color-primary-tint`, AA light/dark) + bordure `primary/30` et anneau `ring-primary/20` sur la pastille pour l'étape `converted_to_task`, libellé i18n (`timeline.actions.*`), acteur (`actor_name`, « Prestataire externe » si vendeur), détail issu du `payload` (transition de statut, nom de fichier, expiration du lien), badge « Signalement » pour les événements `entity_type === 'report'`, lien vers la tâche créée sur `converted_to_task`. Action inconnue → pastille neutre + libellé générique. `linkTasks: false` masque le lien de conversion pour les rôles sans accès aux tâches ; `currentTaskId` fait pointer l'étape de conversion vers le **signalement source** (et inversement depuis `currentReportId` vers la tâche) ; le doublon `task.created` d'une conversion n'est pas rendu. |
| `CTopbarBreadcrumb` | `crumbs[]` (`{ label, to? }`) | ✅ — Fil d'Ariane topbar — `nav` + `aria-current` sur le dernier crumb, troncature : les crumbs intermédiaires cèdent la place en premier |

---

### 3.3 Organismes (`components/organisms/`)

| Composant | Description |
|---|---|
| `AppNav` | Navigation principale (sidebar desktop + bottom nav mobile) |
| `AuthBrandPanel` | Panneau éditorial des écrans d'authentification |
| `CConfirmDialog` | Dialogue de confirmation destructive |
| `CDeleteConfirmModal` | Modale de suppression générique (DDR-006) — props `title`, `message`, `successMessage`, `onDelete` ; émet `deleted`. Remplace les modales de suppression clonées par entité. |
| `CDocumentLinkActionModal` | Modale gestion des documents liés (retrait / suppression) |
| `CFlexGroup` | Conteneur flex groupé |
| `CLinkedDocumentsModal` | Modale liste des documents liés à une entité |
| `CPageError` | Affichage d'erreur pleine page |
| `CReminderCard` | Carte de rappels synthétiques partagée entre vues |
| `CStatusBanner` | Bandeau d'état en-tête (AG : `draft` / `called` / `ongoing` / `closed`) — ✅ |
| `CTopbar` | Barre supérieure avec breadcrumb et actions globales |

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
- Changement de thème dans `Profil > Préférences > Apparence` uniquement (`Clair` / `Sombre` / `Auto`).
- Délai de fermeture automatique des modales après succès, réglable dans le même onglet (`0` = immédiat à `10` s, défaut `3` s — DDR-006) ; l'animation de fermeture du bouton dure exactement le délai choisi.
- `CTopbarBreadcrumb` : fluide (`min-width: 0`), libellés tronquent proprement.

#### Écrans d'authentification

Sur mobile, les écrans publics (`/login`, `/onboarding`, invitation, reset) conservent un en-tête compact de marque sans sortir le formulaire du viewport initial. Le formulaire reste la zone scrollable principale.

#### États de chargement & d'erreur

- Toute vue qui charge ses données au montage passe par `usePageLoader`.
- État d'erreur → `CPageError`. État de chargement → `CPageLoader`.
- Les chargements secondaires non bloquants restent isolés.
- Le backend renvoie des codes d'erreurs au formt i18n. Toutes les entrées doivent être sous `errors`.

#### Documents PDF officiels

- **Aucune génération PDF côté navigateur** (`@media print` interdit) : tout document PDF est généré côté serveur (`genpdf`) et téléchargé en blob.
- L'identité visuelle des documents PDF est centralisée côté backend dans `domain/pdf_template.rs` : **introduction** (1ʳᵉ page — logo image embarquée `assets/images/logo-la-ruche.png` (rasterisé depuis `CLogoMark.svg`) + « La Ruche » + tagline, titre centré, identité copro, « Fait à …, le … » + « Pour le compte de … », paragraphe explicatif), **pas d'en-tête récurrente**, **conclusion** (période de validité légale optionnelle, « Fait à …, le … » rempli, signé « La Ruche »), **pied de page récurrent** (logo miniature + tagline + « Édité le … via La Ruche », petite police grise), police Tinos serif (métrique Times New Roman, sous-ensemble `pyftsubset` Latin-1 + ponctuation, ~32 ko/fichier × 4 styles), logo PNG RGB compressé en FlateDecode en release, palette marque #643fd5. Les renderers documentaires (`domain/logbook_pdf.rs`, …) s'y appuient ; ne pas dupliquer ces styles dans un renderer.
- Le frontend se contente d'appeler l'endpoint PDF et de télécharger le blob (`maintenanceApi.exportPdf`, `etatDateApi.exportPdf`…), avec état de chargement sur le bouton et `CAlert` en cas d'erreur.

#### Cartes de liste

`CCard` est un conteneur générique (background, border-radius, padding, flux). Chaque entité de liste est une `CCard`. La mise en liste est de la responsabilité de la vue :

| Besoin | Solution |
|---|---|
| Liste avec séparateurs visuels entre items | `CFlexGroup` comme wrapper |
| Liste sans séparateurs, espacement seul | `div` Tailwind inline (`flex flex-col gap-*`) |
| Item unique dans une zone de contenu | `CCard` seul sans wrapper de liste |
| Grille responsive d'entités égales | `grid grid-cols-*` inline — pas de `CFlexGroup` |

**Ossature commune d'une carte d'entité :**

```
CCard (conteneur)
├── CCardHeader    — chips de statut / type toujours visibles
├── Titre principal
├── Description courte (optionnel)
├── CMetaLine / CMetaItem  — métadonnées compactes
├── Ligne d'actions    — « Modifier » + action contextuelle accentuée
└── CCardNote          — bloc informatif secondaire (délai, remboursement, lien…)
```

**Règles :** titres en `font-body text-lg`, jamais `font-display` · métadonnées via `CMetaItem` · encarts via `CCardNote` · pas de séparateur entre cartes.

Chaque vue définit ses propres variantes dans ses **règles feature** (§4.2).

#### Actions de ligne (tableaux & cartes)

Les actions répétées sur une ligne de `CDataTable` utilisent **toujours** `CBtn variant="icon"` — bouton carré, icône seule, jamais de libellé textuel.

| Règle | Détail |
|---|---|
| Variante | `variant="icon"` (jamais `ghost`/`secondary` avec `:title` visible) |
| Libellé | `:title` (tooltip natif) **et** `:aria-label` obligatoires — jamais de texte rendu |
| Action destructive | ajouter `class="tone--err"` (Supprimer, Archiver, Révoquer) |
| Conteneur | `flex gap-1 justify-end` (tableau) ou `CCardActions` (carte) |

#### KPI

Les indicateurs et filtres KPI utilisent `CKpiCard` (molécule) + `CKpiMetric` (atome) et `CBtn` avec `.tone--*` pour les pills de filtre. La valeur numérique passe par `CKpiMetric` qui utilise `text-display` (`--text-display`, 40px) + `font-mono lnum tnum`. Ne jamais utiliser la valeur arbitraire `text-[56px]` ni `font-display` (Fraunces) pour des chiffres — non tabulaire, illisible en colonnes.

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

#### Pattern formulaire (DDR-006)

Deux contextes, deux molécules. `CFormFeedback` n'est jamais utilisé directement dans les templates.

**Formulaire modal** — `CFormActions` dans le slot `#footer` de `CModal` :
- Tous les boutons sont enfants de `CFormActions` (slot défaut).
- `CFormFeedback` est rendu automatiquement sous les boutons avec animation push (grid).
- Bouton destructif à gauche : `class="mr-auto"` sur le premier `CBtn`.

**Formulaire inline** — `CForm` remplace tout `<form>` brut dans les vues :
- Champs dans le slot défaut, boutons dans le slot `#actions` → rendu via `CFormActions`.
- Props `status` et `message` transmis depuis `useFormFeedback`.

**Règles communes :**
- `useFormFeedback()` gère `status` / `message`. Appeler `feedback.reset()` à l'ouverture du modal.
- Après succès, la fermeture automatique de la modale suit la **préférence utilisateur de délai** (0–10 s, défaut 3 s — DDR-006) ; à `0`, fermeture immédiate sans fenêtre de succès. L'état `closing` du bouton et son animation durent exactement le délai choisi.
- La logique `required + touched + blur + erreur interne` est mutualisée via le composable `useFieldValidation` (`frontend/src/composables/useFieldValidation.ts`) — ne pas la réimplémenter dans chaque `CForm*`.
- Jamais de `<form class="flex flex-col gap-4">` dans une vue — utiliser `CForm` ou le slot body de `CModal`.
- `<form>` nu autorisé uniquement dans le body d'un `CModal` quand les actions sont dans `#footer`.
- Champ requis (`required`) : après premier focus puis `blur`, si la valeur reste invalide (vide ou contrainte native non respectée), l'erreur est affichée immédiatement sous le champ.
- Action de soumission (`CBtn type="submit"`) : désactivée tant que les champs requis du formulaire ne sont pas valides.

#### Tabs de navigation

Toute vue avec plusieurs filtres d'état utilise `CTabs` (atome §3.1). Le composant reçoit la liste des onglets et expose un `v-model` sur la valeur active — la logique de filtrage reste dans la vue.

```vue
<CTabs
  v-model="filter"
  :tabs="[
    { value: 'all',    label: t('common.all'),    count: total },
    { value: 'active', label: t('common.active'),  count: activeCount },
  ]"
/>
```

**Règles :**
- Flex row, sous l'en-tête de page, au-dessus de la liste.
- Libellé `font-body text-sm font-medium`, neutre en `color: var(--color-on-surface-variant)`.
- Tab actif : `color: var(--color-primary)`, indicateur `border-bottom: 2px solid var(--color-primary)`.
- `count` : badge numérique inline — jamais `CBadge`, toujours un `<span>` stylisé dans le composant.
- Jamais de tab avec icône seule — libellé textuel obligatoire.
- Pas de `<style>` dans la vue qui utilise `CTabs` — toute variation passe par les props.

---

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
- Barre de filtres en trois rangées : onglets période (`CTabs variant="filter"`), recherche plein champ, selects catégorie/source sur une rangée dédiée (même style que la vue Contrats).
- Navigation `précédent / suivant` déplace d'une période complète.
- Bloc réglementaire en tête : adresse immeuble + syndic en exercice.

#### Vue Profil

- Onglets en tête de page (`CTabs variant="tab"`) : `Mon compte` · `Préférences`.
- L'onglet `Préférences` regroupe deux sous-onglets de filtres : `Notifications` · `Apparence` (thème + délai de fermeture automatique des modales). Le bouton global « Réinitialiser mes préférences » (`CBtn` ghost, icône `CIconRefresh`) est aligné à droite sur la même ligne que ces sous-onglets.

#### Vue Calendrier

- Onglets de filtres en tête de page : `Mois` · `Semaine` · `Agenda`. Le bouton « Exporter (.ics) » (`CBtn` ghost, icône `CIconDownload`) est aligné à droite sur la même ligne que ces onglets.
- **Barre de filtres** (`CalendarFilterBar`, feature-local) : rangée de chips toggle par type d'événement — `CBadge` avec la variante de couleur de la source (cohérence AC-CAL-001-B) ; chip actif pleine opacité, inactif `opacity-40`. Seuls les types visibles pour le rôle courant sont affichés.
- **Persistance** : état des filtres en `localStorage` (clé par utilisateur), rechargé au montage — pas de préférence serveur.
- **Mode liste** : deux champs `CFormInput type="date"` (`Du` / `Au`) remplacent la navigation période quand aucune plage libre n'est saisie.
- **Tous filtres désactivés** : `CEmptyState compact` explicite « aucun type sélectionné » — aucun appel réseau.

---

## Appendice — DDR

| Réf | Titre | Résumé |
|---|---|---|
| DDR-001 v3 | [Typographie](ddr/DDR-001-typographie.md) | Fraunces (display) + DM Sans (corps) + JetBrains Mono (data tabulaire) |
| DDR-002 | [Couleurs](ddr/DDR-002-couleurs.md) | Palette lavande/charcoal, light + dark, tokens CSS vars uniquement |
| DDR-003 | [Navigation](ddr/DDR-003-navigation.md) | Bottom nav mobile, sidebar desktop, filtrage par rôle |
| DDR-004 | [Navigation sidebar](ddr/DDR-004-navigation-sidebar.md) | Groupes sidebar (pattern B — labels fixes), dropdown profil, routes `/residence/*` et `/profile/*` |
| DDR-005 | [États de chargement](ddr/DDR-005-page-loading-states.md) | `usePageLoader` + `CPageError` + `CPageLoader` — chargements non bloquants isolés |
| DDR-006 | [Pattern feedback formulaire](ddr/DDR-006-form-feedback-pattern.md) | `CFormActions` + `CForm` — animation push, jamais `CFormFeedback` directement |

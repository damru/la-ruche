# DESIGN.md — La Ruche Design System "The Modern Registry"
> Dernière mise à jour : 2026 (post-étapes C–F)

## Typographie — 3 polices, 3 rôles (DDR-001 v3)

| Rôle | Police | Usage |
|---|---|---|
| Display / Titres | **Fraunces** (variable opsz/wght/ital) | Titres, KPI ≥ 28px |
| Corps / UI | **DM Sans** | Texte, labels, navigation |
| Données tabulaires | **JetBrains Mono** | Montants tableaux, codes, numéros lot |

Fraunces a des chiffres proportionnels non tabulaires → colonnes non alignées en tableau. JetBrains Mono résout ça avec `lnum tnum`.

```
display-lg   Fraunces 56px / 800 / opsz:144 / ls:-0.04em
headline-md  Fraunces 24px / 600 / opsz:48  / italic
headline-sm  Fraunces 20px / 600 / opsz:36
body-md      DM Sans  14px / 400
label-sm     DM Sans  11px / 500 / uppercase / ls:0.06em
data-md      JetBrains Mono 13px / 500
```

### Tokens `--fw-*` — Policy graisse

| Token | Valeur | Usage |
|---|---|---|
| `--fw-regular` | `400` | Corps de texte, descriptions secondaires |
| `--fw-medium` | `500` | Labels formulaire, boutons, nav, contrôles UI |
| `--fw-semibold` | `600` | En-têtes de sections, titres de composants, alertes |
| `--fw-bold` | `700` | Titres de page, valeurs KPI, display numérique |
| `--fw-display` | `800` | `display-lg` uniquement (Fraunces) |

Tout `font-weight` dans les composants doit référencer un token `--fw-*`, jamais une valeur numérique brute.

```html
<link href="https://fonts.googleapis.com/css2?family=Fraunces:ital,opsz,wght@0,9..144,100..900;1,9..144,100..900&family=DM+Sans:ital,opsz,wght@0,9..40,300..500;1,9..40,300..400&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
```

## Palette — Tokens CSS light/dark

| Token | Light | Dark |
|---|---|---|
| `--color-primary` | `#643fd5` | `#c4b5fd` |
| `--color-secondary-container` | `#dde1f9` | `#404659` |
| `--color-on-secondary-container` | `#1a1d2e` | `#dde1f9` |
| `--color-surface` | `#f8f9fa` | `#111318` |
| `--color-surface-container-lowest` | `#ffffff` | `#0c0e13` |
| `--color-surface-container-low` | `#f3f4f5` | `#191c21` |
| `--color-surface-container-high` | `#e7e9ea` | `#28292f` |
| `--color-on-surface` | `#191c1d` | `#e2e3e8` |
| `--color-on-surface-variant` | `#42474e` | `#c3c6cf` |
| `--color-error` / `--color-error-container` | `#ba1a1a` / `#ffdad6` | `#ffb4ab` / `#93000a` |
| `--color-success` / `--color-success-container` | `#2e7d32` / `#e8f5e9` | `#81c784` / `#1b4f23` |
| `--color-warning` / `--color-warning-container` | `#e65100` / `#fff8e1` | `#ffb74d` / `#4d3500` |
| `--gradient-primary` | `linear-gradient(135deg, #643fd5, #7d5bf0)` | `…#7d5bf0 → #c4b5fd` |

**Hiérarchie surfaces :** Page → `surface` · Section → `surface-container-low` · Carte → `surface-container-lowest` · Champ → `surface-container-high`

**Règle No-Line :** Jamais de `1px solid` pour séparer du contenu. Décalage tonal, espacement, ou micro-ombre.

## Élévation

```css
--shadow-md:    0 8px 32px rgba(25,28,29,.07), 0 2px 8px rgba(25,28,29,.04);
--shadow-lg:    0 16px 48px rgba(25,28,29,.10), 0 4px 16px rgba(25,28,29,.06);
--shadow-glass: 0 20px 60px rgba(25,28,29,.14), 0 4px 16px rgba(25,28,29,.08);
/* Glassmorphism modales : rgba(255,255,255,.85) + backdrop-filter:blur(20px) */
```

## Border Radius
`--radius-sm:8px` · `--radius-md:12px` · `--radius-lg:16px` · `--radius-full:9999px`
Jamais 0px sur un élément interactif.

## Composants flottants (tooltips, dropdowns, popovers)

**Règle :** tout élément flottant doit utiliser `Teleport to="body"` + `position: fixed` + calcul de position par `getBoundingClientRect`. Jamais `position: absolute` dans un parent scrollable ou `overflow: hidden`.

Comportement attendu : flip vertical (au-dessus → en-dessous si espace insuffisant), clampage horizontal au viewport avec marge de 10px. L'atome de référence est `CTooltip.vue`.

## Navigation (DDR-003)

| Breakpoint | Pattern |
|---|---|
| ≤ 1024px | Bottom nav — 4 items filtrés par rôle |
| > 1024px | Sidebar fixe 240px |

Mobile — 4 items par rôle : Syndic (Dashboard·AG·Budget·Tâches) · Conseil (Dashboard·AG·Budget·Documents) · Copropriétaire (Dashboard·AG·Budget·Consultations) · Locataire (Dashboard·Consultations·Signalements·—)

## Accessibilité
- Contraste corps ≥ 4.5:1 (WCAG AA), critique ≥ 7:1
- Focus : `outline: 2px solid var(--color-primary); outline-offset: 2px`
- Touch targets : min 44×44px (imposé dans `base.css` sur tous les `button`). Exception : les boutons-icônes *embarqués* dans une zone interactive plus large (ex: `.c-tooltip-trigger`) doivent explicitement déclarer `min-width: unset; min-height: unset` pour ne pas gonfler leur conteneur.
- Jamais couleur seule pour transmettre une information

## Internationalisation
`Intl.NumberFormat('fr-FR', { style:'currency', currency:'EUR' })` · `Intl.DateTimeFormat('fr-FR')`

## Système de tonalités couleur — `tones.css`

Défini dans `frontend/src/styles/tones.css`, importé globalement avant tout composant.

Les classes `.tone--*` sont des **utilitaires purs** : elles définissent uniquement deux variables CSS — sans aucune règle de forme, taille ou layout. Elles s’appliquent à n’importe quel élément (icône, bouton, chip, badge, pill…).

| Classe | `--tone-bg` | `--tone-fg` |
|---|---|---|
| `.tone--default`  | `--color-secondary-container`       | `--color-primary`            |
| `.tone--accent`   | `--gradient-primary`                | `--color-on-primary`         |
| `.tone--warn`     | `--color-warning-container`         | `--color-warning`            |
| `.tone--err`      | `--color-error-container`           | `--color-error`              |
| `.tone--disabled` | `--color-surface-container-highest` | `--color-on-surface-variant` |
| `.tone--valid`    | `--color-success-container`         | `--color-success`            |

Les éléments consommateurs déclarent :
```css
background: var(--tone-bg, <fallback>);
color:      var(--tone-fg, <fallback>);
```

```html
<!-- icône KPI accentée -->
<div class="kpi-icon tone--accent">…</div>

<!-- filtre actif en mode alerte -->
<button class="kpi-filter kpi-filter--active tone--warn">…</button>

<!-- bouton destructif -->
<button class="btn tone--err">Supprimer</button>
```

## Règle UI — Primitives KPI (`kpi.css`)

Mutualisés dans `frontend/src/styles/kpi.css`. Consomment `--tone-bg` / `--tone-fg` fournis par `.tone--*`.

| Classe | Rôle |
|---|---|
| `.kpi-icon`           | Badge icône carré 36px |
| `.kpi-filter-row`     | Ligne de pills KPI |
| `.kpi-filter`         | Pill KPI cliquable (inactif) |
| `.kpi-filter--active` | Pill KPI actif — consomme la tonalité |
| `.kpi-filter__count`  | Valeur numérique (Fraunces 24px) |
| `.kpi-filter__label`  | Libellé secondaire (11px) |
## DDR
| Ref | Décision |
|---|---|
| DDR-001 v3 | Fraunces (display) + DM Sans (corps) + JetBrains Mono (data tabulaire) |
| DDR-002 | Lavande/charcoal, light+dark, tokens CSS vars uniquement |
| DDR-003 | Bottom nav mobile, sidebar desktop, filtrage par rôle |

# DESIGN.md — l'Application Design System "The Modern Registry"

> **Dénomination** — Dans le présent document, l'expression **« l'Application »** désigne exclusivement **La Ruche**, plateforme web open source de gestion participative de copropriété. Ces deux termes sont interchangeables et désignent la même entité logicielle.
> Dernière mise à jour : 2026-04-25

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

**Règle No-Line :** Jamais de `1px solid` pour séparer du contenu hors structures tabulaires. Décalage tonal, espacement, ou micro-ombre.

**Exception tableaux :** les tableaux de données peuvent utiliser des séparateurs discrets si cela améliore la lisibilité des lignes, des colonnes ou de l'en-tête. Dans ce cas, privilégier des bordures à faible contraste, cohérentes avec les tokens de surface, et éviter l'effet grille lourde.

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

## Navigation (DDR-003 + DDR-004)

| Breakpoint | Pattern |
|---|---|
| ≤ 1024px | Bottom nav — 4 items primaires filtrés par rôle + tiroir « Plus » + item Profil |
| > 1024px | Sidebar fixe 240px |

Mobile — 4 items par rôle : Syndic (Dashboard·AG·Budget·Tâches) · Conseil (Dashboard·AG·Budget·Documents) · Copropriétaire (Dashboard·AG·Budget·Consultations) · Locataire (Dashboard·Consultations·Signalements·—)

La bottom nav mobile expose donc 6 actions visibles au total : 4 items primaires, 1 tiroir « Plus » pour les destinations secondaires, et 1 item Profil dédié. Le tiroir mobile est traité comme un panneau de navigation, pas comme une boîte de dialogue modale.

**DDR-004 — Groupes sidebar (pattern B — labels fixes)** : la sidebar desktop est structurée en 5 sections non-cliquables : *(solo)* Tableau de bord · **Gouvernance** (AG, Consultations, Signalements) · **Finance** (Budget, Contrats) · **Opérations** (Tâches, Sinistres, Carnet, Documents) · **Résidence** (Bâtiment, Clés de répartition, Registre, Copropriétaires). Le bloc utilisateur en pied de sidebar est cliquable et ouvre un panel contextuel (Mon compte, Notifications, Déconnexion).

Préférence d'apparence : le changement de thème n'est pas exposé dans la topbar. Il est configuré dans `Profil > Mon compte` via trois états exclusifs : `Clair`, `Sombre`, `Auto`, présentés sous forme de contrôle compact à pastilles avec icône SVG. Un court texte d'aide, contextuel à l'option sélectionnée, peut préciser l'effet du choix sans exposer de détails techniques de stockage.

Breadcrumb de topbar : le bloc gauche doit rester fluide (`min-width: 0`) et les libellés du breadcrumb doivent tronquer proprement si l'espace disponible diminue, sans jamais forcer une largeur fixe sur le conteneur.

Contrôles segmentés : tout conteneur doit rester utilisable sur écran étroit. Si les options dépassent la largeur disponible, le conteneur devient scrollable horizontalement sans casser la hauteur des boutons.

## Écrans publics d'authentification

Sur mobile, les écrans publics (`/login`, `/onboarding`, invitation, reset) doivent conserver un bloc d'information de marque visible **sans** faire sortir le formulaire principal du viewport initial. Le panneau éditorial desktop devient donc un en-tête compact : logo + nom produit + message éditorial court, avec contenu secondaire tronqué ou masqué si nécessaire.

Le formulaire ou wizard reste la zone scrollable principale. Les actions critiques et le début du formulaire doivent rester atteignables sans scroll initial ; l'éditorial ne doit jamais monopoliser la hauteur utile sur écran étroit.

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
## États de chargement et d'erreur de page (DDR-005)

Toute vue qui charge des données au montage doit utiliser le composable `usePageLoader` et le composant `CPageError`.

```vue
const { loading, error, reload } = usePageLoader(loadData)
onMounted(reload)
```

Template conditionnel obligatoire :
1. `v-if="error"` → `<CPageError :message="error" @retry="reload" />`
2. `v-else-if="loading"` → spinner centré via `.c-btn__spinner`
3. `v-else` → contenu normal

Les appels non-critiques (données secondaires, endpoints optionnels) sont isolés dans un `try/catch` séparé après le `Promise.all` principal pour ne pas bloquer le chargement des données primaires.

Références d'implémentation : `frontend/src/composables/usePageLoader.ts`, `frontend/src/components/molecules/CPageError.vue`, `frontend/src/components/molecules/CPageLoader.vue`.

## Règle UI — Vue Contrats

La vue `Contrats` affiche en tête les rappels à traiter, avec une action d’acquittement visible sans navigation secondaire.

Les pièces justificatives d’un contrat restent regroupées dans une modale dédiée au contrat courant, afin de garder la liste principale lisible tout en laissant l’upload documentaire dans le même parcours.

## Règle UI — Vue Carnet d'entretien

La vue `Carnet d'entretien` expose des filtres rapides ancrés sur des périodes calendaires réelles : **mois**, **trimestre** (défaut), **semestre**, **année en cours** et **tout l'historique**.

- La navigation `précédent / suivant` déplace la fenêtre d'une période complète, jamais une durée flottante depuis aujourd'hui.
- Le total des dépenses et la liste affichée sont toujours calculés sur la période active.
- Les champs `Coût` et `Prestataire` de la modale de création restent sur une grille 2 colonnes sur desktop mais repassent en pile sur écran étroit pour éviter tout débordement.

## Règle UI — Vue Signalements

La modale de conversion d'un signalement en tâche doit permettre au gestionnaire de choisir la **priorité** de la tâche au moment de la conversion, sans écran intermédiaire supplémentaire.

- La priorité proposée par défaut est `Normal`.
- La conversion reste une action unique : création de la tâche puis redirection immédiate vers la tâche créée.

## Règle UI — CFormFeedback dans les modales (DDR-006)

**Règle obligatoire :** tout `CModal` qui contient un formulaire ou une action destructive **doit** inclure un `CFormFeedback` dans le slot `#footer`, sur la même ligne que les boutons d'action. Le `CFormFeedback` ne doit jamais être placé dans le corps de la modale (`<template #default>`).

```vue
<CModal :open="showModal" title="…" @close="closeModal">
  <form @submit.prevent="save">
    <!-- champs du formulaire -->
  </form>
  <template #footer>
    <!-- ✅ CFormFeedback TOUJOURS dans #footer, avant les boutons -->
    <CFormFeedback :status="feedback.status.value" :message="feedback.message.value" />
    <CBtn variant="ghost" @click="closeModal">Annuler</CBtn>
    <CBtn variant="primary" :loading="saving" @click="save">Enregistrer</CBtn>
  </template>
</CModal>
```

Références d'implémentation : `frontend/src/components/atoms/CFormFeedback.vue`, `frontend/src/composables/useFormFeedback.ts`. Exemples : `ContractFormModal.vue`, `AllocationKeysView.vue`, `ContractorsView.vue`.

## DDR
| Ref | Décision |
|---|---|
| DDR-001 v3 | Fraunces (display) + DM Sans (corps) + JetBrains Mono (data tabulaire) |
| DDR-002 | Lavande/charcoal, light+dark, tokens CSS vars uniquement |
| DDR-003 | Bottom nav mobile, sidebar desktop, filtrage par rôle |
| DDR-004 | Groupes sidebar (pattern B labels fixes), dropdown profil utilisateur, routes `/residence/*` et `/profile/*` |
| DDR-005 | États de chargement et d'erreur de page — `usePageLoader` + `CPageError` |
| DDR-006 | `CFormFeedback` dans le slot `#footer` de toute `CModal` avec formulaire ou action destructive |

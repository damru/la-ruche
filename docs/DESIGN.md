# DESIGN.md — l'Application Design System "The Modern Registry"

> **Dénomination** — Dans le présent document, l'expression **« l'Application »** désigne exclusivement **La Ruche**, plateforme web open source de gestion participative de copropriété. Ces deux termes sont interchangeables et désignent la même entité logicielle.
> Dernière mise à jour : 2026-05-01

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

Les icônes de navigation doivent rester immédiatement lisibles aux tailles 18px (sidebar) et 22px (bottom nav). Pour l’entrée **Prestataires**, l’icône de référence est le casque de chantier `CIconHardHat`, plus explicite qu’une silhouette avec badge outil à petite taille. Pour l’entrée **Contrats**, l’icône de référence est le document signé à plat `CIconContract`, sans coin plié, afin de conserver une lecture nette en petite taille.

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

- L’en-tête des cartes Contrat reprend le même pattern que les cartes Opérations : icône métier, titre principal et chip de statut sur une même ligne, puis chips secondaires sur une ligne dédiée.
- Les cartes `Prestataires` reprennent la même grammaire d’en-tête, sans chip de statut quand aucune information de cycle de vie n’existe.
- Les tabs de filtre `Contrats` affichent le volume d’éléments par état (`Tous`, `Actifs`, `Expirés`) directement dans chaque segment.
- Les pièces justificatives d’un contrat se gèrent directement depuis la modale de création / modification, sans bouton documentaire dédié sur la carte.
- Chaque carte Contrat rappelle aussi le nombre de documents actuellement liés pour éviter d’ouvrir la modale à vide.
- La modale d’édition liste les pièces déjà liées avec deux issues explicites : retirer le lien seulement, ou retirer le lien puis supprimer définitivement le document.
- La suppression définitive d’un contrat se déclenche depuis la modale d’édition, jamais depuis la carte.

## Règle UI — Vue Carnet d'entretien

La vue `Carnet d'entretien` expose des filtres rapides ancrés sur des périodes calendaires réelles : **mois**, **trimestre** (défaut), **semestre**, **année en cours** et **tout l'historique**.

- La navigation `précédent / suivant` déplace la fenêtre d'une période complète, jamais une durée flottante depuis aujourd'hui.
- Le total des dépenses et la liste affichée sont toujours calculés sur la période active.
- Les champs `Coût` et `Prestataire` de la modale de création restent sur une grille 2 colonnes sur desktop mais repassent en pile sur écran étroit pour éviter tout débordement.
- Un bloc réglementaire précède la timeline et rappelle l'adresse de l'immeuble ainsi que le syndic en exercice.
- Les contrats d'assurance et d'entretien en cours sont affichés dans des sections dédiées, séparées de la timeline, avec leur échéance et un état lisible sans navigation secondaire.

## Règle UI — Vue Sinistres

La vue `Sinistres` affiche pour chaque dossier le délai légal restant avant déclaration à l'assurance, calculé selon le type de sinistre.

- Les alertes de délai doivent être visibles directement sur la carte (approche d'échéance / retard / transmis).
- L'explication juridique des délais est mutualisée dans une carte `CLegalInfoCard` en bas de page.
- La création d'un sinistre reste faisable dans une seule modale et accepte des pièces jointes sans écran intermédiaire.
- Les zones d'ajout de pièces jointes des modales Sinistres reprennent la dropzone standard utilisée sur les Signalements (`bordure pointillée`, état hover/drag, sélection directe de fichiers).
- Chaque carte Sinistre expose toujours deux actions côte à côte pour les profils gestionnaires : `Modifier` puis une action contextuelle dépendante du statut courant.
- Les actions contextuelles suivent strictement ce mapping : `Ouvert → Transmettre`, `En cours → Clore`, `Clos → Supprimer`.
- `Transmettre` ouvre une modale dédiée avec la date de transmission à l'assurance ; `Clore` ouvre une modale dédiée avec remboursement optionnel (date + montant) et ajout de documents ; `Modifier` permet aussi de supprimer le dossier sans revenir à la carte et de gérer les documents déjà joints.
- Les tabs de filtre `Sinistres` affichent aussi le volume par état (`Tous`, `Ouverts`, `En cours`, `Clos`) pour rendre le backlog lisible sans ouvrir chaque segment.

## Règle UI — Cartes Opérations (`Signalements` / `Tâches` / `Sinistres`)

Les cartes de liste des vues `Signalements`, `Tâches` et `Sinistres` partagent une même ossature visuelle afin d'améliorer le balayage :

- ligne 1 : en-tête compact propre à l’entité, avec au minimum le statut visible sans ouvrir la carte ;
- ligne 2 : titre principal ou, pour certaines cartes, reprise du titre juste sous l’en-tête si cela améliore la lisibilité ;
- ligne 3 : description courte éventuelle, en taille de texte standard ;
- ligne 4 : hints documentaires / photo sur leur propre ligne ;
- ligne 5 : métadonnées compactes avec icônes, sans séparateur ;
- séparateur discret ;
- ligne d'actions avec `Modifier` puis une action contextuelle accentée ;
- bloc informatif secondaire pour les éléments métier complémentaires (délai légal, remboursement, lien prestataire, historique, etc.).

Contraintes complémentaires :

- les listes sont triées par `dernière modification` décroissante ;
- le titre des `activity-card` reste en police de texte classique (`font-body`), pas en display ;
- la description utilise une taille de texte standard, en ton principal mais sans graisse excessive ; les metadata / secondary information restent volontairement plus légères via une teinte plus douce (implémentée avec `color-mix(... 88%, white)`) ;
- les lignes compactes `activity-card` n'utilisent plus de séparateur ; le groupement repose sur l'alignement, la proximité et des espacements icône/texte très serrés ;
- la carte `Tâche` compacte son header en `titre + priorité + statut`, sans icône, puis regroupe l’assignation et la date de création relative dans les métadonnées ;
- la carte `Signalement` affiche `icône + (statut + mise à jour relative simple)` sur sa première ligne, avec une icône de mise à jour dédiée ; sa modale de miniatures reste compacte avec des vignettes plafonnées, puis ouvre un viewer photo agrandi dont les boutons de navigation sont alignés sur une même ligne sous l’image ;
- la carte `Sinistre` conserve son icône métier mais sa couleur doit refléter explicitement le type de sinistre ;
- la carte `Sinistre` n'expose jamais `Supprimer` dans la liste ; cette action reste confinée à la modale d'édition ;
- l'ossature reste commune mais chaque entité peut substituer un équivalent métier quand une donnée n'existe pas (ex. `prestataire assigné` au lieu d'une localisation stricte sur les tâches).

## Règle UI — Obligations légales

Les cartes de note légale globales de page utilisent `CLegalInfoCard` et suivent toujours le même schéma :

- un titre explicite ;
- un résumé pédagogique de l'obligation ;
- un ou plusieurs liens légaux libellés avec une référence longue (`Article N …`) vers le texte complet sur une source officielle (`legifrance.gouv.fr` ou `service-public.fr`).

Les obligations légales compactes et strictement locales à un libellé peuvent encore utiliser `CTooltip`, mais les notes juridiques de bas de page doivent être homogènes via `CLegalInfoCard`.

Cette règle s'applique notamment aux délais de déclaration de sinistre, au carnet d'entretien réglementaire, au registre de copropriété et aux consultations informelles.

## Règle UI — Vue Signalements

La modale de conversion d'un signalement en tâche doit permettre au gestionnaire de choisir la **priorité** de la tâche au moment de la conversion, sans écran intermédiaire supplémentaire.

- La priorité proposée par défaut est `Normal`.
- La conversion reste une action unique : création de la tâche puis redirection immédiate vers la tâche créée.
- La modale de modification d’un signalement doit aussi afficher les images déjà jointes et permettre leur gestion sans repasser par la carte.
- L’action `Supprimer` reste disponible uniquement dans la modale d’édition, pas sur la carte de liste.

## Règle UI — Gestion documentaire liée

- Toute entité qui accepte des documents doit permettre leur gestion depuis la modale d’édition et, lorsqu’une vue détaillée existe, depuis cette vue détaillée également.
- L’action de retrait d’un document lié ouvre toujours une confirmation avec deux actions distinctes : `Supprimer le lien seulement` et `Supprimer le lien et le document`.
- Le module `Documents` doit exposer les références d’entités liées pour chaque fichier et rappeler, dans la modale de suppression, qu’un document supprimé sera d’abord retiré des entités auxquelles il est rattaché.
- La suppression d’une entité doit aussi supprimer ses documents devenus orphelins ; un document encore rattaché ailleurs n’est pas supprimé.

## Règle UI — Vue Tâches

Le champ d'assignation prestataire dans `TasksView` doit accepter une saisie libre sans forcer le choix d'un prestataire existant.

- Tant qu'une correspondance existe, l'autocomplete continue de proposer le référentiel.
- Si aucune correspondance n'existe, une suggestion dédiée permet d'ajouter ce prestataire au référentiel sans quitter la modale.
- L'ajout au référentiel reste optionnel : si l'utilisateur n'active pas cette suggestion, la saisie libre ne doit pas bloquer l'enregistrement et doit rester visible sur la tâche enregistrée.

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

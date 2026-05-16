# DESIGN.md — l'Application Design System "The Modern Registry"

> **Dénomination** — Dans le présent document, l'expression **« l'Application »** désigne exclusivement **La Ruche**, plateforme web open source de gestion participative de copropriété. Ces deux termes sont interchangeables et désignent la même entité logicielle.
> Dernière mise à jour : 2026-05-16

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

## Primitives de layout de vue — `base.css`

`frontend/src/styles/base.css` porte les primitives globales qui ne doivent pas être surchargées sauf exception explicites.

## Couche utilitaire Tailwind CSS

Le frontend peut utiliser Tailwind CSS comme couche utilitaire locale, mais les tokens et primitives partagées restent normatifs dans `frontend/src/styles/tokens.css`, `base.css`, `typography.css`, `tones.css`, `forms.css`, `tables.css`, `tabs.css`, `kpi.css` et `activity-cards.css`.

- Tailwind ne remplace pas les composants atomiques/moléculaires existants ni les classes globales déjà documentées dans ce fichier.
- Les valeurs de design récurrentes doivent continuer à être exprimées via les variables CSS du design system, puis consommées depuis Tailwind si nécessaire.
- En configuration Tailwind v4, ces tokens sont exposés aux utilitaires via `@theme inline` dans `frontend/src/styles/tailwind.css`, sans `tailwind.config.js` dédié.
- Une vue ne doit pas introduire un empilement massif de classes utilitaires si une primitive partagée existe déjà (`view-header`, `tabs`, `activity-card`, `tone--*`, etc.).

## Règle UI — Eyebrows de page

- Toute page métier avec un titre principal visible doit exposer un eyebrow très court au-dessus du `h1`.
- L'eyebrow complète le titre ; il ne doit pas répéter le nom de la page ni reprendre mot pour mot le breadcrumb.
- Le contenu attendu est une description de 3 à 6 mots indiquant immédiatement ce que l'utilisateur trouve sur la page.
- La primitive visuelle partagée est portée par l'atome `frontend/src/components/atoms/CEyebrow.vue`, qui encapsule désormais la typo eyebrow en style scoped ; une vue ne doit pas redéfinir localement sa typo sauf besoin exceptionnel déjà justifié.
- Les pages détail peuvent utiliser le même pattern à l'intérieur d'une carte d'en-tête si le `h1` principal est porté par cette carte.

## Règle UI — Reminder cards

Les cartes de rappels synthétiques partagées entre vues métier passent par `frontend/src/components/molecules/CReminderCard.vue`.

La molécule porte le conteneur, le titre de section, l’état vide et la structure de ligne (`titre`, `subtitle`, `meta`, accent temporel, actions optionnelles). Une vue ne doit pas recopier localement les wrappers `section-head`, `reminder-list`, `reminder-row`, `reminder-dot` ou leurs variantes tonales si le besoin correspond à cette carte.

## Règle UI — Vues AG (`EP-003`)

- Les vues AG restent des pages métier dédiées (`AssemblyListView`, `AssemblyDetailView`) et ne doivent plus consommer `src/mock/data.ts` dès le démarrage du chantier `EP-003`.
- Toute vue AG branchée sur l'API réelle applique le pattern DDR-005 : chargement via `usePageLoader`, erreur via `CPageError`, attente via `CPageLoader`.
- La fiche AG doit rendre immédiatement lisibles : statut, date d'AG, date limite de convocation (J-21), résolutions et état documentaire (`convocation`, annexes, `PV`).
- Une AG `closed` bascule en lecture seule visible : les actions mutantes disparaissent ou sont désactivées explicitement, sans ambiguïté sur l'état gelé.
- Les rôles non gestionnaires restent en lecture seule sur la préparation de l'AG ; le locataire consulte les résultats et le PV publié mais ne participe pas au vote.
- Le chantier `EP-003` n'embarque pas les patterns avancés de conduite de séance (`président`, `scrutateurs`, quorum temps réel, notes collaboratives) qui relèvent de `EP-013`.
- `S02` doit exposer dans la fiche AG une section documentaire compacte distinguant au minimum les annexes de convocation et le `PV`, avec upload/liaison côté manager et consultation simple côté rôles autorisés.
- `S03` ajoute une carte latérale de présence lisible sans jargon technique : chaque lot porte un mode de participation explicite (`présent`, `mandat`, `correspondance`, `absent`) et un indicateur de tantièmes représentés ; cette carte ne doit pas dépendre du nombre de votes déjà saisis sur les résolutions.

## Système de tonalités couleur — `tones.css`

Défini dans `frontend/src/styles/tones.css`, importé globalement avant tout composant.

Les classes `.tone--*` sont des **utilitaires purs** : elles définissent uniquement deux variables CSS — sans aucune règle de forme, taille ou layout. Elles s’appliquent à n’importe quel élément (icône, bouton, chip, badge, pill…).

Une vue ne doit jamais redéclarer localement `.tone--default`, `.tone--accent`, `.tone--warn`, `.tone--err`, `.tone--disabled`, `.tone--valid` ou les variantes de rôle `.tone--manager`, `.tone--board-member`, `.tone--co-owner`, `.tone--tenant` dans son CSS scoped. Si un composant a besoin d’une forme locale, il conserve uniquement ses règles de taille / rayon / layout et consomme les variables via `background: var(--tone-bg, ...)` et `color: var(--tone-fg, ...)`.

| Classe | `--tone-bg` | `--tone-fg` |
|---|---|---|
| `.tone--default`  | `--color-secondary-container`       | `--color-primary`            |
| `.tone--info`     | `--color-secondary-container`       | `--color-on-secondary-container` |
| `.tone--accent`   | `--gradient-primary`                | `--color-on-primary`         |
| `.tone--warn`     | `--color-warning-container`         | `--color-warning`            |
| `.tone--err`      | `--color-error-container`           | `--color-error`              |
| `.tone--disabled` | `--color-surface-container-highest` | `--color-on-surface-variant` |
| `.tone--muted`    | `--color-surface-container`         | `--color-on-surface-variant` |
| `.tone--valid`    | `--color-success-container`         | `--color-success`            |
| `.tone--manager`  | `--color-role-manager-bg`           | `--color-role-manager-fg`    |
| `.tone--board-member` | `--color-role-board-bg`        | `--color-role-board-fg`      |
| `.tone--co-owner` | `--color-role-co-owner-bg`          | `--color-role-co-owner-fg`   |
| `.tone--tenant`   | `--color-role-tenant-bg`            | `--color-role-tenant-fg`     |

Les éléments consommateurs déclarent :
```css
background: var(--tone-bg, <fallback>);
color:      var(--tone-fg, <fallback>);

`CBadge` applique désormais ce fallback directement au niveau de sa racine, avec `var(--color-surface-container-high)` et `var(--color-on-surface-variant)` pour les variantes sans tonalité dédiée.
```

```html
<!-- icône KPI accentée -->
<div class="kpi-icon tone--accent">…</div>

<!-- filtre actif en mode alerte -->
<button class="kpi-filter kpi-filter--active tone--warn">…</button>

<!-- bouton destructif -->
<button class="btn tone--err">Supprimer</button>
```

## Règle UI — Activity cards

Les cartes `activity-card` constituent le socle partagé des cartes de liste métier (`Contrat`, `Prestataire`, `Tâche`, `Signalement`, `Sinistre`). Toute règle transversale liée à leur structure, leur hiérarchie visuelle ou leurs primitives CSS doit être documentée ici, et non dispersée dans les sections de vues.

### Ossature commune

- Les cartes reposent sur `frontend/src/styles/activity-cards.css`.
- Dès qu’un header contient une icône métier, il utilise `activity-card__header-row activity-card__header-row--with-icon` pour conserver un centrage vertical stable entre icône, titre et chips.
- Les titres `activity-card` restent compacts (18px), en police de texte classique (`font-body`), jamais en display.
- La description utilise une taille de texte standard, en ton principal mais sans graisse excessive ; les metadata / secondary information restent volontairement plus légères via une teinte plus douce.
- Les lignes compactes `activity-card` n’utilisent pas de séparateur ; le groupement repose sur l’alignement, la proximité et des espacements icône/texte serrés.

### Primitives partagées

Les patterns de contenu récurrents doivent être mutualisés dans `frontend/src/styles/activity-cards.css` au lieu d’être redéclarés localement.

- `activity-card--compact` pour les variantes de carte qui gardent la grammaire `activity-card` mais avec un rythme vertical plus serré.
- `activity-card__chips` pour toute grappe de chips / badges d’une `activity-card`, y compris dans l’en-tête. Une vue ne doit pas introduire de wrapper alternatif dédié si ce conteneur couvre déjà le besoin.
- Les métadonnées de carte (`date`, `auteur`, `lot`, localisation, etc.) passent par la molécule `frontend/src/components/molecules/CMeta.vue` avec style scoped ; une vue ne doit plus utiliser la classe globale `activity-card__meta`.
- Les lignes élémentaires `icône + contenu` à l'intérieur d'une métadonnée passent par `frontend/src/components/molecules/CMetaItem.vue`, avec un slot `icon` dédié. Cette molécule porte les variantes typographiques `footer` par défaut et `sub` (`13px` / `1.5`) ; une vue ne doit plus recopier localement un wrapper `span.activity-card__line` pour ce pattern simple.
- Les encarts d’information secondaire (`activity-card__note`) passent par la molécule `frontend/src/components/molecules/CCardNote.vue` avec style scoped ; une vue ne doit plus utiliser la classe globale `activity-card__note`.
- Les séparateurs discrets entre lignes de métadonnées (point circulaire 3px ou ligne) utilisent l'atome `frontend/src/components/atoms/CSeparator.vue` avec variant "horizontal" ou "vertical" et style scoped.
- `activity-card__contacts` pour la pile verticale email / téléphone.
- `activity-card__contact-link` pour les liens de contact dans une carte, avec préfixe emoji métier (`✉️` pour l’email, `📞` pour le téléphone).
- `activity-card__documents-button` pour le bouton inline ouvrant les pièces liées depuis `activity-card__documents`.

Si une vue réutilise exactement l’un de ces patterns, elle doit consommer la classe partagée et supprimer le CSS local redondant.

### Grammaire des cartes Opérations

Les cartes de liste des vues `Signalements`, `Tâches` et `Sinistres` partagent une même ossature visuelle afin d’améliorer le balayage :

- ligne 1 : en-tête compact propre à l’entité, avec au minimum le statut visible sans ouvrir la carte ;
- ligne 2 : titre principal ou, pour certaines cartes, reprise du titre juste sous l’en-tête si cela améliore la lisibilité ;
- ligne 3 : description courte éventuelle, en taille de texte standard ;
- ligne 4 : hints documentaires / photo sur leur propre ligne ;
- ligne 5 : métadonnées compactes avec icônes, sans séparateur ;
- ligne d’actions avec `Modifier` puis une action contextuelle accentée ;
- bloc informatif secondaire pour les éléments métier complémentaires (délai légal, remboursement, lien prestataire, historique, etc.).

### Variantes métier autorisées

- `Contrat` : header `icône métier + titre + chip statut`, avec chip domaine/type sur une rangée dédiée juste sous l’en-tête ; hint documentaire ouvrable directement depuis la carte quand des pièces existent.
- `Prestataire` : même grammaire d’en-tête que `Contrat`, sans chip de statut quand aucune information de cycle de vie n’existe.
- `Tâche` : header compact `titre + priorité + statut`, sans icône ; l’assignation et la date de création relative sont regroupées dans les métadonnées.
- `Signalement` : première ligne `icône + (statut + mise à jour relative simple)`, titre aligné à gauche sous l’en-tête, viewer photo compact puis agrandi.
- `Sinistre` : icône métier obligatoire avec couleur dépendante du type ; chip de type affichée dans le header à gauche du statut ; hint documentaire positionné au-dessus des métadonnées `date/auteur`.

L’ossature reste commune mais chaque entité peut substituer un équivalent métier lorsqu’une donnée n’existe pas (ex. `prestataire assigné` au lieu d’une localisation stricte sur les tâches).

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

La règle normative reste portée par `docs/ddr/DDR-005-page-loading-states.md`.

Résumé d'usage pour le design system :
- toute vue qui charge ses données au montage passe par `usePageLoader` ;
- l'état d'erreur s'affiche via `CPageError`, l'état de chargement via `CPageLoader` ;
- les chargements secondaires non bloquants restent isolés pour ne pas retarder le rendu principal.

Références d'implémentation : `frontend/src/composables/usePageLoader.ts`, `frontend/src/components/organisms/CPageError.vue`, `frontend/src/components/molecules/CPageLoader.vue`.

## Règle UI — Vue Contrats

La vue `Contrats` affiche en tête les rappels à traiter, avec une action d’acquittement visible sans navigation secondaire.

- Les tabs de filtre `Contrats` affichent le volume d’éléments par état (`Tous`, `Actifs`, `Expirés`) directement dans chaque segment.
- Les pièces justificatives d’un contrat se gèrent directement depuis la modale de création / modification, sans CTA documentaire primaire sur la carte.
- Chaque carte Contrat rappelle aussi le nombre de documents actuellement liés et permet d’ouvrir une modale de consultation/téléchargement directement depuis ce hint lorsque des pièces existent.
- La modale d’édition liste les pièces déjà liées avec deux issues explicites : retirer le lien seulement, ou retirer le lien puis supprimer définitivement le document.
- La suppression définitive d’un contrat se déclenche depuis la modale d’édition, jamais depuis la carte.- Un contrat `active` ne peut pas être modifié sur ses champs financiers (`amount`, `start_date`, `end_date`). L'action « Modifier » est remplacée par « Créer un avenant » qui clôture le contrat courant et ouvre le formulaire pré-rempli pour le nouveau (ADR-009).
- La fiche contrat expose la chaîne des avenants via `predecessor_id` : chaque contrat de la chaîne est affiché avec sa période de validité et ses pièces jointes propres.
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
- L'explication juridique des délais est mutualisée dans une `CCard` contenant `CLegalNotice` en bas de page.
- La création d'un sinistre reste faisable dans une seule modale et accepte des pièces jointes sans écran intermédiaire.
- Les zones d'ajout de pièces jointes des modales Sinistres reprennent la dropzone standard utilisée sur les Signalements (`bordure pointillée`, état hover/drag, sélection directe de fichiers).
- Chaque carte Sinistre expose toujours deux actions côte à côte pour les profils gestionnaires : `Modifier` puis une action contextuelle dépendante du statut courant.
- Les actions contextuelles suivent strictement ce mapping : `Ouvert → Transmettre`, `En cours → Clore`, `Clos → Supprimer`.
- `Transmettre` ouvre une modale dédiée avec la date de transmission à l'assurance ; `Clore` ouvre une modale dédiée avec remboursement optionnel (date + montant) et ajout de documents ; `Modifier` permet aussi de supprimer le dossier sans revenir à la carte et de gérer les documents déjà joints.
- Les tabs de filtre `Sinistres` affichent aussi le volume par état (`Tous`, `Ouverts`, `En cours`, `Clos`) pour rendre le backlog lisible sans ouvrir chaque segment.

## Règle UI — Cartes Opérations (`Signalements` / `Tâches` / `Sinistres`)

La structure partagée et les primitives communes des cartes `Signalements`, `Tâches` et `Sinistres` sont définies dans la section `Règle UI — Activity cards`.

Contraintes complémentaires :

- les listes sont triées par `dernière modification` décroissante ;
- la carte `Sinistre` n'expose jamais `Supprimer` dans la liste ; cette action reste confinée à la modale d'édition ;

## Règle UI — Obligations légales

Les cartes de note légale globales de page utilisent `CCard` + `CLegalNotice` et suivent toujours le même schéma :

- un ou plusieurs `CLegalDescription` avec un titre explicite et un résumé pédagogique de l'obligation ;
- un ou plusieurs liens légaux libellés avec une référence longue (`Article N …`) vers le texte complet sur une source officielle (`legifrance.gouv.fr` ou `service-public.fr`).

`CLegalDescription` porte le bloc textuel `⚖️ + titre + résumé`, tandis que `CLegalNotice` ne gère que la pile verticale et la rangée de liens. Cette séparation permet aussi les cartes juridiques multi-entrées, par exemple pour détailler plusieurs règles de majorité sur une même AG.

Les obligations légales compactes et strictement locales à un libellé peuvent encore utiliser `CTooltip`, mais les notes juridiques de bas de page doivent être homogènes via `CCard` + `CLegalNotice`.

Cette règle s'applique notamment aux délais de déclaration de sinistre, au carnet d'entretien réglementaire, au registre de copropriété et aux consultations informelles.

## Règle UI — Vue Budget & Charges

- Les KPI budget sur mobile restent lisibles en grille 2 colonnes (2 lignes pour 4 KPI), sans défilement horizontal.
- Les KPI budget consomment désormais un composant partagé `CHeroCard` (HeroCard) construit sur `CCard` (header `title + icon`, bloc KPI central `data + subtitle`, footer optionnel).
- Le bloc KPI central de `CHeroCard` est mutualisé via un atome dédié (`CKpiMetric`) également réutilisable dans `CKpiCard`.
- La topbar de la vue budget n'expose qu'une action globale : `Clore l'exercice` (avec confirmation modale).
- Sur écran étroit, le bouton topbar `Clore l'exercice` conserve l'icône visible mais masque son libellé texte via `.btn-label`.
- Les actions métier locales (`Nouvelle ligne`, `Générer les appels`, `Marquer comme payé`) sont portées par leurs cartes respectives, alignées à droite sur la ligne de titre.
- Les sections `Consommation par poste` et `Appels de fonds` n'exposent chacune qu'une seule action primaire de header (pas de doublon d'action en état vide).
- Sur petit écran, les boutons d'action de header de cartes conservent l'icône et masquent le libellé texte.
- Le bloc légal Budget est homogène avec le pattern de bas de page `CLegalCard` + `CLegalNotice` + `CLegalDescription`.
- Le sous-titre de la carte `Appels de fonds` utilise deux métriques séparées (`nombre d'appels`, `taux de recouvrement`) via `CFlexGroup` horizontal + `CMetaItem`.
- Tout sous-titre budget qui exprimait auparavant plusieurs informations dans une seule chaîne avec point médian (`·`) doit être rendu via `CFlexGroup direction="row"` + `CMetaItem variant="sub"` + `CSeparator`.
- Le rappel des appels en retard est affiché en haut de la page Budget, au-dessus des cartes KPI.
- Le tableau des appels repose sur le composant partagé `CDataTable` (pas de table HTML locale), avec total affiché en pied de carte.
- La carte `Consommation par poste` expose une action secondaire à icône de diagramme circulaire ouvrant une modale de répartition par poste ; la modale doit montrer le poids relatif de chaque ligne et une légende textuelle accessible.
- La modale de répartition affiche les montants en typographie tabulaire mono (`font-mono`).
- Les mentions légales liées au fonds travaux doivent citer la base à jour (`art. 14-2-1 loi 1965`, rédaction issue de la loi du 22 août 2021) plutôt qu'une référence générique à ALUR seule.

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
- Les dropzones documentaires inline utilisent la feuille globale `frontend/src/styles/forms.css` et la même base visuelle que l’ancienne `claim-dropzone` : bordure pointillée, état `hover/drag`, icône neutre et hint secondaire.

## Règle UI — Vue Tâches

Le champ d'assignation prestataire dans `TasksView` doit accepter une saisie libre sans forcer le choix d'un prestataire existant.

- Tant qu'une correspondance existe, l'autocomplete continue de proposer le référentiel.
- Si aucune correspondance n'existe, une suggestion dédiée permet d'ajouter ce prestataire au référentiel sans quitter la modale.
- L'ajout au référentiel reste optionnel : si l'utilisateur n'active pas cette suggestion, la saisie libre ne doit pas bloquer l'enregistrement et doit rester visible sur la tâche enregistrée.
- Le bloc `lien prestataire` de la carte doit rester visible tant qu'un token est actif ; les actions `Générer`, `Afficher` et `Révoquer` se placent au-dessus de la ligne d'actions principale, et l'URL active doit être lisible / copiable directement depuis la carte.
- La métadonnée prestataire de la carte n'apparaît que lorsqu'une assignation existe réellement ; elle utilise l'icône métier `CIconWrench` pour un prestataire externe, `CIconUser` pour une auto-assignation, et pointe vers la fiche prestataire si `contractor_id` est connu.
- La modale de création / modification d'une tâche reprend la dropzone documentaire standard des autres modules pour joindre des fichiers sans écran intermédiaire, puis liste aussi les documents déjà liés en édition avec téléchargement et retrait explicites.

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

## Règle UI — Entités comptables gelées (ADR-009)

Toute entité ayant atteint un état terminal comptablement (levy_call `issued`, tâche `closed`, sinistre avec `transmitted_at` renseigné, etc.) doit être visuellement distinguée des entités modifiables.

- **Badge "Gelé"** : une icône cadenas (`CIconLock`) accompagnée du libellé de l'état terminal est affichée dans le header de la carte ou de la vue détail. Elle utilise `.tone--muted`.
- **Actions supprimées** : les boutons Modifier et Supprimer disparaissent de la carte et de la modale — ils ne sont pas simplement désactivés, afin d'éviter toute confusion.
- **Champs en lecture seule** : dans une modale ou vue détail, les champs immuables sont rendus sous forme de `<span>` (pas `<input readonly>`), précédés d'une mention contextuelle discrète « Donnée comptable — non modifiable » en `label-sm` avec `.tone--muted`.
- **Action de remplacement (Pattern B)** : pour les contrats actifs, l'action « Créer un avenant » remplace « Modifier » et s'affiche avec le variant `ghost` standard. Elle ne doit pas être présentée comme une action destructive.
- **Frise d'historique** : sur les entités avec `predecessor_id` (contrats) ou `parent_budget_id` (budgets), une section « Historique » liste la chaîne des versions précédentes avec leur période de validité, sans action disponible sur les entrées historiques.

## DDR
| Ref | Décision |
|---|---|
| DDR-001 v3 | Fraunces (display) + DM Sans (corps) + JetBrains Mono (data tabulaire) |
| DDR-002 | Lavande/charcoal, light+dark, tokens CSS vars uniquement |
| DDR-003 | Bottom nav mobile, sidebar desktop, filtrage par rôle |
| DDR-004 | Groupes sidebar (pattern B labels fixes), dropdown profil utilisateur, routes `/residence/*` et `/profile/*` |
| DDR-005 | États de chargement et d'erreur de page — `usePageLoader` + `CPageError` |
| DDR-006 | `CFormFeedback` dans le slot `#footer` de toute `CModal` avec formulaire ou action destructive |

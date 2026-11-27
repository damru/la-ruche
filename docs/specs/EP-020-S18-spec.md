# Spécification technique — EP-020-S18

**Story :** Passage mobile — densité, débordements et règle des actions.

**Épic :** EP-020 — Design system
**Réf. DESIGN :** `docs/DESIGN.md` §4.1 « Actions de ligne », §4.1 « Cartes de liste », §5 (densité), `docs/EPICS.md` (EP-020-S18)

> Frontend uniquement. Aucune modification backend.

---

## 1. Règle des actions (énoncée par Damien)

| Contexte | Desktop (≥ 640 px) | Mobile (≤ 639 px) |
|---|---|---|
| Boutons d'action des **cartes** (mode liste et mode grille) | icône + libellé | icône seule |
| Boutons d'action des **en-têtes de vue** (topbar `#right`, `CPageHeader #actions`) | icône + libellé | icône seule |
| Actions de **tableau** (cellules, `CCardActions` dans les lignes) | icône seule | icône seule |

Le mécanisme existe déjà dans `CBtn` et ne doit pas être doublé :

- `title` est rendu en texte visible (`.c-btn__title`) : c'est le libellé, pas seulement un tooltip ;
- `shrink="sm"` replie le bouton en boîte d'icône (`width: var(--c-btn-box)`, `gap: 0`, libellé masqué) sous 640 px ;
- `aria-label` est posé automatiquement quand le bouton n'a pas de slot par défaut, donc l'icône seule reste annoncée.

Conséquence : appliquer la règle = passer `shrink="sm"` sur les boutons de cartes et d'en-têtes qui portent un `:icon` et un `:title`. Ne rien ajouter aux actions de tableau (elles gardent `variant="icon"` ou l'absence de libellé).

## 2. Débordement horizontal des tableaux

`CDataTable` défile déjà (`overflow-auto`) mais rien ne signale qu'il déborde : à 375 px une colonne coupée est indiscernable d'une colonne absente.

- Ajout de l'utilitaire `@utility table-scroll` dans `frontend/src/styles/tailwind.css` : deux dégradés `local` (masques des bords) et deux dégradés `scroll` (ombres de bord) peints par le conteneur, sans JS. Les ombres disparaissent naturellement en fin de course.
- `CDataTable` porte la classe en plus de `overflow-auto`.

## 3. État d'implémentation

| Point | État |
|---|---|
| Règle des actions — cartes et en-têtes | ✅ 41 boutons (`shrink="sm"`) sur `ContractsView`, `ContractDetailView`, `ClaimsView`, `TasksView`, `AssemblyDetailView`, `ConsultationsView`, `ContractorsView`, `ReportsView`, `ReportDetailView`, `MaintenanceView`, `DocumentsView`, `AllocationKeysView`, `EtatDateView`, `GovernanceMandatesView`, `JudicialProceedingsView`, `BuildingView`, `BudgetView`, `ResidentsView`, `ProfileView` |
| Débordement des tableaux | ✅ `table-scroll` |
| Héros trop hauts en mobile (`DashboardView`, `RegistryView`, `MyChargesView`, `UnitChargesView`) | ⏳ à traiter |
| Pill « délai légal » en débordement (`ClaimsView`) | ⏳ à traiter — les chips d'en-tête de carte passent en `flex-wrap` via `CCardHeader`, le badge lui-même garde `nowrap` |
| Lignes de tableau gouvernance trop hautes | ⏳ à traiter |
| Boucle de contrôle 375 px sur toutes les captures | ⏳ à traiter |

## 4. Critères d'acceptation

- [x] Un bouton d'action de carte ou d'en-tête de vue affiche icône + libellé en 1440 px et icône seule en 375 px.
- [x] Une action de tableau reste icône seule aux deux largeurs.
- [x] `CDataTable` signale visuellement un débordement horizontal, sans JS ni dépendance.
- [ ] Aucune capture 375 px ne présente de débordement horizontal (`scrollWidth > clientWidth` sur `body`).
- [ ] Densité mobile conforme `DESIGN.md §5` sur les vues à héros.

## 5. Plan de tests

- `npm --prefix frontend run lint`, `typecheck`, `build`.
- Vérification sur les captures CI : `contracts--375--light`, `claims--375--light`, `tasks--375--light`, `documents--1440--light` (tableau débordant).
- Contrôle mécanique : pour chaque route de `e2e/data/access-matrix.ts`, `document.documentElement.scrollWidth <= window.innerWidth` à 375 px (à intégrer aux tests e2e de consistance).

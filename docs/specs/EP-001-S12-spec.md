# Spécification technique — EP-001-S12

**Story :** Follow-up UI/UX de l'onglet EDD (espacements, style du sélecteur de fichier, ajout de lots historiques, clarification du sigle EDD, repositionnement du bouton de création)

**Épic :** EP-001 — Configuration copropriété & clés de répartition
**Réf. LEGAL :** §1.2 · **Dépendance :** EP-001-S11 (onglet EDD fusionné dans `AllocationKeysView.vue`, mergé)
**ADR/DDR :** aucun requis — corrections de style/UX sur composants existants, pas d'alternative structurelle.

**Statut initial :** NON IMPLÉMENTÉ. 5 griefs distincts remontés par l'utilisateur après livraison d'EP-001-S11, vérifiés sur le code réel (`frontend/src/features/residence/EddTab.vue`, main).

---

## 1. Constats vérifiés (état du code, pas des suppositions)

| # | Grief utilisateur | Preuve dans le code |
|---|---|---|
| 1 | Cartes de l'onglet EDD mal espacées | `EddTab.vue` contient un bloc `<style scoped>` (lignes 389-415) — **viole `AGENTS.md` frontend** : *"No `<style>` block in feature views — Tailwind utilities or catalogue components only"*. Les espacements (`.edd-tab-actions`, `.edd-current-reference`, `.shares-section`) sont posés en CSS custom plutôt qu'en utilitaires Tailwind cohérents avec le reste du catalogue. |
| 2 | Sélecteur de document sans bordure cohérente | `EddTab.vue:323-327` utilise un `<input type="file">` brut, sans wrapper catalogue, alors que `CFileDropzone.vue` / `CFormDropzone.vue` existent déjà et sont utilisés ailleurs dans l'app pour ce même besoin. Violation directe de la règle "no raw values… use catalogue components only". |
| 3 | Impossible d'ajouter un lot disparu depuis lors de la saisie | `openForm()` initialise `form.shares` uniquement à partir de `units.value` (lots **actifs actuels**, filtrés `u.active` dans `loadAll()`). Aucun mécanisme pour ajouter une ligne « lot libre » (numéro de lot texte + tantièmes) pour un lot archivé/supprimé depuis la date de l'EDD. |
| 4 | Sigle « EDD » jamais explicité | Toutes les clés i18n (`settings.edd.title`, `newReference`, `empty`, `currentReference`…) et le libellé de nav (`nav.residence-edd` via `AppLayout.vue:80`) utilisent uniquement le sigle. Le terme complet *« État Descriptif de Division »* n'apparaît dans aucun texte affiché à l'utilisateur. |
| 5 | Bouton « Nouvelle référence » mal placé | Le bouton d'action est dans `.edd-tab-actions` en haut du contenu de l'onglet (`EddTab.vue:197-215`), pas dans la topbar de la page comme le bouton « Nouvelle clé » de l'onglet Clés (`CTopbar` de `AllocationKeysView.vue`). Le bouton « Nouvelle clé » disparaît quand on est sur l'onglet EDD (cohérent, il ne s'applique pas), mais rien ne prend sa place au même emplacement. |

## 2. Ce qui NE change PAS

- Le formulaire de saisie de référence EDD reste inchangé fonctionnellement (label, date, upload document, tantièmes par lot).
- Le rapport de cohérence (discrepancies) reste identique.
- Aucune migration, aucun changement backend/API.

## 3. Frontend — changements attendus

### 3.1 Espacements (grief #1)
- Retirer le bloc `<style scoped>` de `EddTab.vue`.
- Remplacer chaque règle par des utilitaires Tailwind existants dans le catalogue (`gap-*`, `mb-*`, `flex`, `border-b` avec token de couleur), au lieu de CSS custom.
- Vérifier visuellement l'espacement entre les cartes (`CCard`) : actuellement aucun `gap`/`mb` explicite entre les 2-3 `CCard` consécutives de l'état "référence existante" (lignes 240-291) — c'est la cause probable du problème remonté.

### 3.2 Style du sélecteur de document (grief #2)
- Remplacer l'`<input type="file">` brut (ligne 323-327) par `CFileDropzone.vue` ou `CFormDropzone.vue` (choisir celui déjà utilisé dans un contexte de formulaire similaire — à vérifier lors de l'implémentation lequel correspond le mieux, ex. `SaveTaskModal.vue` si applicable).
- Adapter `onFileChange` à l'API du composant choisi (probablement `v-model` sur un `File | null` plutôt qu'un event `@change` sur un input natif).

### 3.3 Ajout de lots disparus (grief #3)
- Dans le formulaire de saisie de référence EDD, ajouter un mécanisme pour insérer une ligne « lot manuel » : numéro de lot (texte libre, puisque le lot n'existe plus en base) + valeur de tantièmes.
- Ces lignes manuelles s'ajoutent à `form.shares` à côté des lots actifs pré-remplis, distinguables (ex. lot non résolvable via `unitLabel`).
- Le payload `eddApi.createReference(...)` doit accepter ces entrées ; vérifier si le backend (`edd_reference_share`, `unit_id UUID NOT NULL REFERENCES unit(id)`) autorise un `unit_id` pointant vers un lot soft-deleted (`unit.active = false`) — **point à trancher en implémentation** : soit le lot archivé est sélectionnable dans une liste (dropdown incluant les lots inactifs), soit un champ texte libre nécessite une évolution du schéma (FK stricte vers `unit(id)` n'admet pas un simple texte).
  - Option recommandée (moindre effort, pas de migration) : proposer un sélecteur incluant les lots archivés (`unitsApi.list()` sans filtrer `active`), avec un badge « archivé » — le FK reste valide car le lot existe toujours en base (soft delete), seul le filtre `u.active` dans `EddTab.vue` empêchait de le voir.

### 3.4 Terme complet EDD (grief #4)
- Ajouter le terme complet dans le titre affiché : `settings.edd.title` → *« Contrôle de cohérence — État Descriptif de Division (EDD) »* (ou variante à valider), et dans le libellé de nav si la place le permet (`nav.residence-edd`).
- Ne pas remplacer le sigle partout (trop long dans les badges/messages courts) — seulement au premier titre visible de la page/section, pattern standard « terme complet (sigle) » puis sigle seul ensuite.

### 3.5 Repositionnement du bouton (grief #5)
- Déplacer le bouton « Nouvelle référence EDD » dans le `CTopbar` de `AllocationKeysView.vue`, conditionné à l'onglet actif (`v-if="activeTab === 'edd'"`), au même emplacement que le bouton « Nouvelle clé » (`v-if="activeTab === 'keys'"`).
- Vérifier comment `AllocationKeysView.vue` gère déjà le slot d'actions de la topbar par onglet (probablement déjà conditionnel pour "Nouvelle clé" qui disparaît sur l'onglet EDD — à confirmer en lisant le fichier avant implémentation).

## 4. Cas limites

- Lot manuel ajouté (grief #3) avec une valeur de tantièmes à 0 → exclu du payload à la sauvegarde (cohérent avec le filtre existant `s.share_value > 0` déjà présent pour les autres lots).
- Aucun lot archivé disponible dans le sélecteur (immeuble sans historique de lots supprimés) → pas de section vide affichée, comportement inchangé.
- Bouton topbar repositionné doit rester caché quand le formulaire de saisie est déjà ouvert (`showForm === true`), comme c'est déjà le cas actuellement (`v-if="!showForm"`).

## 5. Critères d'acceptation

1. Aucun bloc `<style scoped>` dans `EddTab.vue` ; espacement visuellement cohérent entre les cartes.
2. Le sélecteur de document EDD utilise un composant catalogue (`CFileDropzone`/`CFormDropzone`), bordure cohérente avec le reste des champs du formulaire.
3. Un lot archivé (non actif) est sélectionnable lors de la saisie d'une référence EDD, avec indication visuelle qu'il est archivé.
4. Le titre de la page/section EDD affiche le terme complet « État Descriptif de Division » au moins une fois, sigle seul ensuite.
5. Le bouton de création d'une nouvelle référence EDD est positionné dans la topbar, à l'emplacement du bouton « Nouvelle clé » quand l'onglet EDD est actif.
6. `npm run build` / `npm run lint` passent sans erreur.

## 6. Plan de tests

- Test manuel (pas de logique backend) : vérifier l'affichage aux 3 états (vide / avec référence / formulaire ouvert), sélection d'un lot archivé, upload via le nouveau composant dropzone, position du bouton topbar par onglet.
- Pas de nouveau test e2e backend requis (aucun changement API).

## 7. Ordre d'implémentation

1. Retirer `<style scoped>`, migrer en Tailwind (grief #1) — vérifier visuellement l'espacement des cartes.
2. Remplacer l'input file brut par le composant catalogue approprié (grief #2).
3. Étendre le sélecteur de lots pour inclure les lots archivés avec badge (grief #3) — vérifier que `unit.active = false` n'empêche pas la FK côté backend (normalement non, simple filtre frontend à lever).
4. Ajouter le terme complet EDD au titre (grief #4).
5. Déplacer le bouton de création dans la topbar, conditionné à l'onglet actif (grief #5).
6. `docs/PROGRESS.md` : entrée datée EP-001-S12.
7. PR + revue Hopper (`kanban_request_review`).

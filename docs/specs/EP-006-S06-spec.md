# Spécification technique — EP-006-S06

**Story :** Lister et consulter les contrats fournisseurs : filtres statut/prestataire/type, vue liste structurée, fiche contrat détaillée.

**Épic :** EP-006 — Contrats fournisseurs
**Réf. PRD :** `docs/product/PRD - Contrats fournisseurs.md` US-CF-002
**Statut : IMPLÉMENTÉ (frontend)** — filtres statut/type/prestataire **côté client** (multi-select `CFormMultiSelect`, options dérivées de la liste chargée), bascule tableau/cartes, fiche contrat `ContractDetailView` sur `/contracts/:id` (en-tête, chaîne d'avenants, documents liés, rappels). Le backend expose `GET /contracts` sans filtre.

**Décision §2 actée (2026-10-22, product owner), révisée 2026-10-23 (alignement UI /contractors) : tableau par défaut + bascule vers les cartes (`CDataTable`).** Le PRD US-CF-002 (AC-CF-002-B) est amendé en conséquence (Q-CF-004 tranchée).

**Décision filtrage (2026-10-23, product owner) : filtrage côté client**, comme sur toutes les pages de l'application (volumétrie faible par résidence). La migration serveur de tous les filtres applicatifs est tracée en tâche technique post-MVP **T-005** (`docs/EPICS.md`).

---

## 1. Ce qui existe déjà (ne pas re-spécifier)

- `ContractsView.vue` : onglets `all/active/expired` avec compteurs (L234-238, L411-413), dérivation de statut côté client (L130-134), recherche plein texte (L199-216), grille de **cartes** (~L638).
- Le type de contrat est affiché sur la carte (L459, L468) mais non filtrable.
- Modales : `ContractFormModal`, `AmendContractModal`, `AmendmentChainModal`, `TerminateContractModal`, `CLinkedDocumentsModal` (L606-614).
- Backend : `GET /contracts` **sans aucun filtre** (`api/contracts.rs` L17-23).

## 2. Ce qui manque (périmètre)

1. Filtres **type** et **prestataire** (multi-selects), combinables avec statut et recherche — **côté client** (décision 2026-10-23, cf. T-005 pour la migration serveur).
2. Une **fiche contrat** (`ContractDetailView`, route dédiée) — aujourd'hui tout passe par des modales.

### Décision de présentation — à trancher avec le PRD

Le PRD US-CF-002 demande une « vue tabulaire ». L'existant est une grille de cartes aboutie. **Recommandation : conserver les cartes** et ajouter une rangée de filtres (statut via onglets existants + selects type/prestataire) — la carte porte déjà toutes les colonnes qu'aurait le tableau, et une table dupliquerait l'existant pour un gain nul à l'échelle d'une copropriété (dizaines de contrats).
→ **Écart PRD assumé** : si les cartes sont conservées, amender le PRD US-CF-002 (« vue tabulaire » → « vue liste filtrable ») ; sinon, implémenter un toggle cartes/table (`CTable` du catalogue). La suite de la spec suppose l'option recommandée (cartes conservées).

---

## 3. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| Liste backend non filtrée | `backend/src/api/contracts.rs` L17-23 → `db/contract.rs::list` |
| Vue liste | `frontend/src/features/contracts/ContractsView.vue` (onglets L234-238/L411-413, statut client L130-134, recherche L199-216, grille ~L638, modales L606-614) |
| Chaîne d'avenants (EP-018-S05) | `AmendmentChainModal.vue`, `GET /contracts/{id}/amendments` |
| Documents liés | `contract_document` (`0001_init.sql` L495-499), endpoints `api/contracts.rs` L168-213 |
| Rappels | `contract_reminder` (`0001_init.sql` L503-512) |
| API client | `frontend/src/api/index.ts` (`contractsApi`) |
| Routes frontend | `frontend/src/router` (suivre le pattern des vues détail existantes, ex. `ContractorDetailView.vue`) |

---

## 4. Modèle de données

Aucune migration. (Le « prestataire » est le champ libre `vendor_name` — pas de référentiel fournisseur, cf. EP-018-S05 §9 : hors périmètre.)

---

## 5. Backend Rust

**Aucun changement** (décision filtrage client, 2026-10-23). `GET /contracts` reste sans filtre ; `GET /contracts/{id}` existe déjà pour la fiche.

---

## 6. Frontend Vue

### 6.1 Filtres — `ContractsView.vue`

- Conserver les onglets de statut ; ajouter une rangée de filtres : multi-select **Type** (valeurs de l'enum `contract_type` présentes dans la liste, libellés FR existants) + multi-select **Prestataire** (noms distincts dérivés de la liste) + recherche existante.
- Filtrage **côté client** (décision 2026-10-23) : les filtres type/prestataire s'appliquent dans le `computed` de la liste, comme le statut (onglets) et la recherche plein texte. Aucun rechargement réseau au changement de filtre — cohérent avec `/contractors` et le reste de l'app.
- Les compteurs d'onglets dérivent de la liste complète, non filtrée.

### 6.2 Nouvelle vue `ContractDetailView.vue`

- Route `/contracts/:id` (pattern `ContractorDetailView`). La carte de la liste devient cliquable → navigation.
- Sections :
  1. **En-tête** : vendor, subject, badge statut, type, montant, dates, préavis, actions existantes (Modifier, Avenant, Résilier — réutiliser les modales existantes).
  2. **Chaîne d'avenants** : réutiliser le fetch `contractsApi.amendmentChain` ; affichage inline (timeline) — la modale `AmendmentChainModal` reste pour la liste.
  3. **Documents liés** : réutiliser les endpoints `contract_document` (liste + rattacher/détacher, composants de `CLinkedDocumentsModal` réutilisés inline).
  4. **Rappels** : liste des `contract_reminder` du contrat (si un endpoint expose les rappels par contrat ; sinon l'ajouter : `GET /contracts/{id}/reminders`, repo trivial).
- Composants : réutiliser le catalogue (`CCard`, `CBadge`, `CTabs`…). **Si un nouveau composant `C*` est créé, le cataloguer dans `docs/DESIGN.md` §3 avant usage** (étape d'implémentation).

### 6.3 `frontend/src/api/index.ts`

`contractsApi.list()` (sans paramètre), `contractsApi.reminders(id)` (si section 4 retenue).

---

## 7. Cas limites

| Cas | Règle |
|---|---|
| Filtres combinés (type + prestataire + onglet + recherche) | Intersection côté client ; état vide avec message dédié. |
| Type/prestataire absent de la liste | N'apparaît pas dans les options du filtre (options dérivées des données). |
| Contrat d'un autre immeuble | `GET /contracts/{id}` → `404` (scoping `building_id` existant). |
| Contrat `draft` (EP-018-S05) | Visible dans la fiche avec bandeau « brouillon — à activer » et bouton Activer. |
| Prestataires en doublon typographique (`vendor_name` libre) | Assumé — champ libre, pas de normalisation dans cette story. |
| Deep-link `/contracts/:id` supprimé | `404` API → page d'erreur/redirect liste. |

---

## 8. Critères d'acceptation

1. La liste combine onglet statut + type + prestataire + recherche, **filtrage côté client** (décision 2026-10-23, T-005 pour la migration serveur).
2. Cliquer une carte ou une ligne ouvre `/contracts/:id` : en-tête complet, chaîne d'avenants, documents liés, rappels.
3. Les actions (modifier/avenant/résilier/documents) fonctionnent depuis la fiche.
4. L'écart « cartes vs vue tabulaire » est acté : PRD amendé + toggle table implémenté (décision §2).

---

## 9. Plan de tests

- **e2e backend** : non-régression S07 (lecture documents liés en `board_member` → 200, attach/detach → 403, acquittement rappel `board_member` → 403 / `manager` → 204) dans `backend/tests/contract_visibility_test.rs`.
- **Frontend** : build + lint ; si vitest configuré, test de `ContractDetailView` (fetch au montage, sections rendues) ; sinon vérification manuelle.

---

## 10. Ordre d'implémentation

1. **Trancher la décision §2 (cartes vs tabulaire) avec le product owner** ; amender le PRD le cas échéant.
2. Frontend api + types.
3. `ContractsView.vue` : rangée de filtres (client), navigation carte → fiche.
4. `ContractDetailView.vue` + route (cataloguer tout nouveau `C*` dans `docs/DESIGN.md` §3 avant usage).
5. `npm run build` + `lint` ; `docs/PROGRESS.md`.

# Spécification technique — EP-006-S06

**Story :** Lister et consulter les contrats fournisseurs : filtres statut/prestataire/type, vue liste structurée, fiche contrat détaillée.

**Épic :** EP-006 — Contrats fournisseurs
**Réf. PRD :** `docs/product/PRD - Contrats fournisseurs.md` US-CF-002
**Statut : PARTIEL** — la liste avec onglets de statut et recherche libre existe ; manquent les filtres type/prestataire, le filtrage côté backend et la fiche contrat.

---

## 1. Ce qui existe déjà (ne pas re-spécifier)

- `ContractsView.vue` : onglets `all/active/expired` avec compteurs (L234-238, L411-413), dérivation de statut côté client (L130-134), recherche plein texte (L199-216), grille de **cartes** (~L638).
- Le type de contrat est affiché sur la carte (L459, L468) mais non filtrable.
- Modales : `ContractFormModal`, `AmendContractModal`, `AmendmentChainModal`, `TerminateContractModal`, `CLinkedDocumentsModal` (L606-614).
- Backend : `GET /contracts` **sans aucun filtre** (`api/contracts.rs` L17-23).

## 2. Ce qui manque (périmètre)

1. Filtres **type** et **prestataire** (selects), combinables avec statut et recherche.
2. Filtrage **côté backend** (query params) au lieu du tout-client.
3. Une **fiche contrat** (`ContractDetailView`, route dédiée) — aujourd'hui tout passe par des modales.

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

### 5.1 `db/contract.rs::list` — filtres

Ajouter des paramètres optionnels : `status: Option<&str>`, `contract_type: Option<&str>`, `vendor_name: Option<&str>` (égalité exacte sur `vendor_name` — la liste des valeurs vient de `SELECT DISTINCT`). SQL : clauses `AND ($n::text IS NULL OR ...)`.

Nouvelle fn `vendor_names(pool, building_id) -> Vec<String>` : `SELECT DISTINCT vendor_name ... ORDER BY vendor_name` (alimente le select frontend).

### 5.2 `api/contracts.rs` + `api/mod.rs`

```rust
#[derive(Deserialize)]
pub struct ListContractsQuery {
    pub status: Option<String>,        // draft|active|terminated|expired
    pub contract_type: Option<String>,
    pub vendor: Option<String>,
}
```

- `list` : `Query<ListContractsQuery>` passé tel quel au repo (zéro logique en handler). Valeurs invalides d'enum → laisser le cast SQL échouer proprement ou pré-valider avec `AppError::validation_code("errors.contracts.invalid_filter")` — retenu : pré-validation dans le repo.
- Nouvelle route `GET /contracts/vendors` → `vendor_names` (rôles : mêmes que `list` — manager + board_member).
- **Aucune nouvelle route détail nécessaire** : `GET /contracts/{id}` existe (sinon, l'ajouter sur le modèle de `get` du repo).

---

## 6. Frontend Vue

### 6.1 Filtres — `ContractsView.vue`

- Conserver les onglets de statut ; ajouter une rangée de filtres : select **Type** (valeurs de l'enum `contract_type`, libellés FR existants) + select **Prestataire** (`GET /contracts/vendors`) + recherche existante.
- Basculer la liste sur le filtrage **backend** : les changements d'onglet/filtre relancent `contractsApi.list({ status, contract_type, vendor })`. La recherche plein texte reste côté client (comportement actuel conservé).
- Les compteurs d'onglets : conserver la dérivation actuelle sur la liste non filtrée par statut (un seul appel `list` sans `status` + filtres type/vendor, onglets filtrant côté client) — **option la plus simple, retenue** : le backend filtre type/vendor, le statut reste client comme aujourd'hui. `status` en query param reste disponible pour d'autres consommateurs.

### 6.2 Nouvelle vue `ContractDetailView.vue`

- Route `/contracts/:id` (pattern `ContractorDetailView`). La carte de la liste devient cliquable → navigation.
- Sections :
  1. **En-tête** : vendor, subject, badge statut, type, montant, dates, préavis, actions existantes (Modifier, Avenant, Résilier — réutiliser les modales existantes).
  2. **Chaîne d'avenants** : réutiliser le fetch `contractsApi.amendmentChain` ; affichage inline (timeline) — la modale `AmendmentChainModal` reste pour la liste.
  3. **Documents liés** : réutiliser les endpoints `contract_document` (liste + rattacher/détacher, composants de `CLinkedDocumentsModal` réutilisés inline).
  4. **Rappels** : liste des `contract_reminder` du contrat (si un endpoint expose les rappels par contrat ; sinon l'ajouter : `GET /contracts/{id}/reminders`, repo trivial).
- Composants : réutiliser le catalogue (`CCard`, `CBadge`, `CTabs`…). **Si un nouveau composant `C*` est créé, le cataloguer dans `docs/DESIGN.md` §3 avant usage** (étape d'implémentation).

### 6.3 `frontend/src/api/index.ts`

`contractsApi.list(params?)`, `contractsApi.vendors()`, `contractsApi.reminders(id)` (si section 4 retenue).

---

## 7. Cas limites

| Cas | Règle |
|---|---|
| Filtres combinés (type + vendor + onglet + recherche) | Intersection ; état vide avec message dédié. |
| `vendor` inexistant en query | Liste vide, `200`. |
| Contrat d'un autre immeuble | `GET /contracts/{id}` → `404` (scoping `building_id` existant). |
| Contrat `draft` (EP-018-S05) | Visible dans la fiche avec bandeau « brouillon — à activer » et bouton Activer. |
| Prestataires en doublon typographique (`vendor_name` libre) | Assumé — champ libre, pas de normalisation dans cette story. |
| Deep-link `/contracts/:id` supprimé | `404` API → page d'erreur/redirect liste. |

---

## 8. Critères d'acceptation

1. `GET /contracts?contract_type=X&vendor=Y` filtre côté serveur ; `GET /contracts/vendors` liste les prestataires distincts.
2. La liste combine onglet statut + type + prestataire + recherche.
3. Cliquer une carte ouvre `/contracts/:id` : en-tête complet, chaîne d'avenants, documents liés, rappels.
4. Les actions (modifier/avenant/résilier/documents) fonctionnent depuis la fiche.
5. L'écart « cartes vs vue tabulaire » est acté : PRD amendé ou toggle table implémenté (décision §2).

---

## 9. Plan de tests

- **e2e backend** :
  1. `GET /contracts?contract_type=...` → seuls les contrats du type.
  2. `GET /contracts?vendor=...` → seuls ceux du prestataire.
  3. Combinaison des deux → intersection.
  4. `GET /contracts/vendors` → distincts, triés.
  5. Filtre invalide → `400 errors.contracts.invalid_filter`.
- **Frontend** : build + lint ; si vitest configuré, test de `ContractDetailView` (fetch au montage, sections rendues) ; sinon vérification manuelle.

---

## 10. Ordre d'implémentation

1. **Trancher la décision §2 (cartes vs tabulaire) avec le product owner** ; amender le PRD le cas échéant.
2. Backend : filtres `db/contract.rs::list`, `vendor_names`, (`reminders` par contrat si retenu), query params + route `GET /contracts/vendors` → `clippy`, e2e.
3. Frontend api + types.
4. `ContractsView.vue` : rangée de filtres, navigation carte → fiche.
5. `ContractDetailView.vue` + route (cataloguer tout nouveau `C*` dans `docs/DESIGN.md` §3 avant usage).
6. `npm run build` + `lint` ; `docs/PROGRESS.md`.

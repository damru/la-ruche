# Spécification technique — EP-005-S06

**Story :** Typer et verrouiller les documents fondateurs (Règlement de copropriété, État Descriptif de Division) avec version courante explicite et historique consultable.

**Épic :** EP-005 — Documents
**Réf. LEGAL :** §1.2 (documents fondateurs, conservation permanente) · **Réf. PRD :** Documents (US-DOC-*)
**Stories liées :** EP-005-S07 (taxonomie complète, s'appuie sur ce modèle), EP-005-S08 (soft-delete, réutilise la chaîne `supersedes_id`)

---

## 1. Quoi (résumé fonctionnel)

Aujourd'hui, tous les documents sont indifférenciés : pas de catégorie, pas de version, suppression physique possible sur n'importe quel document. Or le règlement de copropriété et l'EDD sont des actes fondateurs : ils doivent être **typés**, avoir une **version courante unique** explicite, une **chaîne de versions** consultable (un modificatif publié remplace la version précédente sans l'effacer), et être **verrouillés** contre la suppression physique.

Cible :
- Colonne `category` sur `document` (types fondateurs `reglement_copropriete` / `edd` + valeur générique `autre` par défaut ; la taxonomie complète arrive en S07).
- Chaîne de versions via `supersedes_id` (nouvelle version pointe l'ancienne) + flag `is_current` (au plus une version courante par type fondateur et par immeuble).
- Interdiction du hard delete sur un document fondateur (guard `domain/immutability.rs` + refus API).
- UI : sélection de la catégorie à l'upload, badge « version courante », historique des versions.

---

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| Table `document` (pas de category/version/supersedes) | `backend/migrations/0001_init.sql` L286-301 |
| Repo documents (`DocumentRow`, `list`, `get`, `create`, `delete`, `delete_orphaned_tx`) | `backend/src/db/document.rs` |
| Hard delete DB + S3 | `backend/src/db/document.rs` L172-206 · `backend/src/api/documents.rs` L206-224 |
| `confirm_upload` (POST /documents) | `backend/src/api/documents.rs` L122-167 |
| Guards immutabilité (précédent) | `backend/src/domain/immutability.rs` |
| API client frontend | `frontend/src/api/index.ts` (`documentsApi`) |

---

## 3. Modèle de données

### Migration `backend/migrations/NNNN_document_category.sql`
(le numéro exact sera fixé au moment de l'implémentation)

```sql
-- EP-005-S06: founding document types + version chain (LEGAL §1.2)

CREATE TYPE document_category AS ENUM ('reglement_copropriete', 'edd', 'autre');

ALTER TABLE document
    ADD COLUMN category document_category NOT NULL DEFAULT 'autre',
    ADD COLUMN supersedes_id UUID REFERENCES document(id),
    ADD COLUMN is_current BOOLEAN NOT NULL DEFAULT TRUE;

-- Linear version chain: a version has at most one successor.
CREATE UNIQUE INDEX document_supersedes_unique
    ON document (supersedes_id) WHERE supersedes_id IS NOT NULL;

-- At most one current version per founding category per building.
CREATE UNIQUE INDEX document_current_founding_unique
    ON document (building_id, category)
    WHERE is_current AND category IN ('reglement_copropriete', 'edd');
```

Notes :
- Enum PostgreSQL : S07 ajoutera ses valeurs via `ALTER TYPE ... ADD VALUE` (migration séparée, précédent `contract_status`/`draft`).
- `is_current` reste `TRUE` pour les documents `autre` (sans effet — l'index partiel ne les couvre pas).
- Nouvelle sémantique de table → mettre à jour `docs/ARCHITECTURE.md` **avant** la migration.
- Après migration : `cargo sqlx migrate run` puis `cargo sqlx prepare` (commit `backend/.sqlx/`).

---

## 4. Backend Rust

### 4.1 `domain/immutability.rs`

Ajouter une fonction dédiée (le guard existant est piloté par statut ; ici c'est la catégorie) :

```rust
/// Founding documents (règlement de copropriété, EDD) are permanent legal
/// acts (LEGAL §1.2): hard delete is forbidden, replacement only via a new
/// version (supersedes_id chain).
pub(crate) fn is_document_deletable(category: &str) -> bool {
    !matches!(category, "reglement_copropriete" | "edd")
}
```

### 4.2 `db/document.rs`

- `DocumentRow` / `DocumentListRow` : ajouter `category: String`, `supersedes_id: Option<Uuid>`, `is_current: bool` ; propager dans les `SELECT`.
- `create` : accepter `category: &str` (défaut `autre` côté handler si absent) et, optionnel, `supersedes_id`.
- Nouvelle fn `create_version` (transaction) : vérifie que le document remplacé a la même `category` et `is_current = true`, insère le nouveau document avec `supersedes_id = ancien.id`, `is_current = true`, et passe l'ancien à `is_current = false`. Erreurs : `errors.documents.not_current_version`, `errors.documents.category_mismatch`.
- Nouvelle fn `version_chain(pool, document_id, building_id)` : CTE récursive ascendante/descendante sur `supersedes_id` (modèle exact : `db/contract.rs::amendment_chain`, spec EP-018-S05 §4.2f), ordonnée `created_at`.
- `delete` : refuser si `category` fondatrice → `AppError::validation_code("errors.documents.founding_locked")` ; refuser aussi si le document a un successeur (préserve la chaîne).
- `delete_orphaned_tx` (L208-253) : exclure les catégories fondatrices (`AND d.category NOT IN ('reglement_copropriete','edd')`).

### 4.3 `api/documents.rs` + `api/mod.rs`

- `ConfirmUploadInput` : champ optionnel `category` (validé contre l'enum) + optionnel `supersedes_id`. Si `supersedes_id` fourni → routage vers `create_version`.
- `delete` : le refus vient du repo (i18n), rien à changer d'autre.
- Nouvelle route `GET /documents/{id}/versions` → handler `version_chain` (rôles : mêmes règles de visibilité que `list`).
- `DocumentListItem` : exposer `category`, `is_current`, `supersedes_id`.

Aucune logique en handler : checks catégorie/chaîne dans `db/` + `domain/`.

---

## 5. Frontend Vue

1. `frontend/src/types/domain.ts` : `DocumentCategory = 'reglement_copropriete' | 'edd' | 'autre'` ; étendre le type `Document` (`category`, `is_current`, `supersedes_id`).
2. `frontend/src/api/index.ts` : `category`/`supersedes_id` dans le payload de confirmation d'upload ; `documentsApi.versions(id)`.
3. Vue Documents : select de catégorie dans le formulaire d'upload (défaut « Autre ») ; badge « Version courante » ; action « Publier une nouvelle version » sur un fondateur courant (upload pré-câblé avec `supersedes_id`) ; action « Historique des versions » ouvrant une nouvelle modale `CDocumentVersionsModal` (timeline : date, nom, version courante en tête).
4. **Cataloguer `CDocumentVersionsModal` dans `docs/DESIGN.md` §3 avant usage** (étape d'implémentation — ne pas éditer DESIGN.md dans le cadre de cette spec).

---

## 6. Cas limites

| Cas | Règle |
|---|---|
| Deux uploads « règlement » sans chaînage | Le 2ᵉ upload `is_current` viole l'index partiel → intercepter et retourner `errors.documents.current_already_exists` (l'utilisateur doit passer par « nouvelle version »). |
| `supersedes_id` vers un document non courant | `400 errors.documents.not_current_version`. |
| `supersedes_id` vers une autre catégorie | `400 errors.documents.category_mismatch`. |
| DELETE sur un fondateur (courant ou ancien) | `400 errors.documents.founding_locked` — l'historique est permanent. |
| DELETE sur un document `autre` ayant un successeur | Refusé (chaîne préservée). |
| Documents existants en base | Backfill implicite : `category='autre'`, `is_current=true`, aucune chaîne. |
| Suppression en cascade (`delete_orphaned_tx`) | Ne touche jamais un fondateur. |

---

## 7. Critères d'acceptation

1. Un upload peut être typé `reglement_copropriete`, `edd` ou `autre`.
2. Il ne peut exister qu'une seule version courante par type fondateur et par immeuble.
3. Publier une nouvelle version bascule atomiquement l'ancienne en non-courante et chaîne `supersedes_id`.
4. `GET /documents/{id}/versions` retourne la chaîne complète, identique quel que soit le maillon interrogé.
5. `DELETE` sur un document fondateur échoue avec un code i18n ; le fichier S3 n'est pas touché.
6. L'UI affiche catégorie, badge courant, et l'historique.

---

## 8. Plan de tests

- **Unitaires** (`domain/immutability.rs`) : `is_document_deletable` — fondateurs → false, `autre` → true.
- **e2e** (`backend/tests/api_e2e_test.rs`) :
  1. Upload typé règlement → `201`, `is_current=true`.
  2. 2ᵉ règlement sans `supersedes_id` → `400 current_already_exists`.
  3. Nouvelle version → ancienne `is_current=false`, nouvelle courante, chaîne à 2 maillons.
  4. `supersedes_id` vers version non courante → `400`.
  5. `DELETE` fondateur → `400 founding_locked` ; `DELETE` `autre` sans successeur → `204`.
  6. `GET /versions` sur maillon 1 et maillon N → même tableau.

---

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` (colonnes `document`) → migration `NNNN` → `cargo sqlx migrate run` + `prepare`.
2. `domain/immutability.rs` (`is_document_deletable` + tests) → `cargo test --lib`.
3. `db/document.rs` (rows, `create`, `create_version`, `version_chain`, guards delete) → `cargo build`.
4. `api/documents.rs` + route `GET /documents/{id}/versions` dans `api/mod.rs` → `clippy -D warnings`.
5. Tests e2e → vert.
6. Frontend : types, api, catalogage `CDocumentVersionsModal` dans `docs/DESIGN.md` §3, UI.
7. `docs/PROGRESS.md` (entrée datée EP-005-S06).

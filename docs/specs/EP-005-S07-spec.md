# Spécification technique — EP-005-S07

**Story :** Catégoriser et archiver les pièces réglementaires techniques (DPE, DTG, PPT, PV d'AG…) avec règles de conservation par catégorie.

**Épic :** EP-005 — Documents
**Réf. LEGAL :** §3.3 (diagnostics techniques), §6.1-6.3 (conservation ; PV d'AG = conservation illimitée) · **Réf. PRD :** Documents (US-DOC-*)
**Stories liées :** **EP-005-S06 (prérequis — modèle `document_category`)**, EP-005-S08 (soft-delete respecte la rétention), EP-005-S09 (le job de purge consomme les règles de rétention définies ici)

---

## 1. Quoi (résumé fonctionnel)

S06 introduit `document.category` avec les seuls types fondateurs. Cette story :
- étend la taxonomie aux pièces réglementaires techniques : `dpe`, `dtg`, `ppt`, `pv_ag`, `carnet_entretien`, plus les valeurs existantes ;
- associe à chaque catégorie une **règle de conservation** (durée minimale légale, ou illimitée) portée par le code (`domain/retention.rs`), pas par la base ;
- ajoute un flag d'**archivage** (`archived_at`) : un document archivé sort des listes par défaut mais reste consultable ;
- UI : filtre par catégorie + filtre « archivés », action Archiver/Désarchiver.

La rétention n'est **pas appliquée** ici (aucune purge) : elle est *déclarée* ici et *consommée* par S08 (blocage de purge S3) et S09 (job de nettoyage).

---

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| Table `document` | `backend/migrations/0001_init.sql` L286-301 (+ migration S06 : `category`, `supersedes_id`, `is_current`) |
| Enum `document_category` (créé en S06) | migration S06 |
| Repo / API documents | `backend/src/db/document.rs` · `backend/src/api/documents.rs` |
| Précédent enum étendu en 2 migrations (contrainte PG) | `0030_levy_call_draft.sql` + `0033_levy_call_draft_status.sql` |

---

## 3. Modèle de données

PostgreSQL interdit d'utiliser une valeur d'enum ajoutée dans la même transaction : **deux migrations** (précédent `contract_status`/`draft`).

### Migration `backend/migrations/NNNN_document_category_values.sql` (partie 1)

```sql
-- EP-005-S07: regulatory/technical document categories (LEGAL §3.3, §6.1-6.3)
ALTER TYPE document_category ADD VALUE IF NOT EXISTS 'dpe' BEFORE 'autre';
ALTER TYPE document_category ADD VALUE IF NOT EXISTS 'dtg' BEFORE 'autre';
ALTER TYPE document_category ADD VALUE IF NOT EXISTS 'ppt' BEFORE 'autre';
ALTER TYPE document_category ADD VALUE IF NOT EXISTS 'pv_ag' BEFORE 'autre';
ALTER TYPE document_category ADD VALUE IF NOT EXISTS 'carnet_entretien' BEFORE 'autre';
```

### Migration `backend/migrations/NNNN_document_archive.sql` (partie 2)

```sql
-- EP-005-S07: archive flag. Archived documents leave default listings but
-- remain readable/downloadable. Retention rules live in domain/retention.rs.
ALTER TABLE document ADD COLUMN archived_at TIMESTAMPTZ;
CREATE INDEX document_archived_idx ON document (building_id) WHERE archived_at IS NOT NULL;
```

Mettre à jour `docs/ARCHITECTURE.md` avant migration ; `cargo sqlx prepare` après.

---

## 4. Backend Rust

### 4.1 Nouveau module `backend/src/domain/retention.rs`

Source unique des règles de conservation (consommé par S08/S09) :

```rust
/// Legal minimum retention per document category (LEGAL §6.1-6.3).
/// `None` = unlimited retention: the document may never be purged.
pub(crate) fn retention_months(category: &str) -> Option<u32> {
    match category {
        // Founding acts and AG minutes: unlimited (LEGAL §1.2, §6.1)
        "reglement_copropriete" | "edd" | "pv_ag" | "carnet_entretien" => None,
        // Technical diagnostics: keep 10 years (LEGAL §3.3, §6.2)
        "dpe" | "dtg" | "ppt" => Some(120),
        // Generic documents: 10 years (accounting-grade default, LEGAL §6.3)
        _ => Some(120),
    }
}
```

> Les durées exactes sont à confirmer contre `docs/LEGAL.md` §6.1-6.3 au moment de l'implémentation ; la structure (map catégorie → `Option<mois>`) est le livrable de la story.

Déclarer dans `domain/mod.rs`.

### 4.2 `db/document.rs`

- `DocumentRow` / `DocumentListRow` : ajouter `archived_at: Option<DateTime<Utc>>`.
- `list` : nouveau paramètre `include_archived: bool` (défaut `false` → `AND d.archived_at IS NULL`) et filtre optionnel `category: Option<&str>`.
- Nouvelles fns `archive(pool, id, building_id)` / `unarchive(...)` : `UPDATE document SET archived_at = NOW() / NULL`. Archiver un fondateur **courant** est refusé (`errors.documents.current_cannot_be_archived`) — seules les versions remplacées ou les pièces techniques s'archivent.

### 4.3 `api/documents.rs` + `api/mod.rs`

- `GET /documents` : query params `?category=` et `?archived=true` (extraction `Query<...>` — aucune logique en handler, filtres passés au repo).
- Nouvelles routes `POST /documents/{id}/archive` et `POST /documents/{id}/unarchive` (rôle `manager`).
- `DocumentListItem` : exposer `archived_at`.

---

## 5. Frontend Vue

1. Types : étendre `DocumentCategory` (dpe, dtg, ppt, pv_ag, carnet_entretien) ; `archived_at` sur `Document`.
2. `frontend/src/api/index.ts` : params `category`/`archived` sur la liste ; `archive`/`unarchive`.
3. Vue Documents : select de filtre par catégorie + toggle « Voir les archivés » ; badge catégorie (libellés FR : « DPE », « DTG », « Plan pluriannuel de travaux », « PV d'assemblée générale », « Carnet d'entretien ») ; actions Archiver/Désarchiver (menu du document, rôle manager).
4. Pas de nouveau composant attendu (réutiliser les catalogues existants) ; si un composant `C*` s'avère nécessaire, le cataloguer dans `docs/DESIGN.md` §3 avant usage.

---

## 6. Cas limites

| Cas | Règle |
|---|---|
| Archiver un fondateur courant | `400 errors.documents.current_cannot_be_archived`. |
| Archiver une version remplacée (`is_current=false`) | Autorisé — c'est le cas d'usage principal. |
| Archiver un document déjà archivé | Idempotent (`archived_at` réécrit) ou no-op — choisir no-op, `200`. |
| Document archivé et lié (contrat, rapport…) | Les vues liées continuent de l'afficher ; seul le listing général le masque par défaut. |
| PV d'AG | `retention_months = None` : jamais purgeable, quelle que soit son ancienneté (LEGAL §6.1). |
| Catégorie modifiable après upload ? | Non dans cette story — la catégorie est fixée à l'upload (recatégorisation = story future si besoin). |

---

## 7. Critères d'acceptation

1. Un upload peut être typé dans toute la taxonomie (fondateurs + techniques + `autre`).
2. `domain/retention.rs` expose la règle de conservation de chaque catégorie ; PV d'AG et fondateurs = illimité.
3. Archiver retire le document du listing par défaut ; `?archived=true` le réaffiche ; téléchargement toujours possible.
4. Le filtre par catégorie fonctionne côté API (pas seulement côté client).
5. Aucun document n'est supprimé ni purgé par cette story.

---

## 8. Plan de tests

- **Unitaires** : `retention_months` — chaque catégorie, `None` pour pv_ag/fondateurs/carnet.
- **e2e** :
  1. Upload `dpe` → `201`, listé avec `category="dpe"`.
  2. `GET /documents?category=dpe` → seulement les DPE.
  3. `POST /archive` → absent du listing par défaut, présent avec `?archived=true`, download OK.
  4. `POST /archive` sur fondateur courant → `400`.
  5. `POST /unarchive` → réapparaît.

---

## 9. Ordre d'implémentation

1. Vérifier les durées légales dans `docs/LEGAL.md` §6.1-6.3 ; `docs/ARCHITECTURE.md` (colonne `archived_at`).
2. Migrations enum (partie 1) puis colonne (partie 2) → `sqlx migrate run` + `prepare`.
3. `domain/retention.rs` + tests → `cargo test --lib`.
4. `db/document.rs` (filtres, archive/unarchive) → `api/documents.rs` + routes → `clippy`.
5. Tests e2e → vert.
6. Frontend (types, api, filtres, actions).
7. `docs/PROGRESS.md`.

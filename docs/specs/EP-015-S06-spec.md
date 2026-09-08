# Spécification technique — EP-015-S06

**Story :** Attestation d'assurance (RC pro / décennale) des prestataires : rattachement documentaire avec période de validité, et alerte quand l'attestation est absente ou expirée à la date d'intervention.

**Épic :** EP-015 — Prestataires & interventions
**Réf. PRD :** US-RP-004
**Stories liées :** EP-005 (module Documents — pattern de rattachement identique à `contract_document`)

---

## 1. Quoi (résumé fonctionnel)

Aujourd'hui, rien : la table `contractor` (`0001_init.sql` L519-529) n'a aucune colonne documentaire, aucune table `contractor_document` n'existe, `api/contractors.rs` n'expose aucune route document, et `ContractorDetailView.vue` ne gère aucune attestation.

Cible :
- Table de lien `contractor_document` sur le modèle de `contract_document`, enrichie de **dates de validité** portées par le lien (`valid_from` / `valid_until`) et d'un type d'attestation.
- Rattachement via le module Documents existant (upload présigné + confirm, puis link) — pattern EP-005/EP-006.
- **Alerte non bloquante** à l'assignation d'un prestataire sur une tâche : si aucune attestation n'est valide à la date d'intervention prévue, l'UI affiche un avertissement (l'assignation reste possible — le syndic bénévole reste juge).

---

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| Table `contractor` (aucune colonne doc) | `backend/migrations/0001_init.sql` L519-529 |
| Modèle de lien à répliquer | `contract_document` (`0001_init.sql` L495-499) + endpoints `api/contracts.rs` L168-213 + repo `db/contract.rs` L741/L762 |
| API prestataires (pas de routes doc) | `backend/src/api/contractors.rs` |
| Fiche prestataire | `frontend/src/features/contractors/ContractorDetailView.vue` |
| Tâches : assignation prestataire + date | `backend/src/db/task.rs` (`contractor_id` ajouté par migration 0007) ; formulaire tâche côté frontend |
| Nettoyage doc orphelins (à étendre) | `db/document.rs::delete_orphaned_tx` L208-253 (8 tables de liens) |

---

## 3. Modèle de données

### Migration `backend/migrations/NNNN_contractor_document.sql`

```sql
-- EP-015-S06: contractor insurance certificates (PRD US-RP-004).
-- Validity lives on the LINK (not on document): the same PDF can cover a
-- period; renewals are new documents with new links.

CREATE TYPE contractor_document_kind AS ENUM ('attestation_rc', 'attestation_decennale', 'autre');

CREATE TABLE contractor_document (
    contractor_id  UUID NOT NULL REFERENCES contractor(id) ON DELETE CASCADE,
    document_id    UUID NOT NULL REFERENCES document(id),
    kind           contractor_document_kind NOT NULL DEFAULT 'autre',
    valid_from     DATE,
    valid_until    DATE,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (contractor_id, document_id),
    CHECK (valid_until IS NULL OR valid_from IS NULL OR valid_until >= valid_from)
);

CREATE INDEX contractor_document_contractor_idx ON contractor_document (contractor_id);
```

- Nouvelle table → `docs/ARCHITECTURE.md` **avant** migration. Pas d'ADR (table de lien standard, pas d'alternative structurelle).
- **`delete_orphaned_tx`** (`db/document.rs` L208-253) : ajouter un 9ᵉ `NOT EXISTS` sur `contractor_document` — sinon un document d'attestation devient supprimable en cascade alors qu'il est lié.
- `cargo sqlx prepare` après migration.

---

## 4. Backend Rust

### 4.1 `domain/` — règle de validité

Nouvelle fn pure (dans `domain/contractor.rs` ou module existant des prestataires) :

```rust
/// A contractor is covered at `date` if at least one insurance link
/// (attestation_rc or attestation_decennale) has valid_from <= date <= valid_until
/// (open bounds count as covering). Returns the reason when not covered.
pub(crate) fn insurance_status(links: &[InsuranceLink], date: NaiveDate) -> InsuranceStatus
// InsuranceStatus = Covered | Missing | Expired { latest_valid_until: NaiveDate }
```

### 4.2 `db/contractor.rs`

Sur le modèle de `db/contract.rs` L741/L762 :
- `link_document(pool, contractor_id, building_id, document_id, kind, valid_from, valid_until)` — vérifie que le document appartient à l'immeuble.
- `unlink_document(...)`.
- `list_documents(pool, contractor_id, building_id)` — join `document` (nom, mime, taille) + champs du lien.
- `insurance_links(pool, contractor_id, building_id)` — les liens `kind IN ('attestation_rc','attestation_decennale')` non soft-deleted (si EP-005-S08 livré : `AND d.deleted_at IS NULL`).

### 4.3 `api/contractors.rs` + `api/mod.rs`

Routes (rôle écriture `manager`, lecture `manager` — aligner sur les routes contractor existantes) :

```
POST   /contractors/{id}/documents            body {document_id, kind, valid_from?, valid_until?}
GET    /contractors/{id}/documents
DELETE /contractors/{id}/documents/{document_id}
GET    /contractors/{id}/insurance-status?date=YYYY-MM-DD
```

- `insurance-status` : handler mince → `insurance_links` + `domain::insurance_status` → `{ status: "covered"|"missing"|"expired", latest_valid_until? }`. `date` par défaut = aujourd'hui.
- Erreurs : `errors.contractors.document_not_found`, `errors.contractors.invalid_validity_range` (i18n via `AppError::validation_code`).

### 4.4 Alerte à l'assignation de tâche

Pas de blocage ni de logique dans le handler tâche : le **frontend** interroge `insurance-status` à la date d'intervention lors du choix du prestataire dans le formulaire de tâche. (Alternative rejetée : champ warning dans la réponse de création de tâche — couple inutilement les modules.)

---

## 5. Frontend Vue

1. Types : `ContractorDocumentKind`, `ContractorDocumentLink { document, kind, valid_from, valid_until }`, `InsuranceStatus`.
2. `frontend/src/api/index.ts` : `contractorsApi.linkDocument`, `listDocuments`, `unlinkDocument`, `insuranceStatus(id, date?)`.
3. `ContractorDetailView.vue` : nouvelle section « Attestations & documents » — upload (flux Documents existant : presigned URL → confirm → link avec kind + dates), liste avec badge de validité (valide / expire bientôt / expirée), détachement.
4. Formulaire de tâche (assignation prestataire) : au choix d'un prestataire + date planifiée, appel `insuranceStatus` ; si `missing`/`expired`, bandeau d'avertissement non bloquant (« Aucune attestation d'assurance valide au JJ/MM/AAAA »).
5. Composants : réutiliser le catalogue ; **tout nouveau composant `C*` (ex. `CInsuranceBadge`) est catalogué dans `docs/DESIGN.md` §3 avant usage** (étape d'implémentation).

---

## 6. Cas limites

| Cas | Règle |
|---|---|
| Attestation sans dates (`valid_from`/`valid_until` NULL) | Couvre toute date (bornes ouvertes) — l'utilisateur est responsable de la saisie. |
| Plusieurs attestations, périodes disjointes | Couvert si **au moins une** couvre la date. |
| Tâche sans date planifiée | Statut évalué à aujourd'hui. |
| `valid_until < valid_from` | `400 errors.contractors.invalid_validity_range` (doublé par le CHECK SQL). |
| Suppression du prestataire | `ON DELETE CASCADE` sur le lien ; le document suit le circuit orphelins (`delete_orphaned_tx` étendu, ou soft-delete si S08 livré). |
| Document d'un autre immeuble | Link → `404`. |
| Assignation malgré l'alerte | Toujours autorisée — avertissement, pas de blocage (US-RP-004). |
| Renouvellement | Nouveau document + nouveau lien ; l'ancien reste (historique). |

---

## 7. Critères d'acceptation

1. On peut rattacher à un prestataire un document typé attestation avec période de validité, le lister et le détacher.
2. `GET /contractors/{id}/insurance-status?date=` retourne covered/missing/expired correctement.
3. Le formulaire de tâche affiche un avertissement non bloquant quand le prestataire choisi n'est pas couvert à la date d'intervention.
4. `delete_orphaned_tx` connaît `contractor_document` (un document lié n'est jamais nettoyé en cascade).
5. La fiche prestataire affiche les attestations avec leur état de validité.

---

## 8. Plan de tests

- **Unitaires** (`domain`) : `insurance_status` — couvert, bornes ouvertes, expiré (avec `latest_valid_until`), aucune attestation, périodes multiples disjointes.
- **e2e** :
  1. Link avec kind + dates → `201` ; listing OK.
  2. Link doublon → conflit géré (PK) → `400`/idempotence choisie.
  3. `insurance-status` à une date couverte → `covered` ; hors période → `expired` ; sans lien → `missing`.
  4. Plage invalide → `400`.
  5. Unlink → `204`, listing vide.
  6. Suppression du parent (tâche/contrat) référençant le même document → non supprimé tant que le lien contractor existe.

---

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` (table `contractor_document`) → migration `NNNN` → `sqlx migrate run` + `prepare`.
2. `db/document.rs::delete_orphaned_tx` : 9ᵉ `NOT EXISTS` (boy-scout obligatoire, même commit que la migration).
3. `domain` : `insurance_status` + tests unitaires.
4. `db/contractor.rs` : link/unlink/list/insurance_links → `cargo build`.
5. `api/contractors.rs` + routes dans `api/mod.rs` → `clippy`, e2e.
6. Frontend : types, api, section fiche prestataire, avertissement formulaire tâche (cataloguer les nouveaux `C*` dans `docs/DESIGN.md` §3 avant usage).
7. `docs/PROGRESS.md`, `graphify update .`.

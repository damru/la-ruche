# Spécification technique — EP-002-S21

**Story :** Assembler le **dossier documentaire du pré-état daté** en un PDF unique : règlement de copropriété, EDD, fiche synthétique, carnet d'entretien, DTG, PV des AG des **trois dernières années**, joints à la synthèse financière — art. L721-2 II CCH (LEGIARTI000031110561) + art. 54 IV loi ALUR n°2014-366 (LEGAL.md §7.2)

**Épic :** EP-002 — Copropriétaires & lots
**Réf. LEGAL :** §7.2 — art. L721-2 II CCH (documents annexés au pré-état daté) + art. 54 IV loi ALUR n°2014-366 · §1.3 (fiche synthétique) · §6 (carnet d'entretien, DTG art. L731-1) · **ADR :** aucun requis a priori (assemblage de documents existants)
**Stories liées :** EP-002-S07 ⚠️ (pré-état daté, synthèse financière), EP-005 (module Documents/GED), EP-003 (AG + PV signés), EP-009 (carnet d'entretien), EP-009-S09 ✅ (template PDF serveur + `append_pdf_annex`)

**Statut actuel :** NON IMPLÉMENTÉ — le pré-état daté PDF ne contient que la synthèse financière (`pre_etat_date.typ`) ; aucun assemblage des documents L721-2.

---

## 1. Quoi (résumé fonctionnel)

Le pré-état daté remis avant le compromis est un **PDF unique** composé de :

1. **Page de garde / sommaire** — immeuble, lot concerné, vendeur, date d'édition, liste des annexes avec leur référence GED ;
2. **Synthèse financière L721-2** (déjà livrée, EP-002-S07) : charges des 2 derniers exercices, sommes dues, impayés, dette fournisseurs (EP-002-S23), fonds travaux, procédures en cours ;
3. **Annexes documentaires** (art. L721-2 II) : copie du **règlement de copropriété**, de l'**EDD**, **fiche synthétique**, **carnet d'entretien**, **DTG**, et les **PV signés des AG des trois dernières années**.

Les informations financières sont à jour de l'**AG d'approbation des comptes précédant la signature de la promesse** (art. L721-2 II dernier alinéa) : la date de la dernière AG d'approbation est rappelée en page de garde.

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| Synthèse financière pré-état daté | `backend/src/api/etat_date.rs` (`pre_etat_date`, `pre_etat_date_pdf`), `backend/src/domain/typst/pre_etat_date.typ` |
| Fusion de PDF (annexe ajoutée à un PDF généré) | `backend/src/domain/legal_documents.rs` (`append_pdf_annex`, utilisé pour le questionnaire CSN) |
| Documents GED (règlement, EDD, fiche synthétique, DTG, PV…) | EP-005 : `backend/src/api/documents.rs` (`list`, `get_download_url`, upload pré-signé), catégories de documents |
| PV d'AG signés | EP-003 : `backend/src/api/assemblies.rs` (minutes signées), registre des PV |
| Carnet d'entretien | EP-009 : `backend/src/api/logbook.rs` (+ export PDF EP-009-S09) |
| Dossier documentaire L721-2 assemblé | **inexistant** |

## 3. Modèle de données

**Aucune migration.** Le dossier est assemblé à la volée à partir des documents GED existants (référencés par `document_id`) et des PV d'AG. Une table de « pack pré-état daté » n'est pas justifiée (document reproductible, pas de valeur probante figée).

## 4. Backend

### 4.1 Domain (`backend/src/domain/legal_documents.rs`, extension)

- `select_l721_2_documents(building_id) -> Vec<DocumentRef>` — fonction pure de sélection : pour chaque rubrique L721-2 (règlement, EDD, fiche synthétique, carnet d'entretien, DTG), le document GED le plus récent de la catégorie correspondante ; PV d'AG = les 3 dernières AG clôturées avec minutes signées.
- `assemble_pre_etat_date_pack(...)` — produit la page de garde (sommaire + méta) + ordered list des annexes. Zéro SQL en domain.

### 4.2 DB

- Réutiliser `db/documents.rs` (list par catégorie/bâtiment) et le repo assemblées (3 dernières AG clôturées + minutes). Aucun SQL dupliqué.

### 4.3 API (`backend/src/api/etat_date.rs`, extension)

- `GET /units/{id}/pre-etat-date/export/pdf` — génère le PDF pack : page de garde + synthèse `pre_etat_date.typ` + annexes PDF téléchargées depuis le stockage (via `get_download_url` interne) et fusionnées avec `append_pdf_annex`.
- Document non PDF (ex. PV en markdown) → page de garde avec mention « voir GED » (jamais de conversion arbitraire) ; document absent → ligne « non disponible » dans le sommaire, section présente mais vide (structure fixe, comme S06).
- Erreurs i18n `errors.pre_etat_date.*`.

## 5. Frontend

- Bouton « Exporter le pré-état daté (PDF complet) » à côté de l'aperçu synthèse (fiche lot, feature `units`) — réutilise le pattern `preEtatDatePdf` / téléchargement blob d'EP-009-S09.
- Aperçu du sommaire (liste des annexes incluant manquantes) avant export. Composants catalogue existants ; tout nouveau `C*` → `docs/DESIGN.md §3` avant usage.

## 6. Cas limites

- Moins de 3 AG clôturées → PV des AG existantes uniquement, mention « 3 dernières années : N disponibles ».
- Document L721-2 absent de la GED → ligne « non disponible » dans le sommaire, export possible quand même.
- Document non PDF → référencé en sommaire (téléchargement GED), non fondu dans le PDF.
- Aucune AG d'approbation des comptes précédant la promesse → avertissement affiché (art. L721-2 II dernier alinéa).

## 7. Critères d'acceptation

1. `GET /units/{id}/pre-etat-date/export/pdf` retourne un PDF unique : page de garde + synthèse + annexes (règlement, EDD, fiche synthétique, carnet, DTG, PV 3 dernières AG) dans l'ordre L721-2.
2. Les documents manquants apparaissent explicitement « non disponible » dans le sommaire.
3. La date de la dernière AG d'approbation des comptes précédant la promesse est rappelée en page de garde.
4. Aucune migration ; réutilise GED (EP-005), assemblées (EP-003), `append_pdf_annex`.

## 8. Plan de tests

- **Domaine pur** : `select_l721_2_documents` — GED complète, GED partielle, moins de 3 AG.
- **E2E** : export PDF du pack → nombre de pages = garde + synthèse + annexes présentes ; bâtiment sans documents → sommaire avec « non disponible », 200 ; rôle non autorisé → 403.

## 9. Ordre d'implémentation

1. Domain : `select_l721_2_documents` + page de garde Typst + tests purs.
2. API : endpoint export pdf pack (téléchargement interne des documents + `append_pdf_annex`) + i18n.
3. Frontend : bouton export + sommaire. Catalogue `C*` si besoin.
4. Tests E2E, entrée `docs/PROGRESS.md`.

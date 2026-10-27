# Spécification technique — EP-009-S09

**Story :** Export PDF du carnet d'entretien (en-tête réglementaire, chronologie des interventions, contrats d'assurance et de maintenance actifs).

**Épic :** EP-009 — Carnet d'entretien
**Réf. PRD :** US-CE-004 · **Réf. LEGAL :** carnet d'entretien obligatoire (loi SRU, décret n°2001-477)
**État :** IMPLÉMENTÉ (backend + frontend) — génération PDF **100 % serveur** via le template documentaire partagé `domain/pdf_template.rs` + renderer `domain/logbook_pdf.rs`, endpoint `GET /logbook/export/pdf`, téléchargement blob depuis la vue carnet.

---

## 1. Décision de conception — génération PDF côté serveur + template partagé

**Changement de spec (remplace la décision initiale « HTML imprimable ») : pas de gestion du `@media print`. Toutes les générations de documents PDF passent par une génération backend, et tous les documents partagent le même template.**

**Retenu : génération serveur via `genpdf` + template documentaire réutilisable.**

- **Template commun** (`backend/src/domain/pdf_template.rs`) : identité visuelle unique pour tous les documents officiels —
  - **Introduction (1ʳᵉ page uniquement, « en-tête » au sens spec)** : bloc marque — **logo image embarquée** (`backend/assets/images/logo-la-ruche.png`, rasterisé depuis `frontend/src/components/atoms/CLogoMark.svg`, 7 hexagones dégradé #643fd5 → #7d5bf0, `include_bytes!`) + nom « La Ruche » + **tagline** (« Gestion de copropriété participative, pour les syndics bénévoles et les conseils syndicaux »), titre du document centré en couleur marque, identité de la copro (Immeuble, Adresse, Syndic en exercice, SIRET…), ligne « **Fait à {lieu}, le {date}** » + « **Pour le compte de {syndic}** », **paragraphe explicatif** du contenu du document, espacement généreux — puis seulement le contenu. **Pas d'en-tête récurrente** : les pages 2+ commencent directement par le contenu.
  - **Salutation** optionnelle (documents de type courrier).
  - **Conclusion** : **période de validité légale** optionnelle (quand le document en a une fondée légalement — ex. état daté : 3 mois), puis « Fait à {lieu}, le {date} » **rempli**, signé « **La Ruche** » (signature artificielle — l'app signe, pas d'image de signature).
  - **Pied de page récurrent (toutes les pages)** : filet + **logo miniature** (même image embarquée) + tagline + « Édité le {date} via La Ruche », en petite police grise (décorateur de page custom, hauteur réservée). genpdf n'expose pas de « page X / Y » natif — pas de numérotation de page.
  - **Police** : Liberation Sans embarquée (`include_bytes!`, sous-ensemble `pyftsubset` ASCII + Latin-1 + ponctuation typographique ~40 ko/fichier au lieu de ~410 ko), **palette** : `BRAND` #643fd5 (miroir de `--color-primary` du frontend), gris secondaire, encre. **Poids** : logo PNG RGB (~5 ko, FlateDecode ~4,6 ko/occurrence en release), polices ~160 ko embarquées au total (4 FontFile : regular/bold × 2 car italic = clone de regular) → PDF complet ~250 ko pour 3 pages.
- Chaque document a son renderer dédié dans `domain/` qui assemble les données et s'appuie sur le template (pas de duplication de styles). Réutilisations prévues : compte rendu d'AG, lettre de relance, etc.
- **Avantages** : PDF identique quel que soit le navigateur/OS de l'utilisateur, mise en page maîtrisée (A4, marges, tableaux paginés), réutilisable pour l'envoi automatique futur (e-mail), conforme à ADR-011 (génération serveur).
- **Inconvénient accepté** : la mise en page PDF vit côté backend (et non dans le design system Tailwind) — c'est le template `pdf_template.rs` qui en tient lieu.

**Décision initiale écartée :** vue HTML optimisée impression + `window.print()` — remplacée par le changement de spec ci-dessus (plus de `@media print` géré dans l'app).

## 2. État actuel du code

| Élément | Emplacement | Constat |
|---|---|---|
| Routes carnet | `backend/src/api/mod.rs` | list / summary / get / put / delete / correction + **`GET /logbook/export/pdf`** (S09) |
| Template PDF partagé | `backend/src/domain/pdf_template.rs` | **Nouveau (S09)** — en-tête, titre, salutation, signature, pied de page, polices, palette, formatteurs FR |
| Renderer carnet | `backend/src/domain/logbook_pdf.rs` | **Nouveau (S09)** — `render_logbook(&LogbookPdfData) -> Result<Vec<u8>, String>` |
| Handler export | `api/logbook.rs::export_pdf` | **Nouveau (S09)** — assemble identité (building + mandat syndic + contact), contrats actifs, entrées, rend le PDF |
| Client API carnet | `frontend/src/api/index.ts` (`maintenanceApi`) | **`exportPdf()`** (S09) — `GET /logbook/export/pdf`, réponse blob |
| Vue carnet | `frontend/src/features/maintenance/MaintenanceView.vue` | Bouton « Exporter le carnet (PDF) » → téléchargement blob, état de chargement + `CAlert` en cas d'erreur |
| Contrats actifs | `contractsApi` existant (`GET /contracts`) | Filtrés côté serveur dans `export_pdf` (`status == 'active'` + type assurance/maintenance) |

## 3. Modèle de données

Aucune migration (S10 a ajouté la catégorie ; S09 ne touche pas au schéma).

## 4. Backend

- **`domain/pdf_template.rs`** (pur, sans I/O ni SQL) : `DocumentIdentity`, `DocumentIntro` (titre, méta, lieu, date d'édition, « pour le compte de », paragraphe explicatif), `DocumentConclusion` (lieu/date remplis, période de validité légale optionnelle), `logo_element` (logo image embarquée `assets/images/logo-la-ruche.png`, rasterisé depuis `CLogoMark.svg`), `new_document` (décorateur de page `FooterDecorator` : pied de page récurrent logo + tagline + « Édité le … via La Ruche » en petite police grise, **pas d'en-tête récurrente**), `push_intro`, `push_salutation`, `section`, `body`, `muted`, `push_conclusion` (« Fait à …, le … » rempli, signé « La Ruche »), `finish_document`, formatteurs `format_euro` / `optional_euro` / `format_date` / `format_date_long`. Tests unitaires inclus.
- **`domain/logbook_pdf.rs`** (pur) : `render_logbook` — introduction (bloc marque logo + « La Ruche » + tagline, titre, métas immeuble/adresse/syndic/SIRET, « Fait à {ville}, le {date} » + « Pour le compte de {syndic} », paragraphe explicatif), sections contrats d'assurance et de maintenance actifs (compteur + lignes objet/prestataire/période/montant), tableau chronologie (date, intervention, prestataire, catégorie, source, coût — cellules aérées, lignes paginées proprement), conclusion (période de validité « Document établi à la date d'édition ; il annule et remplace toute édition antérieure », « Fait à {ville}, le {date} » rempli, signé « La Ruche »). Le pied de page récurrent est géré par le décorateur du template. Les libellés sont passés par le caller (renderer agnostique de la langue).
- **`api/logbook.rs::export_pdf`** : `GET /logbook/export/pdf` — tout membre authentifié de l'immeuble (mêmes droits que le summary). Assemble l'identité (`building::get` + `governance_mandate::current_syndic_name` + contact du gestionnaire), les contrats actifs (`is_contract_active` / `is_maintenance_contract`), les entrées (`maintenance_log::list`, tri `intervention_date ASC`), puis rend via `render_logbook`. Réponse `application/pdf`, en-tête `Content-Disposition: attachment; filename="carnet-entretien.pdf"`.
- **Note (hors périmètre)** : `domain/legal_documents.rs` (état daté / pré-état daté) duplique encore certains helpers (formatage, en-tête) — migration vers le template commun envisagée en suivi, sans toucher à un document probant dans ce ticket.

## 5. Frontend

### 5.1 `MaintenanceView.vue`

Bouton « Exporter le carnet (PDF) » dans la topbar : appelle `maintenanceApi.exportPdf()`, télécharge le blob (`carnet-entretien.pdf`) via `URL.createObjectURL` + lien temporaire. État de chargement sur le bouton (`:loading`), `CAlert` sous le header en cas d'erreur (`toErrorMessage`).

### 5.2 `frontend/src/api/index.ts`

`maintenanceApi.exportPdf: () => api.get<Blob>('/logbook/export/pdf', { responseType: 'blob' })` (pattern identique à `etatDateApi.exportPdf`).

### 5.3 Catalogue

Aucun nouveau composant frontend : le template documentaire vit côté backend (`domain/pdf_template.rs`). Règle DESIGN mise à jour : les documents PDF officiels sont générés serveur, jamais via `@media print`.

## 6. Cas limites

| Cas | Règle |
|---|---|
| Carnet vide | Le PDF est généré quand même : en-tête + « Aucune intervention enregistrée » |
| Aucun contrat actif | Bloc affiché avec mention « Aucun contrat en cours » (l'absence d'assurance est une information réglementaire en soi) |
| Volumétrie importante | Tableau multi-pages ; lignes non coupées au milieu (genpdf pagine les lignes), en-tête récurrent sur chaque page |
| Syndic non renseigné | Méta « Syndic en exercice : Syndic non renseigné », signature « Le syndic » conservée |
| Rôles | Même accès que le summary (tout membre authentifié de l'immeuble) ; aucune route frontend supprimée ne contourne de contrôle |
| Montants | Formatés FR (€, deux décimales) depuis `Decimal` (jamais `f64`) — `format_euro` du template |

## 7. Critères d'acceptation

1. Depuis la vue carnet, « Exporter le carnet (PDF) » télécharge un PDF complet (introduction avec logo + « La Ruche » + tagline + identité copro + « Fait à …, le … » + « Pour le compte de … » + paragraphe explicatif, contrats actifs, chronologie, conclusion « Fait à …, le … » remplie signée « La Ruche » + période de validité, pied de page récurrent logo + tagline + « Édité le … via La Ruche »).
2. Le PDF est identique quel que soit le navigateur (génération serveur, A4, tableaux paginés proprement).
3. Le template (`domain/pdf_template.rs`) est réutilisable tel quel pour les prochains documents (CR d'AG, lettre de relance).
4. Carnet vide et absence de contrats gérés sans erreur.
5. Aucune dépendance `@media print` / `window.print` pour les documents PDF.

## 8. Plan de tests

- **Backend** : tests unitaires `pdf_template.rs` + `logbook_pdf.rs` (formatteurs, rendu valide `%PDF`, carnet vide, sans syndic) ; tests e2e `backend/tests/logbook_pdf_test.rs` (200 + `application/pdf` + magic bytes `%PDF` pour les 4 rôles, 401 non authentifié, reflet d'une entrée seedée, carnet vide) — à exécuter en CI avec Postgres.
- **Frontend** : `npm run typecheck && npm run lint && npm run build`.
- Vérification visuelle du rendu (échantillon généré hors repo).

## 9. Ordre d'implémentation

1. Template partagé `domain/pdf_template.rs` + tests unitaires.
2. Renderer `domain/logbook_pdf.rs` + tests unitaires.
3. Handler `export_pdf` + route + tests e2e.
4. Frontend : `maintenanceApi.exportPdf` + bouton de téléchargement dans `MaintenanceView.vue`.
5. Docs (ARCHITECTURE, DESIGN, EPICS) + spec.

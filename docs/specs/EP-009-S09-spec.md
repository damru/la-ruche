# Spécification technique — EP-009-S09

**Story :** Export PDF du carnet d'entretien (lettre de transmission + annexe réglementaire : chronologie des interventions, contrats d'assurance et de maintenance actifs).

**Épic :** EP-009 — Carnet d'entretien
**Réf. PRD :** US-CE-004 · **Réf. LEGAL :** carnet d'entretien obligatoire (loi SRU, décret n°2001-477)
**État :** IMPLÉMENTÉ (backend + frontend) — génération PDF **100 % serveur** via le template documentaire partagé `domain/pdf_template.rs` (layout **v7.1** « lettre + annexe » : logo web exact + polices Fraunces/DM Sans de l'app, en-têtes émetteur/destinataire alignés, corps justifié, destinataire = demandeur) + renderer `domain/logbook_pdf.rs`, endpoint `GET /logbook/export/pdf`, téléchargement blob depuis la vue carnet.

---

## 1. Décision de conception — génération PDF côté serveur + template partagé

**Changement de spec (remplace la décision initiale « HTML imprimable ») : pas de gestion du `@media print`. Toutes les générations de documents PDF passent par une génération backend, et tous les documents partagent le même template.**

**Retenu : génération serveur via `genpdf` + template documentaire réutilisable, layout v7.1 « lettre + annexe ».**

- **Template commun** (`backend/src/domain/pdf_template.rs`) : identité visuelle unique pour tous les documents officiels —
  - **Double en-tête aligné (1ʳᵉ page uniquement, « en-tête » au sens spec)** : le bloc marque « app web » occupe sa propre ligne — **logo image embarquée** (`backend/assets/images/logo-la-ruche.png`, rasterisé depuis `frontend/src/components/atoms/CLogoMark.svg`, 7 hexagones dégradé #643fd5 → #7d5bf0, `include_bytes!`) + « La Ruche » en **Fraunces Bold 20 pt** `#191c1d` (`--on-surface`, police display `--font-display` du web) + tagline « **Gestion participative** » en **DM Sans 14 pt** `#42474e` (`--on-surface-variant`, police d'interface du web — i18n `auth.brandEyebrow`). Sous ce bloc, une **grille à deux colonnes** aligne l'**émetteur** à gauche (intitulé « Le syndic des copropriétaires » en gras, lignes syndic/contact/SIRET en gris) et le **destinataire** à droite (label gris gras + lignes alignées à droite) **sur la même ligne** (retour utilisateur v7.1). Le tout est fermé par un **filet violet** (couleur marque). **Pas d'en-tête récurrent** : les pages suivantes commencent directement par le contenu.
  - **Corps de lettre classique justifié** : **objet en gras** (taille 12) + ligne de **référence** grise (ex. « Réf. LR-CARNET-2026 · Édition du 27/10/2026 — destinataire : {nom} »), **salutation** optionnelle (« Madame, Monsieur, »), paragraphes de description du document **justifiés** (dont **l'annonce explicite de l'annexe**), formule de politesse. genpdf 0.2 n'a pas de justification native : un élément custom `JustifiedParagraph` étire toutes les lignes sauf la dernière du paragraphe sur la largeur utile (wrap greedy + répartition du reste sur les espaces inter-mots, pagination propre). Le contenu détaillé (tableaux) vit en annexe.
  - **Signature (fin de lettre)** : période de validité légale optionnelle (quand le document en a une fondée légalement — ex. état daté : 3 mois), puis bloc **aligné à droite** — « Pour le syndic, », espace de signature, **nom du signataire en gras** (le syndic en exercice ; repli « La Ruche »), **qualité** en petit gris (« Syndic en exercice — {immeuble} »), « **Fait à {lieu}, le {date}** » rempli (signature artificielle — pas d'image de signature).
  - **En-tête d'annexe** : kicker espacé (« A N N E X E »), titre du document en couleur marque, sous-titre (immeuble + adresse), méta-lignes (syndic, SIRET, édition), filet violet — puis le contenu tabulaire de l'annexe.
  - **Pied de page récurrent (toutes les pages)** : filet + **logo miniature** (même image embarquée) + « Gestion participative — La Ruche » en DM Sans à gauche + « Édité le {date} via La Ruche » aligné à droite, en petite police grise (décorateur de page custom, hauteur réservée). Le décorateur applique aussi les **marges de page** (haut 18 mm, côtés 18 mm) et réserve la zone de pied de page — le template n'utilise plus `SimplePageDecorator`. genpdf n'expose pas de « page X / Y » natif — pas de numérotation de page.
  - **Polices** : Tinos embarquée pour le corps (`include_bytes!`, serif métrique Times New Roman, sous-ensemble `pyftsubset` ASCII + Latin-1 + ponctuation typographique ~32 ko/fichier × 4 styles regular/bold/italic/bold-italic) ; **Fraunces-Bold.ttf** et **DMSans-Regular.ttf** (latin, `assets/fonts/`, téléchargées de fonts.gstatic.com) embarquées pour la marque — un seul slot réellement rendu par famille, les slots bold/italic sont mappés sur builtin Helvetica (jamais sélectionnés) pour ne pas tripler le poids. `new_document` enregistre les familles et retourne `(Document, BrandFonts)` ; dépendance directe `printpdf 0.3` (`default-features = false`) pour `BuiltinFont`. **Palette** : `BRAND` #643fd5 (miroir de `--color-primary` du frontend), `ON_SURFACE` #191c1d et `ON_SURFACE_VARIANT` #42474e (miroirs des tokens web), gris secondaire, encre.
- Chaque document a son renderer dédié dans `domain/` qui assemble les données et s'appuie sur le template (pas de duplication de styles). Réutilisations prévues : compte rendu d'AG, lettre de relance, etc.
- **Avantages** : PDF identique quel que soit le navigateur/OS de l'utilisateur, mise en page maîtrisée (A4, marges, tableaux paginés), réutilisable pour l'envoi automatique futur (e-mail), conforme à ADR-011 (génération serveur), forme épistolaire lisible : le lecteur voit d'abord qui émet, à qui, pourquoi — le détail réglementaire suit en annexe.
- **Inconvénient accepté** : la mise en page PDF vit côté backend (et non dans le design system Tailwind) — c'est le template `pdf_template.rs` qui en tient lieu.

**Décision initiale écartée :** vue HTML optimisée impression + `window.print()` — remplacée par le changement de spec ci-dessus (plus de `@media print` géré dans l'app).

## 2. État actuel du code

| Élément | Emplacement | Constat |
|---|---|---|
| Routes carnet | `backend/src/api/mod.rs` | list / summary / get / put / delete / correction + **`GET /logbook/export/pdf`** (S09) |
| Template PDF partagé | `backend/src/domain/pdf_template.rs` | **Nouveau (S09, v7.1)** — bloc marque web (logo + Fraunces/DM Sans), double en-tête aligné, objet, salutation, corps justifié, signature nommée, en-tête d'annexe, pied de page, polices, palette, formatteurs FR |
| Renderer carnet | `backend/src/domain/logbook_pdf.rs` | **Nouveau (S09)** — `render_logbook(&LogbookPdfData) -> Result<Vec<u8>, String>` (lettre + annexe) |
| Handler export | `api/logbook.rs::export_pdf` | **Nouveau (S09)** — assemble identité (building + mandat syndic + contact), contrats actifs, entrées, libellés de la lettre, rend le PDF |
| Client API carnet | `frontend/src/api/index.ts` (`maintenanceApi`) | **`exportPdf()`** (S09) — `GET /logbook/export/pdf`, réponse blob |
| Vue carnet | `frontend/src/features/maintenance/MaintenanceView.vue` | Bouton « Exporter le carnet (PDF) » → téléchargement blob, état de chargement + `CAlert` en cas d'erreur |
| Contrats actifs | `contractsApi` existant (`GET /contracts`) | Filtrés côté serveur dans `export_pdf` (`status == 'active'` + type assurance/maintenance) |

## 3. Modèle de données

Aucune migration (S10 a ajouté la catégorie ; S09 ne touche pas au schéma).

## 4. Backend

- **`domain/pdf_template.rs`** (pur, sans I/O ni SQL) : `DocumentIdentity`, `DocumentHeader` (double en-tête : bloc marque sur sa propre ligne, puis `issuer_lines` à gauche et `DocumentRecipient` à droite **alignés sur la même ligne**), `DocumentAnnexHeader` (kicker, titre, sous-titre, méta), `DocumentConclusion` (closing « Pour le syndic, », `signer_name`, `signer_role`, lieu/date remplis, période de validité légale optionnelle), `logo_element` (logo image embarquée `assets/images/logo-la-ruche.png`, rasterisé depuis `CLogoMark.svg`), `brand_font_family` / `ui_font_family` (Fraunces-Bold / DMSans-Regular embarquées, slots de repli builtin), `BrandFonts`, `new_document(edited_at, title) -> (Document, BrandFonts)` (décorateur de page `FooterDecorator` : marges, pied de page récurrent mini-logo + tagline DM Sans + « Édité le … via La Ruche » aligné à droite, **pas d'en-tête récurrent**), `push_letter_header(layout, &fonts, header)` (bloc marque web + filet violet + grille émetteur/destinataire), `push_objet` (objet gras + référence grise), `push_salutation`, `JustifiedParagraph` + `justified_body` (corps justifié custom — genpdf 0.2 n'a pas de justification native), `push_annex_header` (kicker espacé + titre marque + méta + filet violet), `section`, `body`, `muted`, `push_conclusion` (bloc signature aligné à droite : closing, espace, nom en gras, qualité, « Fait à …, le … » rempli), `brand_hairline` / `hairline` (filets vectoriels gris et couleur marque), `finish_document`, formatteurs `format_euro` / `optional_euro` / `format_date` / `format_date_long`. Tests unitaires inclus.
- **`domain/logbook_pdf.rs`** (pur) : `render_logbook` assemble — **page 1, la lettre** : double en-tête (bloc marque web + émetteur syndic/SIRET à gauche, **destinataire = l'utilisateur qui a demandé le document** à droite), objet + référence, salutation, paragraphes de description **justifiés** (annonce de l'annexe, valeur de l'édition, disponibilité), formule de politesse, signature alignée à droite (nom du syndic, qualité, « Fait à {ville}, le {date} ») ; **page 2+, l'annexe** : en-tête d'annexe (kicker « A N N E X E », titre, identité), sections contrats d'assurance et de maintenance actifs (compteur + lignes objet/prestataire/période/montant), tableau chronologie (date, intervention, prestataire, catégorie, source, coût — cellules aérées, lignes paginées proprement). Pas de signature en annexe : la lettre porte la forme juridique. Le pied de page récurrent est géré par le décorateur du template. Les libellés (dont les paragraphes de la lettre) sont passés par le caller (renderer agnostique de la langue).
- **`api/logbook.rs::export_pdf`** : `GET /logbook/export/pdf` — tout membre authentifié de l'immeuble (mêmes droits que le summary). Assemble l'identité (`building::get` + `governance_mandate::current_syndic_name` + contact du gestionnaire), le **destinataire** (`db::user::find_by_id(auth.user_id)` : « {prénom} {nom} — {rôle FR} » + immeuble + adresse, rôle libellé selon `auth.roles`), les contrats actifs (`is_contract_active` / `is_maintenance_contract`), les entrées (`maintenance_log::list`, tri `intervention_date ASC`), les libellés FR de la lettre (objet « édition {année} », référence « Réf. LR-CARNET-{année} · Édition du {date} — destinataire : {nom} », paragraphes avec annonce de l'annexe, formule de politesse), puis rend via `render_logbook`. Réponse `application/pdf`, en-tête `Content-Disposition: attachment; filename="carnet-entretien.pdf"`.
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
| Carnet vide | Le PDF est généré quand même : lettre complète + annexe « Aucune intervention enregistrée » |
| Aucun contrat actif | Bloc affiché avec mention « Aucun contrat en cours » (l'absence d'assurance est une information réglementaire en soi) |
| Volumétrie importante | Tableau multi-pages en annexe ; lignes non coupées au milieu (genpdf pagine les lignes), pied de page récurrent sur chaque page |
| Syndic non renseigné | Bloc émetteur « Syndic non renseigné », signature « La Ruche » sans qualité |
| Rôles | Même accès que le summary (tout membre authentifié de l'immeuble) ; aucune route frontend supprimée ne contourne de contrôle |
| Montants | Formatés FR (€, deux décimales) depuis `Decimal` (jamais `f64`) — `format_euro` du template |

## 7. Critères d'acceptation

1. Depuis la vue carnet, « Exporter le carnet (PDF) » télécharge un PDF complet : **page 1 = lettre** (bloc marque web — logo + « La Ruche » en Fraunces + « Gestion participative » en DM Sans —, émetteur et destinataire alignés sur la même ligne, objet en gras + référence, salutation, paragraphes **justifiés** avec annonce de l'annexe, formule de politesse, signature nommée alignée à droite avec « Fait à …, le … » rempli), **pages suivantes = annexe** (kicker « A N N E X E », titre, identité, contrats actifs, chronologie), **pied de page récurrent** (mini logo + tagline + « Édité le … via La Ruche »). Le **destinataire** affiché est l'utilisateur connecté qui a demandé la génération.
2. Le PDF est identique quel que soit le navigateur (génération serveur, A4, tableaux paginés proprement).
3. Le template (`domain/pdf_template.rs`) est réutilisable tel quel pour les prochains documents (CR d'AG, lettre de relance).
4. Carnet vide et absence de contrats gérés sans erreur.
5. Aucune dépendance `@media print` / `window.print` pour les documents PDF.

## 8. Plan de tests

- **Backend** : tests unitaires `pdf_template.rs` + `logbook_pdf.rs` (formatteurs, `spaced_caps`, rendu valide `%PDF` avec bloc marque Fraunces/DM Sans + double en-tête aligné + lettre justifiée + annexe, paragraphe justifié multi-pages, carnet vide, sans syndic) ; tests e2e `backend/tests/logbook_pdf_test.rs` (200 + `application/pdf` + magic bytes `%PDF` pour les 4 rôles, 401 non authentifié, reflet d'une entrée seedée, carnet vide) — à exécuter en CI avec Postgres.
- **Frontend** : `npm run typecheck && npm run lint && npm run build`.
- Vérification visuelle du rendu (échantillon `docs/samples/EP-009-S09-carnet-entretien-sample.svg`).

## 9. Ordre d'implémentation

1. Template partagé `domain/pdf_template.rs` (double en-tête, lettre, signature, annexe) + tests unitaires.
2. Renderer `domain/logbook_pdf.rs` (lettre + annexe) + tests unitaires.
3. Handler `export_pdf` + route + tests e2e.
4. Frontend : `maintenanceApi.exportPdf` + bouton de téléchargement dans `MaintenanceView.vue`.
5. Docs (ARCHITECTURE, DESIGN, EPICS) + spec.

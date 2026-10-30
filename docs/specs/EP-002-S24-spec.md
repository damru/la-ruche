# Spécification technique — EP-002-S24

**Story :** Refondre le **formulaire état daté pré-rempli au format La Ruche** (layout v7.1 « lettre + annexe », `base.typ`) en remplacement du clone CSN `questionnaire.typ` — contenu réglementaire identique (informations des parties, 3 parties, annexe actualisation, événements postérieurs, sommes exigibles + certificat art. 20, renseignements complémentaires A1–A12 / B1–B10), mise en page La Ruche (LEGAL.md §7.1, ADR-011, EP-009-S09)

**Épic :** EP-002 — Copropriétaires & lots
**Réf. LEGAL :** §7.1 (contenu art. 5 décret 67-223 v7.0 : 3 parties + annexe + certificat art. 20) · **Réf. design :** ADR-011 + spec EP-009-S09 (layout v7.1) + `backend/src/domain/typst/base.typ` · **ADR :** aucun requis (remplacement de template, même payload)
**Stories liées :** EP-002-S06 ⚠️ (état daté), EP-002-S20 🔲 (certificat art. 20), EP-002-S07 ⚠️ (pré-état daté — même refonte à prévoir en suivi)

**Statut actuel :** NON IMPLÉMENTÉ — `questionnaire.typ` est un clone du formulaire papier CSN (en-tête « DOCUMENT APPROUVÉ PAR LE CSN », police Liberation Sans 9pt, lignes à remplir manuelles, pied de page propre) annexé tel quel au PDF état daté (`append_pdf_annex`), hors du frame partagé `base.typ`.

---

## 1. Quoi (résumé fonctionnel)

Le formulaire état daté pré-rempli est regénéré **dans le design pattern des documents PDF La Ruche**, exactement comme `etat_date.typ` et `logbook.typ` :

- **Page 1 — lettre** (frame partagé `base.typ`) : `brand-block()` (logo + « La Ruche » Fraunces + tagline DM Sans), `letter-header()` (émetteur = syndic en exercice, left ; destinataire = office notarial demandeur, right ; filet marque `#643fd5`), `letter-opening()` (objet « État daté — mutation des lots n° … » + référence dossier + salutation), `letter-paragraphs()` (annonce du document : nature, date de demande, rappel que l'état daté est adressé au notaire avant l'acte authentique, annonce des annexes), `letter-closing()` (formule + signature alignée à droite : « Pour le syndic, » + nom en gras + qualité en gris + « Fait à {lieu}, le {date} »), `letter-footer()` en pied de page courant (filet + ligne d'édition DM Sans 7pt + mention légale italique).
- **Pages suivantes — annexe réglementaire** (comme `etat_date.typ` après `#pagebreak()` + `#set text(font: "Liberation Sans")`) : en-tête d'annexe (kicker espacé « A N N E X E », titre 16pt gras couleur marque, sous-titre immeuble + adresse, filet marque), puis tout le contenu réglementaire du formulaire, pré-rempli :
  - informations des parties (immeuble, cédant, acquéreur, lots, mutation onéreuse/gratuite, date envisagée, office notarial, référence) ;
  - **1ère / 2ème / 3ème partie** via le helper partagé `amount-table(rows)` (label | montant aligné à droite), totaux, modalités de remboursement des avances (art. 45-1, solution 1/2) ;
  - **annexe à la 3ème partie** : quote-part appelée/réelle N-1/N-2 (budget + hors budget), procédures en cours, fonds de travaux (part du lot + dernière cotisation) ;
  - événements postérieurs (nouvelle AG, travaux votés) ; sommes exigibles et **certificat art. 20** (EP-002-S20) ;
  - renseignements complémentaires A1–A12 et B1–B10.

**Règles de pré-remplissage** : valeur connue → **gras** (`strong`) ; champ inconnu → tiret cadratin « — » et case non cochée, **jamais une ligne vide à remplir à la main** (le document est pré-rempli, les sections « néant » restent explicites). Aucune section réglementaire n'est jamais omise.

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| Frame partagé (layout v7.1) | `backend/src/domain/typst/base.typ` : `brand-block`, `letter-header`, `letter-opening`, `letter-paragraphs`, `letter-closing`, `letter-footer`, helpers `amount-table`, `signature-blocks` |
| État daté (déjà au format La Ruche) | `backend/src/domain/typst/etat_date.typ` (lettre + contenu 3 parties + page d'annonce annexe) |
| Formulaire CSN (hors pattern) | `backend/src/domain/typst/questionnaire.typ`, enregistré dans `EMBEDDED_TEMPLATES` (`pdf_template.rs`), compilé puis annexé via `append_pdf_annex` (`domain/legal_documents.rs`) |
| Payload du formulaire | `questionnaire_payload()` dans `backend/src/domain/legal_documents.rs` (inchangé — seule la mise en page change) |
| Formulaire au format La Ruche | **inexistant** |

## 3. Modèle de données

**Aucune migration, aucun changement de payload.** Le payload `questionnaire_payload()` existant alimente le nouveau template. La compilation se fait dans le même document que l'état daté (template unique `etat_date.typ` étendu) ou via un second template `etat_date_form.typ` fusionné par `append_pdf_annex` — le choix retenu : **template unique** (suppression de `questionnaire.typ` et de l'appel `append_pdf_annex` pour le questionnaire).

## 4. Backend

### 4.1 Template (`backend/src/domain/typst/etat_date.typ`, réécrit)

- Réutiliser tel quel le frame `base.typ` (déjà en place) pour la lettre de transmission (émetteur syndic, destinataire notaire, objet état daté) ;
- après la page d'annonce d'annexe existante, rendre **tout le contenu du formulaire** avec les helpers du frame : `amount-table` pour les 3 parties et sommes exigibles, tableaux quote-part N-1/N-2, blocs oui/non (cases `[x]` / `[ ]`), `signature-blocks()` pour le certificat art. 20 ;
- supprimer `questionnaire.typ` de `EMBEDDED_TEMPLATES` et le bloc `append_pdf_annex` correspondant dans `domain/legal_documents.rs`.

### 4.2 Domain (`backend/src/domain/legal_documents.rs`)

- `questionnaire_payload()` inchangé en structure ; ajuster les champs texte pour le pré-remplissage gras vs « — » (les `Option<String>`/`None` rendent désormais « — », plus de ligne vide).
- Tests existants (`render_etat_date_appends_questionnaire_annex`) à mettre à jour : le PDF final est un seul document (plus de pages annexes CSN séparées).

### 4.3 API

Aucun changement d'endpoint : `GET /etat-date-requests/{id}/export/pdf` produit le PDF complet au format La Ruche.

## 5. Frontend

Aucun changement (le PDF est téléchargé tel quel). Si une prévisualisation web du formulaire est ajoutée plus tard, elle suivra le catalogue `docs/DESIGN.md §3`.

## 6. Cas limites

- Champ inconnu → « — » / case non cochée, section toujours présente.
- Cédant à jour → certificat art. 20 « libre de toute obligation », validité 1 mois (EP-002-S20).
- Moins de 2 exercices → annexe quote-part limitée aux exercices existants avec mention.
- Aucune procédure → « Néant ».
- Le pied de page La Ruche (édition + mention légale) est présent sur **toutes** les pages, y compris l'annexe réglementaire ; le bandeau « DOCUMENT APPROUVÉ PAR LE CSN » disparaît.

## 7. Critères d'acceptation

1. Le PDF d'état daté est un **document unique au format La Ruche** : bloc marque page 1, double en-tête, lettre, signature « Pour le syndic » + « Fait à …, le … », pied de page courant sur toutes les pages.
2. Tout le contenu réglementaire du formulaire CSN est présent et pré-rempli (3 parties + annexe + certificat art. 20 + renseignements A1–A12/B1–B10), via `amount-table` et les helpers `base.typ`.
3. Valeurs connues en gras, inconnues en « — » ; aucune ligne à remplir manuellement ; sections « néant » explicites.
4. `questionnaire.typ` supprimé ; plus d'en-tête « DOCUMENT APPROUVÉ PAR LE CSN » dans le rendu.
5. Aucun changement de payload ni d'endpoint API.

## 8. Plan de tests

- **Rendu** : test existant de génération PDF mis à jour — vérifier présence du bloc marque (page 1), du pied de page (toutes pages), l'absence du bandeau CSN, et le nombre de pages attendu.
- **Payload** : tests unitaires existants de `questionnaire_payload` (champs remplis / `None` → « — »).
- **E2E** : `GET /etat-date-requests/{id}/export/pdf` → 200, `application/pdf`, contenu texte du PDF contient « La Ruche » et « État daté » (extraction texte via le moteur de test).

## 9. Ordre d'implémentation

1. Réécriture de `etat_date.typ` (frame complet + contenu formulaire) — dev hot-reload via `LARUCHE_TEMPLATES_DIR`.
2. `legal_documents.rs` : un seul template compilé, suppression de l'annexe CSN ; mise à jour des tests de rendu.
3. Suppression de `questionnaire.typ` et de son entrée `EMBEDDED_TEMPLATES`.
4. Vérification visuelle du PDF (page 1 lettre, annexes réglementaires, certificat art. 20 avec EP-002-S20).
5. Entrée `docs/PROGRESS.md` ; prévoir la même refonte pour le pré-état daté (`pre_etat_date.typ`) en suivi.

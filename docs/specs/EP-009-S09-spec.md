# Spécification technique — EP-009-S09

**Story :** Export PDF du carnet d'entretien (en-tête réglementaire, chronologie des interventions, contrats d'assurance et de maintenance actifs).

**Épic :** EP-009 — Carnet d'entretien
**Réf. PRD :** US-CE-004 · **Réf. LEGAL :** carnet d'entretien obligatoire (loi SRU, décret n°2001-477)
**État :** NON IMPLÉMENTÉ (aucun code PDF ; seules des chaînes de fixture de test dans `domain/processor.rs:52-54`).

---

## 1. Décision de conception — HTML imprimable plutôt que PDF serveur

**Retenu : vue HTML optimisée impression + impression navigateur (print-to-PDF).**

- Zéro nouvelle dépendance (pas de lib PDF Rust ni headless browser).
- Le rendu réutilise les composants/styles du frontend ; la mise en page se règle en CSS `@media print`.
- Le navigateur produit un PDF fidèle via « Imprimer → Enregistrer en PDF » — suffisant pour l'usage (remise du carnet à un acquéreur / au conseil).

**Alternative rejetée : génération PDF côté serveur** (printpdf/typst/wkhtmltopdf/headless Chromium) — dépendance lourde, duplication de la mise en page hors du design system, maintenance disproportionnée pour un document consulté ponctuellement. À reconsidérer seulement si un envoi automatique par e-mail du PDF devient une exigence.

## 2. État actuel du code

| Élément | Emplacement | Constat |
|---|---|---|
| Routes carnet | `backend/src/api/mod.rs:296-304` | list / summary / get / put / delete / correction |
| Endpoint summary (en-tête : adresse, syndic) | bloc carnet, `api/mod.rs:296-304` | **Existant — réutilisé tel quel** |
| Client API carnet | `frontend/src/api/index.ts:400-413` (`maintenanceApi`) | Pas d'export |
| Vue carnet | `frontend/src/features/maintenance/MaintenanceView.vue` | Point d'entrée du bouton « Exporter » |
| Contrats actifs | `contractsApi` existant (`GET /contracts`) | Filtrer côté client `status == 'active'` + type assurance/maintenance |

## 3. Modèle de données

Aucune migration.

## 4. Backend

**Aucun nouveau code requis** si les données suffisent via les endpoints existants :

- en-tête réglementaire : endpoint summary (adresse de l'immeuble + identité du syndic)
- chronologie : `GET` liste carnet (déjà exhaustive)
- contrats actifs : `GET /contracts` (filtrage client par `status` et `contract_type` assurance/maintenance).

Si le summary n'expose pas tous les champs d'en-tête réglementaires (adresse complète, nom du syndic), l'étendre — modification mineure de la couche db/api existante, pas de nouvelle route.

## 5. Frontend

### 5.1 Nouvelle vue `frontend/src/features/maintenance/MaintenancePrintView.vue`

Route dédiée (ex. `/maintenance/print`), accessible Manager/BoardMember (mêmes règles que la vue carnet). Contenu :

1. **En-tête réglementaire** : « Carnet d'entretien de l'immeuble » + adresse + syndic (données du summary) + date d'édition.
2. **Contrats actifs** : deux blocs — assurance(s) en cours, contrats de maintenance en cours (prestataire, objet, dates, montant).
3. **Chronologie** : tableau des interventions (date, titre, description, coût, prestataire, source) trié `intervention_date ASC`.
4. Pied de page : mention « Édité le … via La Ruche ».

Style : Tailwind + `@media print` (masquer navigation/boutons via classe `print:hidden`, `break-inside-avoid` sur les lignes, format A4). Pas de `<style>` bloc si les utilitaires suffisent — sinon tokens/utilitaires print de Tailwind v4 uniquement.

Bouton « Imprimer / Exporter en PDF » appelant `window.print()`.

### 5.2 `MaintenanceView.vue`

Bouton « Exporter le carnet » ouvrant la route print (nouvel onglet `target="_blank"` acceptable).

### 5.3 `frontend/src/api/index.ts`

Aucun nouvel appel si les endpoints existants suffisent (summary + list + contracts). Ne rien ajouter à `maintenanceApi` (`:400-413`) tant que ce n'est pas nécessaire.

### 5.4 Catalogue

`MaintenancePrintView` est une vue, pas un composant catalogue ; si un composant d'en-tête imprimable réutilisable est extrait, l'ajouter à `docs/DESIGN.md §2` avant usage.

## 6. Cas limites

| Cas | Règle |
|---|---|
| Carnet vide | La vue s'imprime quand même : en-tête + « Aucune intervention enregistrée » |
| Aucun contrat actif | Bloc affiché avec mention « Aucun contrat en cours » (l'absence d'assurance est une information réglementaire en soi) |
| Volumétrie importante | Tableau multi-pages ; `break-inside-avoid` par ligne, en-tête de tableau répété (`thead` imprimé par page nativement) |
| Rôles | Mêmes guards que la vue carnet ; la route print ne contourne aucun contrôle d'accès API |
| Montants | Affichage formaté FR (€, deux décimales) — les montants restent `Decimal` côté backend |

## 7. Critères d'acceptation

1. Depuis la vue carnet, « Exporter le carnet » ouvre une page imprimable complète (en-tête réglementaire, contrats actifs, chronologie).
2. `Ctrl+P` / bouton → PDF navigateur propre : A4, sans navigation, tableaux non coupés au milieu d'une ligne.
3. Aucune nouvelle dépendance ajoutée (backend comme frontend).
4. Carnet vide et absence de contrats gérés sans erreur.

## 8. Plan de tests

- **Backend** : aucun nouveau test si aucun endpoint modifié ; si le summary est étendu, test e2e des nouveaux champs.
- **Frontend** : vérification manuelle du rendu print (Chrome + Firefox), carnet vide, carnet > 2 pages.
- Lint/build : `npm --prefix frontend run build && npm --prefix frontend run lint`.

## 9. Ordre d'implémentation

1. Vérifier le contenu du summary ; l'étendre si un champ d'en-tête manque.
2. `MaintenancePrintView.vue` + route + guards.
3. Bouton d'export dans `MaintenanceView.vue`.
4. Ajustements CSS print, tests manuels.

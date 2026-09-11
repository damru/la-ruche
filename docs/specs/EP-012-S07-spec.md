# Spécification technique — EP-012-S07

**Story :** Exporter la fiche synthétique en PDF — document imprimable à en-tête copropriété, depuis la fiche publiée.

**Réfs :** Épic EP-012 (`docs/EPICS.md` §EP-012), LEGAL.md §1.3, PRD `docs/product/PRD - Conformité et reporting.md` US-CR-005 (AC-CR-005-A à C).

> S'appuie sur EP-012-S05 (fiche `published`) et EP-012-S06 (vue lecture + composants de section partagés). **Prérequis : S05 et S06 livrés.** Pas de migration, pas de nouveau endpoint de données.

## 1. Quoi (résumé fonctionnel)

**Décision technique** : aucun moteur PDF n'existe dans le repo et en embarquer un (wkhtmltopdf, headless Chrome, crate `printpdf`) serait disproportionné pour un document d'une page. Le « PDF » est produit **côté client** via une vue HTML optimisée impression (`@media print`) + `window.print()` → « Enregistrer en PDF » du navigateur. C'est le chemin le plus simple qui satisfait AC-CR-005-B/C (contenu identique à l'affichage en ligne, par construction). Si un vrai fichier PDF serveur devient nécessaire (envoi automatique par email), ce sera une story ultérieure avec ADR.

Disponible pour Syndic, Conseil et Copropriétaire (personas US-CR-005), uniquement sur une fiche `published` (AC-CR-005-A).

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| Fiche publiée (données) | `GET /fiche-synthetique/published` (S06) ; `GET /fiche-synthetique/{id}` (Manager, S02) |
| Composants de section partagés | `frontend/src/features/compliance/` (factorisés en S06 §5) |
| Vues consommatrices | `FicheSynthetiqueView.vue` (Syndic), `FichePubliqueView.vue` (résident) |
| Identité en-tête (nom, adresse) | `payload.identification` — déjà dans la fiche, aucun appel supplémentaire |
| Contraintes CSS projet | AGENTS.md : pas de valeurs brutes en `class`/`style`, pas de `<style>` en feature view → les règles print vivent dans une feuille dédiée ou des utilitaires Tailwind `print:` |

## 3. Modèle de données

Aucune migration. Aucune donnée nouvelle : le document imprime exactement le payload publié (AC-CR-005-C).

## 4. Backend Rust

Aucun changement backend. Le contrôle « `published` uniquement » (AC-CR-005-A) est garanti par la source de données : la vue d'impression consomme `GET /fiche-synthetique/published` (résident) ou une fiche dont `status === 'published'` (Syndic) — le frontend n'affiche le bouton d'export que dans ces cas, et le backend ne sert de toute façon jamais un draft aux non-Managers (S06).

## 5. Frontend Vue

### 5.1 Nouveau `frontend/src/features/compliance/FichePrintView.vue`

- Route `compliance/fiche/print` (name `fiche-print`) — accessible aux mêmes rôles que S06 (pas de `Tenant`).
- Charge la fiche via `ficheApi.published()` ; mise en page A4 portrait :
  - **En-tête réglementaire** (AC-CR-005-B) : nom de la copropriété, adresse complète, mention « Fiche synthétique de la copropriété — art. 8-2 loi du 10 juillet 1965 », exercice `fiscal_year`.
  - Les six sections du payload, via les composants partagés (variante dense).
  - **Pied de page** : date de publication (`published_at`) et date de génération du document (impression) — AC-CR-005-B.
- Utilitaires Tailwind `print:` (`print:hidden` pour le chrome applicatif, sauts `break-inside-avoid` par section) ; masquer navigation/sidebar à l'impression.
- Bouton « Imprimer / Enregistrer en PDF » → `window.print()`. Déclenchement auto optionnel via query `?auto=1` posée par les vues appelantes.

### 5.2 Points d'entrée

- `FichePubliqueView.vue` (S06) : bouton « Exporter en PDF » (prévu S06 §5) → navigue vers `fiche-print`.
- `FicheSynthetiqueView.vue` (Syndic) : même bouton, affiché uniquement si `status === 'published'` (jamais sur draft/archived — AC-CR-005-A).
- Nouveau composant/variante → `docs/DESIGN.md` §2 avant usage.

## 6. Cas limites

- Fiche `draft`/`archived` : aucun bouton d'export ; accès direct à `fiche-print` sans fiche publiée → état vide (même comportement que S06).
- Sections nulles (assurance absente) : la section s'imprime avec la mention « Non renseigné » plutôt que d'être omise (AC-CR-005-C : ni ajout ni omission par rapport à l'affichage en ligne — l'affichage en ligne adopte la même mention).
- Payload long (grande copropriété) : `break-inside-avoid` par section, le document peut couler sur 2 pages.
- Mode sombre : la feuille print force le rendu clair (le thème du viewer ne doit pas teinter le document).

## 7. Critères d'acceptation

- **AC-CR-005-A** : export accessible uniquement pour une fiche `published`.
- **AC-CR-005-B** : le document imprimé comporte l'en-tête réglementaire complet, toutes les sections et les dates (publication + génération).
- **AC-CR-005-C** : contenu strictement identique à la consultation en ligne — même source de données, mêmes composants.
- **AC-S07-D** : `Tenant` n'accède ni au bouton ni à la route.

## 8. Plan de tests

- Frontend (composants) : rendu de `FichePrintView.vue` avec payload complet et avec sections nulles ; visibilité du bouton selon `status` ; en-tête et pied de page présents.
- Vérification manuelle (checklist PR) : aperçu avant impression Chrome + Firefox, A4, pas d'élément de navigation visible, rendu clair en thème sombre.
- e2e backend : aucun (pas de changement backend).

## 9. Ordre d'implémentation

1. Variante dense/print des composants de section partagés.
2. `FichePrintView.vue` + route + garde de rôle.
3. Boutons d'entrée dans les deux vues (condition `published`).
4. `docs/DESIGN.md` §2 + entrée `docs/PROGRESS.md` (EP-012-S07).

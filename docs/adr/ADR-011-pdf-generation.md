# ADR-011 — Génération de documents PDF réglementaires

**Date :** 2026-06-30
**Statut :** Accepté
**Références :** LEGAL.md §7.1 (état daté) · §7.2 (pré-état daté) · §4.2 (annexes comptables) · décret 67-223 art. 5 · arrêté du 14/03/2005

---

## Contexte

Plusieurs obligations légales imposent la production de documents PDF officiels générés
côté serveur à partir des données de l'application :

- **État daté** (`EP-002-S06`) — synthèse financière d'un lot lors d'une vente.
- **Pré-état daté** (`EP-002-S07`) — synthèse financière de la copropriété.
- **Annexes comptables I/II/III** (`EP-004-S13`) — états réglementaires de l'arrêté du 14/03/2005.

À ce jour le backend **ne génère aucun PDF** : la table `document` et le stockage S3 ne
servent qu'à héberger des fichiers **téléversés** par les utilisateurs. Il n'existe donc
aucune brique de génération documentaire.

Ces documents partagent des contraintes communes : en-tête copropriété, tableaux
comptables à colonnes, montants en `Decimal`, mise en page déterministe et reproductible
(valeur probatoire), et exécution dans l'image conteneur Debian slim sans dépendance
système lourde.

## Options envisagées

| Option | Avantages | Inconvénients |
|---|---|---|
| **A. Crate Rust pur** (`typst`, `genpdf`, `printpdf`) | Pas de dépendance système ; binaire autonome ; déterministe ; cohérent avec la stack | Mise en page à écrire en Rust / langage de balisage |
| B. HTML → PDF (Chromium headless, `wkhtmltopdf`) | Réutilise des templates HTML/CSS | Dépendance binaire lourde dans l'image ; non déterministe ; surface d'attaque accrue |
| C. Service Python (`weasyprint`) | Templating mûr | Runtime Python séparé à déployer et maintenir ; hors stack |
| D. Génération côté frontend | Décharge le backend | Pas de valeur probatoire serveur ; données sensibles exposées au client |

## Décision

**Option A — génération côté serveur via un crate Rust pur**, sans dépendance binaire
externe ni runtime séparé.

Crate retenu par défaut : **`typst`** (moteur de composition embarqué), pour ses
capacités de tableaux et de mise en page réglementaire, son rendu déterministe et sa
maintenance active. `genpdf` (API Rust directe, sans langage de balisage) reste le repli
si l'intégration de `typst` s'avère trop lourde ; `printpdf` est jugé trop bas niveau pour
des documents tabulaires.

### Principes d'implémentation (pour S06 / S07 / S13)

- La génération vit dans un module `domain/` dédié (ex. `domain/legal_documents.rs`) :
  aucune logique de rendu dans les handlers `api/`.
- Le PDF produit est **persisté dans la GED** (`document` + table de liaison) conformément
  à la règle « Upload = GED » de `ARCHITECTURE.md`, puis servi via flux pré-signé S3.
- Les montants sont formatés depuis `Decimal` (jamais `f64`).
- Le rendu doit être déterministe (mêmes données → même PDF) pour la valeur probatoire.
- Génération réservée au syndic (`Role::Manager`).

## Conséquences

- Ajout d'une dépendance crate (`typst` ou `genpdf`) au `Cargo.toml` lors de la première
  story PDF ; aucun changement de l'image conteneur (Debian slim inchangée).
- Les templates des documents réglementaires (état daté, annexes) suivent les **modèles
  réglementaires standard** (décret 67-223 art. 5 ; arrêté du 14/03/2005), sans gabarit
  client spécifique au MVP.
- `ARCHITECTURE.md` devra référencer ADR-011 lors de l'implémentation de la première
  story de génération PDF.

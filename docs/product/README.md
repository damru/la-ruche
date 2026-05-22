# Product Hub — PRD par feature

> **Dénomination** — Dans le présent document, l'expression **« l'Application »** désigne exclusivement **La Ruche**, plateforme web open source de gestion participative de copropriété. Ces deux termes sont interchangeables et désignent la même entité logicielle.

## Objectif

Ce hub produit complète le `PRD.md` maître avec un **PRD détaillé par feature**.

Le `PRD.md` maître reste le document de synthèse de vision. Le présent dossier porte le niveau de détail opérationnel attendu pour le discovery produit, le backlog, l'UX, les permissions et les cas d'erreur.

## Convention documentaire

- Un document = une feature produit.
- Le niveau attendu est **produit** : interactions, règles métier, permissions, cas d'erreur, hors scope et questions ouvertes.
- L'état d'implémentation réel n'est pas le périmètre du document : la cible produit peut être plus large. Le backlog dans `EPICS.md` reste la source de suivi de livraison.
- Les contraintes légales structurantes sont rappelées de manière courte et renvoient vers `LEGAL.md`.

## Légende permissions

Les matrices du Product Hub sont construites avec :

- une ligne par domaine de gestion (`gestion de X`) ;
- une cellule par rôle contenant les droits CRUD autorisés sur ce domaine.

| Code | Signification |
|---|---|
| `C` | Peut créer |
| `R1` | Peut lire son périmètre propre |
| `R` | Peut lire le périmètre de la feature |
| `U1` | Peut modifier son périmètre propre |
| `U` | Peut modifier le périmètre de la feature |
| `D1` | Peut supprimer son périmètre propre |
| `D` | Peut supprimer le périmètre de la feature |
| `X` | Interdit |

> **Lecture** — Exemple `C / R / U / D` = création autorisée + lecture + mise à jour + suppression sur le périmètre de la feature.

> **Note** — Le pseudo-rôle `Prestataire (token)` désigne un accès ultra-minimal, contextuel et temporaire via lien sécurisé, sans espace authentifié complet.

## Index des features

| Feature | Réf. backlog | Statut produit | PRD détaillé | Note |
|---|---|---|---|---|
| Registre de la copropriété | `EP-001` | en cours | [PRD - Registre de la copropriété](PRD%20-%20Registre%20de%20la%20copropri%C3%A9t%C3%A9.md) | Inclut identité, clés, lots et tantièmes |
| Gestion des propriétaires | `EP-002` | en cours | [PRD - Gestion des propriétaires](PRD%20-%20Gestion%20des%20propri%C3%A9taires.md) | Inclut mutations, invitations et RGPD |
| Assemblée Générale | `EP-003`, `EP-013` | en cours | [PRD - Assemblée Générale](PRD%20-%20Assembl%C3%A9e%20G%C3%A9n%C3%A9rale.md) | Couvre préparation, conduite et PV |
| Budget & Charges | `EP-004` | en cours | [PRD - Budget et charges](PRD%20-%20Budget%20et%20charges.md) | Inclut budget, appels, fonds travaux et arrêté |
| Documents | `EP-005` | en cours | [PRD - Documents](PRD%20-%20Documents.md) | Couche transverse documentaire |
| Profil & préférences | `EP-019`, `DDR-004` | en cours | [PRD - Profil et préférences](PRD%20-%20Profil%20et%20pr%C3%A9f%C3%A9rences.md) | Feature simple self-service utilisateur |
| Contrats fournisseurs | `EP-006` | en cours | [PRD - Contrats fournisseurs](PRD%20-%20Contrats%20fournisseurs.md) | Inclut rappels, avenants et pièces liées |
| Tâches & Prestataires | `EP-007`, `EP-015` | en cours | [PRD - Tâches et prestataires](PRD%20-%20T%C3%A2ches%20et%20prestataires.md) | Inclut sous-parcours prestataire tokenisé |
| Référentiel prestataires | `EP-015` | en cours | [PRD - Référentiel prestataires](PRD%20-%20Référentiel%20prestataires.md) | Annuaire partagé référencé par Tâches et Contrats |
| Sinistres | `EP-008` | en cours | [PRD - Sinistres](PRD%20-%20Sinistres.md) | Inclut délais légaux, transmission, clôture et remboursement |
| Carnet d'entretien | `EP-009` | en cours | [PRD - Carnet d'entretien](PRD%20-%20Carnet%20d'entretien.md) | Inclut alimentation manuelle et automatique depuis tâches/contrats/sinistres |
| Consultations & Sondages | `EP-010` | en cours | [PRD - Consultations et sondages](PRD%20-%20Consultations%20et%20sondages.md) | Participatif, non décisionnel, avec option anonymat |
| Signalements | `EP-011` | en cours | [PRD - Signalements](PRD%20-%20Signalements.md) | Résident → suivi → conversion en tâche |
| Conformité & Reporting réglementaire | `EP-012` | en attente | [PRD - Conformité et reporting](PRD%20-%20Conformité%20et%20reporting.md) | Fiche synthétique obligatoire (art. 8-2) et export RNC |
| Calendrier copropriété | `EP-014` | en attente | [PRD - Calendrier](PRD%20-%20Calendrier.md) | Agrégation read-only de tous les événements datés |
| Paiement en ligne | `EP-016` | en attente | [PRD - Paiement en ligne](PRD%20-%20Paiement%20en%20ligne.md) | Cadrage préliminaire — lot futur, prérequis PSP/DSP2 |
| Journal d'activité & Suivi de cycle de vie | `EP-017` | en attente | [PRD - Journal d'activité](PRD%20-%20Journal%20d'activité.md) | Timeline métier lisible — distincte de l'audit probant backend |
| Immutabilité comptable & auditabilité probante | `EP-018` | en attente | à créer | Contrainte transverse structurante |

## Lot 1

Le lot 1 du Product Hub documente trois features contrastées pour valider la structure cible :

- `Assemblée Générale` pour une feature fortement contrainte juridiquement et riche en statuts.
- `Tâches & Prestataires` pour une feature orientée résolution opérationnelle avec sous-parcours externe.
- `Profil & préférences` pour une feature simple, personnelle et récurrente.
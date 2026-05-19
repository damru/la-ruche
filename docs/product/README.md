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
| Registre de la copropriété | `EP-001` | en cours | à créer | Inclut identité, clés, lots et tantièmes |
| Gestion des propriétaires | `EP-002` | en cours | à créer | Inclut mutations, invitations et RGPD |
| Assemblée Générale | `EP-003`, `EP-013` | en cours | [PRD - Assemblée Générale](PRD%20-%20Assembl%C3%A9e%20G%C3%A9n%C3%A9rale.md) | Couvre préparation, conduite et PV |
| Budget & Charges | `EP-004` | en cours | à créer | Inclut budget, appels, fonds travaux et arrêté |
| Documents | `EP-005` | en cours | à créer | Couche transverse documentaire |
| Profil & préférences | `EP-019`, `DDR-004` | en cours | [PRD - Profil et préférences](PRD%20-%20Profil%20et%20pr%C3%A9f%C3%A9rences.md) | Feature simple self-service utilisateur |
| Contrats fournisseurs | `EP-006` | en cours | à créer | Inclut rappels et pièces liées |
| Tâches & Prestataires | `EP-007`, `EP-015` | en cours | [PRD - Tâches et prestataires](PRD%20-%20T%C3%A2ches%20et%20prestataires.md) | Inclut sous-parcours prestataire tokenisé |
| Référentiel prestataires | `EP-015` | en cours | à créer | Peut rester séparé malgré le lien fort avec les tâches |
| Sinistres | `EP-008` | en cours | à créer | Inclut délais, transmission, clôture |
| Carnet d'entretien | `EP-009` | en cours | à créer | Inclut alimentation manuelle et automatique |
| Consultations & Sondages | `EP-010` | en cours | à créer | Participatif, non décisionnel |
| Signalements | `EP-011` | en cours | à créer | Résident -> suivi -> conversion éventuelle |
| Conformité & Reporting réglementaire | `EP-012` | en attente | à créer | RNC et fiche synthétique |
| Calendrier copropriété | `EP-014` | en attente | à créer | Feature de cadrage |
| Paiement en ligne | `EP-016` | en attente | à créer | Feature de cadrage |
| Journal d’activité & Suivi de cycle de vie | `EP-017` | en attente | à créer | Timeline métier distincte de l'audit probant |
| Immutabilité comptable & auditabilité probante | `EP-018` | en attente | à créer | Contrainte transverse structurante |

## Lot 1

Le lot 1 du Product Hub documente trois features contrastées pour valider la structure cible :

- `Assemblée Générale` pour une feature fortement contrainte juridiquement et riche en statuts.
- `Tâches & Prestataires` pour une feature orientée résolution opérationnelle avec sous-parcours externe.
- `Profil & préférences` pour une feature simple, personnelle et récurrente.
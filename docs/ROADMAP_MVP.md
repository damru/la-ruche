# ROADMAP_MVP.md — l'Application

> **Dénomination** — Dans le présent document, l'expression **« l'Application »** désigne exclusivement **La Ruche**, plateforme web open source de gestion participative de copropriété. Ces deux termes sont interchangeables et désignent la même entité logicielle.

## Objectif

Livrer un MVP réellement exploitable en production sur le périmètre des epics MVP, en supprimant les écarts entre:
- backend réel et frontend mock,
- backlog déclaré et comportements effectifs,
- couverture légale attendue et couverture technique.

## Périmètre MVP concerné

Epics MVP: EP-001, EP-002, EP-003, EP-004, EP-005, EP-006, EP-008, EP-009, EP-010, EP-011.

Hors MVP (pas bloquant livraison MVP): EP-007, EP-012, EP-013, EP-014, EP-016, EP-017.

## État de départ (24/04/2026)

- Solide: EP-001 partiellement avancée, EP-005 majoritairement branchée, onboarding et auth de base.
- Partiel: EP-002, EP-003, EP-004, EP-006.
- Majoritairement stub/mock: EP-008, EP-009, EP-010, EP-011.
- Dette transversale: écrans mock encore présents sur des modules MVP critiques.

## Principes d'exécution

- Priorité aux tranches verticales backend + frontend + tests.
- Priorité légale avant confort UX.
- Aucune story marquée terminée si endpoint en 501 ou vue en mock.
- Chaque lot se termine par:
- validation fonctionnelle,
- validation technique,
- mise à jour LEGAL/EPICS/PROGRESS.

## Roadmap détaillée (10 semaines)

## Lot 0 — Stabilisation d'exécution (Semaine 1)

Objectif: sécuriser le flux de delivery avant la montée en charge.

Scope:
- Geler le mode demo par défaut (pas de données mock en environnement normal).
- Ajouter une checklist de sortie par epic MVP.
- Standardiser la convention de statut: real, partial, stub/mock.

Livrables:
- Front: bannière mock uniquement en mode explicite demo.
- Back: inventaire des routes 501 et priorisation finale.
- Qualité: template de DoD MVP.

Sortie attendue:
- Build vert.
- Liste des dépendances inter-epics validée.

## Lot 1 — Rendre visibles les capacités déjà codées (Semaines 2-3)

Objectif: remplacer rapidement les écrans mock des modules où le backend existe déjà.

Scope principal:
- EP-003 (frontend AG): brancher listes, détail, résolutions, vote, résultats.
- EP-004 (frontend Budget): brancher budgets, lignes, appels, fonds travaux, relevés.
- EP-006 (frontend Contrats): brancher CRUD + rappels.

Stories cibles:
- EP-003-S01 à S06, EP-003-S07.
- EP-004-S01 à S08, EP-004-S10.
- EP-006-S01 à S04.

Sortie attendue:
- Plus de source src/mock/data sur AG/Budget/Contrats.
- Parcours manager complet sur ces 3 modules.

## Lot 2 — Fermer les domaines MVP encore en stub (Semaines 4-6)

Objectif: implémenter le backend manquant et brancher le frontend sur les domaines critiques restants.

Scope principal:
- EP-008 Sinistres: CRUD + délais légaux + alertes + transmission assurance.
- EP-009 Carnet: liste, création, update, suppression, rattachement docs.
- EP-010 Consultations: création, réponse, résultats, clôture.
- EP-011 Signalements: création, listing, changement de statut.

Stories cibles:
- EP-008-S01 à S05.
- EP-009-S01 à S05.
- EP-010-S01 à S03.
- EP-011-S01 à S03.

Sortie attendue:
- Zéro route 501 sur périmètre MVP.
- Zéro écran MVP alimenté en mock.

## Lot 3 — Couverture légale prioritaire propriétaires/comptabilité (Semaines 7-8)

Objectif: couvrir les obligations légales à plus fort risque opérationnel.

Scope principal:
- EP-002: propriétaires, mutation, état daté, pré-état daté, export RGPD.
- EP-004: verrouillage blocage déséquilibre tantièmes bout-en-bout.
- EP-005: typage réglementaire documentaire (EDD, règlement, PV).

Stories cibles:
- EP-002-S01 à S09.
- EP-004-S07, EP-004-S09.
- EP-005-S06, EP-005-S07.

Sortie attendue:
- Flux notarial opérationnel minimum.
- Exports et traces RGPD utilisables.
- Mapping comptable initial en place.

## Lot 4 — Durcissement MVP et préparation release (Semaines 9-10)

Objectif: passer du fonctionnel au déployable stable.

Scope principal:
- Tests d'intégration API sur tous modules MVP.
- Tests E2E des parcours critiques manager et copropriétaire.
- Journalisation/audit et observabilité sur actions légales.
- Revue docs finale LEGAL/EPICS/PROGRESS.

Sortie attendue:
- Pack de non-régression MVP.
- Go/No-Go formalisé.

## Dépendances clés

- EP-001 conditionne EP-004 (tantièmes).
- EP-003 conditionne partiellement EP-002-S06 (travaux votés pour état daté).
- EP-005 conditionne EP-003/EP-009/EP-006 pour pièces justificatives.
- EP-008 et EP-011 partagent un socle de workflow statuts/notifications.

## Critères de sortie MVP

- Aucun module MVP en stub/mock.
- Zéro endpoint 501 sur routes MVP.
- Tous les parcours critiques couverts par tests d'intégration.
- Couverture LEGAL: plus de contradiction docs/code sur périmètre MVP.
- Documentation alignée à date: LEGAL, EPICS, PROGRESS.

## Indicateurs de pilotage hebdomadaires

- Taux de stories MVP terminées vs prévues par lot.
- Nombre de routes 501 restantes (cible: 0 sur MVP).
- Nombre de vues mock restantes (cible: 0 sur MVP).
- Taux de tests passants (API + E2E).
- Nombre d'écarts LEGAL non couverts en MVP.

## Risques principaux et mitigations

- Risque: sous-estimation EP-002 (workflow notarial).
- Mitigation: découpage en incréments livrables (lecture, génération, export).

- Risque: blocage front sur contrats de schéma API.
- Mitigation: contrats DTO gelés en début de lot + tests contractuels.

- Risque: régression sur modules déjà branchés.
- Mitigation: suite non-régression exécutée à chaque fin de lot.

## Prochaines actions immédiates

1. Lancer Lot 0 avec suppression contrôlée du mode mock par défaut.
2. Ouvrir Lot 1 en priorité sur EP-003 frontend (AG) puis EP-004 frontend (Budget).
3. Mettre à jour le statut des stories dans EPICS à la fin de chaque lot.

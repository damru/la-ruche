# EPICS.md — l'Application

> **Dénomination** — Dans le présent document, l'expression **« l'Application »** désigne exclusivement **La Ruche**, plateforme web open source de gestion participative de copropriété. Ces deux termes sont interchangeables et désignent la même entité logicielle.

## Backlog produit

> Ce document est le backlog officiel du projet.
> Chaque feature est organisée en epic. Les stories sont ajoutées à la demande, au moment de l'implémentation.
> Le MVP est l'ensemble des epics marquées **MVP : oui**.
> Statuts possibles : **en attente** · **en cours** · **terminée**
> Dernière mise à jour : 24 avril 2026 (re-baseline code-réel)

> **Lecture d'avancement (code actuel) :**
> - **backend réel** = handlers + DB opérationnels
> - **frontend branché** = vue connectée à l'API réelle
> - **stub/mock** = endpoint `501 NOT_IMPLEMENTED` ou vue alimentée par `src/mock/data.ts`

***

## Epics

### EP-001 — Registre de la copropriété

**Statut :** en cours | **MVP :** oui | **Réf. PRD :** Fondations légales

**État code actuel :** backend réel + frontend branché sur le périmètre registre ; contrôle de blocage strict des appels de fonds en cas de déséquilibre encore partiel.

La fiche identitaire de la copropriété (nom, adresse, SIRET syndic), le règlement de copropriété, les clés de répartition configurables et le registre des lots avec leurs tantièmes par clé.

**Stories :**

* `EP-001-S01` — Créer et modifier la fiche identité de la copropriété (nom, adresse, SIRET syndic, coordonnées de gestion)
* `EP-001-S02` — Importer, stocker et consulter le règlement de copropriété et les documents fondateurs
* `EP-001-S03` — Définir les clés de répartition et leurs règles de calcul
* `EP-001-S04` — Créer, modifier, archiver et consulter les lots avec leurs tantièmes par clé
* `EP-001-S05` — Afficher un registre consolidé copropriété + lots + tantièmes exploitable par les autres modules
* `EP-001-S06` — Versioning des tantièmes : toute modification d'un `share_value` ferme l'enregistrement courant et crée un nouvel enregistrement daté (`valid_from`/`valid_to`) — immuabilité historique (LEGAL.md §2.2, ADR-006)
* `EP-001-S07` — Alerte de déséquilibre : bloquer la génération d'appels de fonds si `Σ tantièmes ≠ total_clé` (LEGAL.md §2.1)
* `EP-001-S08` — Clé Charges Générales système : `key_type` enum (`general` / `special`) + soft delete sur `allocation_key` ; créée automatiquement à l'onboarding, non supprimable (ADR-007, LEGAL.md §2.1)
* `EP-001-S09` — Contrôle de cohérence EDD : comparer les tantièmes configurés avec le dernier EDD déposé et produire un rapport d'écarts (non bloquant) (LEGAL.md §1.2)
* `EP-001-S10` — Journal d'audit des modifications de clés (qui, quoi, quand, justification) pour traçabilité syndic (LEGAL.md §1.2, §2.1)

***

### EP-002 — Gestion des propriétaires

**Statut :** en cours | **MVP :** oui | **Réf. PRD :** Fondations légales

**État code actuel :** partiel ; lecture des utilisateurs et base d'invitation présentes, mais endpoints propriétaires/mutations encore en stub partiel.

Historique de propriété tracé avec dates de cession, invitation par lien tokenisé, anonymisation RGPD à la demande (conservation des données comptables 10 ans).

**Stories :**

* `EP-002-S01` — Créer un propriétaire et l’associer à un ou plusieurs lots
* `EP-002-S02` — Gérer l’historique de propriété avec dates d’entrée, de sortie et cessions
* `EP-002-S03` — Inviter un propriétaire par lien tokenisé pour activer son accès
* `EP-002-S04` — Consulter la fiche propriétaire avec lots détenus, rôle et état d’activation
* `EP-002-S05` — Lancer l'anonymisation RGPD en préservant les obligations de conservation comptable (LEGAL.md §8)
* `EP-002-S06` — Générer l'état daté d'un lot à la demande (charges courantes, arriérés, fonds travaux, travaux votés non réalisés, procédures en cours) dans un délai de 15 jours (LEGAL.md §7.1)
* `EP-002-S07` — Générer le pré-état daté pour signature du compromis de vente (synthèse financière de la copropriété) (LEGAL.md §7.2)
* `EP-002-S08` — Garantir en mutation que les tantièmes restent attachés au lot (pas au propriétaire) et exposer la quote-part vendeur/acquéreur à date (LEGAL.md §2.4)
* `EP-002-S09` — Exporter les données personnelles d'un copropriétaire (droit d'accès RGPD) avec périmètre horodaté et traçabilité de la demande (LEGAL.md §8)

***

### EP-003 — Assemblée Générale

**Statut :** en cours | **MVP :** oui | **Réf. PRD :** Fondations légales

**État code actuel :** backend majoritairement réel ; frontend AG encore en mock (écran démonstratif non branché API).

Cycle de vie complet d'une AG, résolutions typées par majorité, vote par lot avec procurations, calcul automatique des résultats, dépôt du PV signé.

**Stories :**

* `EP-003-S01` — Créer une AG avec date, type et ordre du jour ; calculer et afficher la date limite de convocation (J−21) (LEGAL.md §3.1)
* `EP-003-S02` — Joindre les documents obligatoires à la convocation et alerter en cas de pièce manquante avant envoi (LEGAL.md §3.1)
* `EP-003-S03` — Enregistrer les présences, procurations et votes par correspondance (LEGAL.md §3.2)
* `EP-003-S04` — Saisir les résolutions et calculer les résultats selon la règle de majorité applicable (art. 24/25/25-1/26/unanimité) (LEGAL.md §3.2)
* `EP-003-S05` — Générer et stocker le PV signé en séance (LEGAL.md §3.3)
* `EP-003-S06` — Tracer la date de notification du PV aux copropriétaires absents (délai 2 mois — art. 42 loi 1965) (LEGAL.md §3.3)
* `EP-003-S07` — Exclure explicitement les lots archivés des calculs de quorum et de majorité avec piste d'audit des bases de calcul (LEGAL.md §3.2)

***

### EP-004 — Budget & Charges

**Statut :** en cours | **MVP :** oui | **Réf. PRD :** Fondations légales

**État code actuel :** backend réel sur les flux principaux (génération appels, snapshots, relevé individuel) ; frontend budget encore en mock ; certaines actions backend restent minimales.

Budget prévisionnel par exercice, appels de fonds générés par tantièmes, fonds travaux obligatoire (≥ 5%), arrêté de comptes.

**Stories :**

* `EP-004-S01` — Créer et gérer un budget prévisionnel par exercice (lignes par clé de répartition)
* `EP-004-S02` — Générer les appels de fonds trimestriels au prorata des tantièmes
* `EP-004-S03` — Suivre et alimenter le fonds travaux (≥ 5% du budget prévisionnel, compte séparé)
* `EP-004-S04` — Produire l'arrêté de comptes (réalisé vs prévisionnel)
* `EP-004-S05` — Snapshot des tantièmes dans chaque appel de fonds généré (`shares_snapshot`, `shares_total_snapshot`, `key_id`) — immuabilité comptable (LEGAL.md §2.3, ADR-006)
* `EP-004-S06` — Relevé individuel de charges par lot : liste des appels reçus, montants, statuts, solde courant (LEGAL.md §4.6)
* `EP-004-S07` — Blocage dur côté backend de toute génération d'appel de fonds tant qu'une clé active est déséquilibrée (`Σ tantièmes ≠ total_clé`) (LEGAL.md §2.1)
* `EP-004-S08` — Émission trimestrielle paramétrable et contrôles d'échéances par exercice (quart par défaut) avec piste d'audit (LEGAL.md §4.3)
* `EP-004-S09` — Implémenter le mapping plan comptable copropriété (arrêté 14/03/2005) sur lignes budgétaires et exports comptables (LEGAL.md §4.2)
* `EP-004-S10` — Contrôle et alerte du seuil minimum fonds travaux (>= 5% du budget prévisionnel) avant validation de l'exercice (LEGAL.md §4.4)

***

### EP-005 — Documents

**Statut :** en cours | **MVP :** oui | **Réf. PRD :** Gestion courante

**État code actuel :** backend réel + frontend branché pour listing, upload pré-signé, confirmation, téléchargement et suppression.

Upload direct S3 via pre-signed URL, contrôle d'accès par rôle, téléchargement sécurisé sans exposition publique du bucket.

**Stories :**

* `EP-005-S01` — Déposer un document via URL pré-signée sans transit par le backend
* `EP-005-S02` — Associer un document à une entité métier (copropriété, AG, contrat, sinistre, tâche)
* `EP-005-S03` — Lister, filtrer et rechercher les documents par catégorie, entité et visibilité
* `EP-005-S04` — Appliquer les droits d’accès par rôle sur la consultation et le téléchargement
* `EP-005-S05` — Télécharger un document via un accès sécurisé sans exposition publique du bucket
* `EP-005-S06` — Typer et verrouiller les documents fondateurs (`Règlement`, `EDD`) avec version courante explicite et historique consultable (LEGAL.md §1.2)
* `EP-005-S07` — Catégoriser et archiver les pièces réglementaires techniques (`DPE collectif`, `DTG`, `PPT`, `PV`) avec règles de conservation adaptées (LEGAL.md §3.3, §6.1, §6.2, §6.3)

***

### EP-006 — Contrats fournisseurs

**Statut :** terminée | **MVP :** oui | **Réf. PRD :** Gestion courante

**État code actuel :** backend + frontend réels sur CRUD, rappels, acquittement, résiliation et pièces justificatives liées au contrat.

Fiche contrat avec reconduction tacite et préavis configurables, rappels automatiques calculés à la création/MAJ, acquittement manuel.

**Stories :**

* `EP-006-S01` — Créer et gérer une fiche contrat (prestataire, objet, montant, dates de début/fin)
* `EP-006-S02` — Configurer la reconduction tacite et le préavis de résiliation par contrat (LEGAL.md §5.3)
* `EP-006-S03` — Calculer automatiquement les dates d'alerte : préavis entrant, reconduction imminente, expiration (LEGAL.md §5.3)
* `EP-006-S04` — Acquitter manuellement une alerte ou enregistrer la résiliation d'un contrat
* `EP-006-S05` — Suivre le contrat d'assurance multirisque immeuble (échéance, pièces, rappels) en tant qu'aide de gestion documentaire (LEGAL.md §5.2)

***

### EP-007 — Tâches & Prestataires

**Statut :** terminée | **MVP :** non | **Réf. PRD :** Gestion courante

**État code actuel :** backend réel (CRUD tâches + token prestataire + upload staging + soumission en transaction) + frontend branché (liste, créer, token, page publique prestataire).

Tâches assignées en interne, lien tokenisé éphémère pour accès prestataire sans compte, flux de dépôt staging → validation → bucket final.

**Stories :**

* `EP-007-S01` ✅ — Créer et assigner une tâche à un intervenant interne ou à un prestataire externe
* `EP-007-S02` ✅ — Générer un lien tokenisé éphémère pour l'accès prestataire sans compte
* `EP-007-S03` ✅ — Permettre au prestataire de déposer des documents via le lien tokenisé (zone staging) — `POST /vendor/:token/upload-url`
* `EP-007-S04` ✅ — Valider et archiver les documents déposés par le prestataire (bucket final) — dans la transaction `POST /vendor/:token/submission`
* `EP-007-S05` ✅ — Clôturer une tâche et l'alimenter automatiquement au carnet d'entretien (→ EP-009-S03) — idempotent via unique index

***

### EP-008 — Sinistres

**Statut :** en cours | **MVP :** oui | **Réf. PRD :** Gestion courante

**État code actuel :** stub/mock ; API sinistres non implémentée (`501`) et vue encore alimentée par données fictives.

Déclaration et suivi de sinistres (lot ou parties communes) avec documents associés.

**Stories :**

* `EP-008-S01` — Déclarer un sinistre avec type, description, date de survenance et documents associés
* `EP-008-S02` — Calculer et afficher le délai légal de déclaration restant selon le type de sinistre (LEGAL.md §9)
* `EP-008-S03` — Alerter automatiquement si le délai légal de déclaration approche (LEGAL.md §9)
* `EP-008-S04` — Suivre le statut du sinistre jusqu'à transmission à l'assurance
* `EP-008-S05` — Tracer la date de transmission à l'assurance et contrôler le respect du délai légal selon le type de sinistre (LEGAL.md §9)

***

### EP-009 — Carnet d'entretien

**Statut :** terminée (S01-S03, S05) | **MVP :** oui | **Réf. PRD :** Gestion courante

**État code actuel :** backend réel (list, get, create, update, delete + auto-feed depuis tâche) + frontend branché (timeline, modale création, suppression).

Historique des interventions alimenté manuellement ou automatiquement depuis une tâche clôturée.

**Stories :**

* `EP-009-S01` ✅ — Consulter le carnet d'entretien (liste chronologique des interventions)
* `EP-009-S02` ✅ — Ajouter manuellement une entrée d'intervention (date, description, prestataire, coût)
* `EP-009-S03` ✅ — Alimenter automatiquement le carnet à la clôture d'une tâche (← EP-007-S05) — idempotent via `ON CONFLICT DO NOTHING`
* `EP-009-S04` ❌ (hors scope) — PPT : attacher le Plan Pluriannuel de Travaux et lier ses postes au carnet d'entretien (LEGAL.md §6.1) — différé post-MVP
* `EP-009-S05` ✅ — Registre minimal des contrats et diagnostics liés à l'entretien : champ `contract_id` sur `maintenance_log` + champ `task_id` pour la traçabilité prestataire (LEGAL.md §1.4)

***

### EP-010 — Consultations & Sondages

**Statut :** terminée | **MVP :** oui | **Réf. PRD :** Participatif

**État code actuel :** backend réel (list, get, create as 'open', respond/upsert 1 réponse par utilisateur, results, close) + frontend branché (liste open/closed, modale create, modale respond, clôture, résultats).

Consultations initiables par tout résident sans approbation préalable, résultats en temps réel, mention obligatoire "non décisionnelle".

**Contrainte légale :** mention `INFORMAL_DISCLAIMER` obligatoire sur toutes les réponses API et dans la vue (loi 10 juillet 1965, art. 17 et 24).

**Stories :**

* `EP-010-S01` ✅ — Créer une consultation ou un sondage avec question(s) et options de réponse (minimum 2 options, `visible_to_roles` requis)
* `EP-010-S02` ✅ — Soumettre un vote depuis l'espace résident (une réponse par utilisateur — `UNIQUE(consultation_id, user_id)` + upsert) ; contrôle `visible_to_roles`
* `EP-010-S03` ✅ — Afficher les résultats en temps réel avec mention obligatoire « non décisionnelle »

***

### EP-011 — Signalements

**Statut :** en cours | **MVP :** oui | **Réf. PRD :** Participatif

**État code actuel :** stub/mock ; API signalements non implémentée (`501`) et vue encore alimentée par données fictives.

Signalement par tout résident avec photo(s), suivi de statut par le syndic, conversion possible en tâche.

**Stories :**

* `EP-011-S01` — Créer un signalement avec description, localisation et photo(s)
* `EP-011-S02` — Suivre et mettre à jour le statut d'un signalement (reçu, en cours, résolu)
* `EP-011-S03` — Convertir un signalement en tâche (→ EP-007-S01)

***

### EP-012 — Conformité & Reporting réglementaire

**Statut :** en attente | **MVP :** non | **Réf. LEGAL :** §1.1, §1.3

Export des données de la copropriété au format RNC et génération de la fiche synthétique annuelle obligatoire.

**Stories :**

* `EP-012-S01` — Exporter les données de la copropriété au format attendu par le Registre National des Copropriétés (RNC) pour faciliter la mise à jour annuelle (LEGAL.md §1.1)
* `EP-012-S02` — Générer la fiche synthétique annuelle de copropriété (nombre de lots, budget, données techniques) (LEGAL.md §1.3)
* `EP-012-S03` — Générer un dossier de déclaration annuelle RNC prêt à dépôt (contrôles de complétude + export + journal horodaté) (LEGAL.md §1.1)

***

## Plan de couverture ciblé (LEGAL — toutes sections)

| Réf. LEGAL | Couverture backlog | Écart restant |
|---|---|---|
| §1.1 Immatriculation RNC | EP-012-S01, EP-012-S03 | Dépôt RNC reste manuel (hors API publique gouvernementale) |
| §1.2 Règlement + EDD | EP-005-S06, EP-001-S09, EP-001-S10 | Contrôle documentaire avancé à implémenter |
| §1.3 Fiche synthétique | EP-012-S02 | Génération annuelle non implémentée |
| §1.4 Carnet d'entretien | EP-009-S01/S02/S03/S05 | API + UI encore en mode stub/mock |
| §2.1 Intégrité somme tantièmes | EP-001-S07, EP-004-S07 | Blocage dur backend à finaliser bout-en-bout |
| §2.2 Historique tantièmes | EP-001-S06 | Couvert côté modèle ; contrôles de restitution à renforcer |
| §2.3 Snapshot appels | EP-004-S05 | Couvert ; vérifier adoption sur tous flux d'appels |
| §2.4 Tantièmes et mutations | EP-002-S08, EP-002-S06/S07 | Workflows notariaux à implémenter |
| §3.1 Convocation AG (J-21 + pièces) | EP-003-S01/S02 | Front AG encore en mode mock ; contrôle de complétude à finaliser |
| §3.2 Règles de majorité | EP-003-S03/S04/S07 | Vérifier calculs sur tous cas limites (25-1, unanimité) |
| §3.3 PV + notification absents | EP-003-S05/S06, EP-005-S07 | Workflow de notification probante à compléter |
| §4.1 Compte bancaire séparé | Hors périmètre logiciel | Information uniquement ; pas de flux bancaire |
| §4.2 Plan comptable normé | EP-004-S09 | Mapping comptable non implémenté |
| §4.3 Budget + appels de fonds | EP-004-S01/S02/S08 | Front budget encore en mock ; validations métier à compléter |
| §4.4 Fonds travaux >= 5% | EP-004-S03/S10 | Contrôle automatique du seuil à finaliser |
| §4.5 Arrêté de comptes | EP-004-S04 | Écrans et exports d'arrêté à finaliser |
| §4.6 Relevé individuel de charges | EP-004-S06 | Couvert backend ; UX front à industrialiser |
| §5.1 Contrat de syndic | Hors périmètre logiciel | Pas de gestion du mandat syndic |
| §5.2 Assurance multirisque | EP-006-S05 | Suivi documentaire et alertes à implémenter |
| §5.3 Reconduction contrats | EP-006-S01/S02/S03/S04 | Front contrats encore en mode mock |
| §6.1 Plan pluriannuel de travaux (PPT) | EP-009-S04, EP-005-S07 | Fonctionnalité non implémentée |
| §6.2 DPE collectif | EP-005-S07 | Gestion documentaire à implémenter |
| §6.3 DTG | EP-005-S07 | Gestion documentaire à implémenter |
| §7.1 État daté (15 jours) | EP-002-S06 | Workflow non implémenté |
| §7.2 Pré-état daté | EP-002-S07 | Workflow non implémenté |
| §8 RGPD (anonymisation + export) | EP-002-S05, EP-002-S09 | Export + anonymisation bout-en-bout à finaliser |
| §9 Sinistres (délais) | EP-008-S01/S02/S03/S04/S05 | API + UI encore en mode stub/mock |

***

### EP-015 — Paiement en ligne

**Statut :** en attente | **MVP :** non | **Réf. PRD :** Fonctionnalités avancées

Paiement des appels de fonds via passerelle SEPA, reçu PDF automatique, versement sur compte séparé (art. 18 loi 1965).

**Stories :** _à définir_

***

### EP-013 — Tenue d'AG

**Statut :** en attente | **MVP :** non | **Réf. PRD :** Fonctionnalités avancées — extension EP-003

Conduite de séance : désignation des rôles (président, secrétaire, scrutateurs), feuille de présence numérique, quorum en temps réel, notes collaboratives, génération et validation du PV avec signature numérique.

**Stories :** _à définir_

***

### EP-014 — Calendrier copropriété

**Statut :** en attente | **MVP :** non | **Réf. PRD :** Fonctionnalités avancées

Calendrier partagé centralisant AG, interventions et réunions avec liens contextuels vers les entités correspondantes, export iCal.

**Stories :** _à définir_

***

## Tâches techniques post-MVP

> À traiter en priorité une fois la livraison du MVP validée. Ces tâches ne sont pas des fonctionnalités produit mais des améliorations structurelles de la base de code.

| # | Tâche | Détail | Statut |
|---|-------|--------|--------|
| T-001 | Fusionner les migrations SQL en un fichier unique | Consolider `0001_init.sql` et `0002_tantieme_audit.sql` (et toute migration ultérieure) en un seul fichier `0001_init.sql` pour simplifier l'initialisation d'une nouvelle instance. | en attente |

***

## Fonctionnalités hors scope (non planifiées)

Ces éléments ont été identifiés mais ne font pas l'objet d'une epic active. Ils peuvent être reclassés à la demande.

| Fonctionnalité                | Justification                                                      |
| ----------------------------- | ------------------------------------------------------------------ |
| Notifications email/push      | Infrastructure supplémentaire — `jobs/reminders.rs` prêt à activer |
| App mobile native             | SPA responsive suffisant pour le périmètre bénévole                |
| API publique documentée       | Pas de cas d'usage identifié                                       |
| Multi-copropriétés par compte | Architecture prête, non exposée — décision post-MVP                |
| SSO / fédération d'identité   | Keycloak rejeté (ADR-002)                                          |
| OCR/NLP documents             | Hook prévu (`domain/processor.rs`) — post-MVP                      |

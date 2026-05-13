# EPICS.md — l'Application

> **Dénomination** — Dans le présent document, l'expression **« l'Application »** désigne exclusivement **La Ruche**, plateforme web open source de gestion participative de copropriété. Ces deux termes sont interchangeables et désignent la même entité logicielle.

## Backlog produit

> Ce document est le backlog officiel du projet.
> Chaque feature est organisée en epic. Les stories sont ajoutées à la demande, au moment de l'implémentation.
> Le MVP est l'ensemble des epics marquées **MVP : oui**.
> Statuts possibles : **en attente** · **en cours** · **terminée**
> Dernière mise à jour : 5 mai 2026

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

* `EP-001-S01` ✅ — Créer et modifier la fiche identité de la copropriété (nom, adresse, SIRET syndic, coordonnées de gestion)
* `EP-001-S02` ✅ — Importer, stocker et consulter le règlement de copropriété et les documents fondateurs
* `EP-001-S03` ✅ — Définir les clés de répartition et leurs règles de calcul
* `EP-001-S04` ✅ — Créer, modifier, archiver et consulter les lots avec leurs tantièmes par clé
* `EP-001-S05` ✅ — Afficher un registre consolidé copropriété + lots + tantièmes exploitable par les autres modules
* `EP-001-S06` ✅ — Versioning des tantièmes : toute modification d'un `share_value` ferme l'enregistrement courant et crée un nouvel enregistrement daté (`valid_from`/`valid_to`) — immuabilité historique (LEGAL.md §2.2, ADR-006)
* `EP-001-S07` ✅ — Alerte de déséquilibre : bloquer la génération d'appels de fonds si `Σ tantièmes ≠ total_clé` (LEGAL.md §2.1)
* `EP-001-S08` ✅ — Clé Charges Générales système : `key_type` enum (`general` / `special`) + soft delete sur `allocation_key` ; créée automatiquement à l'onboarding, non supprimable (ADR-007, LEGAL.md §2.1)
* `EP-001-S09` — Contrôle de cohérence EDD : comparer les tantièmes configurés avec le dernier EDD déposé et produire un rapport d'écarts (non bloquant) (LEGAL.md §1.2)
* `EP-001-S10` — Journal d'audit des modifications de clés (qui, quoi, quand, justification) pour traçabilité syndic (LEGAL.md §1.2, §2.1)

***

### EP-002 — Gestion des propriétaires

**Statut :** en cours | **MVP :** oui | **Réf. PRD :** Fondations légales

**État code actuel :** partiel ; lecture des utilisateurs et base d'invitation présentes, mais les flux coeur propriétaires/mutations restent non livrés côté produit (`/units/{id}/owners` et `/units/{id}/transfer` encore en `501 NOT_IMPLEMENTED`).

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
* `EP-002-S10` — Enregistrer et suivre les procédures judiciaires en cours impliquant le syndicat (demandeur ou défendeur), avec date d'ouverture, type, statut ; requis pour l'état daté (LEGAL.md §7.1, §10)

***

### EP-003 — Assemblée Générale

**Statut :** en cours | **MVP :** oui | **Réf. PRD :** Fondations légales

**État code actuel :** backend majoritairement réel ; frontend AG désormais branché sur l'API réelle pour la liste, la fiche, la création/modification d'AG draft, l'ajout de résolutions, la gestion des pièces de convocation et la feuille de présence par lot. Restent à finaliser : dépôt du PV / notification (`S05`/`S06`) et durcissement juridique (`S07`).

Cycle de vie complet d'une AG, résolutions typées par majorité, vote par lot avec procurations, calcul automatique des résultats, dépôt du PV signé.

**Stories :**

* `EP-003-S01` ✅ — Créer une AG avec date, type et ordre du jour ; calculer et afficher la date limite de convocation (J−21) (LEGAL.md §3.1)
* `EP-003-S02` ✅ — Joindre les documents obligatoires à la convocation et alerter en cas de pièce manquante avant envoi (LEGAL.md §3.1)
* `EP-003-S03` ✅ — Enregistrer les présences, procurations et votes par correspondance (LEGAL.md §3.2)
* `EP-003-S04` ✅ — Saisir les résolutions et les votes, et calculer les résultats selon la règle de majorité applicable (art. 24/25/25-1/26/26-1/unanimité) (LEGAL.md §3.2)
* `EP-003-S05` — Générer et stocker le PV signé en séance (LEGAL.md §3.3)
* `EP-003-S06` — Tracer la date de notification du PV aux copropriétaires absents (délai 2 mois — art. 42 loi 1965) (LEGAL.md §3.3)
* `EP-003-S07` ✅ — Exclure explicitement les lots archivés des calculs de quorum et de majorité avec piste d'audit des bases de calcul (LEGAL.md §3.2)
* `EP-003-S08` ✅ — Détecter et traiter l'article 26-1 : si une résolution `double_majority` est rejetée avec > 1/2 des tantièmes totaux en faveur, proposer immédiatement une re-délibération à la majorité absolue (art. 26-1) dans la même séance via une nouvelle résolution liée (LEGAL.md §3.2)

**Hors scope explicite (renvoyé à `EP-013` ou à des epics transverses) :**

- désignation des rôles de séance (`président`, `secrétaire`, `scrutateurs`) ;
- feuille de présence avancée avec quorum temps réel de conduite de séance ;
- prise de notes collaborative par résolution ;
- génération du projet de PV et circuit de validation ;
- signature numérique / signature électronique du PV ;
- workflow d'envoi probant multi-canal aux absents ;
- refonte transverse du module documents au-delà des besoins minimaux `S02` / `S05`.

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
* `EP-004-S05` ✅ — Snapshot des tantièmes dans chaque appel de fonds généré (`shares_snapshot`, `shares_total_snapshot`, `key_id`) — immuabilité comptable (LEGAL.md §2.3, ADR-006)
* `EP-004-S06` ✅ — Relevé individuel de charges par lot : liste des appels reçus, montants, statuts, solde courant (LEGAL.md §4.6)
* `EP-004-S07` — Blocage dur côté backend de toute génération d'appel de fonds tant qu'une clé active est déséquilibrée (`Σ tantièmes ≠ total_clé`) (LEGAL.md §2.1)
* `EP-004-S08` — Émission trimestrielle paramétrable et contrôles d'échéances par exercice (quart par défaut) avec piste d'audit (LEGAL.md §4.3)
* `EP-004-S09` — Implémenter le mapping plan comptable copropriété (arrêté 14/03/2005) sur lignes budgétaires et exports comptables (LEGAL.md §4.2)
* `EP-004-S10` — Contrôle et alerte du seuil minimum fonds travaux (>= 5% du budget prévisionnel) avant validation de l'exercice (LEGAL.md §4.4)
* `EP-004-S11` — Recouvrement des charges impayées : identification des lots en solde débiteur, enregistrement des relances (mise en demeure), suivi des dossiers contentieux jusqu'à clôture (LEGAL.md §4.7)

***

### EP-005 — Documents

**Statut :** en cours | **MVP :** oui | **Réf. PRD :** Gestion courante

**État code actuel :** backend réel + frontend branché pour listing, upload pré-signé, confirmation, téléchargement et suppression ; pas encore de modèle documentaire dédié pour les documents fondateurs/version courante ni pour l'archivage réglementaire typé.

Upload direct S3 via pre-signed URL, contrôle d'accès par rôle, téléchargement sécurisé sans exposition publique du bucket.

**Stories :**

* `EP-005-S01` ✅ — Déposer un document via URL pré-signée sans transit par le backend
* `EP-005-S02` ✅ — Associer un document à une entité métier (copropriété, AG, contrat, sinistre, tâche)
* `EP-005-S03` ✅ — Lister, filtrer et rechercher les documents par catégorie, entité et visibilité
* `EP-005-S04` ✅ — Appliquer les droits d’accès par rôle sur la consultation et le téléchargement
* `EP-005-S05` ✅ — Télécharger un document via un accès sécurisé sans exposition publique du bucket
* `EP-005-S06` — Typer et verrouiller les documents fondateurs (`Règlement`, `EDD`) avec version courante explicite et historique consultable (LEGAL.md §1.2)
* `EP-005-S07` — Catégoriser et archiver les pièces réglementaires techniques (`DPE collectif`, `DTG`, `PPT`, `PV`) avec règles de conservation adaptées (LEGAL.md §3.3, §6.1, §6.2, §6.3)

***

### EP-006 — Contrats fournisseurs

**Statut :** terminée | **MVP :** oui | **Réf. PRD :** Gestion courante

**État code actuel :** backend + frontend réels sur CRUD, rappels, acquittement, résiliation et pièces justificatives liées au contrat.

Fiche contrat avec reconduction tacite et préavis configurables, rappels automatiques calculés à la création/MAJ, acquittement manuel.

**Stories :**

* `EP-006-S01` ✅ — Créer et gérer une fiche contrat (prestataire, objet, montant, dates de début/fin)
* `EP-006-S02` ✅ — Configurer la reconduction tacite et le préavis de résiliation par contrat (LEGAL.md §5.3)
* `EP-006-S03` ✅ — Calculer automatiquement les dates d'alerte : préavis entrant, reconduction imminente, expiration (LEGAL.md §5.3)
* `EP-006-S04` ✅ — Acquitter manuellement une alerte ou enregistrer la résiliation d'un contrat
* `EP-006-S05` ✅ — Suivre le contrat d'assurance multirisque immeuble (échéance, pièces, rappels) en tant qu'aide de gestion documentaire (LEGAL.md §5.2)

***

### EP-007 — Tâches & Prestataires

**Statut :** terminée | **MVP :** non | **Réf. PRD :** Gestion courante

**État code actuel :** backend réel (CRUD tâches + token prestataire + upload staging + pré-validation prestataire en transaction + documents manager via `task_document` + assignation prestataire référentiel ou nom libre sur `task.contractor_name`) + frontend branché (liste, créer, modifier, documents manager, token, page publique prestataire, autocomplete prestataire tolérant avec création optionnelle, clôture avec coût optionnel).

Tâches assignées en interne, lien tokenisé éphémère pour accès prestataire sans compte, flux de dépôt staging → validation → bucket final. Les tâches peuvent être liées à un prestataire du référentiel (EP-015) pour consolider l'historique, ou conserver une saisie libre si aucun prestataire n'est retenu.

**Stories :**

* `EP-007-S01` ✅ — Créer et assigner une tâche à un intervenant interne ou à un prestataire externe
* `EP-007-S02` ✅ — Générer un lien tokenisé éphémère pour l'accès prestataire sans compte
* `EP-007-S03` ✅ — Permettre au prestataire de déposer des documents via le lien tokenisé (zone staging) — `POST /vendor/:token/upload-url`
* `EP-007-S04` ✅ — Valider et archiver les documents déposés par le prestataire (bucket final) puis passer la tâche en pré-validation (`awaiting_validation`) avec commentaire et coût optionnel — dans la transaction `POST /vendor/:token/submission`
* `EP-007-S05` ✅ — Clôturer une tâche avec coût optionnel et l'alimenter automatiquement au carnet d'entretien (→ EP-009-S03), y compris après réouverture ; inclut `vendor_name` du prestataire assigné
* `EP-007-S06` ✅ — Assigner un prestataire du référentiel (EP-015) à une tâche via autocomplete, avec suggestion de création optionnelle si aucune correspondance n'existe et conservation d'un nom libre sur la tâche si aucun prestataire n'est créé — `contractor_id` FK sur `task` + `task.contractor_name`

***

### EP-008 — Sinistres

**Statut :** terminée | **MVP :** oui | **Réf. PRD :** Gestion courante

**État code actuel :** backend réel (`claim_type`, `occurred_at`, `published_at`, `transmitted_at`, remboursement optionnel, calcul d'échéance légale, alertes dérivées, pièces jointes et nettoyage des liaisons documentaires à la suppression) + frontend branché sur API réelle (création, édition, transmission, clôture, suppression, alertes, rattachement documentaire).

Déclaration et suivi de sinistres (lot ou parties communes) avec documents associés.

**Stories :**

* `EP-008-S01` ✅ — Déclarer un sinistre avec type, description, date de survenance et documents associés
* `EP-008-S02` ✅ — Calculer et afficher le délai légal de déclaration restant selon le type de sinistre (LEGAL.md §9)
* `EP-008-S03` ✅ — Alerter automatiquement si le délai légal de déclaration approche (LEGAL.md §9)
* `EP-008-S04` ✅ — Suivre le statut du sinistre avec actions contextuelles (`Modifier`, `Transmettre`, `Clore`, `Supprimer`) selon l'état courant
* `EP-008-S05` ✅ — Tracer la date de transmission à l'assurance, le remboursement éventuel et contrôler le respect du délai légal selon le type de sinistre (LEGAL.md §9)

***

### EP-009 — Carnet d'entretien

**Statut :** terminée (`S04` hors scope post-MVP) | **MVP :** oui | **Réf. PRD :** Gestion courante

**État code actuel :** backend réel (list, get, create, update, delete + auto-feed depuis tâche avec liens de traçabilité souples + résumé réglementaire) + frontend branché (timeline, résumé réglementaire distinct, modale création, suppression, filtres calendaires, listings contrats assurance / maintenance actifs).

Historique des interventions alimenté manuellement ou automatiquement depuis une tâche clôturée. La vue regroupe séparément les mentions minimales obligatoires du décret n°2001-477 (adresse immeuble, identité syndic, contrats d'assurance et d'entretien) au-dessus de la timeline.

**Stories :**

* `EP-009-S01` ✅ — Consulter le carnet d'entretien (liste chronologique des interventions)
* `EP-009-S02` ✅ — Ajouter manuellement une entrée d'intervention (date, description, prestataire, coût)
* `EP-009-S03` ✅ — Alimenter automatiquement le carnet à chaque clôture effective d'une tâche (← EP-007-S05), y compris après réouverture, avec coût optionnel et sans cascade destructive tâche ↔ carnet
* `EP-009-S04` ❌ (hors scope) — PPT : attacher le Plan Pluriannuel de Travaux et lier ses postes au carnet d'entretien (LEGAL.md §6.1) — différé post-MVP
* `EP-009-S05` ✅ — Registre minimal des contrats et diagnostics liés à l'entretien : champ `contract_id` sur `maintenance_log` + champ `task_id` pour la traçabilité prestataire (LEGAL.md §1.4)
* `EP-009-S06` ✅ — En-tête réglementaire du carnet : afficher dans la vue carnet l'adresse de l'immeuble et l'identité du syndic en exercice (depuis EP-001-S01), conformément au décret n°2001-477 art. 1 (LEGAL.md §1.4)
* `EP-009-S07` ✅ — Référencer dans le carnet les contrats d'assurance souscrits par le syndicat (multirisque et dommages-ouvrage) avec leurs dates d'échéance, en lien avec EP-006-S05 (LEGAL.md §1.4)
* `EP-009-S08` ✅ — Lister dans le carnet les contrats d'entretien et de maintenance d'équipements communs en cours (ascenseur, chaufferie…) avec leurs dates d'échéance, en lien avec EP-006 (LEGAL.md §1.4)

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

**Statut :** terminée | **MVP :** oui | **Réf. PRD :** Participatif

**État code actuel :** backend réel (CRUD, documents associés, conversion en tâche avec priorité choisie, lien `report.task_id`, clôture automatique du signalement à la fermeture effective de la tâche liée) + frontend branché (liste, création, photos, mise à jour de statut, conversion en tâche).

Signalement par tout résident avec description et lot optionnel, suivi de statut par le syndic, conversion possible en tâche.

**Stories :**

* `EP-011-S01` ✅ — Créer un signalement (titre, description, lot concerné)
* `EP-011-S02` ✅ — Suivre et mettre à jour le statut (reçu → en cours → clos) — manager/conseil syndical
* `EP-011-S03` ✅ — Convertir un signalement en tâche — crée une tâche avec titre/description + priorité choisie depuis le signalement, passe automatiquement le signalement en `in_progress`, puis le clôture quand la tâche liée est effectivement fermée

***

### EP-012 — Conformité & Reporting réglementaire

**Statut :** en attente | **MVP :** non | **Réf. LEGAL :** §1.1, §1.3

Export des données de la copropriété au format RNC et génération de la fiche synthétique annuelle obligatoire.

**Stories :**

* `EP-012-S01` — Exporter les données de la copropriété au format attendu par le Registre National des Copropriétés (RNC) pour faciliter la mise à jour annuelle (LEGAL.md §1.1)
* `EP-012-S02` — Générer la fiche synthétique annuelle de copropriété (nombre de lots, budget, données techniques) (LEGAL.md §1.3)
* `EP-012-S03` — Générer un dossier de déclaration annuelle RNC prêt à dépôt (contrôles de complétude + export + journal horodaté) (LEGAL.md §1.1)

***

### EP-013 — Tenue d’AG

**Statut :** en attente | **MVP :** non | **Réf. PRD :** Fonctionnalités avancées — extension EP-003

**État code actuel :** statut `ongoing` disponible (migration `0019_meeting_ongoing_status`) ; transitions `POST /{id}/start` (`called` → `ongoing`) et `POST /{id}/close` (`ongoing` → `closed`) opérationnelles backend. Aucune vue frontend dédiée à la conduite de séance n'existe encore.

Conduite de séance : désignation des rôles (président, secrétaire, scrutateurs), feuille de présence numérique, quorum en temps réel, notes collaboratives, génération et validation du PV avec signature numérique. `EP-013` démarre là où `EP-003` s'arrête : préparation juridique, votes et clôture simple restent dans `EP-003`, tandis que l'assistance complète de séance et la validation probante du PV relèvent de cette extension post-MVP.

Le cycle de vie complet d'une AG est : `draft` → `called` → **`ongoing`** → `closed`.

**Stories :**

* `EP-013-S01` — Démarrer la séance : faire passer une AG du statut `called` à `ongoing` via `POST /{id}/start` ; verrouiller l'ordre du jour et l'assiette de tantièmes à l'instant de démarrage ; exposer le statut `ongoing` dans la liste et la fiche AG
* `EP-013-S02` — Désigner les rôles de séance : saisir le président, le secrétaire et les scrutateurs lors du démarrage (`status = ongoing`) ; données figées à la clôture de séance (`status = closed`)
* `EP-013-S03` — Feuille de présence numérique en temps réel : afficher le quorum atteint (tantièmes représentés / total, nombre de copropriétaires participants) dynamiquement à mesure de l'enregistrement des présences, procurations et votes par correspondance ; mise à jour sans rechargement de page
* `EP-013-S04` — Conduire le vote résolution par résolution : en `status = ongoing`, saisir les positions (pour / contre / abstention / non-votant) lot par lot pour chaque résolution et calculer le résultat automatiquement selon la règle de majorité applicable (art. 24 / 25 / 26 / unanimité)
* `EP-013-S05` — Détecter et traiter l'article 25-1 : si une résolution `absolute_majority` est rejetée avec ≥ 1/3 des tantièmes totaux en faveur, proposer immédiatement une re-délibération à la majorité simple (art. 24) dans la même séance ; tracer la résolution originale et sa re-délibération
* `EP-013-S06` — Prendre des notes collaboratives par résolution : saisir des annotations de séance (arguments, amendements de forme) attachées à chaque résolution ; accessibles dans le projet de PV
* `EP-013-S07` — Clôturer la séance : faire passer l'AG de `ongoing` à `closed` via `POST /{id}/close` ; figer les votes et la feuille de présence ; bloquer toute modification ultérieure des résultats
* `EP-013-S08` — Générer et valider le projet de PV : après clôture de séance (`status = closed`), produire un projet de PV structuré (résolutions + résultats + présences + notes) et le soumettre à revue avant signature
* `EP-013-S09` — Signer et publier le PV : finaliser le PV avec signature numérique, l'attacher à l'AG et déclencher la traçabilité de notification aux absents (→ EP-003-S06)
* `EP-013-S10` — Détecter et traiter l'article 26-1 : si une résolution `double_majority` est rejetée avec ≥ 1/2 des tantièmes totaux en faveur, proposer immédiatement une re-délibération à la majorité absolue (art. 25) dans la même séance ; tracer la résolution originale et sa re-délibération

***

### EP-014 — Calendrier copropriété

**Statut :** en attente | **MVP :** non | **Réf. PRD :** Fonctionnalités avancées

Calendrier partagé centralisant AG, interventions et réunions avec liens contextuels vers les entités correspondantes, export iCal.

**Stories :** _à définir_

***

### EP-015 — Référentiel prestataires

**Statut :** en cours | **MVP :** non | **Réf. PRD :** Gestion courante

**État code actuel :** backend réel (CRUD + recherche par nom) + frontend branché (catalogue, création, édition, suppression, autocomplete dans TasksView, création optionnelle depuis la modale tâche).

Catalogue des prestataires de la résidence : nom, email, téléphone, domaine professionnel. Réutilisable lors de l'assignation d'une tâche (autocomplete) et pré-rempli à la migration vers le carnet d'entretien. Accessible en lecture à tous les utilisateurs, modifiable par les managers et membres du conseil syndical.

**Stories :**

* `EP-015-S01` ✅ — Gérer le référentiel prestataires (CRUD) : nom, email, téléphone, domaine — `GET/POST /contractors`, `GET/PUT/DELETE /contractors/:id`
* `EP-015-S02` ✅ — Rechercher un prestataire par nom (autocomplete, `?q=`) pour l'assigner à une tâche
* `EP-015-S03` ✅ — Catalogue accessible à tous les utilisateurs authentifiés de la résidence (lecture), modifiable par manager/conseil syndical
* `EP-015-S04` ✅ — Propagation du nom de prestataire au carnet d'entretien lors de la clôture automatique d'une tâche (`vendor_name`)
* `EP-015-S05` ✅ — Ajouter le numéro de téléphone au profil utilisateur (`user_account.phone`) et l'exposer dans la carte contact syndic sur la page prestataire — migration 0007

***

### EP-016 — Paiement en ligne

**Statut :** en attente | **MVP :** non | **Réf. PRD :** Fonctionnalités avancées

Paiement des appels de fonds via passerelle SEPA, reçu PDF automatique, versement sur compte séparé (art. 18 loi 1965).

**Stories :** _à définir_

***

### EP-017 — Journal d’activité & Suivi de cycle de vie

**Statut :** en attente | **MVP :** non | **Réf. PRD :** Gestion courante

**Arbitrage documentaire ouvert avec `EP-018` / ADR-009 :** le terme `audit_log` est réservé à la piste probante comptable et juridique (immutabilité, `UPDATE` / `DELETE`, diff ciblé, conservation 10 ans). Pour `EP-017`, le besoin porte au contraire sur une frise métier lisible, orientée cycle de vie et expérience utilisateur. La cible documentaire privilégiée est donc une **table `activity_log` polymorphique** (`entity_type` / `entity_id`) dédiée aux événements métier, tandis que `audit_log` reste sous la responsabilité de `EP-018`.

**Contexte et évaluation du méta-objet :** Un signalement peut être « upgradeé » en tâche, créant une chaîne. Plutôt que d’introduire une entité parente (`WorkItem` / `Issue`) qui alourdirait le schéma sans gain proportionnel, l’approche retenue est une **table `activity_log` polymorphique** + un champ `linked_task_id` FK sur `report` pour matérialiser la conversion. Les tables existantes restent inchangées et ne reflètent que l’état courant. Pas de méta-objet.

Vue unifiée du cycle de vie d’un signalement et/ou d’une tâche, de la création à la résolution, basée sur un journal d’audit abstrait couvrant les deux entités. Remplace la page `TaskDetailView` statique par une timeline interactive et intègre le suivi des signalements.

**Actions auditables :**

| Entité | Action | Description |
|--------|--------|-------------|
| `report` | `created` | Signalement créé par un résident |
| `report` | `status_changed` | Changement de statut (`new` → `acknowledged` → `in_progress` → `resolved`) — payload : `old_status`, `new_status` |
| `report` | `photo_added` | Photo attachée au signalement |
| `report` | `converted_to_task` | Converti en tâche — payload : `task_id` |
| `task` | `created` | Tâche créée — payload : `source_report_id` si issue d’un signalement |
| `task` | `status_changed` | Changement de statut — payload : `old_status`, `new_status` |
| `task` | `assignee_changed` | Changement d’assignataire interne — payload : `old_assignee_id`, `new_assignee_id` |
| `task` | `contractor_assigned` | Prestataire référentiel assigné/changé — payload : `contractor_id`, `contractor_name` |
| `task` | `vendor_link_generated` | Lien prestataire externe généré |
| `task` | `document_uploaded` | Document joint par un gestionnaire |
| `task` | `vendor_submission_received` | Soumission prestataire reçue |
| `task` | `closed` | Tâche clôturée |

**Stories :**

* `EP-017-S01` — Migration : créer la table `activity_log` polymorphique (`entity_type TEXT NOT NULL`, `entity_id UUID NOT NULL`, `action TEXT NOT NULL`, `actor_id UUID REFERENCES user_account(id)`, `payload JSONB`, `created_at TIMESTAMPTZ DEFAULT NOW()`) + index `(entity_type, entity_id, created_at DESC)`
* `EP-017-S02` — Backend : émettre des événements auditables aux points clés de mutation : création, changement de statut, changement d’assignataire, attribution prestataire référentiel, génération lien vendeur, soumission vendeur, clôture (tâche) ; création, changement de statut, ajout photo, conversion (signalement). Chaque événement contient un `payload` JSONB avec ancien/nouveau valeur quand pertinent.
* `EP-017-S03` — Backend : ajouter `linked_task_id UUID REFERENCES task(id)` sur `report` pour matérialiser la conversion signalement → tâche ; endpoint `GET /tasks/:id/timeline` remontant la chaîne jusqu’au signalement source si applicable ; endpoint `GET /reports/:id/timeline` symmétrique.
* `EP-017-S04` — Frontend : vue `TaskTimelineView` (remplace `TaskDetailView`) : données statiques de la tâche (titre, statut, priorité, assignataire, prestataire) + timeline chronologique des événements depuis le signalement source (si applicable) jusqu’à la clôture.
* `EP-017-S05` — Frontend : vue `ReportDetailView` : afficher les événements auditables du signalement + lien contextuel vers la tâche créée si conversion effective.
* `EP-017-S06` — Accessibilité : événements visibles par tous les rôles ayant accès à l’entité ; actions sensibles (`assignee_changed`, `vendor_link_generated`) réservées aux rôles manager/conseil syndical.

***

### EP-018 — Immutabilité comptable & auditabilité probante

**Statut :** en attente | **MVP :** oui | **Réf. LEGAL :** §11 · **Réf. ADR :** ADR-009

**État code actuel :** non implémenté ; ADR-009 est accepté mais aucun backlog explicite ne couvrait encore les guards d'immuabilité, les workflows de correction juridique et le journal append-only associé. L'arbitrage documentaire cible est désormais le suivant : `audit_log` reste la piste probante comptable de `EP-018`, tandis que `EP-017` porte une frise métier distincte via `activity_log`.

Durcir les entités comptablement critiques pour empêcher toute modification rétroactive après état terminal, imposer les patterns `gel`, `clôturer + recréer` ou `append-only` selon les cas, et conserver une piste d'audit probante alignée sur l'ADR-009.

**Stories :**

* `EP-018-S01` — Entériner l’arbitrage documentaire avec `EP-017` : `audit_log` réservé aux mutations comptables/juridiques probantes, `activity_log` réservé aux événements métier de timeline, avec frontière de responsabilité documentée et non ambiguë
* `EP-018-S02` — Créer l'infrastructure transverse d'immutabilité : module `domain/immutability`, guards `is_mutable()` / `is_deletable()`, repository de journal append-only et points d'appel obligatoires avant mutation
* `EP-018-S03` — Sécuriser `levy_call` selon le Pattern A : ajout du statut `draft`, endpoint d'émission explicite (`POST /levy-calls/{id}/issue`), blocage des `UPDATE` / `DELETE` hors `draft` et trigger PG défensif
* `EP-018-S04` — Appliquer le Pattern A sur `task`, `claim`, `vote`, `resolution` et `maintenance_log` automatique : champs financiers figés après état terminal, suppression interdite selon les règles ADR-009, et masquage UI des actions `Modifier` / `Supprimer`
* `EP-018-S05` — Appliquer le Pattern B sur `vendor_contract` : ajouter `predecessor_id`, remplacer `Modifier` par `Créer un avenant`, clôturer l'ancien contrat à la date d'effet et exposer la chaîne d'avenants
* `EP-018-S06` — Appliquer le Pattern B sur `budget` / `budget_line` : ajouter `parent_budget_id`, remplacer l'unicité annuelle par une contrainte compatible rectificatifs, versionner `budget_line` via `valid_from` / `valid_to` et créer le workflow de budget rectificatif
* `EP-018-S07` — Appliquer le Pattern C sur les corrections comptables : avoirs sur `levy_call`, corrections append-only sur `maintenance_log` manuel, et journalisation des opérations `UPDATE` / `DELETE` financières avec diff ciblé (`old_values` / `new_values`)

***

## Plan de couverture ciblé (LEGAL — toutes sections)

| Réf. LEGAL | Couverture backlog | Écart restant |
|---|---|---|
| §1.1 Immatriculation RNC | EP-012-S01, EP-012-S03 | Dépôt RNC reste manuel (hors API publique gouvernementale) |
| §1.2 Règlement + EDD | EP-005-S06, EP-001-S09, EP-001-S10 | Contrôle documentaire avancé à implémenter |
| §1.3 Fiche synthétique | EP-012-S02 | Génération annuelle non implémentée |
| §1.4 Carnet d'entretien | EP-009-S01/S02/S03/S05/S06/S07/S08 | Couvert côté carnet ; PPT reste hors scope post-MVP |
| §1.5 Liste des copropriétaires | EP-002-S01, EP-002-S04 | Données partiellement couvertes ; champ email à compléter |
| §1.6 Espace en ligne sécurisé | EP-002/EP-003/EP-004/EP-005/EP-006/EP-009 | Couverture native (c'est l'Application elle-même) |
| §2.1 Intégrité somme tantièmes | EP-001-S07, EP-004-S07 | Blocage dur backend à finaliser bout-en-bout |
| §2.2 Historique tantièmes | EP-001-S06 | Couvert côté modèle ; contrôles de restitution à renforcer |
| §2.3 Snapshot appels | EP-004-S05 | Couvert ; vérifier adoption sur tous flux d'appels |
| §2.4 Tantièmes et mutations | EP-002-S08, EP-002-S06/S07 | Workflows notariaux à implémenter |
| §3.1 Convocation AG (J-21 + pièces) | EP-003-S01/S02 | Couvert en MVP : pièces de convocation liées à l'AG et blocage backend sans document `notice` |
| §3.2 Règles de majorité | EP-003-S03/S04/S07 | Présence par lot désormais distincte du vote ; restent à durcir les cas limites et l'exclusion des lots archivés |
| §3.3 PV + notification absents | EP-003-S05/S06, EP-005-S07 | Workflow de notification probante à compléter |
| §4.1 Compte bancaire séparé | Hors périmètre logiciel | Information uniquement ; pas de flux bancaire |
| §4.2 Plan comptable normé | EP-004-S09 | Mapping comptable non implémenté |
| §4.3 Budget + appels de fonds | EP-004-S01/S02/S08 | Front budget encore en mock ; validations métier à compléter |
| §4.4 Fonds travaux >= 5% | EP-004-S03/S10 | Contrôle automatique du seuil à finaliser |
| §4.5 Arrêté de comptes | EP-004-S04 | Écrans et exports d'arrêté à finaliser |
| §4.6 Relevé individuel de charges | EP-004-S06 | Couvert backend ; UX front à industrialiser |
| §4.7 Recouvrement des impayés | EP-004-S11 | Fonctionnalité non implémentée |
| §5.1 Contrat de syndic | Hors périmètre logiciel | Pas de gestion du mandat syndic |
| §5.2 Assurance multirisque | EP-006-S05 | Suivi documentaire et alertes à implémenter |
| §5.3 Reconduction contrats | EP-006-S01/S02/S03/S04 | Front contrats encore en mode mock |
| §6.1 Plan pluriannuel de travaux (PPT) | EP-009-S04, EP-005-S07 | Fonctionnalité non implémentée |
| §6.2 DPE collectif | EP-005-S07 | Gestion documentaire à implémenter |
| §6.3 DTG | EP-005-S07 | Gestion documentaire à implémenter |
| §7.1 État daté (15 jours) | EP-002-S06 | Workflow non implémenté |
| §7.2 Pré-état daté | EP-002-S07 | Workflow non implémenté |
| §8 RGPD (anonymisation + export) | EP-002-S05, EP-002-S09 | Export + anonymisation bout-en-bout à finaliser |
| §9 Sinistres (délais) | EP-008-S01/S02/S03/S04/S05 | Couvert : workflow réel, alertes de délai, transmission assurance et pièces jointes |
| §10 Représentation en justice + procédures | EP-002-S06, EP-002-S10 | Registre des procédures judiciaires non implémenté |
| §11 Immutabilité comptable + audit log | EP-018-S01/S02/S03/S04/S05/S06/S07 | Implémentation transversale non démarrée ; arbitrage à faire avec EP-017 sur le modèle cible de journal d'audit |

***


## Tâches techniques post-MVP

> À traiter en priorité une fois la livraison du MVP validée. Ces tâches ne sont pas des fonctionnalités produit mais des améliorations structurelles de la base de code.

| # | Tâche | Détail | Statut |
|---|-------|--------|--------|
| T-001 | Fusionner les migrations SQL en un fichier unique | Consolider `0001_init.sql` et `0002_tantieme_audit.sql` (et toute migration ultérieure) en un seul fichier `0001_init.sql` pour simplifier l'initialisation d'une nouvelle instance. | en attente |
| T-002-1 | Utilisation des composants radix-vue (ex: CModal > Dialog) | en attente |
| T-002-2 | Peaufiner les écrans, création de composants atomiques | en-attente |
| T-002-3 | Rationaliser le CSS, généralisation de tailwind, réduction des classes custom | en attente |
| T-003 | Durcir le contrat d'interface entre backend et frontend - utilisation d'openapi ? | en attente |
| T-004 | Etudier l'intérêt de la création d'un crate dédié aux AG | en attente |

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

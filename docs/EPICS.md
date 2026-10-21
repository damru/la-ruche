---
document: EPICS
version: 1.1.0
owner: Damien
status: validé
last_updated: 2026-10-21
description: >
  Backlog officiel du projet La Ruche — épics et stories organisées par domaine fonctionnel.
  Chaque story référence un PRD, un ADR ou une obligation légale (LEGAL.md §X.Y).
rules:
  - Ne jamais marquer ✅ si backend + frontend ne sont pas tous deux sur l'API réelle.
  - Aucun détail d'implémentation dans une story (pas de noms de fichiers, schémas DB, fonctions, endpoints).
  - Toute story d'origine légale doit pointer vers LEGAL.md §X.Y.
  - À chaque création d'item ou livraison effective, mettre à jour le statut de la story, de l'epic et "État code actuel".

---

# EPICS.md — l'Application

> **Dénomination** — Dans le présent document, l'expression **« l'Application »** désigne exclusivement **La Ruche**, plateforme web open source de gestion participative de copropriété. Ces deux termes sont interchangeables et désignent la même entité logicielle.

## Backlog produit

> Ce document est le backlog officiel du projet.
> Chaque feature est organisée en epic. Les stories sont ajoutées à la demande, au moment de l'implémentation.
> Le MVP est l'ensemble des epics marquées **MVP : oui**.
> Statuts possibles : **en attente** · **en cours** · **terminée**
> Dernière mise à jour : 21 octobre 2026

> **Lecture d'avancement (code actuel) :**
> - **backend réel** = handlers + DB opérationnels
> - **frontend branché** = vue connectée à l'API réelle
> - **stub/mock** = endpoint `501 NOT_IMPLEMENTED` ou vue alimentée par `src/mock/data.ts`

## Traçabilité PRD Must/Should (complément)

Objectif : garantir que chaque US de priorité Must ou Should issue des PRD produit est reliée à au moins un item du backlog EPICS.

| Priorité | US PRD | Item EPICS de rattachement |
|---|---|---|
| Must | US-AG-001 | EP-003-S01 |
| Must | US-AG-004 | EP-003-S02, EP-003-S08 |
| Must | US-AG-005 | EP-003-S01, EP-003-S08 |
| Must | US-AG-006 | EP-003-S03, EP-013-S03 |
| Must | US-AG-007 | EP-013-S01 |
| Must | US-AG-008 | EP-003-S04, EP-013-S04 |
| Must | US-AG-009 | EP-013-S07 |
| Must | US-AG-010 | EP-003-S05, EP-013-S08, EP-013-S09 |
| Must | US-AG-011 | EP-003-S06 |
| Should | US-AG-012 | EP-013-S02 |
| Should | US-AG-013 | EP-013-S05, EP-013-S10 |
| Must | US-BC-001 | EP-004-S01 |
| Must | US-BC-002 | EP-004-S01 |
| Must | US-BC-003 | EP-004-S02, EP-004-S05, EP-004-S07 |
| Must | US-BC-004 | EP-004-S08, EP-018-S03 |
| Must | US-BC-005 | EP-004-S03, EP-004-S10 |
| Must | US-BC-006 | EP-004-S06 |
| Must | US-BC-013 | EP-004-S14 |
| Must | US-BC-007 | EP-018-S07 |
| Should | US-BC-008 | EP-004-S04 |
| Should | US-BC-009 | EP-018-S06 |
| Should | US-BC-011 | EP-004-S09 |
| Must | US-CE-001 | EP-009-S01 |
| Must | US-CE-002 | EP-009-S02 |
| Must | US-CE-003 | EP-009-S03 |
| Must | US-CF-001 | EP-006-S01 |
| Must | US-CF-003 | EP-006-S03 |
| Must | US-CF-004 | EP-006-S04 |
| Must | US-CF-005 | EP-006-S04 |
| Should | US-CF-006 | EP-018-S05 |
| Should | US-CF-008 | EP-006-S05 |
| Must | US-CR-001 | EP-012-S02 |
| Must | US-CS-001 | EP-010-S01 |
| Must | US-CS-002 | EP-010-S02 |
| Must | US-CS-003 | EP-010-S03 |
| Must | US-DOC-001 | EP-005-S01 |
| Must | US-DOC-002 | EP-005-S04, EP-005-S05 |
| Must | US-DOC-003 | EP-005-S03, EP-005-S04 |
| Should | US-DOC-005 | EP-005-S02 |
| Should | US-DOC-006 | EP-005-S03 |
| Must | US-GP-005 | EP-002-S05 |
| Should | US-GP-007 | EP-002-S02 |
| Must | US-JA-001 | EP-017-S04 |
| Must | US-JA-002 | EP-017-S04, EP-017-S05 |
| Should | US-JA-003 | EP-017-S05 |
| Must | US-PP-001 | EP-019-S01 |
| Must | US-PP-006 | EP-019-S04 |
| Must | US-RC-001 | EP-001-S01 |
| Must | US-RC-002 | EP-001-S02 |
| Must | US-RC-003 | EP-001-S03 |
| Must | US-RC-004 | EP-001-S04 |
| Must | US-RC-005 | EP-001-S04, EP-001-S06 |
| Must | US-RC-006 | EP-001-S07 |
| Should | US-RC-007 | EP-001-S05 |
| Should | US-RC-008 | EP-001-S04 |
| Must | US-RP-001 | EP-015-S01 |
| Must | US-RP-002 | EP-015-S02 |
| Must | US-RP-003 | EP-007-S06, EP-006-S07 |
| Must | US-RP-005 | EP-015-S03 |
| Must | US-SIG-001 | EP-011-S01 |
| Must | US-SIG-002 | EP-011-S02 |
| Must | US-SIG-003 | EP-011-S02 |
| Must | US-SIG-004 | EP-011-S02 |
| Should | US-SIG-005 | EP-011-S03 |
| Must | US-SIN-001 | EP-008-S01, EP-008-S06 |
| Must | US-SIN-002 | EP-008-S02 |
| Must | US-SIN-003 | EP-008-S05 |
| Must | US-SIN-005 | EP-008-S04 |
| Must | US-TP-001 | EP-007-S01 |
| Must | US-TP-002 | EP-011-S03 |
| Must | US-TP-004 | EP-007-S01 |
| Must | US-TP-006 | EP-007-S06 |
| Must | US-TP-007 | EP-007-S02 |
| Must | US-TP-008 | EP-007-S07 |
| Must | US-TP-009 | EP-007-S07 |
| Must | US-TP-011 | EP-007-S04, EP-007-S05 |
| Must | US-TP-014 | EP-018-S04 |
| Must | US-TP-015 | EP-017-S01, EP-017-S02 |

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
* [EP-001-S09](docs/specs/EP-001-S09-spec.md) ✅ — Contrôle de cohérence EDD : comparer les tantièmes configurés avec le dernier EDD déposé et produire un rapport d'écarts (non bloquant) (LEGAL.md §1.2)
* [EP-001-S10](docs/specs/EP-001-S10-spec.md) ✅ — Journal d'audit des modifications de clés (qui, quoi, quand, justification) pour traçabilité syndic (LEGAL.md §1.2, §2.1)
* [EP-001-S11](docs/specs/EP-001-S11-spec.md) ✅ — Intégrer l'EDD comme onglet de la page « Clés de répartition », avec attache/téléchargement/historique du document EDD réel (LEGAL.md §1.2)
* [EP-001-S12](docs/specs/EP-001-S12-spec.md) ✅ — Follow-up UI/UX onglet EDD : espacements, style du sélecteur de document, ajout de lots archivés à la saisie, clarification du sigle EDD, repositionnement du bouton de création (LEGAL.md §1.2)

***

### EP-002 — Gestion des propriétaires

**Statut :** en cours ⏳ | **MVP :** oui | **Réf. PRD :** Fondations légales

**État code actuel :** S01–S05 et S08 livrés côté backend (domaine + DB + API) et frontend (liste résidents, invitation tokenisée avec TTL, révocation d’invitation, mutation de lot avec historique, anonymisation RGPD, accès par lien `/invite/:token`). S10 (procédures judiciaires) et S11 (registre de gouvernance) livrés backend + frontend. S06, S07, S09, S12 hors scope MVP.

Historique de propriété tracé avec dates de cession, invitation par lien tokenisé, anonymisation RGPD à la demande (conservation des données comptables 10 ans).

**Stories :**

* `EP-002-S01` ✅ — Créer un propriétaire et l’associer à un ou plusieurs lots
* `EP-002-S02` ✅ — Gérer l’historique de propriété avec dates d’entrée, de sortie et cessions
* `EP-002-S03` ✅ — Inviter un propriétaire par lien tokenisé pour activer son accès
* `EP-002-S04` ✅ — Consulter la fiche propriétaire avec lots détenus, rôle et état d’activation
* `EP-002-S05` ✅ — Lancer l’anonymisation RGPD en préservant les obligations de conservation comptable (LEGAL.md §8)
* `EP-002-S06` ⚠️ — Générer l'état daté d'un lot à la demande (charges courantes, arriérés, fonds travaux, travaux votés non réalisés, procédures en cours) dans un délai de 15 jours (LEGAL.md §7.1). Limitation : « travaux votés non réalisés » non alimentée (pas de lien schéma résolution↔tâche) — cf. docs/specs/EP-002-S06-spec.md.
* [EP-002-S07](docs/specs/EP-002-S07-spec.md) ⚠️ — Générer le pré-état daté pour signature du compromis de vente (synthèse financière de la copropriété) (LEGAL.md §7.2). Limitations : dette fournisseurs et travaux votés non réalisés non alimentés (mêmes limitations que S06).
* `EP-002-S08` ✅ — Garantir en mutation que les tantièmes restent attachés au lot (pas au propriétaire) et exposer la quote-part vendeur/acquéreur à date (LEGAL.md §2.4)
* `EP-002-S09` ⏸️ — *(reporté post-MVP → EP-021-S01)* Exporter les données personnelles d'un copropriétaire (droit d'accès RGPD) avec périmètre horodaté et traçabilité de la demande (LEGAL.md §8)
* `EP-002-S10` ✅ — Enregistrer et suivre les procédures judiciaires en cours impliquant le syndicat (demandeur ou défendeur), avec date d'ouverture, type, statut ; requis pour l'état daté (LEGAL.md §7.1, §10)
* `EP-002-S11` ✅ — Tenir un registre de gouvernance (syndic en exercice, membres du conseil syndical, périodes de mandat) pour auditer la séparation des organes (LEGAL.md §1.7, §1.8)
* [EP-002-S12](docs/specs/EP-002-S12-spec.md) — PV du conseil syndical en délégation de pouvoirs : créer, stocker et consulter les PV formels des décisions prises par le conseil syndical sous délégation d'AG (signé par 2 membres, noms + sens des votes, inscrit au registre des PV d'AG) — art. 21-1 décret 67-223, LEGIARTI000042077775 (LEGAL.md §3.5)
* `EP-002-S13` ✅ — Créer un résident sans invitation immédiate et déclencher l'invitation depuis la liste (PRD US-GP-008)
* `EP-002-S14` ✅ — Modifier la fiche d'un résident existant (nom, prénom, email, téléphone, rôle) depuis la liste des résidents — syndic uniquement ; email soumis à contrainte d'unicité (PRD US-GP-001 §U, matrice CRUD « Fiche résident » Syndic = C/R/U/D)
* `EP-002-S15` ✅ — Permettre au syndic de corriger ou renseigner l'adresse email d'un résident au moment de déclencher l'envoi de l'invitation depuis la liste (`SendInviteModal`) ; l'email mis à jour est persisté sur la fiche résident avant génération du token (PRD US-GP-002, US-GP-008)
* `EP-002-S16` ✅ — Archiver un compte résident : désactiver l'accès à une date effective, conserver les données intactes, et permettre la réactivation si le compte n'est pas encore anonymisé ; transition `active → archived → active` (PRD US-GP-004, cycle de vie §4)
* `EP-002-S17` ✅ — Enregistrer la mutation complète d'un lot : saisir date effective, ancien et nouveau propriétaire (existant ou à créer), révoquer l'accès de l'ancien propriétaire à la date effective, créer le nouveau lien `unit_ownership`, gérer l'état `pending_transfer` entre l'archivage vendeur et l'acceptation acquéreur, bloquer deux propriétaires actifs simultanément (422) et tracer les charges courantes non soldées à la date de mutation (PRD US-GP-003)
* `EP-002-S18` ✅ — Attribuer et révoquer les rôles des résidents (`syndic`, `board_member`, `owner`, `tenant`) : unicité du syndic actif par copropriété, invalidation immédiate du cache de permissions à la révocation, contrainte d'auto-révocation du syndic bloquée sans désignation de successeur (PRD US-GP-006)
* `EP-002-S19` ✅ — Vue des charges par lot avec filtre propriétaire historique : dans `UnitChargesView`, sélecteur (syndic uniquement) pour filtrer les appels de fonds par période de propriété quand un lot a eu plusieurs propriétaires successifs (PRD US-GP-009)

***

### EP-003 — Assemblée Générale

**Statut :** en cours | **MVP :** oui | **Réf. PRD :** Fondations légales

**État code actuel :** backend + frontend réels sur tout le cycle de vie (`draft → called → ongoing → closed`). Génération du projet de PV markdown server-side, upload du PV signé via flux pré-signé, traçabilité de la date de notification aux absents avec calcul du délai légal art. 42 loi 1965 (J+60). Calculs de majorité art. 24/25/25-1/26/26-1/unanimité couverts par tests canoniques.

Cycle de vie complet d'une AG, résolutions typées par majorité, vote par lot avec procurations, calcul automatique des résultats, dépôt du PV signé.

**Stories :**

* `EP-003-S01` ✅ — Créer une AG avec date, type et ordre du jour ; calculer et afficher la date limite de convocation (J−21) (LEGAL.md §3.1)
* `EP-003-S02` ✅ — Joindre les documents obligatoires à la convocation et alerter en cas de pièce manquante avant envoi (LEGAL.md §3.1)
* `EP-003-S03` ✅ — Enregistrer les présences, procurations et votes par correspondance (LEGAL.md §3.2)
* `EP-003-S04` ✅ — Saisir les résolutions et les votes, et calculer les résultats selon la règle de majorité applicable (art. 24/25/25-1/26/26-1/unanimité) (LEGAL.md §3.2)
* `EP-003-S05` ✅ — Générer et stocker le PV signé en séance (LEGAL.md §3.3)
* `EP-003-S06` ✅ — Tracer la date de notification du PV aux copropriétaires **opposants ou défaillants** : le syndic doit notifier dans le délai d'**un mois** à compter de la tenue de l'AG (art. 42 al. 3 loi 1965). L'implémentation affiche la date limite de contestation à J+60 depuis la date d'AG (approche conservative : suppose notification immédiate) — le délai légal exact de contestation est J+60 depuis la **date de notification** (LEGAL.md §3.4)
* `EP-003-S07` ✅ — Exclure explicitement les lots archivés des calculs de quorum et de majorité avec piste d'audit des bases de calcul (LEGAL.md §3.2)
* `EP-003-S08` ✅ — Contrôler la complétude réglementaire de la convocation selon les art. 7 à 13 du décret 1967 (ordre du jour, annexes, preuve d'envoi) et l'art. 9 al. 4 (formulaire de **vote par correspondance obligatoirement joint**) avec checklist horodatée (LEGAL.md §3.1)
* `EP-003-S09` ✅ — Tracer la conformité du bureau de séance et des signatures du PV (président, secrétaire, scrutateurs) selon les art. 15, 17 et 17-1 du décret 1967 (LEGAL.md §3.4) 
* [EP-003-S10](docs/specs/EP-003-S10-spec.md) — Valider que le formulaire de vote par correspondance est réceptionné par le syndic **au plus tard J-3 francs** avant la date de l'AG (art. 9 bis décret 67-223, LEGIARTI000042076552) et annuler automatiquement un vote par correspondance si le copropriétaire est présent ou représenté en séance (art. 14-1 décret 67-223, LEGIARTI000042076720) (LEGAL.md §3.1)
* [EP-003-S11](docs/specs/EP-003-S11-spec.md) — Enregistrer et traiter la demande de convocation d'AG formulée par un copropriétaire pour ses propres questions (art. 17-1 AA loi 1965, LEGIARTI000039301594) avec imputation des frais de convocation au copropriétaire demandeur (LEGAL.md §3.5)
* [EP-003-S12](docs/specs/EP-003-S12-spec.md) — Modifier ou supprimer un brouillon d'AG : éditer les métadonnées (date, type, ordre du jour) et supprimer une AG tant qu'elle est en statut `draft` ; le passage à `called` bloque la modification et interdit la suppression (PRD US-AG-002)
* [EP-003-S13](docs/specs/EP-003-S13-spec.md) — Gérer les résolutions d'une AG (CRUD + ordre + qualification de majorité) : créer, modifier, supprimer et réordonner les résolutions avec leur règle de majorité applicable (art. 24/25/25-1/26/26-1/unanimité) ; seul le statut `draft` autorise les modifications (PRD US-AG-003)
* [EP-003-S14](docs/specs/EP-003-S14-spec.md) — Consulter le dossier AG et les décisions votées (vue lecture résidents) : accès en lecture aux résolutions, résultats, convocation et PV pour les résidents selon leur rôle et la visibilité de l'AG (PRD US-AG-014)

**Hors scope explicite (renvoyé à `EP-013` ou à des epics transverses) :**

- désignation des rôles de séance (`président`, `secrétaire`, `scrutateurs`) ;
- feuille de présence avancée avec quorum temps réel de conduite de séance ;
- re-délibérations art. 25-1 (majorité absolue → simple) et art. 26-1 (double majorité → absolue) — implémentées dans `EP-013-S05` et `EP-013-S10` ;
- prise de notes collaborative par résolution ;
- génération du projet de PV et circuit de validation ;
- signature numérique / signature électronique du PV ;
- workflow d'envoi probant multi-canal aux absents ;
- refonte transverse du module documents au-delà des besoins minimaux `S02` / `S05`.

***

### EP-004 — Budget & Charges

**Statut :** en cours | **MVP :** oui | **Réf. PRD :** Fondations légales

**État code actuel :** backend réel sur les flux principaux (génération appels, snapshots, relevé individuel, garde d'équilibrage des clés, clôture conditionnelle fonds travaux) + frontend branché sur l'API réelle (S01-S04) avec carte dédiée de pilotage du fonds travaux et configuration du taux par exercice.

Budget prévisionnel par exercice, appels de fonds générés par tantièmes, fonds travaux obligatoire (≥ 5%), arrêté de comptes.

**Stories :**

* `EP-004-S01` ✅ — Créer et gérer un budget prévisionnel par exercice (lignes par clé de répartition) — `POST /budgets`, `POST /budgets/{id}/lines`, `DELETE /budgets/{id}/lines/{lid}` ; frontend branché (modales création exercice + ligne, suppression ligne, affichage prévisionnel vs réalisé par clé)
* `EP-004-S02` ✅ — Générer les appels de fonds trimestriels au prorata des tantièmes — `POST /budgets/{id}/levies/generate` avec `due_date` + `include_works_fund` ; `PUT /levies/{id}/payment` ; tableau des appels connecté à l'API
* `EP-004-S03` ✅ — Suivre et alimenter le fonds travaux (≥ 5% du budget prévisionnel, compte séparé) — `GET /works-fund/{fiscal_year}` + `PUT /works-fund/{fiscal_year}` ; entrée `works_fund` créée avec l'exercice, carte dédiée de pilotage, objectif annuel, progression d'appel, taux configurable et notice légale
* `EP-004-S04` ✅ — Produire l'arrêté de comptes (réalisé vs prévisionnel) — barres de progression par poste budgétaire avec données réelles API
* `EP-004-S05` ✅ — Snapshot des tantièmes dans chaque appel de fonds généré (`shares_snapshot`, `shares_total_snapshot`, `key_id`) — immuabilité comptable (LEGAL.md §2.3, ADR-006)
* `EP-004-S06` ✅ — Relevé individuel de charges par lot : liste des appels reçus, montants, statuts, solde courant, avec séparation des avoirs (LEGAL.md §4.6)
* `EP-004-S07` ✅ — Blocage dur côté backend de toute génération d'appel de fonds tant qu'une clé active est déséquilibrée (`Σ tantièmes ≠ total_clé`) (LEGAL.md §2.1)
* [EP-004-S08](docs/specs/EP-004-S08-spec.md) — Émission trimestrielle paramétrable et contrôles d'échéances par exercice (quart par défaut) avec piste d'audit (LEGAL.md §4.3) — Prendre en compte les catégories de l'art. 35 décret v6.0 (25/12/2025) : avances PPT, cotisations fonds travaux (art. 14-2-1), provisions délégation CS
* [EP-004-S09](docs/specs/EP-004-S09-spec.md) — Implémenter le mapping plan comptable copropriété (arrêté 14/03/2005) sur lignes budgétaires et exports comptables (LEGAL.md §4.2)
* [EP-004-S10](docs/specs/EP-004-S10-spec.md) — Contrôle et alerte du seuil minimum fonds travaux avant validation de l'exercice : **branche sans PPT partiellement couverte** via alerte UI + blocage de clôture si le taux configuré est < 5% ou si le montant appelé est insuffisant ; **branche PPT 2,5%** encore à implémenter (art. 14-2-1 loi 1965, LEGIARTI000043967792, en vigueur 01/01/2023 — LEGAL.md §4.4)
* [EP-004-S11](docs/specs/EP-004-S11-spec.md) — Recouvrement des charges impayées : identification des lots en solde débiteur, enregistrement des relances (mise en demeure), suivi des dossiers contentieux jusqu'à clôture ; déclencher et signaler la **déchéance du terme à J+30** après mise en demeure infructueuse (toutes provisions restantes deviennent exigibles, art. 19-2 loi 1965, LEGIARTI000043977271) (LEGAL.md §4.7)
* ✅ `EP-004-S12` — Permettre au Conseil syndical et aux copropriétaires de consulter le budget prévisionnel, les lignes budgétaires par clé et le récapitulatif réalisé vs prévisionnel dans leur périmètre de droits (PRD US-BC-010)
* [EP-004-S13](docs/specs/EP-004-S13-spec.md) — Générer les documents comptables légaux obligatoires (Annexes I, II et III décret 1967) à partir des lignes budgétaires et des données d'exercice (PRD US-BC-012, LEGAL.md §4.2)
* `EP-004-S14` ✅ — Borner le relevé copropriétaire à la période de propriété du lot : après mutation, le nouvel acquéreur ne voit pas les appels antérieurs à sa date de début ; Syndic et Conseil conservent la vue historique complète (PRD US-BC-013)

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
* [EP-005-S06](docs/specs/EP-005-S06-spec.md) ✅ — Typer et verrouiller les documents fondateurs (`Règlement`, `EDD`) avec version courante explicite et historique consultable (LEGAL.md §1.2)
* [EP-005-S07](docs/specs/EP-005-S07-spec.md) ✅ — Catégoriser et archiver les pièces réglementaires techniques (`DPE collectif`, `DTG`, `PPT`, `PV`) avec règles de conservation adaptées (LEGAL.md §3.3, §6.1, §6.2, §6.3)
* [EP-005-S08](docs/specs/EP-005-S08-spec.md) ✅ — Supprimer un document : soft-delete avec conservation de la métadonnée ; pour les documents critiques (`Règlement`, `EDD`, PV signé), déclencher un workflow de remplacement avec justification horodatée avant invalidation du fichier courant (PRD US-DOC-004, US-DOC-007)
* [EP-005-S09](docs/specs/EP-005-S09-spec.md) ✅ — Nettoyage périodique des objets S3 orphelins (fichiers sans métadonnée `document` active) et application des règles de rétention par catégorie (PRD US-DOC-008)
* [EP-005-S10](docs/specs/EP-005-S10-spec.md) ✅ — Émettre des événements internes au dépôt d'un document pour permettre aux modules consommateurs (signalements, tâches, AG) de déclencher des notifications contextuelles (PRD US-DOC-009)
* [EP-005-S11](docs/specs/EP-005-S11-spec.md) — Visualiser les documents PDF générés (PV d'AG, relances d'appels de fonds, tout courrier réglementaire envoyé par voie postale et/ou électronique en pièce jointe) directement dans l'Application sans téléchargement préalable, dans le respect des droits d'accès par rôle (→ EP-004-S11, EP-013-S09, LEGAL.md §3.4, §4.7)

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
* [EP-006-S06](docs/specs/EP-006-S06-spec.md) — Lister et consulter les contrats fournisseurs avec filtres par statut (actif / résilié / expiré), prestataire et type d'objet ; vue tabulaire avec accès direct à la fiche contrat (PRD US-CF-002)
* [EP-006-S07](docs/specs/EP-006-S07-spec.md) — Rattacher des documents à un contrat fournisseur (contrat signé, avenants, attestations) via le module Documents (EP-005) avec visibilité réservée au Syndic (PRD US-CF-007)

***

### EP-007 — Tâches & Prestataires

**Statut :** en cours | **MVP :** non | **Réf. PRD :** Gestion courante

**État code actuel :** backend réel (CRUD tâches + token prestataire + upload staging + pré-validation prestataire en transaction + documents manager via `task_document` + assignation prestataire référentiel ou nom libre sur `task.contractor_name`) + frontend branché (liste, créer, modifier, documents manager, token, page publique prestataire, autocomplete prestataire tolérant avec création optionnelle, clôture avec coût optionnel).

Tâches assignées en interne, lien tokenisé éphémère pour accès prestataire sans compte, flux de dépôt staging → validation → bucket final. Les tâches peuvent être liées à un prestataire du référentiel (EP-015) pour consolider l'historique, ou conserver une saisie libre si aucun prestataire n'est retenu.

**Stories :**

* `EP-007-S01` ✅ — Créer et assigner une tâche à un intervenant interne ou à un prestataire externe
* `EP-007-S02` ✅ — Générer un lien tokenisé éphémère pour l'accès prestataire sans compte
* `EP-007-S03` ✅ — Permettre au prestataire de déposer des documents via le lien tokenisé (zone staging)
* `EP-007-S04` ✅ — Valider et archiver les documents déposés par le prestataire (bucket final) puis passer la tâche en pré-validation (`awaiting_validation`) avec commentaire et coût optionnel
* `EP-007-S05` ✅ — Clôturer une tâche avec coût optionnel et l'alimenter automatiquement au carnet d'entretien (→ EP-009-S03), y compris après réouverture ; inclut `vendor_name` du prestataire assigné
* `EP-007-S06` ✅ — Assigner un prestataire du référentiel (EP-015) à une tâche via autocomplete, avec suggestion de création optionnelle si aucune correspondance n'existe et conservation d'un nom libre sur la tâche si aucun prestataire n'est créé
* [EP-007-S07](docs/specs/EP-007-S07-spec.md) — Reprendre en main une tâche depuis une phase devis (`awaiting_quote` ou `quote_received`) : retour à `in_progress` sans annuler les documents reçus, sans révocation automatique du lien prestataire (PRD US-TP-012)
* [EP-007-S08](docs/specs/EP-007-S08-spec.md) — Supprimer ou annuler une tâche : suppression bloquée si une entrée carnet d'entretien ou une soumission prestataire y est rattachée ; annulation disponible comme alternative sans destruction des données (PRD US-TP-016, ADR-009)
* [EP-007-S09](docs/specs/EP-007-S09-spec.md) — Consulter, filtrer et rechercher les tâches : liste paginée avec filtres par statut, priorité, assignataire et prestataire, recherche par titre (PRD US-TP-003)
* [EP-007-S10](docs/specs/EP-007-S10-spec.md) — Gérer les documents rattachés à une tâche côté gestionnaire : ajout append-only après clôture, suppression bloquée si la tâche est clôturée ; indépendant du flux prestataire (PRD US-TP-005)
* [EP-007-S11](docs/specs/EP-007-S11-spec.md) — Déclarer la fin d'intervention depuis l'accès prestataire : le prestataire soumet ses livrables finaux et passe la tâche en `report_received` pour déclencher la validation gestionnaire (PRD US-TP-010)
* [EP-007-S12](docs/specs/EP-007-S12-spec.md) — Réouvrir une tâche clôturée ou annulée vers `in_progress` : le syndic peut relancer une tâche fermée sans perte des documents et de l'historique ; nouvelle clôture génère une entrée carnet d'entretien additionnelle si un coût est renseigné (PRD US-TP-013)

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
* [EP-008-S06](docs/specs/EP-008-S06-spec.md) — Formaliser la déclaration de sinistre : transition explicite `draft → declared` par le syndic, déclenchant le calcul du délai légal et les alertes associées ; un sinistre en `draft` n'est visible que du syndic (PRD US-SIN-001b)
* [EP-008-S07](docs/specs/EP-008-S07-spec.md) — Ajouter des notes de suivi à un sinistre : journal append-only horodaté (auteur + date + note) pendant toute la vie du sinistre, consultable par Syndic et Conseil (PRD US-SIN-004)
* [EP-008-S08](docs/specs/EP-008-S08-spec.md) — Rattacher des documents à un sinistre (PJ déclaration, photos, courrier assureur) via le module Documents (EP-005), avec accès restreint au Syndic (PRD US-SIN-006)

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
* [EP-009-S09](docs/specs/EP-009-S09-spec.md) — Exporter le carnet d'entretien en PDF : document imprimable avec en-tête réglementaire, chronologie des interventions et listing des contrats actifs (PRD US-CE-004)
* [EP-009-S10](docs/specs/EP-009-S10-spec.md) — Filtrer les entrées du carnet par catégorie (travaux, contrat, sinistre), période et source (manuelle / automatique / prestataire) (PRD US-CE-005)

***

### EP-010 — Consultations & Sondages

**Statut :** terminée | **MVP :** oui | **Réf. PRD :** Participatif

**État code actuel :** backend réel (list, get, create as 'open', respond/upsert 1 réponse par utilisateur, results, close) + frontend branché (liste open/closed, modale create, modale respond, clôture, résultats).

Consultations initiables par tout résident sans approbation préalable, résultats en temps réel, mention obligatoire "non décisionnelle".

**Contrainte légale :** mention `INFORMAL_DISCLAIMER` obligatoire sur toutes les réponses API et dans la vue (loi 10 juillet 1965, art. 17 et 24).

**Stories :**

* `EP-010-S01` ✅ — Créer une consultation ou un sondage avec question(s) et options de réponse (minimum 2 options, `visible_to_roles` requis)
* `EP-010-S02` ✅ — Soumettre un vote depuis l'espace résident (une réponse par utilisateur) ; contrôle `visible_to_roles`
* `EP-010-S03` ✅ — Afficher les résultats en temps réel avec mention obligatoire « non décisionnelle »
* [EP-010-S04](docs/specs/EP-010-S04-spec.md) — Gérer le cycle de vie brouillon : créer une consultation en statut `draft`, la modifier librement, puis l'ouvrir explicitement (`draft → open`) pour déclencher la notification aux participants (PRD US-CS-001b)
* [EP-010-S05](docs/specs/EP-010-S05-spec.md) — Relancer les non-répondants d'une consultation ouverte : le Syndic peut envoyer une notification de rappel aux participants n'ayant pas encore soumis de réponse (PRD US-CS-004)

***

### EP-011 — Signalements

**Statut :** terminée | **MVP :** oui | **Réf. PRD :** Participatif

**État code actuel :** backend réel (CRUD, documents associés, conversion en tâche avec priorité choisie, lien `report.task_id`, clôture automatique du signalement à la fermeture effective de la tâche liée) + frontend branché (liste, création, photos, mise à jour de statut, conversion en tâche).

Signalement par tout résident avec description et lot optionnel, suivi de statut par le syndic, conversion possible en tâche.

**Stories :**

* `EP-011-S01` ✅ — Créer un signalement (titre, description, lot concerné)
* `EP-011-S02` ✅ — Suivre et mettre à jour le statut (reçu → en cours → clos) — manager/conseil syndical
* `EP-011-S03` ✅ — Convertir un signalement en tâche — crée une tâche avec titre/description + priorité choisie depuis le signalement, passe automatiquement le signalement en `in_progress`, puis le clôture quand la tâche liée est effectivement fermée
* [EP-011-S04](docs/specs/EP-011-S04-spec.md) — Ajouter un commentaire de suivi à un signalement : le Syndic et le Conseil syndical peuvent poster des commentaires horodatés visibles du déclarant, pour communiquer sur l'avancement (PRD US-SIG-006)
* [EP-011-S05](docs/specs/EP-011-S05-spec.md) — Gérer l'anonymat du déclarant : le résident choisit à la création du signalement de rester anonyme ; son identité n'est jamais exposée aux autres résidents ni au Syndic dans la vue liste (PRD US-SIG-007)

***

### EP-012 — Conformité & Reporting réglementaire

**Statut :** en attente | **MVP :** non | **Réf. LEGAL :** §1.1, §1.3

Export des données de la copropriété au format RNC et génération de la fiche synthétique annuelle obligatoire.

**Stories :**

* [EP-012-S01](docs/specs/EP-012-S01-spec.md) — Exporter les données de la copropriété au format attendu par le Registre National des Copropriétés (RNC) pour faciliter la mise à jour annuelle (LEGAL.md §1.1)
* [EP-012-S02](docs/specs/EP-012-S02-spec.md) — Générer la fiche synthétique annuelle de copropriété (nombre de lots, budget, données techniques) (LEGAL.md §1.3)
* [EP-012-S03](docs/specs/EP-012-S03-spec.md) — Générer un dossier de déclaration annuelle RNC prêt à dépôt (contrôles de complétude + export + journal horodaté) (LEGAL.md §1.1)
* [EP-012-S04](docs/specs/EP-012-S04-spec.md) — Permettre au Syndic de vérifier et corriger le brouillon de fiche synthétique auto-rempli : formulaire pré-rempli depuis les modules actifs (EP-001, EP-004), modifiable avant publication (PRD US-CR-002)
* [EP-012-S05](docs/specs/EP-012-S05-spec.md) — Publier la fiche synthétique annuelle : action explicite du Syndic rendant la fiche accessible en lecture aux copropriétaires et membres du Conseil (PRD US-CR-003)
* [EP-012-S06](docs/specs/EP-012-S06-spec.md) — Consulter la fiche synthétique publiée depuis l'espace résident : accès en lecture pour les copropriétaires et le Conseil syndical à la dernière fiche validée (PRD US-CR-004)
* [EP-012-S07](docs/specs/EP-012-S07-spec.md) — Exporter la fiche synthétique en PDF : génération d'un document imprimable à en-tête copropriété depuis la fiche publiée (PRD US-CR-005)

***

### EP-013 — Tenue d’AG

**Statut :** en attente | **MVP :** non | **Réf. PRD :** Fonctionnalités avancées — extension EP-003

**État code actuel :** statut `ongoing` disponible ; transitions `POST /{id}/start` (`called` → `ongoing`) et `POST /{id}/close` (`ongoing` → `closed`) opérationnelles backend. Aucune vue frontend dédiée à la conduite de séance n'existe encore.

Conduite de séance : désignation des rôles (président, secrétaire, scrutateurs), feuille de présence numérique, quorum en temps réel, notes collaboratives, génération et validation du PV avec signature numérique. `EP-013` démarre là où `EP-003` s'arrête : préparation juridique, votes et clôture simple restent dans `EP-003`, tandis que l'assistance complète de séance et la validation probante du PV relèvent de cette extension post-MVP.

Le cycle de vie complet d'une AG est : `draft` → `called` → **`ongoing`** → `closed`.

**Stories :**

* `EP-013-S01` ✅ — Démarrer la séance : faire passer une AG du statut `called` à `ongoing` ; verrouiller l'ordre du jour et l'assiette de tantièmes à l'instant de démarrage ; exposer le statut `ongoing` dans la liste et la fiche AG
* `EP-013-S02` ✅ — Désigner les rôles de séance : saisir le président, le secrétaire et les scrutateurs lors du démarrage (`status = ongoing`) ; données figées à la clôture de séance (`status = closed`) (LEGAL.md §3.2)
* `EP-013-S03` ✅ — Feuille de présence numérique en temps réel : afficher le quorum atteint (tantièmes représentés / total, nombre de copropriétaires participants) dynamiquement à mesure de l'enregistrement des présences, procurations et votes par correspondance
* `EP-013-S04` ✅ — Conduire le vote résolution par résolution : en `status = ongoing`, saisir les positions (pour / contre / abstention / non-votant) lot par lot pour chaque résolution et calculer le résultat automatiquement selon la règle de majorité applicable (art. 24 / 25 / 26 / unanimité)
* `EP-013-S05` ✅ — Détecter et traiter l'article 25-1 : si une résolution `absolute_majority` est rejetée avec ≥ 1/3 des tantièmes totaux en faveur, proposer immédiatement une re-délibération à la majorité simple (art. 24) dans la même séance ; tracer la résolution originale et sa re-délibération (LEGAL.md §3.3)
* [EP-013-S06](docs/specs/EP-013-S06-spec.md) — Prendre des notes collaboratives par résolution : saisir des annotations de séance (arguments, amendements de forme) attachées à chaque résolution ; accessibles dans le projet de PV
* `EP-013-S07` ✅ — Clôturer la séance : faire passer l'AG de `ongoing` à `closed` via `POST /{id}/close` ; figer les votes et la feuille de présence ; bloquer toute modification ultérieure des résultats (handler backend + bouton UI `Clôturer l'AG`)
* `EP-013-S08` ✅ — Générer et valider le projet de PV : après clôture de séance (`status = closed`), produire un projet de PV structuré (résolutions + résultats + présences + notes) et le soumettre à revue avant signature (LEGAL.md §3.4)
* [EP-013-S09](docs/specs/EP-013-S09-spec.md) — Signer et publier le PV : finaliser le PV avec signature numérique, l'attacher à l'AG et déclencher la traçabilité de notification aux absents (→ EP-003-S06)
* `EP-013-S10` ✅ — Détecter et traiter l'article 26-1 : si une résolution `double_majority` est rejetée avec ≥ 1/2 des tantièmes totaux en faveur, proposer immédiatement une re-délibération à la majorité absolue (art. 25) dans la même séance ; tracer la résolution originale et sa re-délibération (LEGAL.md §3.3)
* [EP-013-S11](docs/specs/EP-013-S11-spec.md) — Garde backend de séance : refuser l'enregistrement des présences et des votes hors statut `ongoing` et figer toute mutation après `closed` — aujourd'hui seule l'UI restreint ces actions, le backend accepte un vote sur une AG `called` ou `closed` (LEGAL.md §11.1)

***

### EP-014 — Calendrier copropriété

**Statut :** en attente | **MVP :** non | **Réf. PRD :** Fonctionnalités avancées

Calendrier partagé centralisant AG, interventions et réunions avec liens contextuels vers les entités correspondantes, export iCal.

**Stories :**

* [EP-014-S01](docs/specs/EP-014-S01-spec.md) ✅ — Afficher la vue calendaire des événements de la copropriété (AG, interventions, réunions) dans les modes mensuel, hebdomadaire et liste avec lien contextuel vers les entités correspondantes (PRD US-CAL-001)
* [EP-014-S02](docs/specs/EP-014-S02-spec.md) — Filtrer les événements du calendrier par type (AG, tâche, sinistre, contrat, autre) et par période (PRD US-CAL-002)
* [EP-014-S03](docs/specs/EP-014-S03-spec.md) ✅ — Créer, modifier et supprimer des événements manuels sur le calendrier (réunion informelle, rappel, rendez-vous) réservé au Syndic et au Conseil (PRD US-CAL-005)
* [EP-014-S04](docs/specs/EP-014-S04-spec.md) ✅ — Exporter le calendrier au format ICS pour intégration dans les agendas personnels (Google Calendar, Outlook, Apple Calendar) (PRD US-CAL-003)
* [EP-014-S05](docs/specs/EP-014-S05-spec.md) ✅ — Afficher une vue agenda (prochains 30 jours) depuis le tableau de bord avec les 5 prochains événements (PRD US-CAL-004)

***

### EP-015 — Référentiel prestataires

**Statut :** en cours | **MVP :** non | **Réf. PRD :** Gestion courante

**État code actuel :** backend réel (CRUD + recherche par nom) + frontend branché (catalogue, création, édition, suppression, autocomplete dans TasksView, création optionnelle depuis la modale tâche).

Catalogue des prestataires de la résidence : nom, email, téléphone, domaine professionnel. Réutilisable lors de l'assignation d'une tâche (autocomplete) et pré-rempli à la migration vers le carnet d'entretien. Accessible en lecture à tous les utilisateurs, modifiable par les managers et membres du conseil syndical.

**Stories :**

* `EP-015-S01` ✅ — Gérer le référentiel prestataires (CRUD) : nom, email, téléphone, domaine
* `EP-015-S02` ✅ — Rechercher un prestataire par nom (autocomplete, `?q=`) pour l'assigner à une tâche
* `EP-015-S03` ✅ — Catalogue accessible à tous les utilisateurs authentifiés de la résidence (lecture), modifiable par manager/conseil syndical
* `EP-015-S04` ✅ — Propagation du nom de prestataire au carnet d'entretien lors de la clôture automatique d'une tâche (`vendor_name`)
* `EP-015-S05` ✅ — Ajouter le numéro de téléphone au profil utilisateur (`user_account.phone`) et l'exposer dans la carte contact syndic sur la page prestataire
* [EP-015-S06](docs/specs/EP-015-S06-spec.md) — Rattacher une attestation d'assurance à un prestataire via le module Documents (EP-005) avec vérification de la validité à la date de l'intervention (PRD US-RP-004)

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

* [EP-017-S01](docs/specs/EP-017-S01-spec.md) — Migration : créer la table `activity_log` polymorphique (`entity_type TEXT NOT NULL`, `entity_id UUID NOT NULL`, `action TEXT NOT NULL`, `actor_id UUID REFERENCES user_account(id)`, `payload JSONB`, `created_at TIMESTAMPTZ DEFAULT NOW()`) + index `(entity_type, entity_id, created_at DESC)`
* [EP-017-S02](docs/specs/EP-017-S02-spec.md) — Backend : émettre des événements auditables aux points clés de mutation : création, changement de statut, changement d’assignataire, attribution prestataire référentiel, génération lien vendeur, soumission vendeur, clôture (tâche) ; création, changement de statut, ajout photo, conversion (signalement). Chaque événement contient un `payload` JSONB avec ancien/nouveau valeur quand pertinent.
* [EP-017-S03](docs/specs/EP-017-S03-spec.md) — Backend : ajouter `linked_task_id UUID REFERENCES task(id)` sur `report` pour matérialiser la conversion signalement → tâche ; endpoint `GET /tasks/:id/timeline` remontant la chaîne jusqu’au signalement source si applicable ; endpoint `GET /reports/:id/timeline` symmétrique.
* [EP-017-S04](docs/specs/EP-017-S04-spec.md) — Frontend : vue `TaskTimelineView` (remplace `TaskDetailView`) : données statiques de la tâche (titre, statut, priorité, assignataire, prestataire) + timeline chronologique des événements depuis le signalement source (si applicable) jusqu’à la clôture.
* [EP-017-S05](docs/specs/EP-017-S05-spec.md) — Frontend : vue `ReportDetailView` : afficher les événements auditables du signalement + lien contextuel vers la tâche créée si conversion effective.
* [EP-017-S06](docs/specs/EP-017-S06-spec.md) — Accessibilité : événements visibles par tous les rôles ayant accès à l’entité ; actions sensibles (`assignee_changed`, `vendor_link_generated`) réservées aux rôles manager/conseil syndical.

***

### EP-018 — Immutabilité comptable & auditabilité probante

**Statut :** en cours | **MVP :** oui | **Réf. LEGAL :** §11 · **Réf. ADR :** ADR-009 · ADR-012

**État code actuel :** S01 à S07 livrées. Le module `domain/immutability` porte les guards `is_mutable()` / `is_deletable()` pour `levy_call`, `task`, `claim`, `vote`, `resolution`, `maintenance_log`, `vendor_contract` et `budget`. Les Patterns A, B et C sont implémentés sur leurs entités respectives. Reste ouvert : le câblage exhaustif de `audit_log` sur l'ensemble des mutations comptables (porté par [EP-021-S03](docs/specs/EP-021-S03-spec.md)), et le lien `resolution_id` sur `budget` (AC-BC-009-F, à trancher avec `EP-003`). L'arbitrage documentaire est le suivant : `audit_log` reste la piste probante comptable de `EP-018`, tandis que `EP-017` porte une frise métier distincte via `activity_log`.

Durcir les entités comptablement critiques pour empêcher toute modification rétroactive après état terminal, imposer les patterns `gel`, `clôturer + recréer` ou `append-only` selon les cas, et conserver une piste d'audit probante alignée sur l'ADR-009.

**Stories :**

* ✅ `EP-018-S01` — Entériner l’arbitrage documentaire avec `EP-017` : `audit_log` réservé aux mutations comptables/juridiques probantes, `activity_log` réservé aux événements métier de timeline, avec frontière de responsabilité documentée et non ambiguë
* ✅ `EP-018-S02` — Créer l'infrastructure transverse d'immutabilité : module `domain/immutability`, guards `is_mutable()` / `is_deletable()`, repository de journal append-only et points d'appel obligatoires avant mutation
* ✅ `EP-018-S03` — Sécuriser `levy_call` selon le Pattern A : ajout du statut `draft`, endpoint d'émission explicite (`POST /levy-calls/{id}/issue`), blocage des `UPDATE` / `DELETE` hors `draft` et trigger PG défensif
* ✅ `EP-018-S04` — Appliquer le Pattern A sur `task`, `claim`, `vote`, `resolution` et `maintenance_log` automatique : champs financiers figés après état terminal, suppression interdite selon les règles ADR-009, et masquage UI des actions `Modifier` / `Supprimer`
* ✅ [EP-018-S05](docs/specs/EP-018-S05-spec.md) — Appliquer le Pattern B sur `vendor_contract` : ajouter `predecessor_id`, remplacer `Modifier` par `Créer un avenant` sur les champs gelés, clôturer l'ancien contrat à l'activation de l'avenant et exposer la chaîne d'avenants
* ✅ [EP-018-S06](docs/specs/EP-018-S06-spec.md) — Appliquer le Pattern B sur `budget` / `budget_line` : ajouter `parent_budget_id`, remplacer l'unicité annuelle par une contrainte compatible rectificatifs, versionner `budget_line` via `valid_from` / `valid_to` et créer le workflow de budget rectificatif
* ✅ `EP-018-S07` — Appliquer le Pattern C sur les corrections comptables : avoirs sur `levy_call`, corrections append-only sur `maintenance_log` manuel, et journalisation des opérations `UPDATE` / `DELETE` financières avec diff ciblé (`old_values` / `new_values`)

***

### EP-019 — Profil & préférences

**Statut :** en cours | **MVP :** oui | **Réf. PRD :** Gestion courante

**État code actuel :** partiel ; la préférence d'apparence est déjà exposée et persistée côté frontend, le profil utilisateur dispose d'un espace dédié, mais la feature reste encore incomplète comme hub personnel structuré de compte, notifications et préférences.

Feature simple self-service permettant à chaque utilisateur authentifié de gérer ses informations personnelles utiles au produit, ses préférences de notifications et sa préférence d'apparence depuis un écran unique à onglets.

**Stories :**

* `EP-019-S01` ✅ — Consulter son espace profil personnel avec rappel lisible du rôle et du contexte de compte
* `EP-019-S02` ✅ — Modifier ses informations personnelles éditables : email (identifiant de connexion) et téléphone uniquement ; `full_name` est `prefilled` depuis EP-002 (non modifiable ici) (PRD US-PP-002)
* [EP-019-S03](docs/specs/EP-019-S03-spec.md) — Gérer ses préférences de notifications par événement (16 événements catalogués : 3 obligatoires toggle désactivé, 13 optionnels) (PRD US-PP-004) — UI livrée mais persistance localStorage uniquement ; reste la persistance serveur
* `EP-019-S04` ✅ — Définir sa préférence d'apparence (`clair`, `sombre`, `automatique`) et l'appliquer immédiatement dans l'interface
* `EP-019-S05` ✅ — Structurer l'écran en onglets `Compte`, `Notifications`, `Apparence` pour éviter la dispersion des réglages
* `EP-019-S06` ✅ — Gérer la première visite : afficher les valeurs par défaut de manière lisible dès la première ouverture du profil ; s'assurer que l'état initial est compréhensible sans action préalable (PRD US-PP-003)
* [EP-019-S07](docs/specs/EP-019-S07-spec.md) — Réinitialiser ses préférences : bouton global unique en bas de page (couvre Notifications + Apparence), avec modale de confirmation avant exécution (PRD US-PP-005) — livré partiellement : reset Notifications seul, placé dans l'onglet ; reste le bouton global couvrant aussi l'Apparence
* `EP-019-S08` ✅ — Afficher le contexte utilisateur : rôle actif, résidence de rattachement et périmètre des droits depuis l'écran profil (PRD US-PP-007)

***

### EP-020 — Design system : consolidation tokens & wrappers Radix

**Statut :** en cours | **MVP :** non | **Réf. PRD :** DESIGN.md

**État code actuel :** S01–S12 livrées (tokens OKLCH, `@theme inline`, wrappers Radix, suppression des CSS legacy). Restent les composants catalogués « 🔲 planifié » dans DESIGN.md §3 et absents du code : `CPageHeader`, `CTopbarBreadcrumb`, `CCardHeader`, `CDocumentList`, `CDataText` (S13–S17). Nota : `CFormGroup` et `CFormRow` existent déjà — DESIGN.md à rafraîchir.

Consolidation technique du design system : migration des tokens de couleur vers OKLCH pour garantir des relations chroma/luminosité vérifiables en thème sombre, enregistrement des tokens manquants dans Tailwind v4, et implémentation des wrappers de primitives Radix Vue restants.

**Stories :**

* `EP-020-S01` ✅ — Migrer les tokens de couleur primaires, secondaires, d'état (error, success, warning) et de surface de `tokens.css` de hex vers OKLCH
* `EP-020-S02` ✅ — Migrer les tokens de couleur de rôle (`role-manager`, `board`, `co-owner`, `tenant`) vers OKLCH avec valeurs dark mode vérifiées
* `EP-020-S03` ✅ — Enregistrer dans `tailwind.css` via `@theme inline` les tokens manquants : `--text-xl`, `--text-2xl`, `--text-display`, `--space-7`, `--space-9`, `--leading-*`, `--z-*`
* `EP-020-S04` ✅ — Implémenter `CDropdownMenu` : wrapper Radix `DropdownMenu` conforme §2.1 et §3.4 de DESIGN.md
* `EP-020-S05` ✅ — Implémenter `CStatusBanner` : bandeau d'état en-tête pour les AG (`draft` / `called` / `ongoing` / `closed`) conforme §2.3 de DESIGN.md
* `EP-020-S06` ✅ — Implémenter les wrappers Radix restants : `CFormCheckbox`, `CFormSelect`, `CTooltip` (§3.4 DESIGN.md — statut 🔲 planifié)
* `EP-020-S07` ✅ — Implémenter `CTabs` : composant atomique de filtres d'état tabulaires, remplace le pattern CSS pur dans toutes les vues consommatrices (§2.1 et §4.1 DESIGN.md) ; supprimer `tabs.css`
* `EP-020-S08` ✅ — Implémenter `CHeading` : atom Fraunces semibold italic (`lg`/`md`/`sm`, prop `as`) ; migrer les usages de `typography.css` (`.display-*`, `.headline-*`) ; supprimer `typography.css`
* `EP-020-S09` ✅ — Créer `CFormRow`, `CFormGroup`, `CFormLabel` molecules ; migrer les vues consommatrices de `forms.css` (`.form-grid`, `.field`, `.search-bar`, `.icon-btn`) vers atoms existants et Tailwind inline
* `EP-020-S10` ✅ — Déplacer les styles `.data-table` dans les scoped blocks des vues consommatrices ; remplacer `.cards-grid` par Tailwind `grid grid-cols-[…]` inline ; supprimer `tables.css`
* `EP-020-S11` ✅ — Nettoyer `base.css` : conserver les seuls resets navigateur et touch targets ; migrer `.view-container`/`.view-header` vers Tailwind inline, `.empty-state` vers `CEmptyState` ; supprimer les classes applicatives (spinner, loading-skeleton, card--terminated)
* `EP-020-S12` ✅ — Supprimer `activity-cards.css` ; remplacer par l'utilisation de CCard
* [EP-020-S13](docs/specs/EP-020-S13-spec.md) — Implémenter `CPageHeader` (molecule) : en-tête de vue (`eyebrow?`, `title`, `meta?`, `icon?`, `frozenLabel?`, slot `#actions`) remplaçant les blocs `view-header` répétés (DESIGN.md §3.2 — 🔲 planifié)
* [EP-020-S14](docs/specs/EP-020-S14-spec.md) — Implémenter `CTopbarBreadcrumb` (molecule) : fil d'Ariane topbar fluide (`min-width: 0`, troncature propre) intégré à `CTopbar` (DESIGN.md §3.2 — 🔲 planifié)
* [EP-020-S15](docs/specs/EP-020-S15-spec.md) — Implémenter `CCardHeader` (molecule) : en-tête de carte d'entité (`withIcon?`, slots `#icon`, `#chips`) conforme à l'ossature commune des cartes de liste (DESIGN.md §3.2, §4.1 — 🔲 planifié)
* [EP-020-S16](docs/specs/EP-020-S16-spec.md) — Implémenter `CDocumentList` (molecule) : liste de documents avec téléchargement/retrait (`documents`, `readonly?`, `emit('remove', doc)`) mutualisant le markup dupliqué des vues contrats/sinistres/tâches (DESIGN.md §3.2 — 🔲 planifié)
* [EP-020-S17](docs/specs/EP-020-S17-spec.md) — Implémenter `CDataText` (atom) : texte tabulaire JetBrains Mono (`size` sm|md|lg, `font-mono font-medium lining-nums tabular-nums`) remplaçant les spans mono ad hoc (DESIGN.md §3.1 — 🔲 planifié)

***

### EP-021 — Conformité différée : RGPD, courriers automatiques & audit log

**Statut :** en attente | **MVP :** non | **Réf. PRD :** LEGAL.md §8, §4.7 ; ADR-009

**État code actuel :** la table `audit_log` existe (ADR-009) mais n'a aucun écrivain applicatif ; aucun export RGPD ni envoi de courrier n'est implémenté. Décision 2026-06-30 (Damien) : prioriser d'abord l'implémentation correcte de tous les évènements fonctionnels, puis traiter ces obligations transverses.

Obligations de conformité reportées après la livraison du MVP : export RGPD du dossier d'un copropriétaire, envoi de courrier automatique (mise en demeure recommandée), et câblage effectif du journal d'audit probant sur les mutations comptables.

**Stories :**

* [EP-021-S01](docs/specs/EP-021-S01-spec.md) — Exporter les données personnelles d'un copropriétaire (droit d'accès RGPD) : bundle horodaté (profil, lots, traçabilité des charges) avec trace de la demande, dans le respect de la conservation comptable 10 ans (ex `EP-002-S09`, LEGAL.md §8)
* [EP-021-S02](docs/specs/EP-021-S02-spec.md) — Envoi de courrier automatique : générer et expédier la mise en demeure recommandée de l'art. 19-2 (déchéance du terme) — au MVP la mise en demeure est purement informative sous forme de notification in-app (LEGAL.md §4.7)
* [EP-021-S03](docs/specs/EP-021-S03-spec.md) — Câbler `audit_log` : écrire un enregistrement append-only à chaque mutation comptable probante (UPDATE/DELETE budget, appels de fonds, corrections) avec diff ciblé `old_values`/`new_values`, sans UPDATE/DELETE applicatif (ADR-009, EP-018-S02)

***

## Tâches techniques post-MVP

> À traiter en priorité une fois la livraison du MVP validée. Ces tâches ne sont pas des fonctionnalités produit mais des améliorations structurelles de la base de code.

| # | Tâche | Détail | Statut |
|---|-------|--------|--------|
| T-001 | Fusionner les migrations SQL en un fichier unique | Consolider `0001_init.sql` et `0002_tantieme_audit.sql` (et toute migration ultérieure) en un seul fichier `0001_init.sql` pour simplifier l'initialisation d'une nouvelle instance. | ✅ livrée |
| T-002-1 | Utilisation des composants radix-vue (ex: CModal > Dialog) | ✅ livrée (wrappers Radix, EP-020-S04/S06) |
| T-002-2 | Peaufiner les écrans, création de composants atomiques | en attente |
| T-002-3 | Rationaliser le CSS, généralisation de tailwind, réduction des classes custom | ✅ livrée (EP-020-S07 à S12, fichiers legacy supprimés) |
| T-002-4 | Vérifier utilisation en dur de height/width sur les icônes SVG sans classes `--icon-` | en attente |
| T-002-5 | Vérifier utilisation inline de balises de titres sans `CHeading` | en attente |
| T-002-6 | Vérifier utilisation inline de formulaires sans `CForm*` | en attente |
| T-002-7 | Améliorer les erreurs au chargement avec des vrais messages | en attente |
| T-002-8 | Améliorer les formulaires dans modale: message de feedback mal placé, bouton `close` inaccessible, délai de fermeture automatique trop long etc. | en attente |
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

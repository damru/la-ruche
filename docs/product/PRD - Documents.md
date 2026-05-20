---
product: Documents
version: 1.0.0
owner: Damien
status: validé
last_updated: 2026-05-21
references:
  - EP-005
  - LEGAL.md §1.6
  - LEGAL.md §8
  - ADR-001
llm_directives:
  temperature: 0.2
  persona: >
    Tu es un agent développeur senior travaillant sur la feature
    « Documents » de La Ruche. Tu DOIS respecter les contraintes
    techniques et les critères d'acceptation sans dérogation.
    Tu NE DOIS PAS implémenter de fonctionnalités listées dans Hors scope
    (GED externe, signature électronique probante, OCR, versioning multiple).
    Le backend NE MANIPULE PAS le contenu des fichiers — tout upload passe
    par une pre-signed URL S3 générée par le backend et utilisée directement
    par le client. Le téléchargement se fait exclusivement via URL signée à
    durée de vie courte — jamais par exposition directe du bucket S3.
    Les fichiers supprimés DOIVENT être marqués logiquement (soft-delete)
    avant toute suppression physique sur S3.
  modification_policy: >
    Ne modifier ce fichier que si une évolution de la feature est ABSOLUMENT
    nécessaire, et EXCLUSIVEMENT après validation explicite d'un PRD additionnel.
    Si une modification est nécessaire : inline, succincte, dans une section
    existante. Si aucune section ne convient, requérir une validation utilisateur.
    Le corps du document ne contient aucune référence technique externe
    (ADR, DDR, numéro de feature ou de ticket).
---

# PRD — Documents

> **Statut** — validé
> **Dernière modification** — 21 mai 2026
> **Références** — `LEGAL.md §1.6`, `LEGAL.md §8`

---

## 1. Vue d'ensemble

### Objectif

Fournir à la copropriété une **couche documentaire transverse** permettant de déposer, organiser et consulter les documents légaux et opérationnels de la copropriété, en répondant à l'obligation de l'espace en ligne sécurisé (art. 18-I-5° loi 1965, en vigueur depuis le 1er janvier 2015).

### Pourquoi

Cette feature est doublement justifiée :

1. **Obligation légale** : tout syndic doit mettre à disposition des copropriétaires un espace en ligne sécurisé donnant accès aux documents essentiels (règlement, PV d'AG, comptes, carnet d'entretien, contrats, diagnostics) — l'Application *est* cet espace.

2. **Infrastructure transverse** : les autres features reposent sur EP-005 pour le stockage de leurs pièces jointes : PV d'AG (EP-003), contrats fournisseurs (EP-006), sinistres (EP-008), carnet d'entretien (EP-009), tâches (EP-007).

Trois risques à couvrir :

- exposition publique du bucket S3, rendant les documents accessibles sans authentification ;
- suppression physique d'un document légalement obligatoire (PV d'AG, règlement) sans traçabilité ;
- absence de contrôle de rôle sur les catégories sensibles.

### KPIs

- Taux de copropriétés avec règlement de copropriété déposé.
- Taux de PV d'AG déposés dans les 30 jours suivant la clôture.
- Taux de téléchargements sans URL signée (cible : 0 — accès direct S3 interdit).

---

## 2. Contraintes techniques

> **CRITICAL** — Ces contraintes sont immuables. Tout agent ou développeur DOIT les respecter sans exception.

### Stack

- **Backend** : Rust (Axum / SQLx / PostgreSQL)
- **Frontend** : Vue 3 Composition API + TypeScript
- **Stockage** : S3-compatible (MinIO self-hosted)
- **Auth** : JWT / tokens opaques

### Upload via pre-signed URL

1. Le backend GÉNÈRE une pre-signed URL S3 valide 10 minutes et la retourne au client.
2. Le client UPLOADE directement sur S3 avec cette URL — le backend ne reçoit jamais le contenu du fichier.
3. À la fin de l'upload, le client notifie le backend : le backend crée l'entrée `document` en base de données.
4. Un upload non notifié dans le délai laisse un objet orphelin en S3 — un job de nettoyage périodique DOIT le supprimer.

### Téléchargement via URL signée

- Chaque téléchargement DOIT passer par une URL signée à durée limitée (défaut : 5 minutes).
- Le backend vérifie les droits de l'utilisateur avant de générer l'URL — aucune URL signée pour un utilisateur non autorisé.
- Le bucket S3 NE DOIT PAS être public — accès exclusivement via URL signée.
- L'URL signée NE DOIT PAS être mise en cache par le frontend au-delà de sa durée de validité.

### Suppression sécurisée

- La suppression DOIT être logique en premier lieu (`deleted_at`) — la suppression physique S3 est différée via job.
- Les documents référencés par une entité juridiquement critique (PV signé, règlement de copropriété en vigueur) NE DOIVENT PAS pouvoir être supprimés directement — le workflow de remplacement (US-DOC-007) s'applique.
- Un document supprimé logiquement NE DOIT PAS générer d'URL signée — retour 404.

### Validation côté backend

- Les types de fichiers autorisés DOIVENT être contrôlés côté backend : PDF, JPEG, PNG, DOCX, XLSX.
- La taille maximale DOIT être appliquée via la policy de la pre-signed URL (défaut : 20 Mo).

### Patterns interdits

- Backend manipulant le contenu des fichiers (lecture, écriture, transformation).
- Exposition directe du chemin S3 ou de l'URL permanente au frontend.
- URL de téléchargement sans expiration.
- Suppression physique immédiate sans soft-delete préalable.
- Ne pas utiliser `unwrap()` / `expect()` dans le code backend — utiliser `Result<T, E>`.

---

## 3. Profils utilisateurs

### Personas

| ID | Persona | Besoin principal | Attente clé |
|---|---|---|---|
| Persona-Syndic | Syndic bénévole | Déposer et organiser les documents légaux | Upload simple, catégories claires |
| Persona-Conseil | Conseil syndical | Accéder aux documents de gestion | Lecture sans friction |
| Persona-Copro | Copropriétaire | Accéder aux documents de sa copropriété | Documents pertinents, sans surcharge |
| Persona-Locataire | Locataire | Accéder aux documents autorisés | Sous-ensemble limité, expérience lisible |

### User stories de haut niveau

- En tant que `Persona-Syndic`, je veux déposer un document et le rattacher à une catégorie ou une entité pour organiser l'espace documentaire.
- En tant que `Persona-Syndic`, je veux contrôler quels rôles peuvent accéder à quelle catégorie de documents.
- En tant que tout utilisateur autorisé, je veux télécharger un document de manière sécurisée sans jamais accéder directement au bucket.
- En tant que `Persona-Copro`, je veux retrouver facilement le règlement de copropriété et les PV d'AG depuis mon espace.

### Matrice CRUD

| Catégorie documentaire | Syndic | Conseil | Copropriétaire | Locataire |
|---|---|---|---|---|
| Règlement de copropriété / EDD | `C/R/U/D¹` | `R` | `R` | `R` |
| Procès-verbal d'AG | `C/R/U/D¹` | `R` | `R` | `R` |
| Documents comptables | `C/R/U/D¹` | `R` | `R` | `X` |
| Appels de fonds (nominatifs) | `C/R/U/D¹` | `R` | `R¹` | `X` |
| Documents budgétaires | `C/R/U/D¹` | `R` | `R` | `X` |
| Contrats fournisseurs | `C/R/U/D¹` | `R` | `X` | `X` |
| Diagnostics (DPE, DTG…) | `C/R/U/D¹` | `R` | `R` | `R` |
| Documents de sinistres | `C/R/U/D¹` | `R` | `R¹` | `X` |
| Documents de tâches | `C/R/U/D¹` | `R/U` | `X` | `X` |
| Documents libres | `C/R/U/D¹` | `R/U` | `R¹` | `X` |

> ¹ `D¹` : suppression logique uniquement ; workflow de remplacement obligatoire pour les documents juridiquement critiques (règlement, PV signé).
> `R¹` (accès restreint) : pour sinistres et documents libres — copropriété de résidence uniquement ; pour appels de fonds — lots de l'utilisateur uniquement.

### Règles de sécurité

- Aucune URL S3 directe NE DOIT être transmise au frontend — uniquement des URL signées à durée limitée.
- La vérification des droits d'accès DOIT se faire côté backend **avant** génération de l'URL signée.
- Les documents supprimés logiquement NE DOIVENT PAS générer d'URL signée.
- Tous les documents sont filtrés par `building_id` — aucun accès cross-copropriété.

---

## 4. Règles de gestion

### Catégories documentaires

| Catégorie | Code | Accès minimum légal | Entité rattachée (optionnel) |
|---|---|---|---|
| Règlement de copropriété / EDD | `REGLEMENT` | Tous résidents | Fiche copropriété |
| Procès-verbal d'AG | `PV_AG` | Tous résidents | AG (`assembly_id`) |
| Documents comptables | `COMPTABILITE` | Syndic + Conseil + Copropriétaire | Copropriété |
| Appels de fonds (nominatif) | `APPEL_FONDS` | Syndic + Conseil + Copropriétaire (son lot uniquement) | Appel de fonds (`levy_call_id`) |
| Documents budgétaires | `BUDGET` | Syndic + Conseil + Copropriétaire | Budget |
| Contrat fournisseur | `CONTRAT` | Syndic + Conseil | Contrat (`contract_id`) |
| Diagnostic technique | `DIAGNOSTIC` | Tous résidents | Copropriété |
| Pièce sinistre | `SINISTRE` | Syndic + Conseil | Sinistre (`claim_id`) |
| Pièce tâche | `TACHE` | Syndic + Conseil | Tâche (`task_id`) |
| Document libre | `LIBRE` | Paramétrable | Optionnel |

### Cycle de vie d'un document

1. Le client demande une pre-signed URL (avec métadonnées : nom, catégorie, entité cible optionnelle).
2. Le backend valide les droits et génère l'URL signée S3 (valide 10 min).
3. Le client uploade directement sur S3.
4. Le client notifie le backend : création de l'entrée `document` en base.
5. Accès au document via URL signée à durée courte (5 min).
6. Suppression : soft-delete (`deleted_at`) — nettoyage S3 différé via job.

### Documents critiques vs standards

| Type | Suppression | Modification |
|---|---|---|
| Standard | Soft-delete direct | Remplacement libre |
| Critique (règlement, PV signé) | Workflow US-DOC-007 obligatoire | Workflow US-DOC-007 obligatoire |

### Obligations légales — Espace en ligne sécurisé (art. 18-I-5° loi 1965)

Le tableau suivant mappe chaque obligation légale à la catégorie documentaire EP-005 qui la satisfait. Le NFR-Legal-001 est considéré rempli si et seulement si toutes les catégories marquées ✅ sont accessibles 24h/24 aux résidents autorisés.

| Obligation légale (art. 18-I-5°) | Catégorie EP-005 | Couverture |
|---|---|---|
| Règlement de copropriété et EDD | `REGLEMENT` | ✅ Couverte |
| PV des AG (3 dernières années) | `PV_AG` | ✅ Couverte |
| Comptes du syndicat (arrêté de comptes) | `COMPTABILITE` | ✅ Couverte |
| Budget prévisionnel de l'exercice | `BUDGET` | ✅ Couverte |
| Appels de fonds individuels (Annexe II) | `APPEL_FONDS` | ✅ Couverte |
| Contrats d'assurance en vigueur | `CONTRAT` | ⚠️ Partielle — accès Syndic+Conseil uniquement (pas résidents) |
| Carnet d'entretien | future EP-009 | ❌ Non couverte — à implémenter en EP-009 |
| DTG / DPE collectif si établi | `DIAGNOSTIC` | ✅ Couverte |
| Plan Pluriannuel de Travaux si adopté | future EP-009 | ❌ Non couverte — à implémenter en EP-009 |

> Note : l'accès aux contrats d'assurance est actuellement réservé au Syndic et au Conseil (voir matrice CRUD §3). Une évolution vers un accès résident en lecture est à envisager pour satisfaire pleinement l'obligation.

### Suppression d'une entité rattachée

Le comportement des documents en cas de suppression de leur entité parente DOIT respecter les règles suivantes :

- **Soft-delete de l'entité** (ex. : tâche annulée, AG archivée) : le lien `entity_id` du document est **préservé** — le document reste accessible, l'entité apparaissant comme archivée dans son contexte.
- **Hard-delete de l'entité** (suppression physique en base) : le lien `entity_id` est mis à `NULL` via cascade DB — le document passe automatiquement en soft-delete (`deleted_at` renseigné). La suppression physique S3 est différée au job de nettoyage (US-DOC-008) selon la période de rétention définie.
- Aucun document NE DOIT être supprimé physiquement en dehors du job dédié (US-DOC-008).

### Événements de notification (architecture transverse)

> **Principe** — EP-005 émet des événements de domaine ; il ne gère pas la livraison. Un module de notification dédié (future EP à définir) consomme ces événements et gère les canaux de distribution (in-app, email), les abonnements par utilisateur et les préférences de fréquence.

Ce pattern est **transverse à toutes les features** de l'Application :

| Feature | Exemples d'événements émis |
|---|---|
| Assemblée Générale | `assembly.convocation_sent`, `assembly.pv_available` |
| Documents | `document.deposited` (catégories `PV_AG`, `COMPTABILITE`, `APPEL_FONDS`, `BUDGET`) |
| Tâches | `task.status_changed`, `task.assigned` |
| Sinistres | `claim.status_changed`, `claim.reimbursed` |
| Signalements | `report.submitted`, `report.resolved` |

Chaque PRD de feature DOIT définir les événements qu'il émet. La structure, les canaux et le stockage de ces événements seront spécifiés dans le PRD du module de notification (futur EP).

---

## 5. Exigences fonctionnelles

### MoSCoW

| Priorité | Ref | Action | Attendu produit |
|---|---|---|---|
| Must | US-DOC-001 | Déposer un document | Upload via pre-signed URL, catégorie, rattachement optionnel |
| Must | US-DOC-002 | Consulter et télécharger un document | Via URL signée, contrôle de rôle |
| Must | US-DOC-003 | Gérer les catégories et droits d'accès | Visibilité par catégorie selon rôle |
| Must | US-DOC-004 | Supprimer un document | Soft-delete, workflow remplacement pour docs critiques |
| Should | US-DOC-005 | Rattacher un document à une entité | Association AG, contrat, sinistre, tâche |
| Should | US-DOC-006 | Lister les documents d'une entité ou catégorie | Vue filtrée, recherche par nom, triée par date |
| Should | US-DOC-007 | Remplacer un document critique | Workflow explicite avec justification et conservation de l'historique |
| Should | US-DOC-008 | Nettoyage périodique S3 | Purge orphelins, purge soft-deleted après rétention |
| Should | US-DOC-009 | Émettre des événements de notification | Dépôt de document émet un événement consommable par le module notif (futur EP) |

### User stories détaillées

#### US-DOC-001 — Déposer un document

**En tant que** `Persona-Syndic` (ou tout rôle autorisé selon la catégorie),
**je veux** déposer un fichier en le catégorisant,
**afin de** le rendre accessible aux utilisateurs autorisés depuis l'espace sécurisé.

**Critères d'acceptation** :

- **AC-DOC-001-A** : Le dépôt DOIT passer par une pre-signed URL S3 générée par le backend — le contenu du fichier NE DOIT PAS transiter par le backend.
- **AC-DOC-001-B** : Les métadonnées obligatoires DOIVENT être saisies avant la génération de l'URL : nom du document, catégorie.
- **AC-DOC-001-C** : Les types de fichiers autorisés DOIVENT être restreints : PDF, JPEG, PNG, DOCX, XLSX — tout autre type DOIT être refusé avec un message explicite.
- **AC-DOC-001-D** : La taille maximale d'un fichier DOIT être limitée à 20 Mo — limite appliquée via la policy de la pre-signed URL.
- **AC-DOC-001-E** : Un upload non complété (client n'a pas notifié le backend) NE DOIT PAS créer d'entrée en base.
- **AC-DOC-001-F** : L'entrée en base DOIT contenir : nom, catégorie, MIME type, taille, `uploaded_by`, `uploaded_at`, clé S3.

#### US-DOC-002 — Consulter et télécharger un document

**En tant que** utilisateur autorisé,
**je veux** accéder à un document depuis l'interface,
**afin de** le consulter ou le télécharger de manière sécurisée.

**Critères d'acceptation** :

- **AC-DOC-002-A** : Tout accès DOIT passer par une URL signée à durée limitée (5 minutes par défaut) — aucune URL S3 permanente.
- **AC-DOC-002-B** : Le backend DOIT vérifier les droits de l'utilisateur sur la catégorie du document avant de générer l'URL signée.
- **AC-DOC-002-C** : Un document supprimé logiquement NE DOIT PAS retourner d'URL signée — retour 404.
- **AC-DOC-002-D** : La liste des documents DOIT afficher : nom, catégorie, date de dépôt, déposé par, taille.
- **AC-DOC-002-E** : Un document non accessible pour le rôle de l'utilisateur NE DOIT PAS apparaître dans la liste (filtrage côté backend).

#### US-DOC-003 — Gérer les catégories et droits d'accès

**En tant que** `Persona-Syndic`,
**je veux** comprendre quels rôles peuvent accéder à quelle catégorie,
**afin de** ne pas exposer de documents sensibles aux mauvais profils.

**Critères d'acceptation** :

- **AC-DOC-003-A** : Les droits d'accès par catégorie DOIVENT correspondre à la matrice définie en §3 — non configurables par le syndic dans le MVP.
- **AC-DOC-003-B** : L'interface DOIT afficher pour chaque catégorie le niveau d'accès applicable (label ou icône).
- **AC-DOC-003-C** : Une tentative d'accès à une catégorie non autorisée DOIT retourner 403.

#### US-DOC-004 — Supprimer un document

**En tant que** `Persona-Syndic`,
**je veux** supprimer un document déposé par erreur,
**afin de** maintenir un espace documentaire propre.

**Critères d'acceptation** :

- **AC-DOC-004-A** : La suppression DOIT être logique en premier lieu (`deleted_at` renseigné) — la suppression physique S3 est différée via job.
- **AC-DOC-004-B** : Un document critique (PV signé, règlement en vigueur) NE DOIT PAS pouvoir être supprimé directement — le workflow US-DOC-007 s'applique.
- **AC-DOC-004-C** : La suppression DOIT demander une confirmation explicite.
- **AC-DOC-004-D** : Un document supprimé DOIT disparaître immédiatement de toutes les listes — aucune URL signée ne DOIT être générée.

#### US-DOC-005 — Rattacher un document à une entité

**En tant que** `Persona-Syndic`,
**je veux** associer un document à une entité (AG, contrat, sinistre, tâche),
**afin de** le retrouver depuis le contexte de cette entité.

**Critères d'acceptation** :

- **AC-DOC-005-A** : Le rattachement DOIT être optionnel — un document peut exister sans entité associée.
- **AC-DOC-005-B** : Un document PEUT être rattaché à plusieurs entités (ex. : un plan rattaché à une tâche et au carnet d'entretien).
- **AC-DOC-005-C** : Depuis une fiche entité (AG, contrat, sinistre), les documents rattachés DOIVENT être visibles sans navigation supplémentaire.

#### US-DOC-006 — Lister les documents d'une entité ou catégorie

**En tant que** utilisateur autorisé,
**je veux** voir les documents disponibles pour une entité ou une catégorie,
**afin de** retrouver rapidement un document sans parcourir toute la bibliothèque.

**Critères d'acceptation** :

- **AC-DOC-006-A** : La liste DOIT être filtrable par catégorie, date de dépôt et entité rattachée.
- **AC-DOC-006-B** : La liste DOIT être triée par date de dépôt décroissante par défaut.
- **AC-DOC-006-C** : Les documents d'autres copropriétés NE DOIVENT PAS apparaître — filtrage strict par `building_id`.
- **AC-DOC-006-D** : La liste DOIT proposer une recherche textuelle sur le nom du document — recherche sur le champ `name` uniquement, côté backend (pas d'OCR, pas de recherche dans le contenu).

#### US-DOC-007 — Remplacer un document critique

**En tant que** `Persona-Syndic`,
**je veux** remplacer un document critique (PV signé, règlement en vigueur),
**afin de** corriger une erreur matérielle sans perdre la traçabilité.

**Critères d'acceptation** :

- **AC-DOC-007-A** : Le remplacement DOIT passer par une modale d'avertissement rappelant les implications juridiques.
- **AC-DOC-007-B** : Une justification textuelle DOIT être obligatoirement saisie avant le dépôt du nouveau document.
- **AC-DOC-007-C** : L'ancien document DOIT rester accessible en historique (soft-delete uniquement — jamais de suppression physique).
- **AC-DOC-007-D** : L'opération DOIT être tracée dans `audit_log` avec les deux références de document (ancien et nouveau) et la justification.

#### US-DOC-008 — Nettoyage périodique S3

**En tant qu'** opérateur système,
**je veux** qu'un job périodique nettoie les objets S3 obsolètes,
**afin d'éviter** l'accumulation d'objets orphelins et de garantir que les suppression logiques sont finalement exécutées physiquement.

**Critères d'acceptation** :

- **AC-DOC-008-A** : Le job DOIT supprimer physiquement de S3 les objets orphelins (pré-signés mais non notifiés au backend) après un délai de **24 heures** depuis la génération de l'URL.
- **AC-DOC-008-B** : Le job DOIT supprimer physiquement de S3 les documents ayant un `deleted_at` renseigné depuis plus de **30 jours** (période de rétention configurable par l'admin).
- **AC-DOC-008-C** : Avant toute suppression physique, le job DOIT vérifier que l'entrée en base existe et que `deleted_at` est bien renseigné — aucune suppression S3 sans confirmation DB.
- **AC-DOC-008-D** : Les documents critiques (règlement, PV signé) NE DOIVENT PAS être supprimés physiquement par le job, même après la période de rétention — ils restent en soft-delete permanent.
- **AC-DOC-008-E** : Chaque exécution du job DOIT écrire un compte-rendu (nombre d'objets supprimés, erreurs éventuelles) dans les logs applicatifs.
- **AC-DOC-008-F** : En cas d'erreur S3 lors d'une suppression individuelle, le job DOIT continuer le traitement des autres objets et logger l'erreur — aucune suppression partielle ne DOIT lever une exception fatale.

#### US-DOC-009 — Émettre des événements de notification

**En tant que** module EP-005,
**je veux** émettre des événements de domaine lors du dépôt de documents clés,
**afin de** permettre au module de notification (futur EP) d'alerter les utilisateurs concernés — sans coupler EP-005 à la logique de livraison.

**Critères d'acceptation** :

- **AC-DOC-009-A** : Le dépôt d'un document dans la catégorie `PV_AG` DOIT émettre un événement `document.deposited` avec : `document_id`, `category`, `building_id`, `uploaded_by`, `uploaded_at`.
- **AC-DOC-009-B** : Le dépôt d'un document dans les catégories `COMPTABILITE`, `APPEL_FONDS` ou `BUDGET` DOIT émettre le même événement `document.deposited`.
- **AC-DOC-009-C** : EP-005 NE DOIT PAS implémenter la logique de livraison (canaux, abonnements, emails) — il émet l'événement et délègue au module de notification (futur EP).
- **AC-DOC-009-D** : Si le module de notification n'est pas encore déployé, l'émission de l'événement DOIT échouer silencieusement (fire-and-forget) — le dépôt du document NE DOIT PAS être bloqué par l'absence du module notif.

---

## 6. Exigences non fonctionnelles

| ID | Catégorie | Exigence | Seuil mesurable |
|---|---|---|---|
| NFR-Sec-001 | Sécurité | Aucune URL S3 permanente | 0 lien de téléchargement sans expiration — contrôle côté backend |
| NFR-Sec-002 | Sécurité | Contrôle de rôle avant URL signée | 0 URL signée générée pour un utilisateur non autorisé |
| NFR-Perf-001 | Performance | Génération URL signée | ≤ 200 ms (p95) |
| NFR-Perf-002 | Performance | Chargement liste documents | ≤ 300 ms (p95) pour 100 documents |
| NFR-Legal-001 | Conformité | Espace en ligne sécurisé (art. 18-I-5° loi 1965) | Documents obligatoires (règlement, PV) accessibles 24h/24 aux résidents autorisés |
| NFR-Privacy-001 | Vie privée | Suppression physique S3 | Uniquement après soft-delete — jamais en direct |

---

## 7. Hors scope

> **CRITICAL** — Ne pas implémenter dans ce lot.

- Intégration avec une GED externe (SharePoint, Google Drive, Nextcloud).
- Signature électronique probante bout-en-bout.
- OCR et indexation full-text des documents.
- Gestion des versions multiples d'un document hors documents critiques (US-DOC-007 couvre uniquement la substitution de référence).
- Contrôle d'accès fin par document individuel (hors catégorie) — configurabilité avancée.
- Génération de documents (PDF, rapports) — hors périmètre du module Documents.
- Quota de stockage par copropriété.
- Catégorie dédiée pour l'état daté et le pré-état daté — à définir à l'implémentation EP-002-S06/S07.

---

## 8. Métriques de succès

| Métrique | Cible | Délai |
|---|---|---|
| Taux de copropriétés avec règlement de copropriété déposé | ≥ 90 % | À 1 mois post-onboarding |
| Taux de PV d'AG déposés dans les 30 jours suivant clôture | ≥ 80 % | À 3 mois |
| Taux de téléchargements sans URL signée (accès direct S3) | 0 % | Continu |
| Temps moyen de génération d'une URL signée | ≤ 200 ms (p95) | Continu |

---

## 9. Questions ouvertes

| # | Question | Owner | Condition de révision |
|---|---|---|---|
| Q-DOC-001 | La durée de vie par défaut des URL signées (5 min) est-elle suffisante pour les connexions lentes sur installations self-hosted ? | Damien | Avant déploiement production |
| Q-DOC-002 | Faut-il une catégorie dédiée pour les documents liés à l'état daté et au pré-état daté (EP-002-S06/S07) ? | Damien | Avant implémentation EP-002-S06 |
| Q-DOC-003 | Le job de nettoyage S3 (orphelins + soft-delete) doit-il être planifiable par l'admin ou tourner à intervalle fixe non configurable ? | Damien | Avant mise en production |
| Q-DOC-004 | Quel EP numéro pour le module de notification transverse (consommateur des événements `document.deposited`, `task.status_changed`, etc.) ? Faut-il créer un EP-018 dédié ou intégrer dans EP-017 (Journal d'activité) ? | Damien | Avant implémentation US-DOC-009 |

---

## Appendice A — Index de traçabilité

| US | Priorité | ACs | Domaine CRUD |
|---|---|---|---|
| US-DOC-001 | Must | AC-DOC-001-A à F | Documents (C) |
| US-DOC-002 | Must | AC-DOC-002-A à E | Documents (R) |
| US-DOC-003 | Must | AC-DOC-003-A à C | Catégories (R) |
| US-DOC-004 | Must | AC-DOC-004-A à D | Documents (D logique) |
| US-DOC-005 | Should | AC-DOC-005-A à C | Rattachement entités (C/U) |
| US-DOC-006 | Should | AC-DOC-006-A à D | Documents (R filtré + recherche) |
| US-DOC-007 | Should | AC-DOC-007-A à D | Remplacement document critique (C + audit) |
| US-DOC-008 | Should | AC-DOC-008-A à F | Nettoyage S3 (job périodique) |
| US-DOC-009 | Should | AC-DOC-009-A à D | Événements notification (fire-and-forget) |

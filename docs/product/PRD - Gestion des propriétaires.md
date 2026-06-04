---
product: Gestion des propriétaires
version: 1.1.0
owner: Damien
status: validé
last_updated: 2026-06-03
references:
  - EP-002
  - LEGAL.md §1.5
  - LEGAL.md §7
  - LEGAL.md §8
  - ADR-009
llm_directives:
  temperature: 0.2
  persona: >
    Tu es un agent développeur senior travaillant sur la feature
    « Gestion des propriétaires » de La Ruche. Tu DOIS respecter les
    contraintes techniques et les critères d'acceptation sans dérogation.
    Tu NE DOIS PAS implémenter de fonctionnalités listées dans Hors scope
    (état daté complet, pré-état daté, procédures judiciaires, envoi automatique d'invitations).
    L'anonymisation RGPD DOIT préserver les données comptables en remplaçant
    le nom par un identifiant neutre — jamais supprimer les lignes comptables.
    Les liens tokenisés d'invitation sont à usage unique et à durée limitée.
    Aucun accès cross-utilisateur n'est autorisé — isolation stricte par building_id + user_id.
---

# PRD — Gestion des propriétaires

## 1. Vue d'ensemble

### Objectif

Permettre au syndic bénévole de **gérer le registre des résidents de sa copropriété** : copropriétaires et locataires, de leur arrivée à leur départ, en couvrant les invitations, les mutations de propriété, les rôles et l'anonymisation RGPD.

### Pourquoi

La liste des copropriétaires est une obligation légale (art. 18-I-1° loi 1965). Elle conditionne :

- l'envoi des convocations d'AG (adresse, email) ;
- le calcul du quorum par lot (tantièmes du lot, présence / absence) ;
- la génération de l'état daté en cas de mutation (charges courantes non soldées) ;
- le respect des droits RGPD (anonymisation à la demande).

Trois risques à couvrir :

- liste obsolète (copropriétaires partis non archivés), faussant les convocations et le quorum ;
- perte de données comptables historiques lors d'une suppression ou anonymisation mal gérée ;
- invitation non sécurisée donnant un accès non maîtrisé à des données de la copropriété.

### KPIs

- Taux de résidents avec email renseigné (condition de réception des convocations).
- Délai moyen entre mutation d'un lot et archivage de l'ancien propriétaire.
- Taux d'invitations acceptées dans les 72 heures.

---

## 2. Contraintes techniques

> **CRITICAL** — Ces contraintes sont immuables. Tout agent ou développeur DOIT les respecter sans exception.

### Stack

- **Backend** : Rust (Axum / SQLx / PostgreSQL)
- **Frontend** : Vue 3 Composition API + TypeScript
- **Auth** : JWT / tokens opaques

### Invitations par lien tokenisé

- Le lien d'invitation DOIT être à usage unique, avec expiration configurable (défaut : 72h).
- Le token DOIT être généré de manière cryptographiquement sûre (`rand::rngs::OsRng` ou équivalent CSPRNG).
- Une invitation expirée DOIT retourner 410 Gone avec message explicite à l'utilisateur.
- Une invitation déjà utilisée DOIT retourner 409 Conflict.
- Le lien NE DOIT PAS contenir d'informations personnelles dans l'URL (ni nom, ni email).

### Anonymisation RGPD

- L'anonymisation DOIT remplacer : nom, prénom, email, téléphone par des valeurs neutres (format : `ANONYMISÉ-{uuid}`).
- Elle NE DOIT PAS supprimer : les `levy_call` rattachés au lot, les `vote`, les `resolution`, les entrées d'`audit_log`.
- Les données de connexion (sessions actives, refresh tokens) DOIVENT être supprimées immédiatement.
- L'opération DOIT être tracée dans `audit_log` avec horodatage et `user_id` du syndic déclencheur.
- L'anonymisation est **irréversible** — une confirmation explicite avec resaisie du nom de l'utilisateur est requise.

### Mutations de propriété

- Une mutation change le propriétaire d'un lot à une date effective.
- L'ancien propriétaire est archivé sur le lot — son accès à la copropriété est révoqué à la date effective.
- Les tantièmes restent attachés au lot : ils ne changent pas avec la mutation (art. 6 loi 1965).
- Les charges courantes non soldées au jour de la mutation sont tracées pour la production future de l'état daté.

### Patterns interdits

- Suppression physique d'un utilisateur ayant des données comptables ou juridiques associées.
- Accès cross-utilisateur aux données personnelles depuis n'importe quel rôle.
- Invitation sans expiration définie.
- Ne pas utiliser `unwrap()` / `expect()` dans le code backend — utiliser `Result<T, E>`.

---

## 3. Profils utilisateurs

### Personas

| ID | Persona | Besoin principal | Attente clé |
|---|---|---|---|
| Persona-Syndic | Syndic bénévole | Maintenir le registre à jour | Être guidé à chaque mutation ou départ |
| Persona-Conseil | Conseil syndical | Consulter la liste des résidents | Visibilité sans modification |
| Persona-Copro | Copropriétaire | Accéder à l'espace sécurisé | Invitation simple et sécurisée |
| Persona-Locataire | Locataire | Accéder à l'espace sécurisé | Invitation simple, périmètre limité |

### User stories de haut niveau

- En tant que `Persona-Syndic`, je veux maintenir la liste complète des copropriétaires pour remplir mon obligation légale (art. 18-I-1° loi 1965).
- En tant que `Persona-Syndic`, je veux inviter un résident par lien tokenisé pour lui donner accès à l'espace sécurisé.
- En tant que `Persona-Syndic`, je veux enregistrer une mutation de lot pour que le nouveau propriétaire soit pris en compte immédiatement.
- En tant que `Persona-Syndic`, je veux anonymiser un compte à la demande pour respecter le RGPD sans perdre les données comptables.
- En tant que `Persona-Copro` ou `Persona-Locataire`, je veux m'inscrire depuis le lien reçu sans parcours complexe.

### Matrice CRUD

| Domaine | Syndic | Conseil | Copropriétaire | Locataire |
|---|---|---|---|---|
| Liste des résidents | `C/R/U/D` | `R` | `X` | `X` |
| Fiche résident | `C/R/U/D` | `R` | `R¹/U¹` | `R¹/U¹` |
| Invitations | `C/R/D` | `X` | `X` | `X` |
| Mutations de propriété | `C/R/U` | `R` | `X` | `X` |
| Rôles des résidents | `C/R/U/D` | `R` | `X` | `X` |
| Archivage compte | `C/R/U` | `X` | `X` | `X` |
| Anonymisation RGPD | `U` (sur demande) | `X` | `X` | `X` |

> ¹ Restreint aux propres données de l'utilisateur — voir PRD Profil et préférences pour la gestion autonome.

### Règles de sécurité

- Un copropriétaire NE DOIT PAS voir les coordonnées des autres copropriétaires.
- La liste des résidents est réservée au Syndic et au Conseil.
- Aucune invitation ne DOIT exposer l'adresse email de l'invitant dans l'URL.
- L'accès à la fiche d'un résident est filtré strictement par `building_id` (copropriété de résidence).

---

## 4. Cycle de vie

### Utilisateurs

```
no_access ──(invitation envoyée)──▶ pending_invite ──(acceptation)──▶ active ──(archivage)──▶ archived ──(RGPD)──▶ anonymized
    │                                                                      │          │
    └──(email ajouté + invitation tardive)──▶ pending_invite              │          └─(réactivation*)──▶ active
                                                                           │          * si non anonymisé
                                               (révocation invite)        │
pending_invite ──────────────────────────────────────────────────▶ no_access (email conservé)
```

| État | Description |
|---|---|
| `no_access` | Résident enregistré dans le registre, sans email ou sans invitation — aucun accès à l'Application |
| `pending_invite` | Lien généré, non encore accepté — pas d'accès à l'Application |
| `active` | Résident actif avec accès selon son rôle |
| `archived` | Accès révoqué, données conservées intactes |
| `anonymized` | Données personnelles remplacées — état irréversible |

| Transition | Déclencheur | Condition | US |
|---|---|---|---|
| création → `no_access` | Syndic | Email absent ou invitation non demandée | US-GP-008 |
| `no_access` → `pending_invite` | Syndic | Email présent + envoi invitation | US-GP-008 |
| `pending_invite` → `no_access` | Syndic | Révocation du lien | US-GP-002 |
| `pending_invite` → `active` | Résident — accept. du lien | Token valide, non expiré | US-GP-002 |
| `active` → `archived` | Syndic | Confirmation + date de départ | US-GP-004 |
| `archived` → `active` | Syndic — réactivation | Compte non anonymisé | US-GP-004 |
| `archived` → `anonymized` | Syndic (demande RGPD) | Confirmation + resaisie nom | US-GP-005 |

### Lots — état de propriété

```
active_owner ──(mutation)──▶ pending_transfer ──(invitation acceptée)──▶ active_owner
```

| État | Description |
|---|---|
| `active_owner` | Propriétaire actif lié au lot |
| `pending_transfer` | Ancien propriétaire archivé, invitation envoyée au nouveau — lot sans propriétaire actif |

---

## 5. Règles de gestion

### Rôles résidents

| Rôle | Description | Restrictions principales |
|---|---|---|
| `syndic` | Administrateur complet de la copropriété | Un seul actif par copropriété |
| `board_member` | Membre du conseil syndical | Plusieurs autorisés |
| `owner` | Copropriétaire propriétaire d'un ou plusieurs lots | Peut posséder plusieurs lots |
| `tenant` | Locataire occupant un lot | Pas de droit de vote AG |

### Historique de propriété

- Chaque lot peut avoir plusieurs propriétaires successifs dans le temps.
- L'enregistrement d'une mutation crée un nouveau lien `unit_ownership` avec `start_date` et archive l'ancien avec `end_date`.
- Un lot NE PEUT PAS avoir deux propriétaires actifs simultanément.

### Archivage vs Anonymisation

- **Archivage** (départ sans demande RGPD) : le compte est désactivé, les données sont conservées intactes, l'accès est révoqué.
- **Anonymisation** (demande RGPD) : les données personnelles identifiantes sont remplacées par des valeurs neutres. Les données comptables et juridiques restent intactes.

### Durées de conservation (LEGAL.md §8)

| Donnée | Durée |
|---|---|
| Données d'identité (copropriétaire actif) | Durée du mandat + 5 ans |
| Données comptables (charges, appels de fonds) | 10 ans (art. L123-22 Code de commerce) |
| Données de connexion | 1 an (CNIL) |

### Isolation multi-bâtiments

Un même utilisateur peut être copropriétaire dans plusieurs copropriétés gérées par la même instance. Chaque contexte est **strictement isolé** : toutes les requêtes DOIVENT être filtrées par `building_id` — aucune donnée d'une copropriété A n'est visible depuis le contexte de la copropriété B, même pour le même `user_id`.

---

## 6. Exigences fonctionnelles

### MoSCoW

| Priorité | Ref | Action | Attendu produit |
|---|---|---|---|
| Must | US-GP-001 | Gérer la liste des résidents | CRUD complet avec état civil, lots, coordonnées — email optionnel |
| Must | US-GP-002 | Inviter un résident | Lien tokenisé, expiration, usage unique |
| Must | US-GP-003 | Enregistrer une mutation de propriété | Archivage ancien, création nouveau lien lot↔propriétaire |
| Must | US-GP-004 | Archiver un compte résident | Désactivation sans suppression des données |
| Must | US-GP-005 | Anonymiser un compte (RGPD) | Remplacement données personnelles, conservation comptable |
| Must | US-GP-008 | Créer un résident sans invitation immédiate | Ajout au registre sans email, invitation différée depuis la liste |
| Should | US-GP-006 | Gérer les rôles des résidents | Attribution et révocation des rôles |
| Should | US-GP-007 | Consulter l'historique de propriété d'un lot | Vue chronologique par lot |
| Should | US-GP-009 | Vue des charges par lot avec filtre propriétaire | Tableau de bord syndic : appels de fonds par lot, filtrable par propriétaire historique |

### User stories détaillées

#### US-GP-001 — Gérer la liste des résidents

**En tant que** `Persona-Syndic`,
**je veux** créer et maintenir la fiche de chaque résident,
**afin de** respecter l'obligation légale de tenue du registre (art. 18-I-1° loi 1965).

**Critères d'acceptation** :

- **AC-GP-001-A** : Une fiche résident DOIT contenir : nom, prénom, email **(optionnel)**, téléphone (optionnel), lot(s) associé(s), rôle.
- **AC-GP-001-B** : Si un email est renseigné, il DOIT être unique dans le système — doublon DOIT retourner une erreur de validation explicite. Un résident sans email n'est pas soumis à cette contrainte.
- **AC-GP-001-F** : Un résident créé sans email se trouve à l'état `no_access` : aucun accès à l'Application, aucune invitation en attente. La liste DOIT l'indiquer visuellement.
- **AC-GP-001-C** : Un copropriétaire PEUT être associé à plusieurs lots de la même copropriété.
- **AC-GP-001-D** : La liste DOIT distinguer clairement les résidents actifs des archivés.
- **AC-GP-001-E** : La liste DOIT être filtrable par rôle (propriétaire / locataire / conseil / syndic) et par lot.

#### US-GP-002 — Inviter un résident

**En tant que** `Persona-Syndic`,
**je veux** générer un lien d'invitation pour un résident,
**afin de** lui permettre d'accéder à l'espace sécurisé de la copropriété.

**Critères d'acceptation** :

- **AC-GP-002-A** : Le lien d'invitation DOIT être à usage unique et expirer après 72h par défaut (configurable).
- **AC-GP-002-B** : Le token DOIT être généré par CSPRNG — aucun élément personnel dans l'URL.
- **AC-GP-002-C** : Une invitation expirée DOIT retourner 410 Gone avec message explicite.
- **AC-GP-002-D** : Une invitation déjà utilisée DOIT retourner 409 Conflict.
- **AC-GP-002-E** : Le syndic DOIT pouvoir révoquer une invitation non utilisée avant expiration.
- **AC-GP-002-F** : Le lien DOIT être transmis manuellement par le syndic — l'Application ne gère pas l'envoi (email / SMS) pour le MVP.
- **AC-GP-002-G** : Pour une invitation expirée non utilisée, le Syndic DOIT pouvoir générer un nouveau lien — l'ancien token DOIT être invalidé automatiquement à la génération du nouveau.
- **AC-GP-002-H** : Pour une invitation au rôle `tenant`, l'interface DOIT préciser le lot de résidence associé dans le formulaire de saisie ; si le copropriétaire du lot est actif dans l'Application, une notification informative (non bloquante) DOIT lui être visible dans son tableau de bord lors de la prochaine connexion.

#### US-GP-003 — Enregistrer une mutation de propriété

**En tant que** `Persona-Syndic`,
**je veux** enregistrer la cession d'un lot à un nouveau propriétaire,
**afin de** maintenir un registre exact et de disposer des données pour l'état daté.

**Critères d'acceptation** :

- **AC-GP-003-A** : Une mutation DOIT saisir : date effective, ancien propriétaire, nouveau propriétaire (existant dans le système ou nouveau à créer).
- **AC-GP-003-B** : L'accès de l'ancien propriétaire à la copropriété DOIT être révoqué à la date effective de la mutation.
- **AC-GP-003-C** : Les tantièmes du lot NE DOIVENT PAS changer lors d'une mutation (conformément à art. 6 loi 1965).
- **AC-GP-003-D** : Les charges courantes non soldées à la date de mutation DOIVENT être tracées et consultables (données nécessaires à l'état daté — cf. §7 Hors scope).
- **AC-GP-003-E** : Un lot NE PEUT PAS avoir deux propriétaires actifs simultanément — tentative DOIT retourner 422.
- **AC-GP-003-F** : Entre l'archivage de l'ancien propriétaire et l'acceptation de l'invitation par le nouveau, le lot est dans l'état `pending_transfer` — les appels de fonds restent rattachés au lot (non à une personne) et sont consultables par le Syndic. Aucun accès résident n'est actif sur le lot durant cette période.

#### US-GP-004 — Archiver un compte résident

**En tant que** `Persona-Syndic`,
**je veux** archiver un compte résident qui quitte la copropriété,
**afin de** révoquer son accès tout en conservant son historique.

**Critères d'acceptation** :

- **AC-GP-004-A** : L'archivage DOIT révoquer l'accès immédiatement (invalidation des tokens actifs).
- **AC-GP-004-B** : Les données personnelles et comptables DOIVENT être conservées intactes après archivage.
- **AC-GP-004-C** : Un compte archivé DOIT pouvoir être réactivé si l'archivage était erroné, sous réserve qu'aucune anonymisation n'ait été effectuée.
- **AC-GP-004-D** : L'archivage DOIT demander une confirmation explicite avec la date de départ.

#### US-GP-005 — Anonymiser un compte (RGPD)

**En tant que** `Persona-Syndic` (sur demande de l'utilisateur),
**je veux** anonymiser les données personnelles d'un résident,
**afin de** respecter son droit à l'effacement (RGPD art. 17) sans compromettre les obligations comptables (art. L123-22 Code de commerce).

**Critères d'acceptation** :

- **AC-GP-005-A** : L'anonymisation DOIT remplacer nom, prénom, email et téléphone par `ANONYMISÉ-{uuid}`.
- **AC-GP-005-B** : Les données comptables (appels de fonds, votes AG, `audit_log`) NE DOIVENT PAS être supprimées ni altérées.
- **AC-GP-005-C** : Une confirmation explicite avec resaisie du nom complet de l'utilisateur DOIT être requise avant exécution.
- **AC-GP-005-D** : L'opération DOIT être tracée dans `audit_log` avec horodatage, `user_id` du syndic et `building_id`.
- **AC-GP-005-E** : L'anonymisation est irréversible — un avertissement explicite DOIT être affiché avant la confirmation.
- **AC-GP-005-F** : Les données de connexion (sessions actives, tokens) DOIVENT être supprimées immédiatement à l'anonymisation.

#### US-GP-006 — Gérer les rôles des résidents

**En tant que** `Persona-Syndic`,
**je veux** attribuer et révoquer les rôles des résidents,
**afin de** contrôler précisément les droits d'accès à la copropriété.

**Critères d'acceptation** :

- **AC-GP-006-A** : Les rôles disponibles DOIVENT être : `syndic`, `board_member`, `owner`, `tenant`.
- **AC-GP-006-B** : Un seul `syndic` PEUT être actif à la fois par copropriété.
- **AC-GP-006-C** : La révocation d'un rôle DOIT être immédiate (invalidation du cache de permissions).
- **AC-GP-006-D** : Le Syndic NE PEUT PAS révoquer son propre rôle sans désigner un successeur au préalable.

#### US-GP-007 — Consulter l'historique de propriété d'un lot

**En tant que** `Persona-Syndic`,
**je veux** consulter la chronologie des propriétaires d'un lot,
**afin de** disposer d'une traçabilité complète pour les besoins légaux.

**Critères d'acceptation** :

- **AC-GP-007-A** : L'historique DOIT afficher pour chaque propriétaire : nom (ou `ANONYMISÉ-{uuid}` si anonymisé), période de propriété (date début – date fin).
- **AC-GP-007-B** : La consultation DOIT être réservée au Syndic et au Conseil.
- **AC-GP-007-C** : Les données anonymisées NE DOIVENT PAS afficher d'informations permettant la réidentification.

#### US-GP-008 — Créer un résident sans invitation immédiate

**En tant que** `Persona-Syndic`,
**je veux** pouvoir enregistrer un résident dans le registre sans lui envoyer d'invitation immédiatement,
**afin de** maintenir un registre exact des occupants même lorsque l'adresse email n'est pas disponible ou que l'invitation doit être différée.

**Contexte** : Un syndic bénévole peut recevoir les informations d'un nouveau copropriétaire (nom, lot, date d'entrée) avant d'avoir son adresse email. Il doit pouvoir créer la fiche résident dans le registre, puis déclencher l'invitation ultérieurement depuis la liste — sans avoir à recréer la fiche.

**Critères d'acceptation** :

- **AC-GP-008-A** : Le formulaire de création d'un résident DOIT accepter la soumission sans adresse email — le résident est alors créé à l'état `no_access`.
- **AC-GP-008-B** : Si une adresse email est saisie lors de la création, un bouton **« Inviter maintenant »** optionnel DOIT apparaître. L'invitation N'EST PAS déclenchée automatiquement — c'est un choix explicite du Syndic.
- **AC-GP-008-C** : Depuis la liste des résidents, le Syndic DOIT pouvoir ouvrir la fiche d'un résident à l'état `no_access` pour y ajouter ou modifier son adresse email.
- **AC-GP-008-D** : Depuis la liste des résidents, un résident à l'état `no_access` disposant d'un email DOIT afficher un bouton **« Envoyer l'invitation »** déclenchant le flux d'invitation existant (US-GP-002) sans quitter la liste.
- **AC-GP-008-E** : Un résident à l'état `no_access` DOIT être distingué visuellement dans la liste par un badge dédié — distinct de `pending_invite` et `active`.
- **AC-GP-008-F** : L'invitation tardive DOIT appliquer les mêmes règles que l'invitation initiale : token CSPRNG, usage unique, expiration 72 h, invalidation du précédent token en attente (AC-GP-002-A, AC-GP-002-B, AC-GP-002-G).
- **AC-GP-008-G** : Un résident à l'état `no_access` PEUT être associé à un lot et avoir un rôle — ces données sont fonctionnelles (tantièmes, quorum AG) même sans accès à l'Application.

#### US-GP-009 — Vue des charges par lot avec filtre propriétaire

**En tant que** `Persona-Syndic`,
**je veux** consulter les appels de fonds lot par lot et filtrer par propriétaire lorsqu'un lot a changé de main,
**afin de** suivre les charges courantes et historiques indépendamment des mutations de propriété.

**Contexte** : Après une mutation, un même lot peut avoir plusieurs propriétaires successifs dans l'historique. Le syndic doit pouvoir identifier les charges rattachées à chaque période de propriété — notamment pour préparer un état daté ou vérifier qu'un appel de fonds impayé était bien adressé à l'ancien ou au nouveau propriétaire.

**Critères d'acceptation** :

- **AC-GP-009-A** : La vue des charges DOIT afficher les appels de fonds regroupés par lot (numéro de lot), pour le propriétaire courant par défaut.
- **AC-GP-009-B** : Si un lot a eu plusieurs propriétaires, un sélecteur DOIT permettre au syndic de filtrer les appels de fonds par propriétaire historique (liste déroulante avec période de propriété). Ce sélecteur est visible uniquement pour le rôle `syndic` ; le filtre s'applique côté client sur les données de la vue détail du lot (`UnitChargesView`).
- **AC-GP-009-C** : Les appels de fonds DOIVENT rester rattachés au lot, pas à la personne — la mutation ne doit pas altérer ni déplacer les enregistrements existants.
- **AC-GP-009-D** : La vue des charges par lot (`UnitChargesView`) est accessible aux rôles `syndic`, `board_member` et `owner` (copropriétaire pour ses propres lots). Le filtre propriétaire historique (AC-GP-009-B) est réservé au rôle `syndic` uniquement.
- **AC-GP-009-E** : Un lot en état `pending_transfer` (sans propriétaire actif) DOIT afficher ses appels de fonds sans propriétaire associé, avec une indication visuelle claire.

---

## 7. Exigences non fonctionnelles

| ID | Catégorie | Exigence | Seuil mesurable |
|---|---|---|---|
| NFR-Sec-001 | Sécurité | Token d'invitation | Généré par CSPRNG, usage unique, expiration — 0 réutilisation possible |
| NFR-Privacy-001 | Vie privée | Anonymisation RGPD | Données personnelles effacées sans exception sur les champs ciblés — 0 fuite résiduelle |
| NFR-Privacy-002 | Vie privée | Données comptables après anonymisation | Conservées intégralement — 0 ligne supprimée |
| NFR-Privacy-003 | Vie privée | Durée conservation données connexion | Supprimées à l'anonymisation — conformité CNIL |
| NFR-Perf-001 | Performance | Chargement liste résidents | ≤ 300 ms (p95) pour 200 résidents |
| NFR-Sec-002 | Sécurité | Isolation cross-user | Aucune donnée d'un autre utilisateur accessible — 0 exception |

---

## 8. Hors scope

> **CRITICAL** — Ne pas implémenter dans ce lot.

- Production de l'état daté complet (art. 5 décret 1967) — lot futur.
- Pré-état daté (art. L721-2 Code de la construction) — lot futur.
- Suivi des procédures judiciaires — lot futur.
- Gestion des délégations complexes du conseil syndical (art. 21-1 décret 1967) — lot futur.
- Export RGPD détaillé des données personnelles (droit d'accès) — lot futur.
- Gestion du mot de passe, MFA — couverts par la feature Authentification.
- Envoi automatisé des invitations (email transactionnel) — lot futur.
- PV du conseil syndical en délégation de pouvoirs (art. 21-1 décret 1967) — lot futur.

---

## 9. Métriques de succès

| Métrique | Cible | Délai |
|---|---|---|
| Taux de résidents avec email renseigné | ≥ 90 % | À 1 mois post-onboarding |
| Taux d'invitations acceptées sous 72h | ≥ 70 % | À 3 mois |
| Délai moyen archivage après mutation | < 7 jours | À 3 mois |
| Nombre de suppressions physiques de données comptables | 0 | Continu |

---

## 10. Questions ouvertes

| # | Question | Owner | Condition de révision |
|---|---|---|---|
| Q-GP-001 | Un locataire peut-il s'auto-inviter via un lien public de copropriété, ou faut-il systématiquement une invitation nominative du syndic ? | Damien | Avant implémentation US-GP-002 |
| Q-GP-002 | Le délai d'expiration d'invitation à 72h est-il adapté aux syndics bénévoles peu disponibles — faut-il monter à 7 jours par défaut ? | Damien | Avant implémentation US-GP-002 |
| Q-GP-003 | Les charges non soldées à la mutation doivent-elles déjà être exportables en PDF à ce stade, ou simplement consultables à l'écran ? | Damien | Avant implémentation US-GP-003 |

---

## Appendice A — Index de traçabilité

| US | Priorité | ACs | Domaine CRUD |
|---|---|---|---|
| US-GP-001 | Must | AC-GP-001-A à F | Liste résidents (C/R/U) — email optionnel |
| US-GP-002 | Must | AC-GP-002-A à H | Invitations (C/R/D) |
| US-GP-003 | Must | AC-GP-003-A à F | Mutations de propriété (C/R) |
| US-GP-004 | Must | AC-GP-004-A à D | Archivage compte (U) |
| US-GP-005 | Must | AC-GP-005-A à F | Anonymisation RGPD (U irréversible) |
| US-GP-006 | Should | AC-GP-006-A à D | Rôles résidents (C/U/D) |
| US-GP-007 | Should | AC-GP-007-A à C | Historique propriété (R) |
| US-GP-008 | Must | AC-GP-008-A à G | Création sans invite + invitation différée (C/U) |
| US-GP-009 | Should | AC-GP-009-A à E | Vue charges par lot avec filtre propriétaire (R) |

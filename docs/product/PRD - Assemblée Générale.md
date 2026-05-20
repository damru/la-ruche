---
product: Assemblée Générale
version: 2.0.0
owner: Damien
status: validé
last_updated: 2026-05-20
references:
  - EP-003
  - EP-013
  - LEGAL.md §3
  - ADR-009
  - ADR-006
llm_directives:
  temperature: 0.2
  persona: >
    Tu es un agent développeur senior travaillant sur la feature
    « Assemblée Générale » de La Ruche. Tu DOIS respecter les contraintes
    techniques et les critères d'acceptation sans dérogation.
    Tu NE DOIS PAS implémenter de fonctionnalités listées dans Hors scope.
    Les données figées après clôture de séance (votes, résultats, feuille de
    présence, substance des résolutions) sont immuables — aucun PUT/PATCH sur
    ces données après l'état `closed`. Le PV signé déposé est append-only.
    Le moteur de vote DOIT appliquer les règles art. 24, 25, 25-1, 26, 26-1
    et unanimité sans raccourci.
  modification_policy: >
    Ne modifier ce fichier que si une évolution de la feature est ABSOLUMENT
    nécessaire, et EXCLUSIVEMENT après validation explicite d'un PRD additionnel.
    Si une modification est nécessaire : inline, succincte, dans une section
    existante. Si aucune section ne convient, requérir une validation utilisateur.
    Le corps du document ne contient aucune référence technique externe
    (ADR, DDR, numéro de feature ou de ticket).
---

# PRD — Assemblée Générale

> **Statut** — en cours
> **Dernière modification** — 20 mai 2026
> **Références** — `EP-003`, `EP-013`, `LEGAL.md §3`, `ADR-009`, `ADR-006`

---

## 1. Vue d'ensemble

### Objectif

Permettre au syndic bénévole de **préparer, tenir et clôturer une Assemblée Générale juridiquement exploitable**, depuis la convocation jusqu'à la notification du PV, en distinguant clairement :

- la préparation juridique avant séance ;
- la conduite de séance ;
- le traitement post-AG et la preuve documentaire.

### Pourquoi

L'AG est le point de bascule entre décision collective, opposabilité et exécution. La feature doit réduire trois risques majeurs :

- risque de non-conformité documentaire avant convocation ;
- risque d'erreur de calcul ou de procédure pendant la séance ;
- risque de perte de traçabilité après clôture, en particulier sur le PV et sa notification.

### KPIs

- Délai moyen entre `clôture de séance` et `notification du PV`.
- Taux d'AG convoquées avec dossier complet avant envoi.
- Taux de résolutions finalisées sans correction manuelle post-séance.

---

## 2. Contraintes techniques

> **CRITICAL** — Ces contraintes sont immuables. Tout agent ou développeur DOIT les respecter sans exception avant d'implémenter une quelconque fonctionnalité.

### Stack

- **Backend** : Rust (Axum / SQLx / PostgreSQL) — voir ADR-001
- **Frontend** : Vue 3 Composition API + TypeScript — voir DDR-001/002
- **Auth** : JWT / tokens opaques — voir ADR-002

### Immutabilité comptable et juridique (ADR-009 — Pattern A)

Les données suivantes sont **figées dès leurs états respectifs** et ne peuvent plus être modifiées :

| Données | Figées depuis | Interdit après |
|---|---|---|
| Votes par lot par résolution | `closed` | PUT / PATCH / DELETE |
| Résultats par résolution | `closed` | PUT / PATCH |
| Substance des résolutions votées | `closed` | PUT / PATCH |
| Feuille de présence effective | `ongoing` (démarrage) | PUT / PATCH après gel |
| PV signé déposé | Dépôt explicite | DELETE / PATCH |

Toute mutation sur des données juridiquement engagées DOIT écrire dans `audit_log` via `db::audit_log::record()`.

### Dépendances obligatoires

- **Module tantièmes** (ADR-006) : DOIT être disponible et cohérent avant tout démarrage de séance. Le calcul du quorum et des résultats dépend des clés de répartition en vigueur.
- **Ordre du jour verrouillé** : figé à la transition `convoque` → `en_seance` — tout ajout post-démarrage est interdit.

### Moteur de vote

Le moteur DOIT implémenter les règles suivantes **sans raccourci** :

- **Art. 24** : majorité des voix exprimées
- **Art. 25** : majorité des voix de tous les copropriétaires
- **Art. 25-1** : second vote à l'art. 24 si majorité absolue non atteinte mais résultat > 1/3
- **Art. 26** : double majorité (nombre et tantièmes)
- **Art. 26-1** : second vote à l'art. 25 si conditions remplies
- **Unanimité** : tous les copropriétaires présents ou représentés

### Patterns interdits

- Aucun `PUT /assemblies/:id` générique ne DOIT modifier une donnée figée.
- Aucune transition d'état ne DOIT contourner la machine à états décrite en §4.
- Suppression d'une résolution dont le vote a été saisi : **interdite**.
- Recalcul manuel d'un résultat de vote finalisé : **interdit** — utiliser un amendement explicite avec workflow.
- Ne pas utiliser `unwrap()` / `expect()` dans le code backend — utiliser `Result<T, E>`.
### Navigation

- La fiche AG DOIT être une page unique organisée en trois onglets : **`Préparation`**, **`Séance`**, **`Post-AG`**.
- Les transitions d’état DOIVENT être accessibles depuis l’onglet pertinent — pas de pages séparées par phase.
---

## 3. Profils utilisateurs

### Personas

| ID | Persona | Besoin principal | Attente clé |
|---|---|---|---|
| Persona-Syndic | Syndic bénévole | Piloter l'ensemble du cycle AG | Sécuriser la conformité sans outil expert |
| Persona-Conseil | Conseil syndical | Contrôler le dossier et suivre la séance | Visibilité sans porter l'administration |
| Persona-Copro | Copropriétaire | Comprendre, participer, voter | Voir les informations utiles et exercer son droit |
| Persona-Locataire | Locataire | Suivre les décisions sans voter | Accéder à l'information sans confusion sur ses droits |

### User stories de haut niveau

**Préparation avant séance**

- En tant que `Persona-Syndic`, je veux créer un dossier d'AG complet pour sécuriser la convocation avant l'envoi.
- En tant que `Persona-Syndic`, je veux rédiger, ordonner et qualifier les résolutions pour préparer un vote juridiquement exploitable.
- En tant que `Persona-Syndic`, je veux voir ce qui bloque la convocation pour ne pas envoyer une AG incomplète.
- En tant que `Persona-Conseil`, je veux relire le dossier préparatoire pour vérifier le contenu sans administrer le cycle AG.

**Conduite de séance**

- En tant que `Persona-Syndic`, je veux démarrer la séance dans un parcours ordonné pour limiter les oublis.
- En tant que `Persona-Syndic`, je veux désigner les rôles de séance pour documenter correctement le bureau.
- En tant que `Persona-Copro`, je veux exprimer mon vote par résolution selon ma situation de présence ou de représentation.
- En tant que `Persona-Syndic`, je veux voir immédiatement les résultats et les cas de passerelle 25-1 / 26-1.

**Post-AG et preuve**

- En tant que `Persona-Syndic`, je veux générer un projet de PV fidèle à la séance pour accélérer la formalisation.
- En tant que `Persona-Syndic`, je veux rattacher le PV signé et tracer sa notification pour sécuriser l'opposabilité.
- En tant que `Persona-Copro`, je veux retrouver les décisions et leurs résultats pour comprendre ce qui a été adopté.
- En tant que `Persona-Locataire`, je veux consulter les décisions accessibles sans être placé dans un faux parcours de vote.

### Matrice CRUD

| Domaine | Syndic | Conseil | Copropriétaire | Locataire |
|---|---|---|---|---|
| Dossier d'AG | `C/R/U/D` | `R` | `R¹` | `R¹` |
| Résolutions | `C/R/U/D` | `R` | `R¹` | `R¹` |
| Convocation | `C/R/U/D¹` | `R` | `R¹` | `R¹` |
| Rôles de séance | `C/R/U/D¹` | `R` | `R¹` | `R¹` |
| Présence et quorum | `C/R/U/D` | `R` | `R¹` | `R¹` |
| Votes et résultats | `C/R/U/D` | `R` | `C/R¹/U¹` | `R¹` |
| PV | `C/R/U/D¹` | `R` | `R¹` | `R¹` |
| Notification PV | `C/R/U/D¹` | `R` | `R¹` | `R¹` |

> ¹ Périmètre restreint à la copropriété de résidence de l'utilisateur. Pour `Persona-Copro` sur votes : ses propres votes uniquement.

### Règles de sécurité

- Les actions de transition d'état doivent être des endpoints dédiés — non fondues dans un `Modifier` générique.
- Les données votées et les éléments figés de séance suivent ADR-009 après clôture.
- `Persona-Locataire` n'entre jamais dans un parcours de vote — aucun lien ou bouton de vote NE DOIT lui être présenté.
- Les droits de lecture résidentielle distinguent l'information consultable du pouvoir d'action.

---

## 4. Cycle de vie et machine à états

### États AG

| État | Description |
|---|---|
| `draft` | Dossier créé, non encore convoqué |
| `called` | Convocation envoyée, AG programmée |
| `ongoing` | Séance démarrée, OJ et assiette gelés |
| `closed` | Séance clôturée, données de séance figées (ADR-009) |
| `cancelled` | AG annulée définitivement |

### Transitions autorisées

| De | Vers | Déclencheur |
|---|---|---|
| `draft` | `called` | Convoquer l'AG (checklist OK + délai légal respecté) |
| `draft` | `cancelled` | Annuler le brouillon |
| `called` | `ongoing` | Démarrer la séance |
| `called` | `cancelled` | Annuler après convocation |
| `ongoing` | `closed` | Clôturer la séance (toutes résolutions complètes) |

### Sous-états du PV (après `closed`)

| Sous-état | Description |
|---|---|
| `no_minutes` | Aucun projet PV généré |
| `draft_minutes` | Projet PV disponible (non signé) |
| `signed_minutes` | PV signé déposé — immuable (append-only) |
| `notified_minutes` | Notification tracée aux opposants et défaillants |

> Le PV signé est **append-only** après dépôt. Tout remplacement DOIT passer par un workflow de remplacement explicite avec justification horodatée.

---

## 5. Exigences fonctionnelles

### MoSCoW

| Priorité | Ref | Action | Attendu produit |
|---|---|---|---|
| Must | US-AG-001 | Créer une AG | Brouillon avec date, type, OJ, contraintes calendrier |
| Must | US-AG-002 | Modifier ou supprimer un brouillon | Tant que statut = `brouillon` |
| Must | US-AG-003 | Gérer les résolutions | Ajouter, modifier, supprimer, typer, ordonner |
| Must | US-AG-004 | Gérer le dossier de convocation | Pièces obligatoires, checklist, alertes bloquantes |
| Must | US-AG-005 | Convoquer l'AG | Verrouiller le niveau de préparation requis à l'envoi |
| Must | US-AG-006 | Gérer la présence et le quorum | Présences, procurations, correspondance, base de quorum |
| Must | US-AG-007 | Démarrer la séance | Gel OJ + assiette, saisie rôles de séance |
| Must | US-AG-008 | Saisir les votes et calculer les résultats | Art. 24, 25, 25-1, 26, 26-1 et unanimité |
| Must | US-AG-009 | Clôturer la séance | Figer votes, présence, résultats |
| Must | US-AG-010 | Générer et publier le PV | Projet PV structuré + dépôt PV signé |
| Must | US-AG-011 | Notifier le PV | Action explicite + suivi délai légal |
| Must | US-AG-014 | Consulter le dossier et les décisions (résidents) | Vue lecture selon rôle |
| Should | US-AG-012 | Désigner les rôles de séance | Président, secrétaire, scrutateurs |
| Should | US-AG-013 | Gérer les passerelles art. 25-1 et 26-1 | Détection auto + traçabilité seconde délibération |

### User stories détaillées

#### US-AG-001 — Créer une AG

**En tant que** `Persona-Syndic`,
**je veux** créer un dossier d'AG,
**afin de** disposer d'un espace structuré pour préparer la séance.

**Critères d'acceptation** :

- **AC-AG-001-A** : La création DOIT ouvrir un dossier avec statut `draft`.
- **AC-AG-001-B** : La date de séance est obligatoire — soumission sans date DOIT retourner une erreur de validation.
- **AC-AG-001-C** : La date de séance DOIT piloter le calcul automatique de la date limite de convocation.
- **AC-AG-001-D** : L'AG DOIT être rattachée à une copropriété existante.
- **AC-AG-001-E** : Une création en doublon manifeste (même copropriété, même date) DOIT afficher un avertissement non bloquant.

#### US-AG-002 — Modifier ou supprimer un brouillon

**En tant que** `Persona-Syndic`,
**je veux** modifier ou supprimer une AG en brouillon,
**afin de** corriger les informations avant convocation.

**Critères d'acceptation** :

- **AC-AG-002-A** : La modification est autorisée uniquement si statut = `draft`.
- **AC-AG-002-B** : La suppression est autorisée si statut = `draft` et aucune pièce engagée ne s'y rattache.
- **AC-AG-002-C** : Toute tentative de modification ou suppression sur une AG avec statut ≠ `draft` DOIT retourner 403.
- **AC-AG-002-D** : La confirmation de suppression DOIT être explicite (modal de confirmation requis).

#### US-AG-003 — Gérer les résolutions

**En tant que** `Persona-Syndic`,
**je veux** ajouter, modifier, ordonner et supprimer des résolutions,
**afin de** construire un ordre du jour juridiquement exploitable.

**Critères d'acceptation** :

- **AC-AG-003-A** : Une résolution DOIT comporter au minimum un titre, un texte, un ordre et une règle de majorité.
- **AC-AG-003-B** : Ajout, modification, suppression et réordonnancement DOIVENT être possibles tant que l'AG est au statut `draft` ou `called` (avant démarrage).
- **AC-AG-003-C** : La substance d'une résolution votée DOIT être immuable après `cloture` — tentative de modification DOIT retourner 403.
- **AC-AG-003-D** : La suppression d'une résolution dont le vote a été saisi DOIT être refusée avec un message explicite.
- **AC-AG-003-E** : Un ordre en doublon DOIT être détecté et signalé à la sauvegarde.

#### US-AG-004 — Gérer le dossier de convocation

**En tant que** `Persona-Syndic`,
**je veux** composer et vérifier le dossier de convocation,
**afin de** ne pas envoyer une convocation incomplète juridiquement.

**Critères d'acceptation** :

- **AC-AG-004-A** : Le dossier DOIT lister les pièces obligatoires avec leur statut (présent / absent).
- **AC-AG-004-B** : Les éléments bloquant juridiquement la convocation DOIVENT être clairement distingués des éléments recommandés.
- **AC-AG-004-C** : Toute pièce obligatoire manquante DOIT bloquer la transition vers `called`.
- **AC-AG-004-D** : Un document illisible ou non rattaché correctement DOIT être signalé avant la transition.
- **AC-AG-004-E** : Le formulaire de vote par correspondance (art. 9 al. 4 décret n°67-223) DOIT figurer dans les pièces obligatoires de la checklist — son absence DOIT bloquer la transition vers `called`.

#### US-AG-005 — Convoquer l'AG

**En tant que** `Persona-Syndic`,
**je veux** envoyer la convocation pour passer l'AG en statut `convoque`,
**afin de** figer le dossier minimal requis et lancer le délai légal.

**Critères d'acceptation** :

- **AC-AG-005-A** : La convocation DOIT être une transition d'état explicite `draft` → `called`.
- **AC-AG-005-B** : La checklist de convocation DOIT être conforme avant d'autoriser la transition.
- **AC-AG-005-C** : Le délai légal DOIT être validé (≥ 21 jours pour AG ordinaire, conformément à l'art. 9 du décret n°67-223).
- **AC-AG-005-D** : L'horodatage et le canal de convocation DOIVENT être tracés.
- **AC-AG-005-E** : Une AG déjà convoquée NE DOIT PAS être reconvoquée sans annulation préalable.

#### US-AG-006 — Gérer la présence et le quorum

**En tant que** `Persona-Syndic`,
**je veux** enregistrer la présence des copropriétaires et suivre le quorum,
**afin de** disposer d'une base de calcul fiable avant et pendant la séance.

**Critères d'acceptation** :

- **AC-AG-006-A** : Les statuts de présence DOIVENT inclure : présent, représenté, correspondance, absent.
- **AC-AG-006-B** : Le quorum affiché DOIT se baser sur l'assiette valide au moment de la séance.
- **AC-AG-006-C** : Les lots archivés ou non éligibles DOIVENT être exclus des calculs de quorum.
- **AC-AG-006-D** : Une procuration incohérente (double mandataire sur un même lot, auto-procuration) DOIT être signalée.
- **AC-AG-006-E** : Un vote saisi pour un lot absent ou sans représentation valide DOIT être refusé avec 422.

#### US-AG-007 — Démarrer la séance

**En tant que** `Persona-Syndic`,
**je veux** passer l'AG en statut `en_seance`,
**afin de** figer l'ordre du jour effectif et lancer la conduite formelle.

**Critères d'acceptation** :

- **AC-AG-007-A** : Le démarrage DOIT être une transition explicite `called` → `ongoing`.
- **AC-AG-007-B** : Cette transition DOIT figer l'ordre du jour effectif et l'assiette de calcul (tantièmes).
- **AC-AG-007-C** : Démarrage sur une AG avec statut ≠ `called` DOIT retourner 422.
- **AC-AG-007-D** : Les rôles de séance (président, secrétaire) PEUVENT être saisis lors du démarrage.

#### US-AG-008 — Saisir les votes et calculer les résultats

**En tant que** `Persona-Syndic`,
**je veux** saisir les votes résolution par résolution et voir les résultats calculés automatiquement,
**afin de** conduire la séance sans recalcul manuel.

**Critères d'acceptation** :

- **AC-AG-008-A** : Le vote se fait par résolution, par lot, avec calcul automatique et immédiat du résultat.
- **AC-AG-008-B** : Le moteur DOIT appliquer les règles art. 24, 25, 25-1, 26, 26-1 et unanimité.
- **AC-AG-008-C** : Les résolutions éligibles à une seconde délibération DOIVENT être signalées sans ambiguïté.
- **AC-AG-008-D** : Vote saisi pour un lot non éligible DOIT être refusé avec 422.
- **AC-AG-008-E** : Vote saisi hors séance (statut AG ≠ `ongoing`) DOIT être refusé avec 403.

#### US-AG-009 — Clôturer la séance

**En tant que** `Persona-Syndic`,
**je veux** clôturer la séance pour figer les décisions,
**afin de** sécuriser l'opposabilité des votes et déclencher le traitement post-AG.

**Critères d'acceptation** :

- **AC-AG-009-A** : La clôture DOIT être une transition explicite `ongoing` → `closed`.
- **AC-AG-009-B** : Les votes, feuille de présence et résultats DOIVENT être figés à la clôture (ADR-009 Pattern A).
- **AC-AG-009-C** : Des résolutions incomplètes (sans vote saisi) DOIVENT bloquer la transition avec liste des résolutions manquantes.
- **AC-AG-009-D** : Toute tentative de modifier un vote ou un résultat après clôture DOIT retourner 403.
- **AC-AG-009-E** : La transition DOIT écrire une entrée dans `audit_log` avec snapshot des données figées.

#### US-AG-010 — Générer et publier le PV

**En tant que** `Persona-Syndic`,
**je veux** générer un projet de PV et déposer le PV signé,
**afin de** disposer d'une preuve documentaire exploitable de la séance.

**Critères d'acceptation** :

- **AC-AG-010-A** : La génération du projet PV DOIT être impossible si statut AG ≠ `closed`.
- **AC-AG-010-B** : Le projet PV DOIT être structuré à partir des données figées de séance.
- **AC-AG-010-C** : Le dépôt du PV signé DOIT être une action distincte du projet de PV.
- **AC-AG-010-D** : Une fois le PV signé déposé, la matière de séance NE DOIT PAS être modifiable librement.
- **AC-AG-010-E** : Republication d’un PV déjà déposé DOIT passer par un workflow de remplacement explicite : modale d’avertissement rappelant les conséquences juridiques + saisie obligatoire d’une justification textuelle + dépôt du nouveau fichier. Le PV précédent DOIT rester accessible en historique. L’opération DOIT être tracée dans `audit_log` avec les deux versions et la justification.

#### US-AG-011 — Notifier le PV

**En tant que** `Persona-Syndic`,
**je veux** marquer la notification du PV aux opposants et défaillants,
**afin de** faire courir le délai légal d'opposition (art. 42 loi 65-557).

**Critères d'acceptation** :

- **AC-AG-011-A** : La notification DOIT être impossible sans PV signé déposé.
- **AC-AG-011-B** : La date de notification DOIT être tracée pour les opposants et les défaillants (horodatage interne — pas de canal ou de preuve externe pour le MVP).
- **AC-AG-011-C** : La fiche AG DOIT afficher le délai restant pour notifier le PV (J+30 depuis la date d’AG — art. 42 al. 3 loi 65-557), avec alerte visuelle si J+21 est dépassé.
- **AC-AG-011-D** : Une notification déjà marquée NE DOIT PAS être duplicable.
- **AC-AG-011-E** : La fiche AG DOIT afficher le délai de contestation (J+60 depuis la date de notification tracée — art. 42 al. 2 loi 65-557), calculé à partir de `notification_date`, pas de `assembly_date`.

#### US-AG-012 — Désigner les rôles de séance

**En tant que** `Persona-Syndic`,
**je veux** saisir le bureau de séance (président, secrétaire, scrutateurs),
**afin de** documenter correctement la composition du bureau pour le PV.

**Critères d'acceptation** :

- **AC-AG-012-A** : Les rôles DOIVENT pouvoir être saisis avant ou lors du démarrage de séance.
- **AC-AG-012-B** : Les rôles figés après clôture NE DOIVENT PAS être modifiables.
- **AC-AG-012-C** : Absence de rôles obligatoires DOIT afficher un avertissement (non bloquant au démarrage).

#### US-AG-013 — Gérer les passerelles art. 25-1 et 26-1

**En tant que** `Persona-Syndic`,
**je veux** que le système détecte automatiquement les cas de seconde délibération,
**afin de** ne pas manquer une passerelle légale en cours de séance.

**Critères d'acceptation** :

- **AC-AG-013-A** : La détection des cas de redélibération DOIT être automatique après enregistrement du résultat initial.
- **AC-AG-013-B** : La seconde délibération DOIT rester explicitement liée à la résolution d'origine.
- **AC-AG-013-C** : La traçabilité DOIT indiquer pourquoi la bascule a été proposée (règle et seuil atteint).
- **AC-AG-013-D** : Double redélibération sur la même résolution DOIT être refusée avec un message explicite.
- **AC-AG-013-E** : Une passerelle NE DOIT PAS pouvoir être créée manuellement si les conditions légales ne sont pas remplies.

#### US-AG-014 — Consulter le dossier et les décisions (résidents)

**En tant que** `Persona-Copro` ou `Persona-Locataire`,
**je veux** consulter les résolutions, résultats et documents visibles selon mon rôle,
**afin de** comprendre les décisions prises et y accéder après séance.

**Critères d'acceptation** :

- **AC-AG-014-A** : `Persona-Copro` DOIT avoir accès aux résolutions, résultats et PV de sa copropriété.
- **AC-AG-014-B** : `Persona-Locataire` DOIT pouvoir consulter les décisions accessibles sans être placé dans un parcours de vote.
- **AC-AG-014-C** : `Persona-Locataire` NE DOIT PAS voir d'action de vote ou de bouton de participation.
- **AC-AG-014-D** : Les droits de lecture DOIVENT être restreints à la copropriété de résidence de l'utilisateur.

---

## 6. Exigences non fonctionnelles

| ID | Catégorie | Exigence | Seuil mesurable |
|---|---|---|---|
| NFR-Perf-001 | Performance | Calcul de résultat de vote | ≤ 500 ms (p95) pour une AG jusqu'à 200 lots |
| NFR-Perf-002 | Performance | Chargement de la vue séance | ≤ 800 ms (p95) |
| NFR-Sec-001 | Sécurité | Données figées après clôture protégées en écriture | Retourne 403 sur tout PUT/PATCH interdit — 0 exception |
| NFR-Audit-001 | Traçabilité | Toute transition d'état AG | Entrée dans `audit_log` avec `old_values`/`new_values` — 100 % de couverture |
| NFR-Legal-001 | Conformité | Délai légal de convocation | Calculé automatiquement (≥ 21 jours AG ordinaire, art. 9 décret n°67-223) |
| NFR-Integ-001 | Intégration | Calcul quorum et résultats | DOIT utiliser les clés de répartition du module tantièmes (ADR-006) — 0 calcul hardcodé |

---

## 7. Hors scope

> **CRITICAL** — Les fonctionnalités suivantes NE DOIVENT PAS être implémentées dans ce lot. Tout agent DOIT rejeter toute demande d'implémentation de ces éléments.

- Signature électronique probante bout-en-bout.
- Envoi probant multi-canal automatisé (AR électronique, recommandé numérique).
- Co-édition temps réel riche de notes de séance.
- Gestion des délégations complexes du conseil syndical hors flux principal.
- Portail externe dédié aux copropriétaires pour le vote en ligne avant séance.
- Intégration avec un système de gestion de documents externe (GED).

---

## 8. Métriques de succès

| Métrique | Cible | Délai |
|---|---|---|
| Délai moyen `clôture séance` → `notification PV` | < 14 jours | À 3 mois post-déploiement |
| Taux d'AG convoquées avec dossier complet dès l'envoi | ≥ 80 % | À 3 mois |
| Taux de résolutions sans correction manuelle post-séance | ≥ 90 % | À 6 mois |
| Temps de saisie d'une séance complète (20 résolutions) | < 45 min | À 6 mois |

---

## 9. Questions ouvertes

| # | Question | Owner | Condition de révision |
|---|---|---|---|
| Q-AG-001 | Quel niveau de preuve documentaire viser pour la notification du PV au-delà du simple horodatage (AR électronique, recommandé numérique) ? | ✅ **Résolu** — Horodatage interne simple (date + action syndic). Pas de canal tracé ni de preuve externe pour le MVP. AR électronique hors scope. | Damien | 2026-05-20 |
| Q-AG-002 | Quel workflow autoriser si un PV signé doit être remplacé (erreur matérielle) — circuit de validation, délai, acteurs ? | ✅ **Résolu** — Syndic seul, avec modale d’avertissement forte (conséquences juridiques) + justification textuelle obligatoire. PV précédent conservé en historique. Tracé dans `audit_log`. | Damien | 2026-05-20 |
| Q-AG-003 | Faut-il séparer visuellement la phase `préparation juridique` de la phase `conduite de séance` dans la navigation (pages distinctes vs onglets) ? | ✅ **Résolu** — Onglets dans la même page fiche AG (`Préparation` / `Séance` / `Post-AG`). Page unique par entité AG, transition d’état accessible depuis chaque onglet. Cohérent avec DDR-004. | Équipe | 2026-05-20 |

---

## Appendice A — Index de traçabilité

| US | Priorité | ACs | Domaine CRUD |
|---|---|---|---|
| US-AG-001 | Must | AC-AG-001-A à E | Dossier d'AG (C) |
| US-AG-002 | Must | AC-AG-002-A à D | Dossier d'AG (U/D) |
| US-AG-003 | Must | AC-AG-003-A à E | Résolutions (C/U/D) |
| US-AG-004 | Must | AC-AG-004-A à E | Convocation (C/U) |
| US-AG-005 | Must | AC-AG-005-A à E | Convocation (transition) |
| US-AG-006 | Must | AC-AG-006-A à E | Présence et quorum (C/U) |
| US-AG-007 | Must | AC-AG-007-A à D | Séance (démarrage) |
| US-AG-008 | Must | AC-AG-008-A à E | Votes et résultats (C/U) |
| US-AG-009 | Must | AC-AG-009-A à E | Séance (clôture) |
| US-AG-010 | Must | AC-AG-010-A à E | PV (C/U) |
| US-AG-011 | Must | AC-AG-011-A à E | Notification PV (C) |
| US-AG-012 | Should | AC-AG-012-A à C | Rôles de séance (C/U) |
| US-AG-013 | Should | AC-AG-013-A à E | Votes et résultats (passerelles) |
| US-AG-014 | Must | AC-AG-014-A à D | Dossier d'AG / Résolutions (R) |

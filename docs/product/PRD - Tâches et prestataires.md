---
product: Tâches et prestataires
version: 1.0.0
owner: Damien
status: en cours
last_updated: 2026-05-22
references:
  - EP-007
  - EP-015
  - EP-009-S03
  - EP-018-S04
  - LEGAL.md §1.4
  - ADR-009
llm_directives:
  temperature: 0.2
  persona: >
    Tu es un agent développeur senior travaillant sur la feature
    « Tâches et prestataires » de La Ruche. Tu DOIS respecter les
    contraintes techniques et les critères d'acceptation sans dérogation.
    Tu NE DOIS PAS implémenter de fonctionnalités listées dans Hors scope.
    Les champs figés après clôture (completion_cost, closed_at,
    accepted_quote, assigned_contractor, documents reçus du prestataire)
    sont immuables — aucun PUT/PATCH sur ces champs après l'état `closed`.
    La timeline est append-only — aucun DELETE ni UPDATE sur ses entrées.
  modification_policy: >
    Ne modifier ce fichier que si une évolution de la feature est ABSOLUMENT
    nécessaire, et EXCLUSIVEMENT après validation explicite d'un PRD additionnel.
    Si une modification est nécessaire : inline, succincte, dans une section
    existante. Si aucune section ne convient, requérir une validation utilisateur.
    Le corps du document ne contient aucune référence technique externe
    (ADR, DDR, numéro de feature ou de ticket).
---

# PRD — Tâches et prestataires

## 1. Vue d'ensemble

### Objectif

Permettre aux gestionnaires de **résoudre un problème opérationnel jusqu'à sa clôture**, avec deux parcours complémentaires :

- un parcours interne de gestion de tâche ;
- un sous-parcours ultra-minimal prestataire via lien tokenisé.

### Pourquoi

La valeur métier de la feature n'est pas la création d'une tâche en soi, mais la **résolution traçable d'un besoin d'intervention**. Le système doit :

- faciliter la coordination interne ;
- ouvrir ponctuellement un accès prestataire sans créer de compte ;
- conserver une trace fiable de la clôture, du coût proposé et des documents ;
- limiter les retours arrière sur des données déjà figées ou réutilisées ailleurs.

---

## 2. Contraintes techniques

> **CRITICAL** — Ces contraintes sont immuables. Tout agent ou développeur DOIT les respecter sans exception avant d'implémenter une quelconque fonctionnalité.

### Stack

- **Backend** : Rust (Axum / SQLx / PostgreSQL) — voir ADR-001
- **Frontend** : Vue 3 Composition API + TypeScript — voir DDR-001/002
- **Auth** : JWT / tokens opaques — voir ADR-002

### Immutabilité comptable (ADR-009 — Pattern A)

Les champs suivants sont **figés dès l'état `closed`** et ne peuvent plus être modifiés, même après réouverture :

| Champ | Interdit après clôture |
|---|---|
| `completion_cost` | PUT / PATCH |
| `closed_at` | PUT / PATCH |
| `assigned_contractor` | PUT / PATCH |
| `accepted_quote` | PUT / PATCH |
| Documents reçus du prestataire | DELETE / PATCH |

**Ajout uniquement** : les documents restent ajoutables après clôture (append-only) — jamais modifiables ni supprimables.

Toute mutation financière DOIT écrire dans `audit_log` via `db::audit_log::record()`.

### Token prestataire

- Un seul token actif par tâche. Générer un nouveau token DOIT révoquer l'ancien.
- Durée de vie : 30 jours (configurable via `CONTRACTOR_TOKEN_TTL_DAYS`).
- Token révoqué ou expiré DOIT être conservé en `audit_log` — inutilisable mais traçable.
- L'endpoint prestataire DOIT valider le token avant toute lecture ou écriture.

### Timeline

- Chaque transition d'état DOIT créer une entrée en `task_timeline` (append-only).
- Aucune entrée de timeline ne peut être `UPDATE`d ni `DELETE`d.
- Champs obligatoires par entrée : `actor_id`, `timestamp`, `from_state`, `to_state`, `snapshot JSONB`.

### Patterns interdits

- Aucun `PUT /tasks/:id` générique ne DOIT modifier un champ figé.
- Aucune transition d'état ne DOIT contourner la machine à états décrite en §4.
- Toute suppression de tâche ayant alimenté le carnet d'entretien est **interdite**.
- Ne pas utiliser `unwrap()` / `expect()` dans le code backend — utiliser `Result<T, E>`.

---

## 3. Profils utilisateurs

### Personas

| ID | Persona | Besoin principal | Attente clé |
|---|---|---|---|
| Persona-Syndic | Syndic bénévole | Faire aboutir une intervention | Pilotage simple et traçable |
| Persona-Conseil | Conseil syndical | Suivre et co-gérer l'opérationnel | Agir sans ouvrir l'administration générale |
| Persona-Prestataire | Prestataire externe | Répondre à une demande précise | Lien simple sans onboarding |
| Persona-Copro | Copropriétaire | Être indirectement servi par la résolution | Pas de back-office tâche |
| Persona-Locataire | Locataire | Signaler en amont, pas gérer | Comprendre que la résolution passe côté gestionnaires |

### User stories de haut niveau

- En tant que `Persona-Syndic`, je veux créer et qualifier une tâche pour piloter une résolution sans perdre d'information.
- En tant que `Persona-Syndic`, je veux convertir un signalement en tâche pour éviter une ressaisie.
- En tant que `Persona-Conseil`, je veux suivre et mettre à jour l'avancement pour aider à la résolution.
- En tant que `Persona-Syndic`, je veux solliciter un prestataire via un lien temporaire sans créer de compte.
- En tant que `Persona-Prestataire`, je veux déposer un devis et un rapport depuis un parcours minimal.
- En tant que `Persona-Syndic`, je veux clôturer proprement la tâche et figer les éléments réutilisés par le carnet d'entretien.

### Matrice CRUD

| Domaine | Syndic | Conseil | Copropriétaire | Locataire | Prestataire (token) |
|---|---|---|---|---|---|
| Tâches | `C/R/U/D` | `R` | `X` | `X` | `R¹` |
| Signalement source lié | `C/R/U` | `R` | `R¹` | `R¹` | `X` |
| Pièces de tâche | `C/R/U/D` | `R` | `X` | `X` | `C/R¹` |
| Référentiel prestataires | `C/R/U/D` | `C/R/U/D` | `R` | `R` | `X` |
| Affectation prestataire | `C/R/U/D¹` | `R` | `X` | `X` | `X` |
| Lien prestataire | `C/R/U/D` | `R` | `X` | `X` | `X` |
| Réponse prestataire | `R/U` | `R` | `X` | `X` | `C/R¹/U¹` |

> ¹ Périmètre restreint à la tâche concernée uniquement.

### Règles de sécurité

- Le lien prestataire ne donne accès qu'à un périmètre ultra-minimal et contextuel.
- Le coût proposé par le prestataire n'est jamais traité comme une validation de dépense.
- Les actions destructives doivent être masquées ou bloquées après état terminal ou alimentation du carnet.
- Les champs figés après clôture suivent les principes d'immutabilité portés par `EP-018`.

---

## 4. Cycle de vie et machine à états

### États

| État | Description |
|---|---|
| `open` | Tâche créée, non encore prise en charge |
| `in_progress` | Tâche en traitement actif par un gestionnaire |
| `awaiting_quote` | Demande de devis envoyée au prestataire |
| `quote_received` | Le prestataire a soumis un devis — en attente de validation syndic |
| `awaiting_intervention` | Devis validé (ou intervention directe) — prestataire doit intervenir |
| `report_received` | Le prestataire a déclaré la fin d'intervention — en attente de validation syndic |
| `closed` | Résolution validée — champs sensibles figés |
| `cancelled` | Tâche définitivement abandonnée |

### Transitions autorisées

| De | Vers | Déclencheur |
|---|---|---|
| `open` | `in_progress` | Prise en charge |
| `open` | `cancelled` | Abandon avant traitement |
| `in_progress` | `awaiting_quote` | Demande de devis envoyée |
| `in_progress` | `awaiting_intervention` | Intervention directe (sans phase devis) |
| `in_progress` | `closed` | Clôture directe sans prestataire |
| `in_progress` | `cancelled` | Abandon |
| `awaiting_quote` | `quote_received` | Prestataire soumet le devis |
| `awaiting_quote` | `in_progress` | Action « Reprendre en main » |
| `quote_received` | `awaiting_intervention` | Syndic valide le devis |
| `quote_received` | `in_progress` | Syndic rejette / reprend en main |
| `awaiting_intervention` | `report_received` | Prestataire déclare fin d'intervention |
| `report_received` | `closed` | Syndic valide l'intervention |
| `report_received` | `awaiting_intervention` | Syndic rejette l'intervention |
| `closed` | `in_progress` | Réouverture |
| `cancelled` | `in_progress` | Réouverture |

> `cancelled` = abandon définitif. « Reprendre en main » ramène la tâche à `in_progress` sans effacer l'historique et sans passer par `cancelled`.

### Cycles de résolution

Chaque clôture constitue un **cycle de résolution** historisé : `completion_cost`, `closed_at`, prestataire impliqué, documents reçus. Une réouverture démarre un nouveau cycle sans écraser les précédents. Tous les cycles sont visibles dans la timeline.

---

## 5. Exigences fonctionnelles

### MoSCoW

| Priorité | Ref | Action | Attendu produit |
|---|---|---|---|
| Must | US-TP-001 | Créer une tâche | Titre, description, priorité, contexte, assignation |
| Must | US-TP-002 | Convertir un signalement en tâche | Reprise du contexte, priorité obligatoire, lien bidirectionnel |
| Must | US-TP-003 | Consulter, filtrer et rechercher | Vue orientée résolution, 8 états filtrables |
| Must | US-TP-004 | Modifier une tâche ouverte | Tant qu'aucun verrou métier n'est atteint |
| Must | US-TP-005 | Gérer les documents côté gestionnaire | Pièces utiles, append-only après clôture |
| Must | US-TP-006 | Assigner un prestataire | Référentiel, création opportuniste, reprendre en main |
| Must | US-TP-007 | Générer et révoquer un lien prestataire | 30 j, 1 seul actif, révocation manuelle, audit log |
| Must | US-TP-008 | Soumettre un devis (prestataire) | Commentaire, documents, montant — transition `quote_received` |
| Must | US-TP-009 | Valider ou rejeter le devis (syndic) | Validation → `awaiting_intervention`, rejet → `in_progress` |
| Must | US-TP-010 | Déclarer la fin d'intervention (prestataire) | Commentaire, documents — transition `report_received` |
| Must | US-TP-011 | Valider ou rejeter l'intervention et clôturer | Validation → `closed` + carnet, rejet → `awaiting_intervention` |
| Must | US-TP-012 | Reprendre en main depuis une phase devis | `awaiting_quote` / `quote_received` → `in_progress` sans annuler |
| Must | US-TP-013 | Réouvrir une tâche | `closed` / `cancelled` → `in_progress`, cycles archivés préservés |
| Must | US-TP-014 | Immutabilité des champs sensibles | HTTP 422 sur champ figé, audit_log sur mutation financière |
| Must | US-TP-015 | Timeline append-only | Suivi immuable de bout en bout |
| Must | US-TP-016 | Supprimer ou annuler une tâche | Suppression bloquée si carnet ou soumission prestataire |
| Could | — | Relances prestataires | Coordination, non critique |
| Won't | — | Portail prestataire persistant multi-tâches | Hors périmètre actuel |

---

### User stories et critères d'acceptation

---

#### US-TP-001 — Créer une tâche

**En tant que** `Persona-Syndic`,
**je veux** créer et qualifier une tâche,
**afin de** piloter une résolution sans perdre d'information.

**Critères d'acceptation**

- **AC-001-A** : Le système DOIT accepter la création avec : titre (obligatoire), description (obligatoire), priorité parmi `low/normal/high/urgent` (obligatoire), contexte (optionnel), prestataire (optionnel).
- **AC-001-B** : Le système DOIT rejeter si le titre est absent ou vide — code `errors.tasks.title_required`.
- **AC-001-C** : Le système DOIT rejeter si la priorité est absente — code `errors.tasks.priority_required`.
- **AC-001-D** : La tâche créée DOIT être initialisée à l'état `open`.
- **AC-001-E** : Le système DOIT avertir si une tâche potentiellement en doublon (même titre, même contexte, état non terminal) existe déjà — code `errors.tasks.potential_duplicate`.

---

#### US-TP-002 — Convertir un signalement en tâche

**En tant que** `Persona-Syndic`,
**je veux** convertir un signalement qualifié en tâche,
**afin d'** éviter une ressaisie et conserver la continuité de traitement.

**Mapping de champs à la conversion**

| Champ signalement | Comportement dans la tâche |
|---|---|
| Titre / description | Repris automatiquement |
| Lot / unité concernée | Repris automatiquement |
| Documents joints | Repris automatiquement |
| Catégorie / type | Saisie manuelle obligatoire |
| Priorité | Saisie manuelle obligatoire |
| Prestataire | Vide par défaut — optionnel |

**Critères d'acceptation**

- **AC-002-A** : Le système DOIT reprendre automatiquement titre, description, lot/unité et documents du signalement source. La catégorie de la tâche DOIT être saisie manuellement par le Syndic lors de la conversion — elle n'est pas héritée automatiquement.
- **AC-002-B** : Le système DOIT exiger la saisie manuelle de la priorité avant de valider la conversion.
- **AC-002-C** : Le système DOIT créer un lien bidirectionnel `signalement ↔ tâche` visible dans les deux vues.
- **AC-002-D** : Le système DOIT rejeter si le signalement est déjà converti — code `errors.tasks.signalement_already_converted`.
- **AC-002-E** : Le système DOIT rejeter si le signalement est fermé ou non convertible.
- **AC-002-F** : Le signalement source DOIT être figé après conversion — aucune modification de ses champs existants. Seuls de nouveaux documents peuvent y être ajoutés (append-only).

---

#### US-TP-003 — Consulter, filtrer et rechercher les tâches

**En tant que** `Persona-Syndic` ou `Persona-Conseil`,
**je veux** consulter la liste des tâches avec filtres et recherche,
**afin de** repérer rapidement les urgences et les actions en attente.

**Critères d'acceptation**

- **AC-003-A** : La liste DOIT afficher par défaut les tâches triées par date de dernière modification décroissante.
- **AC-003-B** : La liste DOIT permettre le filtrage par chacun des 8 états : `open`, `in_progress`, `awaiting_quote`, `quote_received`, `awaiting_intervention`, `report_received`, `closed`, `cancelled`.
- **AC-003-C** : La liste DOIT permettre le filtrage par priorité.
- **AC-003-D** : La liste DOIT permettre la recherche textuelle sur titre et description.
- **AC-003-E** : Un état vide (aucun résultat) DOIT afficher un message explicatif.
- **AC-003-F** : Les tâches inaccessibles selon le rôle NE DOIVENT PAS apparaître dans la liste.

---

#### US-TP-004 — Modifier une tâche ouverte

**En tant que** `Persona-Syndic`,
**je veux** modifier le contenu d'une tâche,
**afin de** corriger ou compléter les informations avant clôture.

**Champs figés après clôture**

| Champ | Raison |
|---|---|
| `completion_cost` | Coût validé — réutilisé par le carnet d'entretien |
| `closed_at` | Date de traçabilité légale |
| `assigned_contractor` | Traçabilité de l'intervenant |
| `accepted_quote` | Preuve du devis validé |
| Documents reçus du prestataire | Preuves opérationnelles |

**Critères d'acceptation**

- **AC-004-A** : Le système DOIT autoriser la modification de titre, description, priorité et contexte sur une tâche non clôturée.
- **AC-004-B** : Le système DOIT refuser toute modification de `completion_cost`, `closed_at`, `assigned_contractor`, `accepted_quote` ou des documents prestataires sur une tâche `closed` — code `errors.tasks.field_frozen`.
- **AC-004-C** : En cas de conflit d'édition concurrente, le système DOIT retourner `errors.tasks.concurrent_edit_conflict` sans écraser silencieusement les données.

---

#### US-TP-005 — Gérer les documents côté gestionnaire

**En tant que** `Persona-Syndic`,
**je veux** joindre, consulter et gérer des documents sur une tâche,
**afin de** cadrer le besoin, briefer un prestataire ou justifier une clôture.

**Critères d'acceptation**

- **AC-005-A** : Le gestionnaire DOIT pouvoir ajouter des documents à n'importe quel état, y compris après `closed` (append-only).
- **AC-005-B** : Le gestionnaire DOIT pouvoir supprimer ses propres documents gestionnaire tant que la tâche n'est pas `closed`.
- **AC-005-C** : Le système DOIT refuser la suppression de tout document sur une tâche `closed` — code `errors.tasks.document_frozen`.
- **AC-005-D** : Le gestionnaire DOIT pouvoir contrôler la visibilité de chaque document (visible prestataire : oui/non).
- **AC-005-E** : Le système DOIT rejeter un fichier dont le format ou la taille dépasse la politique de la plateforme — code `errors.storage.file_rejected`.

---

#### US-TP-006 — Assigner un prestataire

**En tant que** `Persona-Syndic`,
**je veux** assigner un prestataire à une tâche,
**afin d'** identifier l'intervenant et déclencher le parcours tokenisé.

**Critères d'acceptation**

- **AC-006-A** : Le gestionnaire DOIT pouvoir assigner un prestataire depuis le référentiel (autocomplete).
- **AC-006-B** : Le gestionnaire DOIT pouvoir assigner un prestataire via un nom libre si la relation n'est pas encore structurée.
- **AC-006-C** : Le système DOIT rejeter l'assignation d'un prestataire archivé — code `errors.contractors.archived`.
- **AC-006-D** : L'action « Reprendre en main » DOIT être accessible depuis `awaiting_quote` et `quote_received` — elle DOIT ramener la tâche à `in_progress` sans l'annuler et sans effacer son historique.

---

#### US-TP-007 — Générer et révoquer un lien prestataire

**En tant que** `Persona-Syndic`,
**je veux** générer et révoquer un lien tokenisé pour un prestataire,
**afin de** lui donner un accès minimal et temporaire à la tâche.

**Critères d'acceptation**

- **AC-007-A** : Le système DOIT générer un token opaque à durée de vie de 30 jours (configurable via `CONTRACTOR_TOKEN_TTL_DAYS`).
- **AC-007-B** : Le système DOIT n'autoriser qu'un seul token actif par tâche — générer un nouveau token DOIT révoquer l'ancien.
- **AC-007-C** : Le gestionnaire DOIT pouvoir révoquer manuellement le token à tout moment.
- **AC-007-D** : Un token révoqué ou expiré DOIT retourner une erreur explicite — code `errors.contractor_token.invalid`.
- **AC-007-E** : Les tokens révoqués ou expirés DOIVENT être conservés en audit log (inutilisables, traçables).
- **AC-007-F** : Le parcours prestataire DOIT être accessible sans compte utilisateur, sans navigation globale, sans exposition de données d'autres tâches.

---

#### US-TP-008 — Soumettre un devis (prestataire — Phase 1)

**En tant que** `Persona-Prestataire`,
**je veux** soumettre un devis depuis le lien tokenisé,
**afin de** proposer un coût et des pièces justificatives au gestionnaire.

**Critères d'acceptation**

- **AC-008-A** : Le système DOIT refuser la soumission si le token est expiré ou révoqué — code `errors.contractor_token.invalid`.
- **AC-008-B** : Le système DOIT refuser la soumission si la tâche n'est pas à l'état `awaiting_quote`.
- **AC-008-C** : Le prestataire DOIT pouvoir soumettre : commentaire (obligatoire), documents (optionnel), montant proposé (optionnel).
- **AC-008-D** : À la soumission, le système DOIT faire passer la tâche de `awaiting_quote` à `quote_received`.
- **AC-008-E** : Le montant soumis DOIT être stocké comme `proposed_quote` — non engageant tant que le syndic n'a pas clôturé.
- **AC-008-F** : La soumission DOIT créer une entrée timeline immuable (`from_state = awaiting_quote`, `to_state = quote_received`).

---

#### US-TP-009 — Valider ou rejeter le devis (syndic)

**En tant que** `Persona-Syndic`,
**je veux** valider ou rejeter le devis soumis par le prestataire,
**afin de** décider de la suite de l'intervention.

**Critères d'acceptation**

- **AC-009-A** : La validation DOIT être possible depuis `quote_received` uniquement — transition vers `awaiting_intervention`.
- **AC-009-B** : Le rejet DOIT être possible depuis `quote_received` — transition vers `in_progress`.
- **AC-009-C** : À la validation, le système DOIT marquer le devis comme `accepted_quote` (valeur figée dès cet instant).
- **AC-009-D** : La décision (validation ou rejet) DOIT créer une entrée timeline immuable avec le motif.

---

#### US-TP-010 — Déclarer la fin d'intervention (prestataire — Phase 2)

**En tant que** `Persona-Prestataire`,
**je veux** déclarer la fin d'intervention depuis le lien tokenisé,
**afin de** soumettre le rapport et les preuves de travaux au gestionnaire.

**Critères d'acceptation**

- **AC-010-A** : Le système DOIT refuser la déclaration si le token est expiré ou révoqué.
- **AC-010-B** : Le système DOIT refuser la déclaration si la tâche n'est pas à l'état `awaiting_intervention`.
- **AC-010-C** : Le prestataire DOIT pouvoir soumettre : commentaire (obligatoire), documents de preuve (optionnel).
- **AC-010-D** : À la soumission, le système DOIT faire passer la tâche de `awaiting_intervention` à `report_received`.
- **AC-010-E** : La déclaration DOIT créer une entrée timeline immuable.

---

#### US-TP-011 — Valider ou rejeter l'intervention et clôturer

**En tant que** `Persona-Syndic`,
**je veux** valider ou rejeter le rapport d'intervention,
**afin de** clôturer la tâche et alimenter le carnet d'entretien.

**Critères d'acceptation**

- **AC-011-A** : La validation DOIT être possible depuis `report_received` uniquement — transition vers `closed`.
- **AC-011-B** : Le rejet DOIT être possible depuis `report_received` — transition vers `awaiting_intervention`.
- **AC-011-C** : À la clôture, le gestionnaire DOIT confirmer le `completion_cost` (montant retenu).
- **AC-011-D** : Le système DOIT alimenter le carnet d'entretien avec : titre, description, `completion_cost`, `closed_at`, prestataire impliqué, documents de l'intervention.
- **AC-011-E** : L'alimentation du carnet DOIT être atomique avec la clôture — si l'alimentation échoue, la clôture DOIT être annulée, code `errors.tasks.maintenance_log_failed`.
- **AC-011-F** : Le système DOIT archiver un cycle de résolution immuable dans la timeline (snapshot des champs sensibles).
- **AC-011-G** : Après clôture, `completion_cost`, `closed_at`, `assigned_contractor`, `accepted_quote` et documents prestataires DOIVENT être en lecture seule.

---

#### US-TP-012 — Reprendre en main depuis une phase devis

**En tant que** `Persona-Syndic`,
**je veux** reprendre le contrôle de la tâche sans l'annuler,
**afin d'** abandonner un prestataire non réactif et repartir sur de nouvelles bases.

**Critères d'acceptation**

- **AC-012-A** : L'action DOIT être accessible depuis `awaiting_quote` et `quote_received`.
- **AC-012-B** : L'action DOIT faire passer la tâche à `in_progress` sans créer d'état `cancelled`.
- **AC-012-C** : L'historique (timeline, documents, devis soumis) DOIT être conservé intégralement.
- **AC-012-D** : L'action DOIT créer une entrée timeline immuable mentionnant l'acteur et la cause.

---

#### US-TP-013 — Réouvrir une tâche

**En tant que** `Persona-Syndic`,
**je veux** réouvrir une tâche clôturée ou annulée,
**afin de** traiter une résolution incomplète ou non conforme.

**Critères d'acceptation**

- **AC-013-A** : La réouverture DOIT être possible depuis `closed` et `cancelled` uniquement.
- **AC-013-B** : La réouverture DOIT faire passer la tâche à `in_progress`.
- **AC-013-C** : Les données du cycle de résolution précédent DOIVENT rester archivées et non modifiables.
- **AC-013-D** : La réouverture DOIT créer une entrée timeline immuable (acteur + motif).
- **AC-013-E** : Le système DOIT refuser la réouverture depuis tout autre état — code `errors.tasks.invalid_transition`.

---

#### US-TP-014 — Immutabilité des champs sensibles après clôture

**En tant qu'** architecte système,
**je veux** que les champs sensibles soient immuables après clôture,
**afin de** garantir la traçabilité légale et financière (ADR-009).

> Voir §2 pour la liste des champs figés.

**Critères d'acceptation**

- **AC-014-A** : Tout endpoint ciblant un champ figé sur une tâche `closed` DOIT retourner HTTP 422 — code `errors.tasks.field_frozen`.
- **AC-014-B** : Tout endpoint `DELETE` sur un document prestataire d'une tâche `closed` DOIT retourner HTTP 422.
- **AC-014-C** : Toute mutation réussie sur un champ financier DOIT produire une entrée dans `audit_log` avec `old_values` / `new_values` JSONB.

---

#### US-TP-015 — Timeline append-only de résolution

**En tant que** `Persona-Syndic` ou `Persona-Conseil`,
**je veux** consulter la timeline immuable d'une tâche,
**afin de** suivre l'historique complet des transitions et cycles.

**Critères d'acceptation**

- **AC-015-A** : La timeline DOIT afficher chaque transition avec : acteur, horodatage, état précédent, nouvel état.
- **AC-015-B** : La timeline DOIT être en lecture seule — aucune action d'édition ou de suppression ne DOIT être exposée.
- **AC-015-C** : Les cycles de résolution successifs DOIVENT être visuellement distingués dans la timeline.
- **AC-015-D** : Le snapshot des champs clés au moment de chaque transition DOIT être consultable.

---

#### US-TP-016 — Supprimer ou annuler une tâche

**En tant que** `Persona-Syndic`,
**je veux** supprimer ou annuler une tâche,
**afin de** purger les tâches erronées ou définitivement abandonnées.

**Critères d'acceptation**

- **AC-016-A** : La suppression DOIT être refusée si la tâche a alimenté le carnet d'entretien — code `errors.tasks.has_maintenance_log`.
- **AC-016-B** : La suppression DOIT être refusée si la tâche a reçu une soumission prestataire — code `errors.tasks.has_contractor_submission`.
- **AC-016-C** : L'annulation (transition vers `cancelled`) DOIT être possible depuis `open` et `in_progress` uniquement.
- **AC-016-D** : L'annulation DOIT créer une entrée timeline immuable avec le motif.

---

## 6. Exigences non-fonctionnelles

> **Note** — Seuils quantitatifs à affiner post-lancement (baseline an 1).

| ID | Catégorie | Exigence | Seuil cible |
|---|---|---|---|
| NFR-Perf-001 | Performance | Temps de réponse API liste tâches (p95) | < 500 ms |
| NFR-Perf-002 | Performance | Temps de réponse API création tâche | < 300 ms |
| NFR-Sec-001 | Sécurité | Token prestataire non-devinable (entropie minimale) | 128 bits |
| NFR-Sec-002 | Sécurité | Durée de vie token configurable via env var | `CONTRACTOR_TOKEN_TTL_DAYS` |
| NFR-Sec-003 | Sécurité | Validation token avant toute lecture/écriture prestataire | Obligatoire |
| NFR-Audit-001 | Traçabilité | Toute mutation de champ financier dans `audit_log` | 100 % |
| NFR-Integ-001 | Intégrité | Clôture + alimentation carnet atomiques | Transaction DB unique |
| NFR-Ret-001 | Rétention | Entrées timeline jamais supprimées | Politique append-only |

---

## 7. Hors scope

> **CRITICAL** — Ne pas implémenter les fonctionnalités ci-dessous pour éviter le scope creep et l'hallucination d'agent.

- Portail prestataire persistant avec historique d'interventions multi-tâches.
- Messagerie complète gestionnaire-prestataire (hors commentaires contextuels par tâche).
- Contractualisation ou commande formelle dans le module tâche.
- Gestion fine des achats ou validation comptable de facture.
- Notifications push ou e-mail hors du périmètre actuel.

---

## 8. Métriques de succès

- Délai moyen entre création et première prise en charge.
- Délai moyen entre création et clôture.
- Taux de tâches réouvertes après clôture.
- Part des tâches confiées à un prestataire externe.
- Part des tâches clôturées avec `completion_cost` et pièces jointes exploitables.

> Cibles, baselines et counter-metrics à définir post-lancement (baseline an 1).

---

## 9. Questions ouvertes

- Quelle granularité d'annulation distinguer entre `cancelled`, `abandoned` et `duplicate` ? — *Owner : Product* — *Revisit : avant spécification UX de US-TP-016.*
- Jusqu'où pousser la réouverture sans fragiliser la traçabilité du carnet d'entretien ? — *Owner : Product + Architecture* — *Revisit : lors de la conception du module carnet d'entretien.*
- Faut-il séparer visuellement les tâches purement internes des tâches impliquant un prestataire externe ? — *Owner : UX* — *Revisit : avant le premier prototype de la liste tâches.*

---

## Appendice A — Index de traçabilité

| US | Description | AC couverts |
|---|---|---|
| US-TP-001 | Créer une tâche | AC-001-A … AC-001-E |
| US-TP-002 | Convertir un signalement | AC-002-A … AC-002-F |
| US-TP-003 | Consulter / filtrer / rechercher | AC-003-A … AC-003-F |
| US-TP-004 | Modifier une tâche | AC-004-A … AC-004-C |
| US-TP-005 | Gérer les documents | AC-005-A … AC-005-E |
| US-TP-006 | Assigner un prestataire | AC-006-A … AC-006-D |
| US-TP-007 | Lien prestataire | AC-007-A … AC-007-F |
| US-TP-008 | Soumettre un devis | AC-008-A … AC-008-F |
| US-TP-009 | Valider / rejeter le devis | AC-009-A … AC-009-D |
| US-TP-010 | Déclarer fin d'intervention | AC-010-A … AC-010-E |
| US-TP-011 | Valider / rejeter + clôturer | AC-011-A … AC-011-G |
| US-TP-012 | Reprendre en main | AC-012-A … AC-012-D |
| US-TP-013 | Réouvrir | AC-013-A … AC-013-E |
| US-TP-014 | Immutabilité système | AC-014-A … AC-014-C |
| US-TP-015 | Timeline | AC-015-A … AC-015-D |
| US-TP-016 | Supprimer / annuler | AC-016-A … AC-016-D |

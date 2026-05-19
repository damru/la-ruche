# PRD - Tâches et prestataires

> **Statut** — en cours
> **Dernière modification** — 19 mai 2026
> **Références** — `EP-007`, `EP-015`, `EP-009-S03`, `EP-018-S04`, `LEGAL.md §1.4`

## 1. En-tête et contexte

### Objectif de la feature

Permettre aux gestionnaires de **résoudre un problème opérationnel jusqu'à sa clôture**, avec deux parcours complémentaires :

- un parcours interne de gestion de tâche ;
- un sous-parcours ultra-minimal prestataire via lien tokenisé.

### Pourquoi

La valeur métier de la feature n'est pas la création d'une tâche en soi, mais la **résolution traçable d'un besoin d'intervention**. Le système doit :

- faciliter la coordination interne ;
- ouvrir ponctuellement un accès prestataire sans créer de compte ;
- conserver une trace fiable de la clôture, du coût proposé et des documents ;
- limiter les retours arrière sur des données déjà figées ou réutilisées ailleurs.

### KPIs

- Délai moyen entre création et première prise en charge.
- Délai moyen entre création et clôture.
- Taux de tâches réouvertes après clôture.
- Part des tâches confiées à un prestataire externe.
- Part des tâches clôturées avec coût et pièces jointes exploitables.

## 2. Profils utilisateurs et cas d'usage

### Personas

| Persona | Besoin principal | Attente clé |
|---|---|---|
| Syndic bénévole | Faire aboutir une intervention | Disposer d'un pilotage simple et traçable |
| Conseil syndical | Suivre et co-gérer l'opérationnel | Pouvoir agir sans ouvrir l'administration générale |
| Prestataire externe | Répondre à une demande précise | Recevoir un lien simple sans onboarding |
| Copropriétaire | Être indirectement servi par la résolution | Pas de back-office tâche, mais une exécution efficace |
| Locataire | Signaler en amont, pas gérer la tâche | Comprendre que la résolution passe ensuite côté gestionnaires |

### User stories

- En tant que `syndic`, je veux créer et qualifier une tâche pour piloter une résolution sans perdre d'information.
- En tant que `syndic`, je veux pouvoir convertir un signalement en tâche pour éviter une ressaisie et garder la continuité de traitement.
- En tant que `conseil syndical`, je veux suivre et mettre à jour l'avancement pour aider à la résolution.
- En tant que `syndic`, je veux pouvoir solliciter un prestataire via un lien temporaire plutôt que créer un compte.
- En tant que `prestataire`, je veux déposer un commentaire, des pièces et un coût proposé depuis un parcours minimal.
- En tant que `syndic`, je veux clôturer proprement la tâche et figer les éléments réutilisés par le carnet d'entretien.

## 3. Exigences fonctionnelles

### MoSCoW — actions et périmètre

| Priorité | Action | Attendu produit |
|---|---|---|
| Must | Créer une tâche | Titre, description, priorité, contexte, assignation |
| Must | Convertir un signalement en tâche | Reprise du contexte source, priorité, continuité de suivi |
| Must | Lire, filtrer et rechercher les tâches | Vue orientée résolution et tri par dernière modification |
| Must | Modifier une tâche ouverte | Tant qu'aucun verrou métier n'est atteint |
| Must | Attacher des documents côté gestionnaire | Pièces utiles avant ou pendant l'intervention |
| Must | Assigner un prestataire | Référentiel, création opportuniste ou nom libre |
| Must | Générer et révoquer un lien prestataire | Accès contextuel et temporaire |
| Must | Permettre la soumission prestataire | Commentaire, documents, coût proposé |
| Must | Valider puis clôturer la tâche | Avec coût éventuel et alimentation du carnet |
| Must | Réouvrir une tâche si nécessaire | Sans effacer la traçabilité passée |
| Must | Empêcher les modifications incohérentes après état terminal | Principes d'immutabilité sur les champs sensibles |
| Should | Exposer une timeline de résolution | Suivi métier lisible de bout en bout |
| Should | Mieux distinguer validation prestataire et clôture finale | Réduire les ambiguïtés d'état |
| Could | Relances prestataires | Aides à la coordination, pas critiques |
| Won't (lot actuel) | Portail prestataire complet avec historique multi-tâches | Le parcours prestataire reste volontairement minimal |

### Actions détaillées

#### 3.1 Créer une tâche

**Logique métier**

- Une tâche formalise un besoin de résolution, pas seulement une note interne.
- La création doit permettre de qualifier l'urgence, le contexte et l'assignation initiale.
- Une tâche peut exister sans prestataire référentiel, avec un nom libre si nécessaire.

**Cas d'erreur**

- titre ou description insuffisants ;
- priorité absente ;
- assignation incohérente ;
- doublon manifeste déjà en cours.

#### 3.2 Consulter, filtrer et rechercher les tâches

**Logique métier**

- La liste doit servir la résolution rapide, donc privilégier les tâches récemment modifiées et l'état courant.
- Les filtres doivent rendre visibles les urgences, les tâches en attente prestataire et les tâches à valider.
- La vue doit distinguer les tâches ouvertes, en cours, en attente, clôturées ou annulées.

**Cas d'erreur**

- résultat vide mal expliqué ;
- filtres contradictoires ;
- tâche inaccessible selon le rôle.

#### 3.2 bis Convertir un signalement en tâche

**Logique métier**

- Une tâche peut être créée directement ou résulter d'un signalement existant.
- La conversion doit reprendre le maximum de contexte utile sans imposer une ressaisie au gestionnaire.
- Le lien entre signalement source et tâche créée doit rester visible pour préserver la continuité de résolution.

**Cas d'erreur**

- signalement déjà converti ;
- signalement fermé ou non convertible ;
- priorité de tâche absente au moment de la conversion ;
- perte d'informations du signalement dans la tâche cible.

#### 3.3 Modifier une tâche ouverte

**Logique métier**

- Les gestionnaires peuvent modifier le contenu d'une tâche tant qu'aucun verrou métier ne l'interdit.
- Les champs réutilisés par d'autres modules deviennent plus contraints à l'approche de la clôture.
- La feature doit refléter les principes d'immutabilité sur les champs sensibles une fois la tâche clôturée.

**Cas d'erreur**

- modification d'une tâche clôturée sur un champ figé ;
- état incompatible avec l'action demandée ;
- conflit d'édition concurrente.

#### 3.4 Gérer les documents côté gestionnaire

**Logique métier**

- Les gestionnaires doivent pouvoir joindre des pièces dès la création ou l'édition.
- Les documents peuvent servir à cadrer le besoin, à briefer un prestataire ou à justifier une clôture.
- Les documents visibles dans le parcours prestataire doivent rester sous contrôle du gestionnaire.

**Cas d'erreur**

- document non transféré correctement ;
- fichier supprimé alors qu'il est encore utile à la tâche ;
- visibilité mal configurée ;
- format ou taille non acceptés.

#### 3.5 Assigner un prestataire

**Logique métier**

- L'assignation peut se faire à un prestataire du référentiel ou via un nom libre si la relation n'est pas encore structurée.
- Le système doit encourager la réutilisation du référentiel sans rendre l'action bloquante.
- La tâche doit conserver une trace lisible du prestataire effectivement visé.

**Cas d'erreur**

- prestataire archivé ou introuvable ;
- création opportuniste incomplète ;
- ambiguïté entre nom libre et prestataire référentiel.

#### 3.6 Générer et révoquer un lien prestataire

**Logique métier**

- Le lien prestataire est un accès restreint à une seule tâche.
- Il doit pouvoir être régénéré ou révoqué sans casser la tâche elle-même.
- Le parcours doit rester sans compte, sans navigation superflue et sans exposition d'autres données.

**Cas d'erreur**

- lien expiré ;
- lien révoqué ;
- lien déjà remplacé par un plus récent ;
- génération tentée sur une tâche non compatible.

#### 3.7 Soumettre une réponse prestataire

**Logique métier**

- Le prestataire peut déposer un commentaire, des pièces et un coût proposé.
- Le coût proposé est une **proposition non engageante** tant que le syndic n'a pas validé la clôture.
- La soumission fait passer la tâche dans un état intermédiaire, pas en clôture automatique.

**Cas d'erreur**

- lien expiré ;
- tâche déjà clôturée ;
- dépôt incomplet ;
- document non transféré ;
- commentaire vide si la politique métier impose un minimum.

#### 3.8 Valider la soumission et clôturer la tâche

**Logique métier**

- La clôture reste un acte de gestionnaire.
- Le gestionnaire décide du coût retenu, valide le résultat et déclenche l'alimentation automatique du carnet d'entretien si applicable.
- Une fois clôturée, la tâche ne doit plus permettre l'édition libre des champs sensibles réutilisés comme preuve opérationnelle ou financière.

**Cas d'erreur**

- clôture sans décision explicite ;
- coût incohérent ;
- tâche déjà clôturée ;
- alimentation du carnet impossible ;
- pièces de preuve manquantes selon la politique interne retenue.

#### 3.9 Réouvrir une tâche

**Logique métier**

- La réouverture doit exister pour les cas de résolution incomplète ou non conforme.
- Elle ne doit pas effacer les traces de la clôture précédente.
- Le produit doit garder lisibles les différents temps de vie de la tâche.

**Cas d'erreur**

- réouverture interdite par un verrou d'immutabilité sur certains champs ;
- tâche annulée au lieu d'être réouvrable ;
- incohérence entre état de tâche et état du carnet associé.

#### 3.10 Supprimer ou annuler une tâche

**Logique métier**

- La suppression doit rester exceptionnelle et réservée aux cas sans impact durable.
- L'annulation est préférable à la suppression quand une trace d'existence doit rester visible.
- Une tâche déjà clôturée ou ayant alimenté le carnet doit suivre des règles plus strictes.

**Cas d'erreur**

- suppression demandée après alimentation du carnet ;
- suppression d'une tâche avec soumission prestataire déjà reçue ;
- suppression interdite par un principe de traçabilité.

## 4. Sécurité et permissions

### Matrice CRUD simplifiée

| Domaine | Entrée UX principale | Syndic | Conseil | Copropriétaire | Locataire | Prestataire (token) |
|---|---|---|---|---|---|---|
| Tâches | Liste tâches + fiche tâche | `C / R / U / D` | `C / R / U / D` | `X` | `X` | `R1` |
| Signalement source lié | Vue signalement + fiche tâche | `C / R / U` | `C / R / U` | `R1` | `R1` | `X` |
| Pièces de tâche | Modale tâche / section pièces | `C / R / U / D` | `C / R / U / D` | `X` | `X` | `C / R1 / U1` |
| Référentiel prestataires | Catalogue + autocomplete | `C / R / U / D` | `C / R / U / D` | `R` | `R` | `X` |
| Affectation prestataire | Modale tâche / champ prestataire | `C / R / U / D1` | `C / R / U / D1` | `X` | `X` | `X` |
| Lien prestataire | Carte tâche / bloc lien prestataire | `C / R / U / D` | `C / R / U / D` | `X` | `X` | `X` |
| Réponse prestataire | Page prestataire / formulaire | `R / U` | `R / U` | `X` | `X` | `C / R1 / U1 / D1` |

### Règles de sécurité

- Le lien prestataire ne donne accès qu'à un périmètre ultra-minimal et contextuel.
- Le coût proposé par le prestataire n'est jamais traité comme une validation de dépense.
- Les actions destructives doivent être masquées ou bloquées après état terminal ou alimentation du carnet.
- Les champs figés après clôture suivent les principes d'immutabilité portés par `EP-018`.

## 5. UX/UI

### User flows attendus

#### Flow 0 — Entrée depuis un signalement

1. Le gestionnaire ouvre un signalement qualifié.
2. Il choisit de le convertir en tâche et renseigne la priorité de traitement.
3. La tâche est créée avec reprise du contexte utile.
4. Le lien entre signalement source et tâche reste visible dans les deux sens.

#### Flow 1 — Gestionnaire interne

1. Le gestionnaire crée la tâche.
2. Il qualifie la priorité, l'assignation et le contexte.
3. Il joint les documents utiles.
4. Il suit l'avancement, valide la soumission éventuelle et clôture.

#### Flow 2 — Sous-parcours prestataire tokenisé

1. Le gestionnaire génère le lien prestataire.
2. Le prestataire ouvre une page unique, sans navigation globale.
3. Il lit le contexte minimal, ajoute ses pièces, son commentaire et son coût proposé.
4. Il soumet puis sort du parcours.

#### Flow 3 — Clôture et trace

1. Le gestionnaire vérifie les éléments reçus.
2. Il valide le résultat de l'intervention.
3. Il choisit le coût retenu.
4. La tâche passe en clôture et le carnet d'entretien est alimenté si nécessaire.

### Cas d'erreur et états UX

- État vide : aucune tâche sur le périmètre courant.
- État conversion depuis signalement : expliquer clairement ce qui est repris et ce qui doit encore être complété.
- État lien prestataire invalide : message simple, sans jargon, avec cause explicite.
- État attente prestataire : visibilité claire de ce qui est attendu.
- État pré-validation reçue : différence nette entre `soumis` et `clôturé`.
- État clôturé figé : les champs non modifiables sont affichés comme historiques.
- Erreur de clôture : indiquer précisément si le blocage vient du coût, des pièces, du statut ou d'une règle d'immutabilité.

## 6. Hors scope et questions ouvertes

### Hors scope actuel

- portail prestataire persistant avec historique d'interventions ;
- messagerie complète gestionnaire-prestataire ;
- contractualisation ou commande formelle dans le module tâche ;
- gestion fine des achats ou validation comptable de facture.

### Questions ouvertes

- Quelle granularité d'annulation faut-il distinguer entre `annulée`, `abandonnée` et `doublon` ?
- Jusqu'où pousser la réouverture sans fragiliser la traçabilité du carnet d'entretien ?
- Faut-il séparer visuellement les tâches purement internes des tâches impliquant un prestataire externe ?
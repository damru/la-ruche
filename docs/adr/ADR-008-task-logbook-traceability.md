# ADR-008 — Traçabilité souple tâches / carnet d'entretien / soumissions prestataire

**Date :** 29 avril 2026  
**Statut :** Accepté  
**Contexte :** EP-007-S04/S05, EP-009-S03/S05

---

## Contexte

Le modèle initial imposait implicitement une relation **1 tâche = 1 ligne de carnet** :

1. index unique partiel `maintenance_log(task_id)` ;
2. alimentation automatique idempotente via `ON CONFLICT DO NOTHING` ;
3. FKs de suppression trop rigides pour les traces métier autour d'une tâche clôturée.

Ce modèle casse trois usages légitimes :

- une tâche rouverte puis re-clôturée doit créer une nouvelle ligne de carnet ;
- la suppression d'une tâche ne doit pas supprimer l'historique du carnet ;
- la pré-validation prestataire doit pouvoir proposer un coût avant la validation finale par le syndic.

## Décision

### §1 — `maintenance_log.task_id` devient un lien de traçabilité

La colonne `maintenance_log.task_id` reste présente mais n'impose plus une relation forte.

- suppression de l'index unique `1 tâche → 1 entrée` ;
- FK conservée avec `ON DELETE SET NULL` ;
- chaque transition **non closed → closed** crée une nouvelle entrée automatique dans le carnet.

### §2 — `vendor_submission.task_id` suit le même principe

La soumission prestataire reste historisée même si la tâche est supprimée.

- `vendor_submission.task_id` devient nullable ;
- FK passée en `ON DELETE SET NULL`.

### §3 — coût courant porté par la tâche

Le coût proposé ou confirmé est stocké sur `task.completion_cost`.

- le prestataire peut le renseigner lors de la pré-validation ;
- le syndic le retrouve pré-rempli avant clôture ;
- le carnet reçoit un snapshot du coût à chaque clôture effective.

### §4 — statut intermédiaire de pré-validation

Ajout du statut `awaiting_validation` sur `task_status`.

- `awaiting_vendor` = token envoyé, attente du prestataire ;
- `awaiting_validation` = retour prestataire reçu, validation finale côté syndic encore requise ;
- `closed` = seule transition qui crée une ligne de carnet.

## Alternatives rejetées

| Option | Raison du rejet |
|---|---|
| Conserver l'unicité `maintenance_log.task_id` | Empêche l'historique multi-interventions d'une même tâche |
| Table de jointure dédiée tâche ↔ carnet | Plus lourde pour un besoin couvert par un lien informatif simple |
| Supprimer totalement `task_id` du carnet | Perte de traçabilité et de navigation depuis l'UI |
| Auto-clôturer après soumission prestataire | Empêche la validation humaine du coût et de la fin d'intervention |

## Conséquences

- migration SQL : nouveau statut `awaiting_validation`, colonne `task.completion_cost`, FKs `ON DELETE SET NULL`, suppression de l'index unique `maintenance_log.task_id` ;
- backend : la clôture de tâche devient le point unique de création automatique dans le carnet ;
- frontend : la page prestataire capture un coût optionnel, et la modale de clôture du syndic le réutilise ;
- la vue carnet peut afficher plusieurs entrées historiques liées à une même tâche.

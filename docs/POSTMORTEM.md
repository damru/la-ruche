# POSTMORTEM.md — Post-mortems

> **Règle de processus** — Une tâche ne peut être close `wont do` qu'accompagnée
> d'un post-mortem écrit : cause racine, pourquoi elle n'est pas corrigeable
> maintenant, et risque résiduel le cas échéant. Un `wont do` nu, sans
> justification, est une violation de processus. (Cette règle doit aussi être
> reflétée dans `AGENTS.md` §Workflow, fichier protégé — approbation humaine
> requise.)

---

## 2026-09-15 — 4 crashs du test de concurrence audit-log (t_13e08ee0)

### Contexte

`t_13e08ee0` (test de concurrence sur le journal d'audit des clés de
répartition) a été déposée par Hopper suite à la rétro `t_10a31fc1` d'EP-001-S10.
Le test devait vérifier le risque TOCTOU signalé : la lecture `before` hors
verrou entre le SELECT et l'UPDATE dans
`db::allocation_key::{update, delete, set_share}`. La tâche a « crashé » 4 fois
puis a été close `wont do` sans justification.

### Chronologie

| Date | Événement |
|---|---|
| 2026-09-13 18:02–18:05 | 4 runs (282/286/287/288), chacun ~60 s, terminés en `protocol_violation` |
| 2026-09-14 10:36 | Close `wont do` (run 291) — 7 caractères, aucun commentaire, aucun résumé |
| 2026-09-14 | Rétro : `t_15ad7bbf` déposée pour refaire la tâche correctement |
| 2026-09-15 | `t_15ad7bbf` : test écrit, 2 bugs réels reproduits, correctif, PR #100 mergée |

### Cause racine des 4 crashs

Les 4 « crashs » ne sont **pas des échecs de test** : ce sont 4
`protocol_violation` du dispatcher Hermes — le worker s'est terminé proprement
(rc=0) **sans appeler `kanban_complete` ni `kanban_block`**. Facteurs :

- workspace `scratch` (répertoire temporaire, aucun checkout git) ;
- pas de cargo/docker/PostgreSQL disponibles (déjà signalé par la rétro parente
  `t_10a31fc1`) ;
- le test n'a donc jamais été écrit ni exécuté durant `t_13e08ee0`.

Autrement dit : les 4 crashs sont une défaillance du worker (processus agent
terminé sans émettre l'appel kanban terminal), **pas** de la flakiness de test,
**pas** une instabilité d'infra de test. Le risque « flakiness résiduelle masquée
par la réécriture » évoqué dans la rétro est donc écarté : il n'y avait aucun
test à masquer.

### Le bug TOCTOU était réel (et distinct des crashs)

Quand `t_15ad7bbf` a réellement écrit le test
(`backend/tests/allocation_key_concurrency_test.rs`), le premier run a reproduit
**deux vrais bugs** :

1. `set_share` concurrent sur le même `(unit_id, key_id)` → violation de l'index
   unique partiel `uak_unique_active` (23505) : deux racers lisaient la même
   ligne « courante », la clôturaient et inséraient tous les deux.
2. `delete` concurrent sur la même clé → **8** entrées `allocation_key.deleted`
   en audit pour une seule suppression réelle : chaque racer lisait la ligne
   pré-suppression hors verrou et écrivait sa propre entrée d'audit.

### Confirmation : correctif réel, pas un contournement

Le correctif (`e171e73`, PR #100, mergé sur `origin/main`) :

- `set_share` et `delete` ouvrent leur transaction puis prennent
  `pg_advisory_xact_lock(hashtextextended(...))` sur l'identité de la ligne
  **avant** le read-before-write → sérialise les racers sur la même clé sans
  bloquer les clés distinctes ;
- `delete` vérifie `rows_affected() == 0` après l'`UPDATE` → un racer ayant perdu
  la course n'écrit jamais d'entrée d'audit orpheline ;
- `update` n'est pas modifié : son écriture d'audit est déjà conditionnée au
  résultat de `UPDATE ... RETURNING`.

C'est une vraie correction de la fenêtre TOCTOU (sérialisation côté base), pas un
contournement ni un affaiblissement du test. Vérifié par la revue (`t_15ad7bbf`
run 360) : 3/3 tests de concurrence, 4/4 tests d'audit existants,
`clippy --all-targets -D warnings` et `fmt --check` propres.

### Conclusion

1. Les 4 crashs initiaux = défaillance worker/dispatcher (rc=0 sans appel kanban
   terminal) dans un workspace sans outillage. Aucune flakiness, aucune
   instabilité d'infra de test.
2. Le risque TOCTOU était réel et est corrigé par un verrou consultatif
   transactionnel — correction de fond, pas un contournement.
3. Lacune de processus : `wont do` appliqué sans justification après 4 échecs.
   Règle introduite : `wont do` exige désormais un post-mortem écrit.

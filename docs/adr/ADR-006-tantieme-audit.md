# ADR-006 — Versioning des tantièmes et snapshot dans les appels de fonds

**Date :** 22 avril 2026  
**Statut :** Accepté  
**Contexte :** LEGAL.md §2.2 et §2.3 — EP-001 (Registre), EP-004 (Budget)

---

## Contexte

La table `unit_allocation_key` stockait une seule ligne par `(unit_id, key_id)` avec un `share_value` mutable (simple upsert). La table `levy_call` stockait uniquement le montant final calculé, sans référence aux tantièmes utilisés pour le calcul.

Ces deux lacunes rendent impossibles :
1. L'audit de toute modification des tantièmes (obligation légale — art. 10 loi 1965, décret 1967)
2. La justification d'un appel de fonds passé si les tantièmes ont été corrigés depuis (art. 14-1 loi 1965)
3. La production du relevé individuel de charges par lot (décret 1967 art. 48)

## Décision

### §2.2 — Versioning de `unit_allocation_key`

Ajout de `valid_from DATE` et `valid_to DATE` (NULL = enregistrement courant). Le schéma passe d'une PK composite `(unit_id, key_id)` à un `id UUID` avec un index partiel unique sur `(unit_id, key_id) WHERE valid_to IS NULL`.

**Règle de modification :** toute mise à jour de `share_value` ferme l'enregistrement courant (`valid_to = CURRENT_DATE`) et insère un nouvel enregistrement actif. Si la valeur est identique, aucune action (pas d'entrée d'historique inutile).

### §2.3 — Snapshot dans `levy_call`

Ajout de trois colonnes à `levy_call` :
- `key_id UUID` — clé de répartition utilisée pour le calcul
- `shares_snapshot INTEGER` — tantièmes du lot au moment de la génération
- `shares_total_snapshot INTEGER` — total des tantièmes de la clé au moment de la génération

Ces valeurs sont **immuables après INSERT** : elles représentent l'état du registre au moment de l'émission de l'appel. Aucune mise à jour rétroactive n'est autorisée.

## Alternatives écartées

| Alternative | Raison du rejet |
|---|---|
| Table d'audit séparée (triggers) | Complexité infra, pas de garantie d'intégrité applicative |
| JSONB history column | Non queryable, pas typé, complique les requêtes d'audit |
| Garde uniquement les valeurs courantes | Illégal : rend toute justification comptable historique impossible |
| `valid_from`/`valid_to` via `tstzrange` (temporal table) | Nécessite `btree_gist`, complexifie les requêtes sans bénéfice au stade actuel |

## Conséquences

- **`upsert_share` supprimé** → remplacé par `set_share` qui implémente le pattern close-and-insert dans une transaction.
- **Toutes les requêtes sur `unit_allocation_key`** doivent filtrer sur `valid_to IS NULL` pour obtenir les tantièmes courants.
- **`generate_levies`** capture le snapshot et le passe à l'INSERT dans `levy_call`.
- **`ComputedLevy`** (domain) porte désormais `key_id`, `shares_snapshot`, `shares_total_snapshot` pour les propager jusqu'à la couche DB sans recalcul.
- Les colonnes snapshot de `levy_call` sont nullable pour rétrocompatibilité avec les données existantes avant migration.
- Référencé par LEGAL.md §2.2, §2.3.

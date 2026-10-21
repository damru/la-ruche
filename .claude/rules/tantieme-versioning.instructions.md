---
description: 'Versionnement des tantièmes : valid_from/valid_to, snapshot à l’appel de fonds, blocage de cohérence Σ tantièmes (ADR-006/007).'
applyTo: 'backend/src/**,frontend/src/**,backend/migrations/**'
---

# Versionnement des tantièmes

- Les répartitions de tantièmes sont **datées** : `valid_from` / `valid_to` ; une modification de répartition n'écrase jamais l'historique, elle clôture la version courante et en crée une nouvelle (cf. ADR-006/ADR-007).
- Un appel de fonds **snapshotte** les tantièmes en vigueur à sa création : les décomptes ne sont jamais recalculés rétroactivement.
- Invariant de cohérence : la somme des tantièmes d'une répartition doit être cohérente (Σ par bâtiment/type conforme aux clés de répartition) ; toute écriture violant ce blocage est rejetée.
- Les migrations ne peuvent pas rétro-agir sur les snapshots existants.

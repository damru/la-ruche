# ADR-003 — Structure projet Rust

**Date :** 2026  
**Statut :** Accepté

---

## Contexte

Choisir entre workspace multi-crates et single crate modulaire.

## Décision

**Single crate modulaire.**

## Justification

- Projet OSS avec contributeurs occasionnels : barrière d'entrée minimale
- Domaine métier borné : pas de lib réutilisable indépendamment prévue
- Frontières tenues par convention (`pub(crate)`, pas de `pub` sans raison)
- Migration vers workspace possible sans réécriture si besoin futur
- Un seul `Cargo.toml`, une seule commande `cargo build`

## Règles de module — à respecter impérativement

```
domain/   → aucune dépendance vers db/, api/, auth/, storage/
db/       → dépend de domain/ uniquement
auth/     → dépend de db/ et domain/
storage/  → aucune dépendance interne
api/      → dépend de tout (point d'assemblage)
jobs/     → dépend de db/ et domain/
```

La violation de ces règles est un motif de refus de PR.

## Conséquences

- Les tests unitaires de `domain/` s'exécutent sans DB
- Les tests d'intégration dans `db/` nécessitent une DB de test
- `cargo test` en CI : deux étapes (unit sans DB, integration avec DB)

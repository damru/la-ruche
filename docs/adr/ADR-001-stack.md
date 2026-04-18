# ADR-001 — Stack technique & format API

**Date :** 2026  
**Statut :** Accepté

---

## Contexte

Choix de la stack backend pour une webapp OSS de gestion de copropriété,
self-hosted, binaire léger, contribuable par des tiers.

## Décisions

### Rust + Axum
- Binaire unique ~10MB, démarrage < 100ms
- Pas de runtime JVM ou Node à installer pour le self-hosting
- Axum : framework async mature, basé sur tower, excellent écosystème middleware
- SQLx : vérification des requêtes SQL à la compilation → zéro runtime SQL error

### REST JSON + OpenAPI via utoipa
- REST : universel, simple à comprendre pour les contributeurs
- utoipa : génération OpenAPI depuis les annotations Rust, pas de fichier YAML à maintenir
- Versioning via préfixe `/api/v1/`

### SQLx migrate (pas Flyway/Liquibase)
- Intégré à SQLx, pas de dépendance externe
- Fichiers `.sql` purs, lisibles, réversibles
- Exécution au démarrage ou via `sqlx migrate run`
- Compatible avec le binaire unique (migrations embarquées)

## Alternatives rejetées

| Alternative | Raison du rejet |
|---|---|
| Actix-web | Moins ergonomique, modèle acteur plus complexe |
| Warp | Moins mature, moins de contributeurs |
| GraphQL | Overhead de complexité non justifié pour ce scope |
| Diesel | Synchrone, moins adapté à Axum async |
| SeaORM | Trop abstrait, cache le SQL, moins de contrôle |

## Conséquences

- Les requêtes SQL doivent être vérifiables à la compilation → `DATABASE_URL` requis au build
- CI doit avoir accès à une instance PostgreSQL pour compiler avec SQLx
- Alternative : utiliser `sqlx::query!` avec `SQLX_OFFLINE=true` + fichier `.sqlx/` committé

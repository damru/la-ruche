# La Ruche

Application web de gestion de copropriété participative, conçue pour les syndics bénévoles et les conseils syndicaux. Elle couvre l'ensemble du cycle de vie d'une copropriété : registre des lots, assemblées générales, budget et appels de fonds, contrats et prestataires, carnet d'entretien, tâches et sinistres, et mise à disposition des documents aux copropriétaires.

## Fonctionnalités

- **Registre** : lots, copropriétaires, locataires, tantièmes et répartition.
- **Assemblées générales** : convocations, résolutions, votes, procès-verbaux.
- **Budget & appels de fonds** : budget prévisionnel, charges, appels de fonds, décomptes.
- **Contrats** : prestataires, contrats, suivis d'interventions.
- **Tâches & sinistres** : suivi des interventions et sinistres.
- **Carnet d'entretien** : historique d'entretien de l'immeuble.
- **Documents (GED)** : téléversement, classement et accès aux documents, liés aux entités du registre.
- **Consultations** : consultations en ligne du conseil syndical.
- **Calendrier** : événements de la copropriété.

## Rôles

- **Syndic (manager)** : accès complet.
- **Conseil syndical (board_member)** : accès élargi, sans gestion du budget.
- **Copropriétaire (owner)** et **locataire (tenant)** : accès à leurs lots et aux documents partagés.
- **Prestataire (token)** : accès par jeton, limité à l'intervention concernée.

## Stack technique

Backend Rust (Axum, SQLx) sur PostgreSQL 16, frontend Vue 3 + TypeScript + Tailwind v4, stockage objet RustFS, reverse-proxy Caddy, conteneurs Docker Compose. Détails : [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Démarrage rapide (développement)

```bash
make setup      # base de données, secrets, seeds
make dev        # backend + frontend
```

## Déploiement

### Démo

Un environnement de démonstration autonome, avec données factices réinitialisables.

```bash
LA_RUCHE__APP__DEMO_MODE=true docker compose up --build
```

- Application : http://localhost:443 (Caddy) ou http://localhost:8080 (backend direct).
- Console RustFS : http://localhost:9001 (stockage : port 9000).
- La base est (re)jouée depuis `backend/seeds/demo.sql`. L'endpoint administrateur `POST /admin/reset-demo` réinitialise la démo.

### Production

```bash
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

- Image de production : `docker.io/damru/la-ruche`.
- Ports PostgreSQL et RustFS non exposés publiquement; Caddy écoute sur le port 80 (HTTPS automatique via ACME si un domaine est configuré dans le Caddyfile).
- Surchargez impérativement les secrets par variables d'environnement : `JWT_SECRET`, `POSTGRES_PASSWORD`, `S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY`.
- Logs applicatifs JSON en rotation (50 Mo × 5), `RUST_LOG=info`.

### Variables de configuration

La configuration suit le préfixe `LA_RUCHE__` (structure `LA_RUCHE__SECTION__CLE`, ex. `LA_RUCHE__APP__DEMO_MODE`). Voir `config/` et `docker-compose.yml`.

## Licence

Double licence : AGPL-3.0 (usage non commercial) et licence commerciale. Voir [LICENSE](LICENSE), [LEGAL.md](docs/LEGAL.md) et [CLA.md](CLA.md). Aucune télémétrie.

## Contribuer

Voir [CONTRIBUTING.md](CONTRIBUTING.md). Pull requests vers `main`, commits au format Conventional Commits.

## Documentation

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) : architecture technique.
- [docs/PRD.md](docs/PRD.md) et [docs/EPICS.md](docs/EPICS.md) : produit et épopées.
- [docs/DESIGN.md](docs/DESIGN.md) : design system.
- [docs/specs/](docs/specs/), [docs/adr/](docs/adr/), [docs/ddr/](docs/ddr/) : implémentation.

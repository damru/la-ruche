<!-- logo -->
<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset=".github/assets/lockup-dark.svg">
    <source media="(prefers-color-scheme: light)" srcset=".github/assets/lockup-light.svg">
    <img src=".github/assets/lockup-light.svg" alt="La Ruche" width="340">
  </picture>
</p>

<p align="center">
  <strong>La copropriété, sans le syndic professionnel.</strong><br>
  Application web open source de gestion de copropriété participative, pour les syndics bénévoles et les conseils syndicaux.
</p>

<p align="center">
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/Rust-1.85%2B%20(edition%202024)-DEA584?logo=rust&logoColor=white" alt="Rust"></a>
  <a href="https://vuejs.org"><img src="https://img.shields.io/badge/Vue-3.5-42B883?logo=vuedotjs&logoColor=white" alt="Vue 3"></a>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/licence-AGPL--3.0%20%2F%20commerciale-643fd5" alt="Licence AGPL-3.0 / commerciale"></a>
  <a href="docs/"><img src="https://img.shields.io/badge/docs-la--ruche-643fd5?logo=gitbook&logoColor=white" alt="Documentation"></a>
</p>

---

Application web de gestion de copropriété participative. Elle couvre l'ensemble du cycle de vie d'une copropriété : registre des lots, assemblées générales, budget et appels de fonds, contrats et prestataires, carnet d'entretien, tâches et sinistres, et mise à disposition des documents aux copropriétaires.

## 🐝 Fonctionnalités

- 🗂️ **Registre** : lots, copropriétaires, locataires, tantièmes et répartition.
- 🗳️ **Assemblées générales** : convocations, résolutions, votes, procès-verbaux.
- 💰 **Budget & appels de fonds** : budget prévisionnel, charges, appels de fonds, décomptes.
- 📄 **Contrats** : prestataires, contrats, suivis d'interventions.
- 🛠️ **Tâches & sinistres** : suivi des interventions et sinistres.
- 🏠 **Carnet d'entretien** : historique d'entretien de l'immeuble.
- 📁 **Documents (GED)** : téléversement, classement et accès aux documents, liés aux entités du registre.
- 🔄 **Consultations** : consultations en ligne du conseil syndical.
- 📅 **Calendrier** : événements de la copropriété.

## 👥 Rôles

- 🪄 **Syndic (manager)** : accès complet.
- 🧭 **Conseil syndical (board_member)** : accès élargi, sans gestion du budget.
- 🔑 **Copropriétaire (owner)** et **locataire (tenant)** : accès à leurs lots et aux documents partagés.
- 🎫 **Prestataire (token)** : accès par jeton, limité à l'intervention concernée.

## 🧱 Stack technique

Backend Rust (Axum, SQLx) sur PostgreSQL 16, frontend Vue 3 + TypeScript + Tailwind v4, stockage objet RustFS, reverse-proxy Caddy, conteneurs Docker Compose. Détails : [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## 🚀 Démarrage rapide (développement)

```bash
make setup      # base de données, secrets, seeds
make dev        # backend + frontend
```

## 📦 Déploiement

### 🎬 Démo

Un environnement de démonstration autonome, avec données factices réinitialisables.

```bash
LA_RUCHE__APP__DEMO_MODE=true docker compose up --build
```

- Application : http://localhost:8080 (Caddy, HTTP) ou https://localhost:443 (HTTPS).
- Console RustFS : http://localhost:9001 (stockage : port 9000).
- La base est (re)jouée depuis `backend/seeds/demo.sql`. L'endpoint administrateur `POST /admin/reset-demo` réinitialise la démo.

### 🏭 Production

```bash
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

- Image de production : `docker.io/damru/la-ruche`.
- Ports PostgreSQL et RustFS non exposés publiquement ; l'application est servie par Caddy sur 443 (HTTPS via ACME si un domaine est configuré dans le Caddyfile) et sur 8080 (HTTP interne).
- Surchargez impérativement les secrets par variables d'environnement : `JWT_SECRET`, `POSTGRES_PASSWORD`, `S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY`.
- Logs applicatifs JSON en rotation (50 Mo × 5), `RUST_LOG=info`.

### ⚙️ Variables de configuration

La configuration suit le préfixe `LA_RUCHE__` (structure `LA_RUCHE__SECTION__CLE`, ex. `LA_RUCHE__APP__DEMO_MODE`). Voir `config/` et `docker-compose.yml`.

## ⚖️ Couverture légale

La Ruche implémente et respecte les obligations légales de la copropriété française. Le référentiel complet, article par article, est maintenu dans [docs/LEGAL.md](docs/LEGAL.md) (couverture : ✅ couverte · ⚠️ partielle · 🔲 hors périmètre logiciel · ❌ non couverte).

### Textes de référence

- **Loi n°65-557 du 10 juillet 1965** — statut de la copropriété (syndicat, syndic, conseil syndical, charges, majorités).
- **Décret n°67-223 du 17 mars 1967** — application de la loi de 1965 (convocations, PV, état daté, appels de fonds).
- **Loi ALUR n°2014-366 du 24 mars 2014** — espace en ligne sécurisé, immatriculation RNC, pré-état daté.
- **Loi ELAN n°2018-1021 du 23 novembre 2018** — évolutions de la gouvernance et de la transaction.
- **Loi Climat et Résilience n°2021-1104 du 22 août 2021** — plan pluriannuel de travaux, fonds travaux.
- **Arrêté du 14 mars 2005** — plan comptable des copropriétés.
- **Décret n°2001-477 du 30 mai 2001** — carnet d'entretien de l'immeuble.
- **RGPD (Règlement UE 2016/679)** — protection des données personnelles.

### Obligations couvertes (✅)

| Obligation | Base légale |
|---|---|
| Espace en ligne sécurisé des copropriétaires | Art. 18-I-5° loi 1965 (ALUR) |
| Historique des tantièmes par lot | Art. 10 loi 1965 |
| Snapshot des tantièmes par appel de fonds | Art. 14-1 loi 1965 · art. 35 décret 1967 |
| Tenue de séance (président, secrétaire, scrutateurs) | Art. 15 décret 1967 |
| Règles de majorité (art. 24, 25, 26, 25-1) | Art. 24, 25, 25-1, 26 loi 1965 |
| PV signable (J+8), notification (1 mois), contestation (2 mois) | Art. 17, 18 décret 1967 · art. 42 loi 1965 |
| Relevé individuel de charges | Art. 18 loi 1965 · arrêté 14/03/2005 |
| Contrats et reconductions tacites | Art. 18 loi 1965 |
| Sinistres (délais de déclaration) | Code des assurances |

### Couvertures partielles et en cours (⚠️ / ❌)

Le détail exhaustif des couvertures partielles (RNC, carnet d'entretien, budget et appels de fonds, fonds travaux, état daté, RGPD, PPT…) et des écarts à combler, avec les épopées et stories correspondantes, figure dans le tableau synthétique de [docs/LEGAL.md](docs/LEGAL.md).

## 📜 Licence

Double licence : AGPL-3.0 (usage non commercial) et licence commerciale. Voir [LICENSE](LICENSE), [LEGAL.md](docs/LEGAL.md) et [CLA.md](CLA.md). Aucune télémétrie.

## 🤝 Contribuer

Voir [CONTRIBUTING.md](CONTRIBUTING.md). Pull requests vers `main`, commits au format Conventional Commits.

## 📚 Documentation

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) : architecture technique.
- [docs/PRD.md](docs/PRD.md) et [docs/EPICS.md](docs/EPICS.md) : produit et épopées.
- [docs/DESIGN.md](docs/DESIGN.md) : design system.
- [docs/specs/](docs/specs/), [docs/adr/](docs/adr/), [docs/ddr/](docs/ddr/) : implémentation.

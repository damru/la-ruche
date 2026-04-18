# Contribuer à La Ruche

Merci de l'intérêt pour La Ruche. Ce guide explique comment contribuer efficacement.

***

## Licence des contributions

La Ruche est distribuée sous **double licence** : AGPL-3.0 pour les usages non commerciaux, et une licence commerciale pour les syndics professionnels (voir [LICENSE](LICENSE) et [LICENSE-COMMERCIAL.md](LICENSE-COMMERCIAL.md)).

Pour permettre cette double licence, tout contributeur externe doit signer le **Contributor License Agreement (CLA)** avant qu'une Pull Request soit mergée. Le CLA vous permet de conserver vos droits sur vos contributions tout en accordant au mainteneur le droit de les redistribuer sous les deux licences.

**Signature :** Lors de votre première PR, un bot CLA Assistant vous demandera de signer en commentant la phrase indiquée. Vous ne signez qu'une seule fois, pour toutes vos contributions futures. Voir [CLA.md](CLA.md) pour le texte complet.

## Ce projet n'accepte pas les contributions qui :

* Introduiraient une dépendance sous licence incompatible avec l'AGPL-3.0
* Ajouteraient des mécanismes de télémétrie ou de tracking utilisateur
* Restreindraient l'accès au code source de quelque manière que ce soit
* Ajouteraient de la logique métier dans les handlers API (violation ADR-003)
* Utiliseraient `unwrap()` hors des tests

***

## Démarrage rapide

```Shell
git clone https://github.com/damru/la-ruche
cd laruche
make install-tools
make setup        # copie .env, démarre postgres+stockage S3, migrations
make dev          # backend hot-reload
make dev-frontend # frontend Vite (autre terminal)
```

***

## Structure du projet

```
laruche/
├── Makefile               ← cibles make (dev, test, docker…)
├── Dockerfile             ← image unifiée (Node + Rust + Caddy sur Debian slim)
├── docker-compose.yml     ← stack complète
├── backend/               ← Rust (Axum + SQLx)
│   ├── src/
│   ├── migrations/
│   └── Cargo.toml
└── frontend/              ← Vue 3 + Vite + TypeScript
    ├── src/
    └── package.json
```

Voir [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) pour l'architecture technique et la description des modules.

**Règles de modules Rust — violations = refus de PR :**

```
domain/   → aucune dépendance vers db/, api/, auth/, storage/
db/       → dépend de domain/ uniquement
auth/     → dépend de db/ et domain/
storage/  → aucune dépendance interne
api/      → point d'assemblage, dépend de tout
jobs/     → dépend de db/ et domain/
```

***

## Workflow de contribution

1. **Fork** le dépôt
2. **Crée une branche** : `git checkout -b feat/ma-fonctionnalite`
3. **Code** en respectant les principes ci-dessous
4. **Tests** : `make test-all` doit passer
5. **Lint** : `make lint` sans erreurs
6. **Commit** avec un message conventionnel (voir ci-dessous)
7. **Pull Request** vers `main`

***

## Messages de commit

Ce projet suit la spécification **[Conventional Commits](https://www.conventionalcommits.org/)**.
Le format est **obligatoire** à minima pour les Pull Requests.

```
<type>(<scope>): <description courte>

[corps optionnel]

[pied de page optionnel — BREAKING CHANGE: ..., Closes #...]
```

| Type         | Usage                                           |
| ------------ | ----------------------------------------------- |
| `feat`       | Nouvelle fonctionnalité                         |
| `fix`        | Correction de bug                               |
| `refactor`   | Refactoring sans changement de comportement     |
| `perf`       | Amélioration de performance                     |
| `test`       | Ajout ou modification de tests                  |
| `docs`       | Documentation uniquement                        |
| `style`      | Formatage, virgules manquantes, etc.            |
| `chore`      | Build, dépendances, outillage                   |
| `ci`         | Workflows CI/CD                                 |
| `build`      | Système de build, Dockerfile, Makefile          |
| `revert`     | Annulation d'un commit précédent                |

**Breaking change :** ajouter `!` avant les deux-points ou un pied de page `BREAKING CHANGE:`.

Exemples :

```
feat(ag): calcul automatique résultats art. 26
fix(backend): guard levy_call mutation après statut issued
test(domain): cas limites vote unanimité
docs: mettre à jour ARCHITECTURE.md — table vendor_contract
feat!: supprimer l'endpoint PUT /levy-calls/:id (remplacé par POST /issue)
chore(ci): ajouter job commitlint sur pull_request
```

***

## Standards de code

### Rust (backend/)

* `cargo fmt` avant tout commit
* `cargo clippy -- -D warnings` sans erreurs
* Pas de `unwrap()` hors `#[cfg(test)]`
* `Decimal` pour tous les montants financiers, jamais `f64`
* Toutes les erreurs remontent via `AppError` et `?`

### Vue 3 / TypeScript (frontend/)

* `npm run typecheck` sans erreurs
* Composants : props typées, pas de `any`
* Traductions : toutes les chaînes dans `fr.json`, jamais en dur dans les templates
* Styles : uniquement les tokens CSS `var(--...)`, jamais de couleur en dur
* Design system atomique : atomes → molécules → organismes (voir [docs/DESIGN.md](docs/DESIGN.md))

### Tests

* Toute logique dans `domain/` doit avoir des tests unitaires
* Les handlers critiques (auth, votes, appels de fonds) doivent avoir des tests d'intégration
* Les tests d'intégration utilisent une DB de test dédiée, jamais la DB de dev

***

## Signaler un bug

Utilise les Issues GitHub avec le template fourni. Pour les failles de sécurité, **ne pas ouvrir une issue publique** — envoyer un email à bonjour@damienrubio.com.

***

## Questions légales

La Ruche ne fournit aucun conseil juridique. Le code implémente les règles légales telles que documentées dans le PRD — le syndic reste responsable de la conformité légale de sa copropriété.

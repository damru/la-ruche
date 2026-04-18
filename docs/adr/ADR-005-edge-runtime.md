# ADR-005 — Runtime Edge Debian + Caddy

## Contexte

L'image Docker unifiée doit servir les assets frontend et reverse proxyfier les routes backend Rust dans un seul conteneur. Le runtime `caddy:2` initial reposait sur Alpine, alors que le backend Rust était historiquement compilé dans un environnement Debian `glibc`.

Cette différence de libc provoquait un échec au démarrage du binaire dans l'image finale. Le besoin côté edge reste simple :

- servir une SPA Vue avec fallback `index.html`
- proxyfier `/api/*`, `/vendor/*`, `/swagger-ui/*`, `/api-doc/*` et `/notice`
- garder un binaire et une exploitation simples pour `docker run` comme pour Docker Compose

## Décision

Conserver Caddy comme runtime edge, mais l'installer dans une image finale `debian:trixie-slim`.

- le backend Rust est de nouveau compilé dans un environnement Debian `glibc`
- Caddy est installé via `apt`
- les assets frontend restent servis depuis `/usr/share/caddy`
- l'entrypoint démarre le backend, attend sa disponibilité, puis lance Caddy au premier plan

## Alternatives rejetées

| Option | Raison du rejet |
|---|---|
| Garder `caddy:2` (Alpine) | Incompatibilité de libc avec le binaire Rust glibc |
| Passer le backend en musl statique | Complexité de build OpenSSL plus élevée sans bénéfice produit immédiat |
| Basculer vers Nginx | Sort du besoin initial et augmente le bruit documentaire sans résoudre mieux le problème |

## Conséquences

- L'image unifiée conserve les ports `80` et `443`
- Le runtime edge reste Caddy, avec une configuration versionnée dans `docker/Caddyfile`
- La chaîne de build redevient homogène autour de Debian pour éviter les écarts de compatibilité libc
---
description: 'Conventions base de données : UUID v4, TIMESTAMPTZ, soft-delete unique par table, triggers updated_at, baseline migratoire unique.'
applyTo: 'backend/migrations/**,backend/src/**'
---

# Conventions PostgreSQL

- Clés primaires : UUID v4 générés côté application, type `uuid` dans PostgreSQL.
- Horodatage : `TIMESTAMPTZ` uniquement, jamais `TIMESTAMP` sans fuseau.
- Une table possède **un seul mécanisme de soft-delete** (`deleted_at` ou équivalent) ; on n'ajoute jamais un second drapeau redondant.
- Chaque table mutable porte un trigger `updated_at` ; l'application ne met pas à jour `updated_at` à la main.
- Baseline migratoire unique : `backend/migrations/0001_init.sql` (squash). Toute évolution de schéma = nouvelle migration incrémentale, jamais de modification de la baseline.
- Toute nouvelle table doit refléter ses invariants dans `docs/ARCHITECTURE.md` et respecter les ADR en vigueur (cf. ADR-006/007/012 selon le domaine).

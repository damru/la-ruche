# ADR-007 — Clé de répartition générale système (`key_type` enum + soft delete)

**Date :** 22 avril 2026  
**Statut :** Accepté  
**Contexte :** LEGAL.md §2.1, §2.2 — EP-001-S03, EP-001-S07, EP-001-S08

---

## Contexte

La clé de répartition "Charges Générales" est la clé fondatrice de toute copropriété. Elle est créée à l'onboarding et sert de base à l'obligation légale de répartition des charges (art. 10 loi 1965 + EDD). Elle ne peut pas être supprimée sans violer l'audit comptable.

Deux lacunes structurelles existaient :

1. **Aucun moyen d'identifier la clé générale** côté système — la protection était absente, n'importe quel manager pouvait la supprimer via `DELETE /building/keys/{id}`.
2. **Suppression physique des clés** (`DELETE FROM allocation_key`) — impossible sans violer l'immuabilité de l'historique des appels de fonds (qui référencent `key_id` via snapshot).

## Décision

### §1 — `key_type` enum sur `allocation_key`

Ajout d'une colonne `key_type allocation_key_type` (enum PG : `'general' | 'special'`) sur `allocation_key`.

- Une seule clé `general` par copropriété (index unique partiel).
- Créée automatiquement à l'onboarding avec `key_type = 'general'`.
- Le backend rejette toute tentative de suppression d'une clé `general` avec `409 GENERAL_KEY_PROTECTED`.
- Le frontend masque le bouton "Supprimer" pour les clés `general`.

### §2 — Soft delete des clés spéciales (`deleted_at`)

Ajout d'une colonne `deleted_at TIMESTAMPTZ` sur `allocation_key`.

- `list()` filtre `deleted_at IS NULL`.
- `delete()` effectue `UPDATE SET deleted_at = NOW()` au lieu d'un `DELETE`.
- Les shares actifs de la clé sont fermés (`valid_to = CURRENT_DATE`) avant le soft-delete pour respecter l'invariant ADR-006.
- Les données historiques (snapshots dans `levy_call.key_id`) restent intactes.

### §3 — Tantièmes généraux à la création d'un lot

Le formulaire de création/édition d'un lot dans `ResidentsView` n'expose plus qu'un seul champ "Tantièmes généraux", assigné automatiquement à la clé `general`. Les clés spéciales sont gérées exclusivement dans `AllocationKeysView`.

## Alternatives rejetées

| Option | Raison du rejet |
|---|---|
| Convention sur le nom ("Charges Générales") | Le nom est modifiable — protection fragile |
| `is_system: bool` | Moins expressif ; ne permet pas d'étendre à d'autres types système à terme |
| Table `system_key` séparée | Sur-ingénierie pour 2 types ; joins supplémentaires inutiles |
| Hard delete avec contrainte FK | Casse l'audit comptable — `levy_call.key_id` référence des clés potentiellement supprimées |

## Conséquences

- `AllocationKeyRow` expose `key_type: String`.
- `AllocationKeyResponse` expose `key_type`.
- `db::allocation_key::list()` filtre `deleted_at IS NULL`.
- `db::allocation_key::delete()` soft-delete uniquement, garde `GENERAL_KEY_PROTECTED` si `key_type = 'general'`.
- `api/onboarding.rs::bootstrap()` crée la clé avec `key_type = 'general'` et nom "Charges Générales".
- `AllocationKey` en TypeScript a `key_type: 'general' | 'special'`.
- `ResidentsView` : un seul champ tantièmes à la création de lot.
- Migration 0003 : crée l'enum, ajoute les colonnes, migre l'existant.

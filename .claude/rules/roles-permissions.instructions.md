---
description: 'Rôles et permissions : manager/board_member/owner/tenant + jeton prestataire, 403 plutôt que 404, isolation building_id, garde v-if par rôle.'
applyTo: 'backend/src/**,frontend/src/**'
---

# Rôles & permissions

- Rôles applicatifs : `manager` (syndic), `board_member` (conseil syndical), `owner` (copropriétaire), `tenant` (locataire). Les prestataires accèdent via un **jeton dédié**, limité à leur intervention.
- Refus d'accès → **403**, pas 404. Ne pas masquer l'existence d'une ressource à un utilisateur authentifié qui a un droit de lecture nul mais une légitimité partielle ; en revanche l'isolation par `building_id` est stricte.
- Toute requête backend vérifie le rôle **et** l'appartenance au bâtiment (`building_id`), sans exception.
- Côté frontend, les éléments d'UI réservés sont gardés par `v-if` sur le rôle (composable/store d'auth) ; le masquage frontend ne remplace jamais la vérification backend.
- Toute nouvelle route backend doit définir son rôle minimum dans la couche d'auth et toute vue son niveau d'accès.

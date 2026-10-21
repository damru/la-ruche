---
description: 'Règle GED : tout fichier téléversé devient une entrée document avec liaison entité ; jamais de pièce jointe hors GED.'
applyTo: 'backend/src/**,frontend/src/**'
---

# Documents & GED

- Tout téléversement de fichier crée une ligne `document` (GED) dans PostgreSQL, puis une liaison `{entite}_document` vers l'entité concernée (lot, AG, contrat, tâche, etc.).
- Le stockage binaire vit dans RustFS (S3) ; PostgreSQL ne stocke que les métadonnées.
- Aucune pièce jointe n'est jamais stockée hors du circuit GED (pas de base64 en colonne, pas de fichier servi hors RustFS).
- Supprimer une entité ne supprime pas physiquement les documents : on délie/supprime logiquement la ligne `document` selon la convention soft-delete de la table.
- Les accès aux documents suivent les règles de rôles (cf. rule roles-permissions).

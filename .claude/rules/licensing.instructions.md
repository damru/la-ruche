---
description: 'Règles de licence : double licence AGPL-3.0 / commerciale, dépendances AGPL-compatibles, zéro télémétrie, CLA obligatoire.'
applyTo: 'Cargo.toml', 
---

# Licence & conformité

- Le projet est sous **double licence** : AGPL-3.0 pour l'usage non commercial, licence commerciale pour l'usage commercial (cf. `LICENSE`, `docs/LEGAL.md`).
- Toute nouvelle dépendance (Rust ou npm) doit être **AGPL-3.0 compatible** ; vérifier la licence avant d'ajouter au manifeste, et mettre à jour `NOTICE` si requise.
- **Zéro télémétrie** : aucun envoi de données d'usage, aucune dépendance de tracking, aucun appel réseau sortant non fonctionnel.
- Toute contribution externe passe par le CLA (`CLA.md`).
- Ne jamais introduire de code dont la licence imposerait des obligations incompatibles avec la double licence.

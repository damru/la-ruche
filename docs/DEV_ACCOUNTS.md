# Comptes de développement — La Ruche

> ⚠️ **Données de seed uniquement** — ne jamais utiliser en production.  
> Fichier source : `backend/seeds/seed_dev.sql`

**Mot de passe commun à tous les comptes : `Demo1234!`**

## Comptes utilisateurs

| Email | Nom | Prénom | Rôle(s) | Lots détenus |
|---|---|---|---|---|
| alice.martin@laruche.test | Martin | Alice | `manager` | — |
| bernard.dupont@laruche.test | Dupont | Bernard | `board_member`, `co_owner` | Lot 6 (A · 3e) |
| camille.rousseau@laruche.test | Rousseau | Camille | `board_member`, `co_owner` | Lot 7 (A · 3e) |
| david.laurent@laruche.test | Laurent | David | `co_owner` | Lot 1 (A · RDC) |
| emma.lefebvre@laruche.test | Lefebvre | Emma | `co_owner` | Lots 2 & 8 |
| francois.moreau@laruche.test | Moreau | François | `co_owner` | Lot 4 (A · 2e) |
| gisele.petit@laruche.test | Petit | Gisèle | `co_owner` | Lot 5 (A · 2e) |
| henri.chevalier@laruche.test | Chevalier | Henri | `co_owner` | Lots 3, 9, 10 |
| isabelle.fontaine@laruche.test | Fontaine | Isabelle | `tenant` | occupe Lot 2 |
| jules.martin@laruche.test | Martin | Jules | `tenant` | occupe Lot 3 |

## Lots (Résidence Les Cerisiers)

| N° lot | Type | Bloc · Étage | Propriétaire actuel |
|---|---|---|---|
| 1 | Appartement 3P | A · RDC | David Laurent |
| 2 | Appartement 2P | A · 1er | Emma Lefebvre *(locataire : Isabelle Fontaine)* |
| 3 | Appartement 2P | A · 1er | Henri Chevalier *(locataire : Jules Martin)* |
| 4 | Appartement 3P | A · 2e | François Moreau |
| 5 | Appartement 2P | A · 2e | Gisèle Petit |
| 6 | Appartement 4P | A · 3e | Bernard Dupont |
| 7 | Appartement 3P | A · 3e | Camille Rousseau |
| 8 | Appartement 4P duplex | A · 4e | Emma Lefebvre |
| 9 | Cave | S | Henri Chevalier |
| 10 | Parking | P | Henri Chevalier |

## Clés de répartition

| Nom | Type | Total tantièmes | Lots concernés |
|---|---|---|---|
| Charges Générales | `general` | 8 000 | Tous (1–10) |
| Ascenseur | `special` | 1 110 | Lots 2–8 |
| Eau Froide | `special` | 750 | Lots 1–9 |

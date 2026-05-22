# ADR-010 — PV du conseil syndical (délégation de pouvoirs)

**Date :** 2026-05-22
**Statut :** Accepté
**Références :** LEGAL.md §3.5 · Art. 21-1 décret 67-223 · LEGIARTI000042077775 · EP-002-S12

---

## Contexte

L'art. 21-1 du décret 67-223 impose que les décisions prises par le conseil syndical sous délégation de pouvoirs AG soient formalisées dans un PV signé par au moins deux membres, avec noms et sens des votes, et inscrites au registre des PV d'AG.

EP-002-S12 couvre la création, le stockage et la consultation de ces PV.

La question est : faut-il une table dédiée `council_meeting` ou réutiliser `general_meeting` ?

---

## Alternatives évaluées

| Option | Avantages | Inconvénients |
|---|---|---|
| **Nouvelle table `council_meeting`** | Schéma explicite, pas de couplage avec AG | Duplication de toute l'infrastructure `general_meeting` (documents, attendance, index) ; migration lourde |
| **Extension `meeting_type` ENUM + réutilisation `general_meeting`** | Zéro migration de données, réutilise GED existante, `minutes_document_id` déjà présent, consistance des index | `meeting_type` doit distinguer les contextes dans les requêtes AG classiques |

---

## Décision

**Extension de l'ENUM `meeting_type`** : ajout de la valeur `council` aux valeurs existantes `annual` / `extraordinary`.

Les PV du conseil syndical sont stockés comme des enregistrements `general_meeting` avec `meeting_type = 'council'`.

Modèle de données résultant :

- `general_meeting.meeting_type = 'council'` — identifie la séance CS
- `general_meeting.minutes_document_id` — référence le PV signé (FK → `document`)
- `general_meeting_document` — pièces jointes associées (via `meeting_document_kind`)
- `general_meeting_attendance` — signataires (au moins 2 membres) avec `attendance_mode = 'present'`

Toutes les requêtes qui listent les AG standard doivent filtrer `meeting_type IN ('annual', 'extraordinary')`. Les requêtes PV CS filtrent `meeting_type = 'council'`.

Migration nécessaire : `ALTER TYPE meeting_type ADD VALUE 'council'` (non destructive).

---

## Conséquences

- La migration est non destructive — `ADD VALUE` sur un ENUM PostgreSQL ne nécessite pas de réécriture de table.
- Les handlers AG existants (`assemblies.rs`) doivent exclure `meeting_type = 'council'` de leurs listings.
- Un handler dédié ou une route distincte (`/api/v1/council-meetings`) peut être ajouté dans `assemblies.rs` ou dans un fichier séparé.
- `general_meeting_attendance` sert de registre des signataires CS (minimum 2 membres requis par art. 21-1).
- Pas de table `vote` ni `resolution` pour les PV CS — ces champs restent `NULL`  ; la délibération est capturée dans le corps du PV (document signé).

## Alternatives rejetées

**Table dédiée `council_meeting`** — duplication de schéma non justifiée. Le modèle existant couvre tous les besoins documentaires d'EP-002-S12 avec une migration minimale.

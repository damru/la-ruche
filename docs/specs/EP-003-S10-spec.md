# Spécification technique — EP-003-S10

**Story :** Valider la réception du formulaire de vote par correspondance au plus tard J-3 francs avant l'AG (art. 9 bis décret 67-223) et rendre caduc le vote par correspondance d'un copropriétaire présent ou représenté en séance (art. 14-1 décret 67-223).

**Épic :** EP-003 — Assemblées générales
**Réf. LEGAL :** §3.1 · **Réf. PRD :** US-AG-* (vote par correspondance) · **Réf. ADR :** aucun (pas d'alternative structurelle — colonnes additives sur table existante)

**Statut actuel : NON IMPLÉMENTÉ.**

---

## 1. Quoi (résumé fonctionnel)

- Le syndic enregistre la **date de réception** du formulaire de vote par correspondance d'une unité.
- Un formulaire reçu **après J-3 francs** avant l'AG est refusé (le mode `correspondence` ne peut pas être enregistré).
- Si le copropriétaire est finalement **présent ou représenté** en séance, ses votes par correspondance deviennent **caducs** : exclus des décomptes et signalés comme tels.

## 2. État actuel du code

| Élément | Emplacement |
|---|---|
| Enum mode de présence incluant `correspondence` | `backend/migrations/0001_init.sql` L135 |
| Usage du mode dans le repo | `backend/src/db/assembly.rs` L113, L121 |
| Table `general_meeting_attendance` (sans `form_received_at` ni `cancelled_at`) | `0001_init.sql` L352-361 |
| `save_attendance` (aucune auto-annulation) | `backend/src/db/assembly.rs` L1033-1130 |
| `cast_vote` (upsert `(resolution_id, unit_id)`, aucun arbitrage correspondance/présence) | `backend/src/db/assembly.rs` L1266-1302 |
| Checklist convocation : document « formulaire de vote par correspondance » | `backend/src/domain/convocation_checklist.rs` L78-79 |
| Routes meetings | `backend/src/api/mod.rs` L113-174 |

## 3. Modèle de données

Migration `backend/migrations/NNNN_correspondence_vote_rules.sql` :

```sql
-- EP-003-S10: vote par correspondance — réception J-3 francs (art. 9 bis
-- décret 67-223) et caducité si présent/représenté (art. 14-1).
ALTER TABLE general_meeting_attendance
    ADD COLUMN form_received_at DATE,           -- réception du formulaire (mode correspondence)
    ADD COLUMN correspondence_cancelled_at TIMESTAMPTZ;  -- caducité (présent/représenté en séance)
```

- `form_received_at` obligatoire côté applicatif quand `mode = 'correspondence'` ; NULL pour les autres modes.
- `correspondence_cancelled_at` posé automatiquement quand une présence `present`/`represented` remplace/complète un enregistrement `correspondence` — trace probante, jamais supprimé.
- Nouvelle colonne sur table existante → pas de mise à jour `docs/ARCHITECTURE.md` obligatoire si la table y est déjà décrite, sinon compléter la description.

Après migration : `cargo sqlx migrate run` puis `cargo sqlx prepare` (commit `backend/.sqlx/`).

## 4. Backend

### 4.1 Domaine — `backend/src/domain/correspondence.rs` (nouveau)

Calcul des **jours francs** (définition précise, réutilisable et testable pure) :

```rust
/// Art. 9 bis décret 67-223: the form must be received at least 3 "jours
/// francs" before the meeting. A "jour franc" is a full calendar day: neither
/// the reception day nor the meeting day counts. Latest valid reception date
/// is therefore meeting_date - 4 days (leaving days J-3, J-2, J-1 free).
pub(crate) fn latest_valid_reception(meeting_date: NaiveDate) -> NaiveDate {
    meeting_date - chrono::Duration::days(4)
}

pub(crate) fn is_form_received_in_time(received: NaiveDate, meeting_date: NaiveDate) -> bool {
    received <= latest_valid_reception(meeting_date)
}
```

> Décision explicitée dans la spec : « J-3 francs » = 3 jours pleins entre réception et AG, jour de réception et jour d'AG exclus. Exemple : AG le 20 → dernière réception valide le 16.

### 4.2 DB — `backend/src/db/assembly.rs`

- `save_attendance` :
  - payload étendu avec `form_received_at: Option<NaiveDate>` ;
  - si `mode == "correspondence"` : `form_received_at` requis (`errors.meetings.correspondence_form_date_required`) et validé via `is_form_received_in_time` contre `scheduled_at::date` du meeting (`errors.meetings.correspondence_form_late`) ;
  - si `mode` passe à `present`/`represented` pour une unité ayant un enregistrement `correspondence` : poser `correspondence_cancelled_at = NOW()` (caducité art. 14-1) au lieu d'écraser silencieusement.
- Décomptes de votes (tallies) : exclure les votes d'une unité dont la présence porte `correspondence_cancelled_at IS NOT NULL` **et** dont le vote a été enregistré au titre de la correspondance. Approche minimale : les votes caducs sont exclus par jointure sur `general_meeting_attendance` au moment du calcul des résultats — pas de suppression de lignes `vote` (trace conservée).
- `cast_vote` : refuser un vote « correspondance » pour une unité déjà `present`/`represented` (`errors.meetings.correspondence_superseded`).

### 4.3 API

- Pas de nouvelle route : `save_attendance` et `cast_vote` existants transportent les nouveaux champs/erreurs.
- Handlers inchangés dans leur rôle (zéro logique — validation dans domain/db).
- Erreurs i18n : `errors.meetings.correspondence_form_date_required`, `errors.meetings.correspondence_form_late`, `errors.meetings.correspondence_superseded`.

## 5. Frontend

- `frontend/src/api/index.ts` : étendre le payload d'attendance avec `form_received_at?: string`.
- Vue feuille de présence (features/assemblies) : champ date « Formulaire reçu le » affiché uniquement quand le mode `correspondence` est sélectionné ; message d'erreur i18n si hors délai.
- Affichage d'un badge « Vote par correspondance caduc (présent en séance) » sur les lignes concernées (attendance avec `correspondence_cancelled_at`).
- Aucun nouveau composant C* attendu ; si un badge dédié s'avère nécessaire, le cataloguer dans `docs/DESIGN.md §3` avant usage (étape, ne pas éditer DESIGN.md dans cette spec).

## 6. Cas limites

| Cas | Règle |
|---|---|
| Formulaire reçu exactement à la dernière date valide (J-4 calendaire) | Accepté (3 jours francs pleins restants). |
| Formulaire reçu J-3 calendaire ou après | Refusé `errors.meetings.correspondence_form_late`. |
| Unité passe `correspondence` → `present` puis re-`correspondence` | La caducité posée n'est pas effacée ; le retour en `correspondence` exige un nouveau `form_received_at` valide et repart d'une ligne « propre » (nouvelle validation), la trace `correspondence_cancelled_at` de l'épisode précédent est conservée en historique applicatif ou écrasée — décision : écrasée (une seule ligne par unité, la table a une ligne par (meeting, unit)) mais le flag est remis à NULL uniquement via ce chemin explicite. |
| Meeting sans `scheduled_at` | Validation impossible → refuser l'enregistrement `correspondence` (`errors.meetings.correspondence_form_late` ou erreur dédiée) — cas normalement impossible après convocation. |
| Vote déjà enregistré par correspondance puis présence en séance | Vote exclu des décomptes, ligne conservée, badge frontend. |

## 7. Critères d'acceptation

1. Enregistrer un mode `correspondence` sans date de réception → 400 `errors.meetings.correspondence_form_date_required`.
2. Date de réception hors délai (moins de 3 jours francs) → 400 `errors.meetings.correspondence_form_late`.
3. Date de réception valide → attendance enregistrée avec `form_received_at`.
4. Passage à `present`/`represented` d'une unité en `correspondence` → `correspondence_cancelled_at` posé, votes par correspondance exclus des résultats.
5. Les votes caducs restent en base (trace probante) et sont identifiables.

## 8. Plan de tests

- **Unitaires domaine** (`correspondence.rs`) : `latest_valid_reception` et `is_form_received_in_time` — bornes exactes (J-4 accepté, J-3 refusé, jour même refusé).
- **E2E** (`backend/tests/api_e2e_test.rs`) :
  1. attendance `correspondence` sans date → 400 ;
  2. date tardive → 400 ;
  3. date valide → 200 ;
  4. vote par correspondance puis attendance `present` → résultat de la résolution exclut ce vote, attendance porte la caducité ;
  5. re-vote en séance après caducité → compté normalement.

## 9. Ordre d'implémentation

1. Migration `NNNN` (colonnes) → `cargo sqlx migrate run` + `cargo sqlx prepare`.
2. `domain/correspondence.rs` + tests unitaires jours francs.
3. `db/assembly.rs` : `save_attendance` (validation + caducité), exclusion dans les tallies, guard `cast_vote`.
4. Tests e2e, boucler jusqu'au vert (`cargo test --lib --tests`, `cargo clippy -- -D warnings`).
5. Frontend : payload api, champ date, badge caducité.
6. `docs/PROGRESS.md` (entrée datée).

# Spécification technique — EP-002-S22

**Story :** Rubrique « frais de délivrance de l'état daté » (partie 1, rubrique 7) plafonnée à **380 € TTC** à la charge du cédant + facturation — art. 10-1 b) loi 1965, décret n°2020-153 du 21/02/2020 (en vigueur 01/06/2020) (LEGAL.md §7.1)

**Épic :** EP-002 — Copropriétaires & lots
**Réf. LEGAL :** §7.1 — art. 10-1 b) loi 65-557 (honoraires et frais d'établissement de l'état daté, imputables au seul copropriétaire concerné, plafonnés par décret) + décret n°2020-153 (montant fixé à **380 € TTC**, quel que soit le nombre de lots de la même mutation ; tout dépassement est illégal — confirmé par le Conseil d'État, déc. n°441005) · mutation à titre **onéreux** uniquement · **ADR :** aucun requis (champ de config + ligne de document)
**Stories liées :** EP-002-S06 ⚠️ (état daté, rubrique 7 rendue à vide), EP-004 (appels de fonds / écritures), EP-001 (fiche identité bâtiment, config)

**Statut actuel :** NON IMPLÉMENTÉ — la rubrique 7 « Frais de délivrance de l'état daté » est rendue avec un montant vide (`None`) dans `domain/legal_documents.rs` ; aucun plafond ni facturation.

---

## 1. Quoi (résumé fonctionnel)

L'état daté est payant, à la charge du **cédant** (art. 10-1 b loi 1965), dans la limite d'un plafond réglementaire de **380 € TTC** par mutation à titre onéreux (décret n°2020-153, quel que soit le nombre de lots concernés).

- La **rubrique 7 de la partie 1** de l'état daté affiche le montant réel facturé, **clampé à 380 € TTC** (`min(montant configuré, 380)`) ;
- le montant est configurable par copropriété (paramètre bâtiment « frais d'état daté TTC », défaut 380) ;
- une **facture** est émise au cédant pour ce montant (mutation à titre onéreux uniquement ; mutation à titre gratuit → rubrique 7 à 0 € avec mention « non applicable — mutation à titre gratuit »).

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| Rubrique 7 rendue à vide | `backend/src/domain/legal_documents.rs` (`ligne("7- Frais de délivrance de l'état daté", None)`) |
| Assemblage partie 1 | `backend/src/domain/etat_date.rs` (`assemble`, `LigneMontant`) |
| Config bâtiment | `backend/src/api/building.rs`, table `building` (EP-001) |
| Écritures / appels de fonds | EP-004 : `backend/src/api/budget.rs`, repos levy |
| Plafond 380 € TTC + facture | **inexistant** |

## 3. Modèle de données

Migration `backend/migrations/NNNN_etat_date_fee.sql` :

```sql
-- EP-002-S22: état daté fee config, capped at 380 EUR TTC (art. 10-1 b loi 1965, décret 2020-153)
ALTER TABLE building ADD COLUMN etat_date_fee_ttc NUMERIC(10,2) NOT NULL DEFAULT 380.00
    CHECK (etat_date_fee_ttc >= 0 AND etat_date_fee_ttc <= 380.00);
```

Le clamp est aussi appliqué en domain (défense en profondeur) : `fee = min(config, 380 €)`.

## 4. Backend

### 4.1 Domain (`backend/src/domain/etat_date.rs`, extension)

- `pub(crate) const ETAT_DATE_FEE_CAP_TTC: Decimal = Decimal::from(380)` — constante nommée (plafond légal).
- `etat_date_fee(building_fee_ttc: Decimal, mutation_onereuse: bool) -> Decimal` — `0` si mutation gratuite, sinon `min(building_fee_ttc, CAP)`.
- `assemble` reçoit le fee et rend la rubrique 7 avec le montant (jamais `None` quand onéreux).

### 4.2 DB

- `db/building.rs` : lecture/écriture de `etat_date_fee_ttc` (PATCH config bâtiment, syndic uniquement).
- Facture : réutiliser le mécanisme EP-004 (écriture « autres sommes exigibles » / appel de fonds exceptionnel au cédant) — pas de nouveau repo comptable.

### 4.3 API

- `PATCH /buildings/{id}` (ou endpoint config existant) — accepte `etat_date_fee_ttc`, validé `0 ≤ x ≤ 380` → sinon `AppError::validation_code("errors.etat_date.fee_above_cap")`.
- `POST /etat-date-requests/{id}/generate` — intègre le fee dans le snapshot (rubrique 7) et déclenche l'écriture de facturation au cédant (mutation onéreuse).
- Le PDF d'état daté affiche la rubrique 7 remplie.

## 5. Frontend

- Champ « Frais d'état daté TTC (plafond légal 380 €) » dans les paramètres de la copropriété (feature `building`), avec hint du plafond légal.
- Sur la fiche de la demande d'état daté : montant facturé affiché, mention « à la charge du cédant ».
- Composants catalogue existants ; nouveau `C*` → `docs/DESIGN.md §3` avant usage.

## 6. Cas limites

- Config > 380 € → clamp 380 (DB + domain), jamais de dépassement.
- Mutation à titre gratuit → rubrique 7 = 0 € « non applicable ».
- Fee = 0 → rubrique 7 à 0 € (état daté gratuit), facture de 0 € non émise.
- Plusieurs lots, une même mutation → un seul fee (380 € max pour l'ensemble), pas un fee par lot.

## 7. Critères d'acceptation

1. La rubrique 7 de l'état daté (PDF + snapshot JSON) affiche le montant clampé à 380 € TTC.
2. Le paramètre bâtiment est borné à [0, 380] (DB CHECK + validation API i18n).
3. Une facture/écriture est émise au cédant à la génération (mutation onéreuse), pas pour une mutation gratuite.
4. `docs/ARCHITECTURE.md` documente la colonne.

## 8. Plan de tests

- **Domaine pur** : `etat_date_fee` — 380, 500 → 380, 0, mutation gratuite → 0.
- **E2E** : PATCH fee à 500 → 422 ; génération état daté → snapshot rubrique 7 = min(fee, 380) ; mutation gratuite → 0.

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` → migration → `cargo sqlx migrate run` → `cargo sqlx prepare`.
2. Domain (constante + clamp + rubrique 7) + tests purs.
3. API config bâtiment + génération (snapshot + écriture facture) + i18n.
4. Frontend (champ config + affichage demande).
5. Tests E2E, entrée `docs/PROGRESS.md`.

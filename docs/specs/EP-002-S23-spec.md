# Spécification technique — EP-002-S23

**Story :** Alimenter la **dette fournisseurs** du pré-état daté (art. L721-2 II CCH) depuis les contrats/factures fournisseurs (EP-006) et l'état des impayés du syndicat (LEGAL.md §7.2)

**Épic :** EP-002 — Copropriétaires & lots
**Réf. LEGAL :** §7.2 — art. L721-2 II CCH (le pré-état daté comporte l'état des **dettes envers les prestataires/fournisseurs** du syndicat) · §5.3 (contrats fournisseurs) · **ADR :** requis uniquement si l'on passe à une vraie comptabilité fournisseurs — MVP = solde impayé saisi par contrat (voir §3).
**Stories liées :** EP-002-S07 ⚠️ (pré-état daté, `dette_fournisseurs` actuellement `None`), EP-006 ✅ (contrats fournisseurs), EP-004 (comptabilité)

**Statut actuel :** NON IMPLÉMENTÉ — `dette_fournisseurs: Option<Decimal>` vaut toujours `None` dans `domain/etat_date.rs` (rendu « non disponible » dans le pré-état daté).

---

## 1. Quoi (résumé fonctionnel)

L'art. L721-2 II CCH exige que le pré-état daté comporte les **dettes du syndicat envers les prestataires** (fournisseurs). Le produit ne dispose pas d'une comptabilité fournisseurs (AP) : la dette est suivée comme **solde impayé par contrat fournisseur**.

- Chaque contrat fournisseur (EP-006) porte un champ « **montant impayé échu** » (saisi par le syndic, `Decimal`, défaut 0) ;
- le pré-état daté affiche la **dette fournisseurs totale** = somme des impayés échus des contrats actifs du bâtiment, et le détail par contrat (prestataire, montant) ;
- si aucun contrat n'a d'impayé saisi → dette = 0 € (et non plus « non disponible »).

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| `dette_fournisseurs: Option<Decimal> = None` | `backend/src/domain/etat_date.rs` (`assemble_pre_etat_date`) |
| Contrats fournisseurs | EP-006 : `backend/src/api/contracts.rs`, `contractors.rs`, table `contract` |
| Rendu « non disponible » | `backend/src/domain/typst/pre_etat_date.typ` |
| Dette fournisseurs alimentée | **inexistant** |

## 3. Modèle de données

Migration `backend/migrations/NNNN_contract_outstanding_balance.sql` :

```sql
-- EP-002-S23: solde impayé échu par contrat fournisseur (dette fournisseurs du pré-état daté, art. L721-2 II CCH)
ALTER TABLE contract ADD COLUMN outstanding_balance NUMERIC(12,2) NOT NULL DEFAULT 0
    CHECK (outstanding_balance >= 0);
```

Pas de table de factures fournisseurs en MVP (pas de comptabilité AP) — évolution possible plus tard sans casser le calcul (la dette resterait la somme des impayés). **ADR** à rédiger uniquement si l'on passe à une vraie comptabilité fournisseurs (alternatives : table `supplier_invoice` vs champ sur `contract`).

## 4. Backend

### 4.1 Domain (`backend/src/domain/etat_date.rs`, extension)

- `dette_fournisseurs_total(contrats: &[ContractSummary]) -> Decimal` — somme des `outstanding_balance` des contrats **actifs** du bâtiment. Fonction pure, `Decimal`.
- `assemble_pre_etat_date` : `dette_fournisseurs = Some(total)` + `detail_dettes_fournisseurs: Vec<(prestataire, montant)>` pour le rendu. Le `None` « non disponible » disparaît.

### 4.2 DB

- `db/contracts.rs` : lecture des contrats actifs du bâtiment avec `outstanding_balance` ; écriture du champ via l'API contrat existante (PATCH, syndic uniquement).

### 4.3 API

- `PATCH /contracts/{id}` — accepte `outstanding_balance` (≥ 0) → `AppError::validation_code("errors.contracts.invalid_outstanding")`.
- `GET /units/{id}/pre-etat-date` — retourne `dette_fournisseurs: Decimal` + détail par prestataire ; le PDF (`pre_etat_date_pdf`) affiche la rubrique avec le total et le détail.

## 5. Frontend

- Champ « Montant impayé échu (€) » sur la fiche contrat fournisseur (EP-006), saisi par le syndic.
- Pré-état daté (fiche lot) : rubrique « dettes fournisseurs » avec total + liste des prestataires concernés.
- Composants catalogue existants ; nouveau `C*` → `docs/DESIGN.md §3` avant usage.

## 6. Cas limites

- Aucun contrat → dette = 0 € (plus de « non disponible »).
- Contrats résiliés/archivés → exclus du total.
- Impayé saisi sur un contrat sans échéance → compté quand même (saisie manuelle = responsabilité du syndic).
- Total nul → rubrique affichée « 0 € », jamais omise.

## 7. Critères d'acceptation

1. `PATCH /contracts/{id}` persiste `outstanding_balance` (≥ 0, i18n sinon).
2. Le pré-état daté (JSON + PDF) affiche la dette fournisseurs = somme des impayés des contrats actifs, avec détail par prestataire.
3. `dette_fournisseurs` n'est plus jamais `None` dans le pré-état daté.
4. `docs/ARCHITECTURE.md` documente la colonne.

## 8. Plan de tests

- **Domaine pur** : `dette_fournisseurs_total` — contrats avec/sans impayé, contrats résiliés exclus, liste vide → 0.
- **E2E** : PATCH impayé → pré-état daté reflète le total ; contrat résilié → exclu ; valeur négative → 422.

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` → migration → `cargo sqlx migrate run` → `cargo sqlx prepare`.
2. Domain (somme + spec `assemble_pre_etat_date`) + tests purs.
3. API contrat (PATCH) + pré-état daté (JSON/PDF) + i18n.
4. Frontend (champ contrat + rubrique pré-état daté).
5. Tests E2E, entrée `docs/PROGRESS.md`.

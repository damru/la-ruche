# Spécification technique — EP-002-S07

**Story :** Générer le pré-état daté pour signature du compromis de vente (synthèse financière de la copropriété) (LEGAL.md §7.2)

**Épic :** EP-002 — Copropriétaires & lots
**Réf. LEGAL :** §7.2, art. L721-2 CCH · **ADR :** aucun requis
**Dépendance forte :** **EP-002-S06** (état daté) — le pré-état daté réutilise le même assemblage de données (module `domain/etat_date.rs`). Implémenter S06 d'abord ; cette spec ne couvre que le delta.

**Statut initial :** NON IMPLÉMENTÉ — rien n'existe.

---

## 1. Quoi (résumé fonctionnel)

Avant la signature du compromis, le vendeur (ou son notaire) doit fournir à l'acquéreur les informations financières de l'art. L721-2 CCH : montants des charges courantes du budget prévisionnel et hors budget payés par le cédant sur les **2 derniers exercices**, sommes dues au syndicat, état global des impayés et de la dette fournisseurs, montant du **fonds travaux** (quote-part du lot et dernière cotisation).

Contrairement à l'état daté (S06) :
- document **plus léger**, purement informatif, sans structure 3 parties ;
- **pas de délai légal de 15 jours** ni de suivi d'échéance ;
- généré **à la demande, sans workflow** : pas de statut, pas de table de suivi — génération directe avec snapshot optionnel non requis (le document est reproductible à tout moment).

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| Sources financières (charges, appels, fonds travaux) | identiques à S06 : données EP-004 ✅, EP-004-S03 ✅ |
| Procédures judiciaires (mention synthétique) | `backend/migrations/0034_judicial_proceedings.sql`, `backend/src/api/judicial_proceedings.rs` |
| Propriété du lot | `backend/src/api/units.rs:279-350` (EP-002-S17 ✅) |
| Assemblage partagé | `backend/src/domain/etat_date.rs` — **créé par S06** ; le pré-état daté en consomme les mêmes structures d'entrée |
| Pré-état daté | **inexistant** |

## 3. Modèle de données

**Aucune migration.** Le pré-état daté est un document dérivé, régénérable à volonté à partir des données vivantes ; il n'a pas de valeur probante figée (contrairement au snapshot S06) ni de suivi de demande. Pas de nouvelle table → pas de mise à jour `docs/ARCHITECTURE.md`, pas d'ADR.

*(Si le besoin d'archiver le document émis apparaît plus tard, il passera par le module Documents EP-005 — hors scope ici.)*

## 4. Backend

### 4.1 Domain (`backend/src/domain/etat_date.rs`, extension)

Ajouter une fonction pure `assemble_pre_etat_date(inputs) -> PreEtatDate` consommant les **mêmes structures d'entrée** que `assemble` (S06) :

```rust
pub(crate) struct PreEtatDate {
    pub charges_courantes_n1: Decimal,      // exercice N-1
    pub charges_courantes_n2: Decimal,      // exercice N-2
    pub charges_hors_budget_n1: Decimal,
    pub charges_hors_budget_n2: Decimal,
    pub sommes_dues_par_cedant: Decimal,
    pub impayes_globaux_syndicat: Decimal,
    pub dette_fournisseurs: Option<Decimal>, // None => rendu « non disponible »
    pub fonds_travaux_quote_part: Decimal,
    pub fonds_travaux_derniere_cotisation: Decimal,
    pub procedures_en_cours: usize,
}
```

`Decimal` partout ; zéro SQL en domain ; zéro logique en handler.

### 4.2 DB

Aucun nouveau repo : réutiliser les fonctions de chargement introduites par S06 (`db/etat_date.rs` ou repos levy/works-fund existants). Si S06 a factorisé un chargeur `load_etat_date_inputs(unit_id)`, ce même chargeur alimente le pré-état daté.

### 4.3 API (`backend/src/api/etat_date.rs`, extension ; route dans `backend/src/api/mod.rs`)

- `GET /units/{id}/pre-etat-date` — manager (et le copropriétaire du lot en lecture, s'il est vendeur : à confirmer, par défaut manager uniquement). Charge les inputs, appelle `assemble_pre_etat_date`, retourne le JSON.
- Lot inexistant/archivé → 404 ; données financières partiellement absentes → document avec zéros / « non disponible », jamais 500. Erreurs i18n `AppError::validation_code("errors.pre_etat_date.*")` si validation nécessaire.

## 5. Frontend

- `frontend/src/api/index.ts` : `etatDateApi.preEtatDate(unitId)`.
- `frontend/src/types/domain.ts` : `PreEtatDate`.
- UI : action « Pré-état daté » sur la fiche lot (feature `units`), ouvrant une **vue imprimable** synthétique (tableau des rubriques L721-2, en-tête copropriété + lot + date d'édition). Réutiliser la mise en page imprimable introduite par S06.
- Composants catalogue existants ; **tout nouveau `C*` doit être ajouté à `docs/DESIGN.md §3` avant usage** (étape dédiée).

## 6. Cas limites

- Moins de 2 exercices clos → rubriques N-2 rendues « non disponible » avec mention explicite.
- Dette fournisseurs non modélisée finement → `None` → « non disponible » (limitation affichée, pas d'invention de chiffre).
- Emprunts collectifs : non gérés (même limitation actée que S06) → mention « néant — non gérés ».
- Lot sans aucun appel émis → toutes les rubriques à zéro, document quand même généré.
- Le document affiche sa **date d'édition** : deux générations à des dates différentes peuvent différer, c'est attendu (pas de snapshot).

## 7. Critères d'acceptation

1. `GET /units/{id}/pre-etat-date` retourne la synthèse L721-2 : charges 2 exercices, sommes dues, impayés globaux, fonds travaux (quote-part + dernière cotisation), nombre de procédures en cours.
2. Montants en `Decimal` ; rubriques indisponibles explicitement marquées, jamais omises silencieusement.
3. Vue imprimable accessible depuis la fiche lot.
4. Aucune table ni migration ajoutée ; aucun impact sur le workflow S06.

## 8. Plan de tests

- **Domaine pur** : `assemble_pre_etat_date` — lot avec historique complet ; 1 seul exercice ; lot vierge (zéros) ; `dette_fournisseurs = None`.
- **E2E** (`backend/tests/api_e2e_test.rs`, `common::isolated_test_server()`) :
  - lot avec appels sur 2 exercices → rubriques correctes ;
  - lot sans données → document à zéros, 200 ;
  - lot inexistant → 404 ; rôle non autorisé → 403.

## 9. Ordre d'implémentation

1. **Pré-requis : EP-002-S06 livrée** (module `domain/etat_date.rs` + chargeur d'inputs).
2. `assemble_pre_etat_date` en domain + tests purs.
3. Handler + route `api/mod.rs` (+ `cargo sqlx prepare` si nouvelles requêtes).
4. Frontend (type, api client, vue imprimable) — cataloguer les `C*` nouveaux d'abord.
5. Tests E2E, entrée `docs/PROGRESS.md`.

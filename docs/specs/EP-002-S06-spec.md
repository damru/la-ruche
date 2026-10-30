# Spécification technique — EP-002-S06

**Story :** Générer l'état daté d'un lot à la demande du notaire ou du cédant (charges courantes, arriérés, fonds travaux, travaux votés non réalisés, procédures en cours), à remettre avant l'acte authentique (LEGAL.md §7.1)

**Épic :** EP-002 — Copropriétaires & lots
**Réf. LEGAL :** §7.1 — art. 5 + 5-1 décret 67-223 v7.0 (25/12/2025) : structure en **3 parties obligatoires** + annexe quote-part 2 exercices, adressée au notaire **avant l'établissement de l'acte** (aucun délai légal chiffré — les 15 jours sont ceux de l'art. 20-I : avis de mutation et opposition) · art. 20 loi 1965 + art. 20-II ALUR (certificat art. 20, validité 1 mois) · art. 6 décret 1967 (notification notaire) · art. 10-1 b) loi 1965 + décret n°2020-153 : frais d'état daté plafonnés **380 € TTC** à la charge du cédant · art. 33 loi 1965 · art. 45-1 décret 1967 (avances remboursables, modalités solution 1/2) · **ADR :** aucun requis a priori (assemblage en lecture + table de suivi simple)
**Stories liées :** EP-004 ✅ (appels/charges), EP-004-S03 ✅ (fonds travaux), EP-002-S10/registre des procédures ✅, EP-002-S17 ✅ (mutation de lot), EP-002-S07 (pré-état daté, réutilise l'assemblage)

**Statut actuel :** ⚠️ IMPLÉMENTÉ (backend + frontend) avec limitations actées — génération PDF serveur (template EP-009-S09) : lettre + état daté 3 parties (`etat_date.typ`) + formulaire pré-rempli en annexe (`questionnaire.typ`, à refondre au format La Ruche via EP-002-S24, dont certificat art. 20) ; limitations : travaux votés non réalisés non alimentés (suivi t_5adbc63d), emprunts collectifs rendus « néant — non gérés », rubrique 7 frais de délivrance non facturée (EP-002-S22). Améliorations de format actées : voir §1 (références art. 20, art. 10-1 b + décret 2020-153, art. 33, art. 45-1, annexe art. 5 3°).

---

## 1. Quoi (résumé fonctionnel)

Lors d'une vente, le notaire **ou le copropriétaire cédant** demande au syndic un **état daté** du lot cédé (art. 5 décret 67-223 : adressé au notaire avant l'établissement de l'acte — aucun délai légal chiffré ; l'Application garde une échéance interne conservatoire J+15). Le document suit la structure réglementaire :

- **Partie 1** — sommes dues par le **cédant** au syndicat : provisions exigibles (budget + hors budget), charges impayées des exercices antérieurs, sommes devenues exigibles du fait de la vente (art. 33 loi 1965), avances exigibles (réserve, provisions spéciales, emprunt syndicat art. 45-1 al. 4), cotisations annuelles au fonds de travaux, autres sommes exigibles (quote-part de prêt, condamnations), **frais de délivrance de l'état daté plafonnés à 380 € TTC** (art. 10-1 b loi 1965, décret n°2020-153) ; et à des tiers (emprunts de copropriétaires gérés par le syndic).
- **Partie 2** — sommes dont le **syndicat est débiteur** envers le cédant : avances perçues (art. 45-1, remboursables — **modalités de remboursement à cocher** : solution 1 l'acquéreur rembourse directement le vendeur / solution 2 l'acquéreur verse au syndic), provisions rendues exigibles par déchéance du terme (art. 19-2), solde créditeur de l'exercice antérieur.
- **Partie 3** — sommes incombant à l'**acquéreur** : reconstitution des avances, provisions non encore exigibles (budget + hors budget, avec dates d'exigibilité).
- **Annexe à la 3ᵉ partie** (art. 5, 3° décret 1967) — quote-part du lot dans le budget prévisionnel et les dépenses hors budget sur les **2 exercices précédents** (appelée vs réelle), état des **procédures en cours** (registre EP-002-S10, art. L721-2 2° d), existence du fonds de travaux (part rattachée au lot principal + dernière cotisation), travaux votés non réalisés, événements postérieurs à la délivrance du questionnaire avant-contrat (nouvelle AG, nouveaux travaux votés).
- **Certificat de l'article 20** (art. 20 loi 1965 + 20-II ALUR, joint à l'état daté, validité **1 mois**) : si le cédant est à jour, certificat « libre de toute obligation » (le notaire est dispensé d'avis de mutation et adresse la notification art. 6 décret 1967) ; sinon mention du solde débiteur + avis de mutation (15 jours) et opposition du syndic (15 jours, art. 5-1 décret 1967). Génération automatique : cf. **EP-002-S20**.
- **Formulaire état daté pré-rempli** (contenu du modèle CSN `backend/assets/QUESTIONNAIRE-SYNDIC-ETAT-DATE.pdf` : informations des parties, 3 parties, annexe actualisation, événements postérieurs, sommes exigibles pour le certificat art. 20, renseignements complémentaires A1–A12 et B1–B10) — rendu actuellement par le clone CSN `questionnaire.typ`, **à refondre au format La Ruche (layout v7.1) via EP-002-S24**.

La story couvre : (a) le **suivi de la demande** (date de réception, échéance interne J+15, statut), (b) la **génération à la demande, la plus automatisée possible**, d'un état daté structuré (JSON + PDF serveur via template partagé EP-009-S09) : lettre de transmission + état daté 3 parties + questionnaire CSN pré-rempli en annexe.

**Limitation actée :** les emprunts collectifs ne sont pas modélisés dans l'application → la rubrique correspondante est rendue avec la mention explicite « néant — emprunts collectifs non gérés », section vide mais présente.

**Limitation actée (2) :** le schéma ne relie pas les résolutions d'AG (`resolution`, `vote_type` impliquant des travaux) à un suivi d'achèvement des travaux (`task` n'a pas de colonne `resolution_id` — aucune jointure possible entre une résolution votée et sa réalisation effective). La rubrique `travaux_votes_non_realises` est donc toujours vide, section présente mais non alimentée. Cette limitation nécessite une évolution de schéma hors périmètre de cette story ; suivi : t_5adbc63d. LEGAL.md §7.1 reste marquée ⚠️ partielle tant que ce lien n'existe pas.

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| Appels de fonds / charges par lot | EP-004 ✅ (relevé individuel EP-004-S06) — sources dans `backend/src/db/` (levy) |
| Fonds travaux | EP-004-S03 ✅ |
| Registre des procédures judiciaires | migration `backend/migrations/0034_judicial_proceedings.sql`, API `backend/src/api/judicial_proceedings.rs`, routes `backend/src/api/mod.rs:337-347`, vue `frontend/src/features/registers/JudicialProceedingsView.vue` |
| Propriété / mutation de lot | `backend/src/api/units.rs:279-350` (EP-002-S17 ✅) |
| État daté | **inexistant** |

## 3. Modèle de données

Migration `backend/migrations/NNNN_etat_date_request.sql` (numéro en flux sur cette branche) :

```sql
-- EP-002-S06: etat date request tracking (LEGAL.md §7.1, 15-day legal deadline)
CREATE TYPE etat_date_status AS ENUM ('requested', 'generated', 'sent', 'cancelled');

CREATE TABLE etat_date_request (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    unit_id UUID NOT NULL REFERENCES unit(id),
    requested_at DATE NOT NULL,                      -- date de la demande du notaire
    due_date DATE NOT NULL,                          -- requested_at + 15 jours (calculé en domain)
    requester_name TEXT NOT NULL,                    -- notaire / étude
    status etat_date_status NOT NULL DEFAULT 'requested',
    snapshot JSONB NULL,                             -- état daté figé à la génération
    generated_at TIMESTAMPTZ NULL,
    created_by UUID NOT NULL REFERENCES app_user(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX etat_date_request_unit_idx ON etat_date_request (unit_id);
```

Le `snapshot` JSONB fige le document au moment de la génération (valeur probante : les données vivantes continuent d'évoluer). Étape associée : **mettre à jour `docs/ARCHITECTURE.md`** avant la migration.

## 4. Backend

### 4.1 Domain (`backend/src/domain/etat_date.rs`, nouveau)

- `compute_due_date(requested_at) -> Date` : échéance interne conservatoire J+15 (le délai légal est « avant l'établissement de l'acte », art. 5 décret 67-223 — cf. LEGAL.md §7.1).
- `assemble(inputs) -> EtatDate` : fonction pure recevant les données déjà chargées (soldes du lot, appels, fonds travaux, travaux votés, procédures) et produisant la structure 3 parties + annexes. `Decimal` pour tous les montants. Zéro SQL.
- Structure de sortie : `EtatDate { partie1: Vec<LigneMontant>, partie2: ..., partie3: ..., annexe_quote_part: [Exercice; 2], procedures: Vec<ProcedureSummary>, travaux_votes_non_realises: Vec<...>, emprunts_collectifs: SectionNeant }`.
- Cette fonction d'assemblage est **partagée avec EP-002-S07** (pré-état daté) — la concevoir comme la source unique.

### 4.2 DB (`backend/src/db/etat_date.rs`, nouveau)

`create_request`, `get`, `list_for_building`, `store_snapshot` (statut → `generated`, `generated_at`, `snapshot`), `update_status`. Les données financières sont lues via les repos existants (levy, works fund, judicial proceedings) — pas de duplication SQL.

### 4.3 API (`backend/src/api/etat_date.rs`, nouveau ; routes dans `backend/src/api/mod.rs`)

- `POST /units/{id}/etat-date-requests` — enregistre la demande (manager ; demandeur = notaire ou cédant, art. 5 décret 67-223). Le formulaire prévoit aussi la désignation de l'acquéreur (art. 20-II loi 1965, ALUR), l'office notarial, la référence dossier et la date envisagée de la mutation. Validation dates → `AppError::validation_code("errors.etat_date.invalid_request_date")`.
- `GET /etat-date-requests` — liste avec échéances (tri par `due_date`, badge dépassement).
- `POST /etat-date-requests/{id}/generate` — charge les sources, appelle `domain::etat_date::assemble`, fige le `snapshot`. Regénération autorisée tant que non `sent` ; sinon `AppError::validation_code("errors.etat_date.already_sent")`.
- `GET /etat-date-requests/{id}` — retourne le snapshot (ou 409 `errors.etat_date.not_generated`).

Handlers sans logique métier ; le calcul J+15 et l'assemblage vivent en domain.

## 5. Frontend

- `frontend/src/api/index.ts` : `etatDateApi` (create, list, generate, get).
- `frontend/src/types/domain.ts` : `EtatDateRequest`, `EtatDate` (3 parties, annexes).
- Vue `frontend/src/features/registers/EtatDateView.vue` (ou feature `units`) : liste des demandes avec échéance J+15 (alerte visuelle si proche/dépassée), action « Générer », et **vue imprimable** du document (mise en page 3 parties + annexes, `@media print` via utilitaires Tailwind — pas de `<style>` block dans la feature view).
- **Tout nouveau composant `C*` doit être catalogué dans `docs/DESIGN.md §3` avant usage** (étape dédiée).

## 6. Cas limites

- Cédant à jour de toutes ses obligations → certificat art. 20 « libre de toute obligation » (validité 1 mois), pas de solde débiteur.
- Frais de délivrance (rubrique 7, partie 1) : toujours affiché, plafonné à **380 € TTC** (art. 10-1 b loi 1965, décret n°2020-153) — cf. EP-002-S22 pour la facturation.
- Modalités de remboursement des avances (art. 45-1 décret 1967) : solution 1 ou 2 cochée, jamais les deux ; montant global renseigné.
- Lot sans arriéré ni avance → parties rendues avec lignes à zéro, jamais omises (structure réglementaire fixe).
- Moins de 2 exercices d'historique → annexe quote-part limitée aux exercices existants, avec mention.
- Aucune procédure en cours → section « néant ».
- Emprunts collectifs → section « néant — non gérés » (limitation actée §1).
- Regénération après correction de données → nouveau snapshot écrase l'ancien tant que `status != 'sent'`.
- Mutation du lot entre demande et génération → la génération porte sur le propriétaire cédant au moment de la demande (utiliser l'historique EP-002-S17).

## 7. Critères d'acceptation

1. Une demande enregistrée affiche son échéance J+15 et son statut.
2. La génération produit un document structuré conforme (parties 1/2/3 + annexe quote-part 2 exercices + procédures + travaux votés), montants en `Decimal`, figé en JSONB.
3. Les procédures proviennent du registre 0034 ; les charges/fonds travaux des données EP-004.
4. Section emprunts collectifs présente et explicitement « néant ».
5. Vue imprimable fidèle à la structure. Erreurs i18n `errors.etat_date.*`.
6. `docs/ARCHITECTURE.md` documente la table.

## 8. Plan de tests

- **Domaine pur** : `compute_due_date` (J+15, fin de mois) ; `assemble` — lot avec arriérés, lot créditeur, lot vierge (zéros, sections néant), 1 seul exercice disponible.
- **E2E** (`backend/tests/api_e2e_test.rs`, `common::isolated_test_server()`) : demande → génération → lecture snapshot ; regénération après `sent` → 422 ; génération sans données financières → document à zéros ; droits (copropriétaire → 403 sur POST).

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` → migration `NNNN_etat_date_request.sql` → `cargo sqlx migrate run` → `cargo sqlx prepare`.
2. `domain/etat_date.rs` + tests purs (structure partagée avec S07 en tête).
3. `db/etat_date.rs`.
4. `api/etat_date.rs` + routes `api/mod.rs` + erreurs i18n.
5. Frontend (types, api client, vue liste + imprimable) — cataloguer les `C*` nouveaux d'abord.
6. Tests E2E, entrée `docs/PROGRESS.md`.

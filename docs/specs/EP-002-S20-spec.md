# Spécification technique — EP-002-S20

**Story :** Générer automatiquement le **certificat de l'article 20** et le joindre à l'état daté : certificat « libre de toute obligation » (validité 1 mois) si le cédant est à jour, sinon solde débiteur + avis de mutation (15 jours) et opposition du syndic (15 jours) — art. 20-I/20-II loi 1965 (ALUR art. 55), art. 6 décret 67-223 (LEGAL.md §7.1)

**Épic :** EP-002 — Copropriétaires & lots
**Réf. LEGAL :** §7.1 — art. 20-I loi 1965 (avis de mutation par le notaire dans les 15 jours du transfert de propriété si aucun certificat de moins d'un mois ; opposition du syndic dans les 15 jours de la réception de l'avis, par acte extrajudiciaire ; versement au syndicat dans les 3 mois) · art. 5-1 décret 67-223 (l'opposition énonce précisément les créances : 4 masses — année courante et deux dernières années échues, deux années antérieures, créances garanties par hypothèque légale, autres créances) · art. 20-II loi 1965 (désignation de l'acquéreur, loi ALUR) · art. 6 décret 67-223 (notification par le notaire en cas de certificat) · **ADR :** aucun requis (dérivé du snapshot S06)
**Stories liées :** EP-002-S06 ✅⚠️ (état daté + snapshot figé, source du certificat), EP-002-S17 ✅ (mutation de lot, date de vente)

**Statut actuel :** NON IMPLÉMENTÉ — le template `questionnaire.typ` rend la page « certificat art. 20 » mais les champs `date_certificat` / `certificat_art20_joint` sont à `None` (`domain/legal_documents.rs`) ; seul `copro_a_jour` est calculé.

---

## 1. Quoi (résumé fonctionnel)

À chaque génération d'état daté, le syndic délivre en même temps le **certificat de l'article 20** (il ne peut être délivré qu'en connaissance de la date de vente) :

- si le cédant est **à jour** de ses charges, provisions et avances → certificat attestant qu'il est « **libre de toute obligation à l'égard du syndicat** », **valable un mois** ; le notaire est alors dispensé de l'avis de mutation et adresse au syndic la notification de l'art. 6 du décret 1967 ;
- sinon → mention du **solde débiteur** ; à défaut de règlement, le notaire adresse l'avis de mutation (art. 20-I, LRAR dans les 15 jours du transfert) et le syndic exerce l'**opposition** (art. 20 + art. 5-1 décret 1967 : 4 masses de créances détaillées), frais à la charge du vendeur.

Le certificat est **joint à l'état daté** (page dédiée du questionnaire CSN, déjà rendue par `questionnaire.typ`) et porte : lieu, date, somme débitrice éventuelle, case « certificat daté et signé joint ».

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| Demande d'état daté (snapshot figé) | `backend/src/api/etat_date.rs` (`create`, `generate`), `backend/src/db/etat_date.rs`, table `etat_date_request` (migration S06) |
| Assemblage état daté + calcul `a_jour` | `backend/src/domain/etat_date.rs` (`assemble`) ; `backend/src/domain/legal_documents.rs` (`copro_a_jour = total_part1.is_zero()`, `somme_debitrice`) |
| Page certificat art. 20 (champs vides) | `backend/src/domain/typst/questionnaire.typ` (section « sommes exigibles / certificat art. 20 » ; template à refondre au format La Ruche via EP-002-S24), payload `certificat_art20_joint`, `date_certificat` |
| Date de vente envisagée | à ajouter sur `etat_date_request` (champ `planned_mutation_date`, nullable) — cf. §3 |
| Certificat art. 20 auto-rempli | **partiellement câblé, champs non alimentés** |

## 3. Modèle de données

Migration `backend/migrations/NNNN_etat_date_planned_mutation.sql` :

```sql
-- EP-002-S20: date de vente connue, requise pour délivrer le certificat art. 20
ALTER TABLE etat_date_request ADD COLUMN planned_mutation_date DATE NULL;
```

Le certificat est un **document dérivé du snapshot** (pas de table dédiée) : il est régénéré avec l'état daté et figé dans le `snapshot` JSONB.

## 4. Backend

### 4.1 Domain (`backend/src/domain/etat_date.rs`, extension)

- `assemble_certificat_art20(snapshot: &EtatDate, planned_mutation_date: Option<NaiveDate>, edited_at: NaiveDate) -> CertificatArt20` — fonction pure :
  - `a_jour = total_part1 - total_part2 <= 0` (aucune somme exigible) ;
  - si `a_jour` → `CertificatArt20 { libre: true, somme_debitrice: 0, date_certificat: edited_at, validite_jusquau: edited_at + 1 mois }` ;
  - sinon → `libre: false`, `somme_debitrice = total_part1 - total_part2`, et **pas de délivrance possible sans date de vente connue** (`planned_mutation_date` requis pour l'opposition, art. 20-I) ;
  - `Decimal` pour les montants, zéro SQL.
- Étendre `EtatDate` (ou le payload PDF) avec `certificat_art20: CertificatArt20`.

### 4.2 DB

Aucun repo nouveau : `planned_mutation_date` lu/écrit via `db/etat_date.rs` (`create` accepte le champ optionnel ; `update` si la date de vente change avant `sent`).

### 4.3 API (`backend/src/api/etat_date.rs`, extension)

- `POST /units/{id}/etat-date-requests` — accepte `planned_mutation_date` (optionnel) ; si absent, la génération du certificat est **bloquée avec avertissement** (`errors.etat_date.missing_mutation_date`) mais l'état daté reste générable.
- `POST /etat-date-requests/{id}/generate` — calcule aussi `assemble_certificat_art20` et fige le tout dans le `snapshot`.
- `GET /etat-date-requests/{id}/export/pdf` — le PDF d'état daté inclut la page certificat art. 20 remplie (libre / solde débiteur + date + validité 1 mois), via le template existant.

## 5. Frontend

- `frontend/src/types/domain.ts` : `CertificatArt20` ; champ `planned_mutation_date` sur `EtatDateRequest` (modale de demande, déjà prévue par le formulaire CSN « date envisagée pour la mutation »).
- Badge sur la liste des demandes : « certificat art. 20 délivré (valable jusqu'au …) » ou « solde débiteur — opposition ».
- Aucun composant `C*` nouveau attendu (réutilise la vue imprimable/PDF de S06) ; si un nouveau composant apparaît, le cataloguer dans `docs/DESIGN.md §3` avant usage.

## 6. Cas limites

- Cédant à jour → certificat « libre », somme débitrice à zéro, validité 1 mois affichée.
- Cédant non à jour → solde débiteur = total partie 1 − total partie 2 (les lots non visés par la mutation restent en rubrique 2 du certificat, à zéro si aucun).
- Date de vente inconnue → état daté générable, certificat non délivré (avertissement explicite, jamais de date inventée).
- Validité 1 mois dépassée à la signature → le certificat est régénéré (nouveau snapshot) tant que `status != 'sent'`.
- Mutation à titre gratuit → certificat art. 20 non requis (art. 20 ne vise que les mutations à titre onéreux) : section rendue « non applicable ».

## 7. Critères d'acceptation

1. La génération d'un état daté produit aussi le certificat art. 20 figé dans le snapshot (libre ou solde débiteur, date, validité 1 mois).
2. Le PDF exporté contient la page certificat remplie, jointe à l'état daté.
3. Sans `planned_mutation_date`, la génération signale l'absence de date de vente (i18n `errors.etat_date.missing_mutation_date`) sans bloquer l'état daté.
4. Montants en `Decimal` ; aucune date inventée ; mutation gratuite → certificat « non applicable ».
5. `docs/ARCHITECTURE.md` documente la colonne `planned_mutation_date`.

## 8. Plan de tests

- **Domaine pur** : `assemble_certificat_art20` — cédant à jour, cédant débiteur, `planned_mutation_date` absent, validité = date + 1 mois.
- **E2E** : demande avec date de vente → génération → snapshot contient le certificat → PDF exporté contient la page ; demande sans date → avertissement ; régénération après `sent` → 422 (cohérent S06).

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` → migration `planned_mutation_date` → `cargo sqlx migrate run` → `cargo sqlx prepare`.
2. `assemble_certificat_art20` en domain + tests purs.
3. `db/etat_date.rs` (champ) + `api/etat_date.rs` (création/génération/export) + i18n.
4. Frontend (champ date de vente, badge certificat).
5. Tests E2E, entrée `docs/PROGRESS.md`.

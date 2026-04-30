# PRD.md — l'Application

> **Dénomination** — Dans le présent document, l'expression **« l'Application »** désigne exclusivement **La Ruche**, plateforme web open source de gestion participative de copropriété. Ces deux termes sont interchangeables et désignent la même entité logicielle.
## Product Requirements Document

> Document produit. Vision, utilisateurs, fonctionnalités et contraintes centralisés ici.
> Pour le backlog détaillé (epics, stories, statuts), voir [EPICS.md](EPICS.md).
> Liens vers ADR/DDR pour les contraintes techniques associées.
> Dernière mise à jour : 25 avril 2026

---

## Vision produit

**l'Application** est une webapp open source de gestion participative de copropriété, conçue pour les syndics bénévoles, les membres du conseil syndical et les copropriétaires. Elle vise à rendre la gestion d'immeuble transparente, collaborative et accessible — sans les coûts et la complexité des outils professionnels.

**Principe directeur :** La gestion participative dans les limites légales. Tout ce qui peut être partagé avec les résidents l'est. Les décisions informelles (consultations, sondages) sont clairement distinguées des actes légaux (votes AG).

---

## Utilisateurs cibles

| Rôle | Profil | Besoins principaux |
|---|---|---|
| **Syndic bénévole** | Copropriétaire élu, pas forcément technique | Administration complète, conformité légale, gain de temps |
| **Conseil syndical** | 2–5 membres élus en AG | Supervision, validation pre-AG, accès comptabilité |
| **Copropriétaire** | Résident propriétaire | Transparence, votes AG, consultation de ses charges |
| **Locataire** | Résident non-propriétaire | Signalements, consultations informelles, informations pratiques |
| **Prestataire** | Entreprise externe | Dépôt de documents via lien tokenisé, clôture de tâche |

---

## Fonctionnalités

### Fondations légales (loi 10/07/1965 + décret 17/03/1967)

**Registre de la copropriété** — l'Application permet de constituer la fiche identitaire de la copropriété (nom, adresse, SIRET syndic), de déposer le règlement de copropriété, de configurer les clés de répartition et de gérer le registre des lots avec leurs tantièmes par clé. → EP-001

**Gestion des propriétaires** — L'historique de propriété est tracé avec les dates de cession. Les résidents sont invités par lien tokenisé. L'anonymisation RGPD à la demande efface le nom tout en conservant les données comptables (10 ans, art. L123-22 Code de commerce). → EP-002

**Assemblée Générale** — Cycle de vie complet d'une AG (brouillon → convoquée → tenue → clôturée), avec rappel du délai légal de convocation (21 jours, art. 9 décret 1967). Les résolutions sont typées par majorité (art. 24 / 25 / 26 / unanimité), le vote se fait par lot avec procurations, et les résultats sont calculés automatiquement. Le PV signé est déposé à la clôture. → EP-003

**Budget & Charges** — Budget prévisionnel par exercice découpé en lignes par clé de répartition. Les appels de fonds sont générés automatiquement au prorata des tantièmes. Le fonds travaux obligatoire (≥ 5%, art. 14-2 ALUR) est suivi sur un compte séparé. L'arrêté de comptes compare réalisé et prévisionnel. → EP-004

### Gestion courante

**Documents** — Upload direct vers S3 via pre-signed URL ; le backend ne manipule pas les fichiers. L'accès est contrôlé par rôle, le téléchargement se fait via une URL signée à courte durée de vie, sans exposition publique du bucket. → EP-005

**Profil & préférences** — Chaque utilisateur gère ses informations personnelles, ses notifications et sa préférence d'apparence depuis son profil. Le thème peut être forcé en clair ou sombre, ou rester en mode automatique en suivant la préférence système de l'appareil. → DDR-004

**Contrats fournisseurs** — Chaque contrat recense fournisseur, objet, dates, montant et périodicité. Les reconductions tacites et préavis sont configurables. Des rappels (reconduction, échéance, expiration, résiliation) sont calculés automatiquement et acquittables manuellement. Le contrat d’assurance multirisque immeuble peut en plus regrouper ses pièces justificatives dans le même espace de suivi. → EP-006

**Tâches & Prestataires** — Les tâches sont assignées en interne avec priorité et statut. Un lien tokenisé éphémère permet aux prestataires externes d'accéder sans compte, de déposer photos et documents, puis de pré-valider leur intervention avec un commentaire et un coût optionnel avant validation finale par le syndic. La clôture d'une tâche capture un montant optionnel, réutilisable pour alimenter le carnet d'entretien. Les fichiers transitent par un bucket staging avant validation. Les tâches peuvent être liées à un prestataire du référentiel pour consolider l'historique. → EP-007

**Référentiel prestataires** — Catalogue des entreprises et intervenants de la résidence (nom, email, téléphone, domaine). Accessible à tous les résidents (catalogue pratique), modifiable par les managers. Réutilisé à l'assignation des tâches et propagé au carnet d'entretien. → EP-015

**Sinistres** — Déclaration d'un sinistre (libellé, date, lot ou parties communes) avec suivi de statut jusqu'à clôture et attachement de documents associés. → EP-008

**Carnet d'entretien** — Historique des interventions (libellé, date, coût, prestataire, pièces jointes), alimenté manuellement pour l'historique antérieur au logiciel ou automatiquement à chaque clôture effective d'une tâche, y compris après réouverture. Les liens avec les tâches restent purement de traçabilité et ne bloquent pas la suppression d'une tâche ou d'une entrée du carnet. → EP-009

### Participatif (pas de valeur décisionnelle légale)

**Consultations & Sondages** — Tout résident (copropriétaire ou locataire) peut initier une consultation sans approbation préalable. Les résultats sont visibles en temps réel. La mention "Informelle — non décisionnelle" est obligatoire dans l'interface. → EP-010

**Signalements** — Tout résident peut signaler un problème avec photo(s) optionnelle(s). Le syndic suit le statut et peut convertir un signalement en tâche en choisissant son niveau d'urgence ; la conversion fait automatiquement passer le signalement en cours, puis la clôture effective de la tâche clôture aussi le signalement lié. → EP-011

### Fonctionnalités avancées

**Paiement en ligne** — Les copropriétaires règlent leurs appels de fonds via une passerelle SEPA depuis l'interface. Un reçu PDF est généré à la confirmation. Les fonds sont versés sur le compte bancaire séparé de la copropriété (art. 18 loi 1965). → EP-016

**Tenue d'AG** — Module dédié à la conduite de séance : désignation des rôles (président, secrétaire, scrutateurs), feuille de présence numérique avec calcul du quorum en temps réel, prise de notes collaborative par résolution, génération du projet de PV et circuit de validation avec signature numérique (délai légal : art. 17 décret 1967). → EP-013

**Calendrier copropriété** — Calendrier partagé (mensuel / hebdomadaire / liste) centralisant AG, interventions prestataires et réunions du conseil, avec lien contextuel vers les entités correspondantes et export iCal. → EP-014

---

## 🪦 Cimetière des fonctionnalités abandonnées

### Keycloak pour l'authentification
**Proposé :** Gestion auth via Keycloak.
**Rejeté :** Trop lourd pour le self-hosting sans compétences DevOps. Remplacé par auth Rust intégrée (ADR-002).

### Commons Clause sur la licence
**Proposée :** Addendum pour limiter la commercialisation.
**Rejeté :** Non reconnue OSI — frein aux contributions. Remplacée par un modèle dual-license AGPL-3.0 + licence commerciale explicite, avec CLA pour les contributeurs.

### AGPL-3.0 pure (modèle initial)
**Utilisée initialement.**
**Remplacée :** L'AGPL-3.0 seule ne restreint pas l'usage commercial. Migration vers double licence AGPL-3.0 (non commercial) + licence commerciale (syndics professionnels). Voir `LICENSE`, `LICENSE-COMMERCIAL.md`, `CLA.md`.

### SaaS OSS (hébergement centralisé)
**Proposé :** Option SaaS en plus du self-hosted.
**Rejeté :** Hors scope, complexité opérationnelle, contradictoire avec la philosophie bénévole.

### Comptabilité en partie double
**Proposée :** Plan comptable copropriété (arrêté 14/03/2005).
**Rejeté :** Dépasse le périmètre d'un outil bénévole. Nécessite expertise comptable.

---

## Rôles & permissions

> **Note locataire** : le locataire a le même niveau de participation que le copropriétaire, à l'exception du vote en AG (remplacé par un droit de consultation — lecture des résolutions et PV). Ses initiatives (consultations, signalements) ne nécessitent **aucune approbation préalable** du syndic.

| Action | Syndic | Conseil | Copropriétaire | Locataire |
|---|---|---|---|---|
| Admin copropriété | ✅ | ❌ | ❌ | ❌ |
| Créer/modifier AG | ✅ | ❌ | ❌ | ❌ |
| Voter en AG | ✅ | ✅ | ✅ | ❌ |
| Consulter résultats AG | ✅ | ✅ | ✅ | ✅ |
| Voir comptabilité | ✅ | ✅ | Ses charges | ❌ |
| Créer budget | ✅ | ❌ | ❌ | ❌ |
| Gérer contrats | ✅ | ❌ | ❌ | ❌ |
| Gérer tâches | ✅ | ✅ | ❌ | ❌ |
| Voir documents | ✅ | ✅ | Selon rôle | Selon rôle |
| Signaler | ✅ | ✅ | ✅ | ✅ |
| Consulter/sonder | ✅ | ✅ | ✅ | ✅ |
| Initier consultation | ✅ | ✅ | ✅ | ✅ |
| Inviter résidents | ✅ | ❌ | ❌ | ❌ |

---

## Contraintes légales (non interprétées — textes officiels)

- **Loi n°65-557 du 10/07/1965** — statut de la copropriété
- **Décret n°67-223 du 17/03/1967** — application de la loi de 1965
- **Art. 9 décret 1967** — délai minimal de convocation AG : 21 jours
- **Art. 24 loi 1965** — majorité des voix des présents et représentés
- **Art. 25 loi 1965** — majorité absolue de tous les copropriétaires
- **Art. 26 loi 1965** — double majorité (2/3 tantièmes + majorité en nombre)
- **Art. 14-2 loi ALUR** — fonds travaux obligatoire ≥ 5% budget prévisionnel
- **Art. L123-22 Code de commerce** — conservation données comptables 10 ans
- **RGPD** — droit à l'effacement (anonymisation avec conservation données légales)

> ⚠️ l'Application ne fournit aucun conseil juridique. Les textes sont affichés tels quels. Le syndic reste responsable de la conformité légale.

---

## Architecture technique (résumé)

| Couche | Choix | Référence |
|---|---|---|
| Backend | Rust + Axum — binaire unique | ADR-001 |
| Base de données | PostgreSQL 16 + SQLx | ADR-001 |
| Migrations | SQLx migrate | ADR-001 |
| Auth | JWT httpOnly + Argon2id + WebAuthn | ADR-002 |
| Structure | Single crate modulaire | ADR-003 |
| Frontend | Vue 3 + Vite + TypeScript | DDR-003 |
| Typographie | Fraunces + DM Sans + JetBrains Mono | DDR-001 |
| Design | "The Modern Registry" — lavande/charcoal | DDR-002 |
| Stockage | S3-compatible (MinIO self-hosted) | schéma DB |
| Licence | AGPL-3.0 + licence commerciale | LICENSE, LICENSE-COMMERCIAL.md |
| Déploiement | Docker Compose (backend + postgres + minio + caddy) | Dockerfile |

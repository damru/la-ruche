---
product: La Ruche
version: 2.1.0
owner: Damien
status: validé
last_updated: 2026-10-21
features:
  - product/PRD - Registre de la copropriété.md
  - product/PRD - Gestion des propriétaires.md
  - product/PRD - Assemblée Générale.md
  - product/PRD - Budget et charges.md
  - product/PRD - Documents.md
  - product/PRD - Profil et préférences.md
  - product/PRD - Contrats fournisseurs.md
  - product/PRD - Tâches et prestataires.md
  - product/PRD - Référentiel prestataires.md
  - product/PRD - Sinistres.md
  - product/PRD - Carnet d'entretien.md
  - product/PRD - Consultations et sondages.md
  - product/PRD - Signalements.md
  - product/PRD - Journal d'activité.md
  - product/PRD - Calendrier.md
  - product/PRD - Conformité et reporting.md
  - product/PRD - Paiement en ligne.md
references:
  - LEGAL.md
  - ADR-001
  - ADR-002
  - ADR-003
  - ADR-006
  - ADR-009
  - DDR-001
  - DDR-002
  - DDR-003
  - DDR-004
llm_directives:
  temperature: 0.2
  persona: >
    Ce document est le PRD maître de La Ruche. Il décrit la vision produit,
    les utilisateurs, les fonctionnalités et les contraintes transversales.
    Les PRD détaillés par feature sont référencés dans le frontmatter `features`.
    Ne pas modifier les sections des features individuelles ici — renvoyer
    vers le PRD dédié pour tout détail fonctionnel.
---

# PRD — La Ruche

> **Dénomination** — « l'Application » désigne exclusivement **La Ruche**, plateforme web open source de gestion participative de copropriété.

> Document produit. Vision, utilisateurs, fonctionnalités et contraintes centralisés ici.
> Pour le backlog détaillé (epics, stories, statuts), voir [EPICS.md](EPICS.md).
> Pour le détail feature par feature, voir le [Product Hub](product/README.md).

---

## Vision produit

**La Ruche** est une webapp open source de gestion participative de copropriété, conçue pour les syndics bénévoles, les membres du conseil syndical et les copropriétaires. Elle vise à rendre la gestion d'immeuble transparente, collaborative et accessible — sans les coûts et la complexité des outils professionnels.

**Principe directeur :** La gestion participative dans les limites légales. Tout ce qui peut être partagé avec les résidents l'est. Les décisions informelles (consultations, sondages) sont clairement distinguées des actes légaux (votes AG).

---

## Utilisateurs cibles

| Rôle | Profil | Besoins principaux | Contexte d'usage |
|---|---|---|---|
| **Syndic bénévole** (primaire) | Copropriétaire élu, pas forcément technique | Administration complète, conformité légale, guidage, ne jamais se sentir seul face à une obligation légale | Mobile sur le terrain + bureau le soir |
| **Conseil syndical** (secondaire) | 2–5 membres élus en AG | Supervision, validation pre-AG, accès comptabilité, flux décisionnels clairs | Desktop + mobile |
| **Copropriétaire** (tertiaire) | Résident propriétaire | Transparence, votes AG, consultation de ses charges, signalements | Mobile principalement |
| **Locataire** (tertiaire) | Résident non-propriétaire | Signalements, consultations informelles, informations pratiques, se sentir entendu | Mobile principalement |
| **Prestataire** (périphérique) | Entreprise externe | Dépôt de documents via lien tokenisé, clôture de tâche, zéro apprentissage | Mobile, première visite |
|---|---|---|
| **Syndic bénévole** | Copropriétaire élu, pas forcément technique | Administration complète, conformité légale, gain de temps |
| **Conseil syndical** | 2–5 membres élus en AG | Supervision, validation pre-AG, accès comptabilité |
| **Copropriétaire** | Résident propriétaire | Transparence, votes AG, consultation de ses charges |
| **Locataire** | Résident non-propriétaire | Signalements, consultations informelles, informations pratiques |
| **Prestataire** | Entreprise externe | Dépôt de documents via lien tokenisé, clôture de tâche |

> **Note locataire** : le locataire participe au même niveau que le copropriétaire, sauf pour le vote en AG (remplacé par un droit de consultation).

---

## Fonctionnalités

> Les PRD détaillés par feature sont listés dans le frontmatter `features` et centralisés dans le [Product Hub](product/README.md).

### Fondations légales

| Feature | Usages principaux | PRD |
|---|---|---|
| **Registre de la copropriété** | Fiche identitaire (nom, adresse, SIRET syndic), règlement de copropriété, clés de répartition, registre des lots avec tantièmes versionnés | [PRD](product/PRD%20-%20Registre%20de%20la%20copropri%C3%A9t%C3%A9.md) |
| **Gestion des propriétaires** | Historique de propriété avec dates de cession, invitation par lien tokenisé, anonymisation RGPD (conservation comptable 10 ans) | [PRD](product/PRD%20-%20Gestion%20des%20propri%C3%A9taires.md) |
| **Assemblée Générale** | Cycle de vie complet (brouillon → convoquée → tenue → clôturée), résolutions typées par majorité, vote par lot avec procurations, calcul automatique des résultats, dépôt du PV signé | [PRD](product/PRD%20-%20Assembl%C3%A9e%20G%C3%A9n%C3%A9rale.md) |
| **Budget & Charges** | Budget prévisionnel par exercice et clé de répartition, appels de fonds au prorata des tantièmes, fonds travaux ALUR (≥ 5 %), arrêté de comptes réalisé vs prévisionnel | [PRD](product/PRD%20-%20Budget%20et%20charges.md) |

### Gestion courante

| Feature | Usages principaux | PRD |
|---|---|---|
| **Documents** | Upload S3 via pre-signed URL, accès contrôlé par rôle, téléchargement via URL signée à courte durée de vie | [PRD](product/PRD%20-%20Documents.md) |
| **Profil & préférences** | Informations personnelles, notifications, thème clair/sombre/auto | [PRD](product/PRD%20-%20Profil%20et%20pr%C3%A9f%C3%A9rences.md) |
| **Contrats fournisseurs** | Fiche contrat (fournisseur, objet, dates, montant, périodicité), reconduction tacite et préavis, rappels automatiques acquittables, suivi assurance MRI | [PRD](product/PRD%20-%20Contrats%20fournisseurs.md) |
| **Tâches & Prestataires** | Tâches assignées avec priorité et statut, accès prestataire par lien tokenisé, dépôt de photos/documents, pré-validation d'intervention, clôture avec montant optionnel alimentant le carnet d'entretien | [PRD](product/PRD%20-%20T%C3%A2ches%20et%20prestataires.md) |
| **Référentiel prestataires** | Annuaire centralisé (nom, spécialité, contact, SIRET), réutilisé dans tâches et contrats, consultation simplifiée pour copropriétaires, archivage soft-delete | [PRD](product/PRD%20-%20R%C3%A9f%C3%A9rentiel%20prestataires.md) |
| **Sinistres** | Déclaration (type, date, description), calcul du délai légal, transmission à l'assureur avec date et montant figés, clôture avec remboursement ou refus, traçabilité complète | [PRD](product/PRD%20-%20Sinistres.md) |
| **Carnet d'entretien** | Historique des interventions (libellé, date, coût, prestataire), alimentation automatique à la clôture de tâche, en-tête réglementaire, listing contrats en cours | [PRD](product/PRD%20-%20Carnet%20d%27entretien.md) |

### Participatif

| Feature | Usages principaux | PRD |
|---|---|---|
| **Consultations & Sondages** | Consultation binaire ou sondage à choix multiples, mode anonyme ou nominatif, résultats en temps réel, mention non-décisionnelle obligatoire | [PRD](product/PRD%20-%20Consultations%20et%20sondages.md) |
| **Signalements** | Signalement de problème par tout résident (catégorie, description, photo, urgence), anonymat optionnel, conversion en tâche, suivi de statut par le déclarant | [PRD](product/PRD%20-%20Signalements.md) |

### Transversal

| Feature | Usages principaux | PRD |
|---|---|---|
| **Journal d'activité** | Timeline opérationnelle des transitions de cycle de vie (signalements + tâches), filtrage par rôle et visibilité, append-only | [PRD](product/PRD%20-%20Journal%20d%27activit%C3%A9.md) |
| **Calendrier** | Vue partagée (mensuel/hebdo/liste) des AG, interventions et réunions, lien contextuel vers les entités, export iCal | [PRD](product/PRD%20-%20Calendrier.md) |
| **Conformité et reporting** | Fiches réglementaires (immatriculation, RGPD, diagnostics, assurance, carnet d'entretien), calcul de conformité, export PDF | [PRD](product/PRD%20-%20Conformit%C3%A9%20et%20reporting.md) |

### Avancé (lots futurs)

| Feature | Usages principaux | PRD |
|---|---|---|
| **Paiement en ligne** | Règlement des appels de fonds via passerelle SEPA, reçu PDF, versement sur compte séparé (art. 18 loi 1965) | [PRD](product/PRD%20-%20Paiement%20en%20ligne.md) |
| **Tenue d'AG** | Conduite de séance (rôles, feuille de présence, quorum temps réel, prise de notes, génération PV, signature numérique) | Inclus dans [PRD AG](product/PRD%20-%20Assembl%C3%A9e%20G%C3%A9n%C3%A9rale.md) |

---

## Cimetière des fonctionnalités abandonnées

### Keycloak pour l'authentification
**Proposé :** Gestion auth via Keycloak.
**Rejeté :** Trop lourd pour le self-hosting sans compétences DevOps. Remplacé par auth Rust intégrée.

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
| Voir comptabilité | ✅ | ✅ | Budget général + ses charges | ❌ |
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
- **Art. 14-2-1 loi du 10 juillet 1965** — fonds travaux obligatoire (seuil minimal selon le droit en vigueur, modifié notamment par la loi du 22 août 2021)
- **Art. L123-22 Code de commerce** — conservation données comptables 10 ans
- **RGPD** — droit à l'effacement (anonymisation avec conservation données légales)

> ⚠️ l'Application ne fournit aucun conseil juridique. Les textes sont affichés tels quels. Le syndic reste responsable de la conformité légale.

---

## Architecture technique (résumé)

| Couche | Choix |
|---|---|
| Backend | Rust + Axum — binaire unique |
| Base de données | PostgreSQL 16 + SQLx |
| Migrations | SQLx migrate |
| Auth | JWT httpOnly + Argon2id + WebAuthn |
| Structure | Single crate modulaire |
| Frontend | Vue 3 + Vite + TypeScript + Tailwind CSS (couche utilitaire) |
| Typographie | Fraunces + DM Sans + JetBrains Mono |
| Design | "The Modern Registry" — lavande/charcoal |
| Stockage | S3-compatible (self-hosted) |
| Licence | AGPL-3.0 + licence commerciale |
| Déploiement | Docker Compose (backend + postgres + stockage S3 + caddy) |

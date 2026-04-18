# EPICS.md — La Ruche
## Backlog produit

> Ce document est le backlog officiel du projet.
> Chaque feature est organisée en epic. Les stories sont ajoutées à la demande, au moment de l'implémentation.
> Le MVP est l'ensemble des epics marquées **MVP : oui**.
> Statuts possibles : **en attente** · **en cours** · **terminée**
> Dernière mise à jour : avril 2026

---

## Epics

### EP-001 — Registre de la copropriété
**Statut :** en cours | **MVP :** oui | **Réf. PRD :** Fondations légales

La fiche identitaire de la copropriété (nom, adresse, SIRET syndic), le règlement de copropriété, les clés de répartition configurables et le registre des lots avec leurs tantièmes par clé.

**Stories :** _à définir_

---

### EP-002 — Gestion des propriétaires
**Statut :** en cours | **MVP :** oui | **Réf. PRD :** Fondations légales

Historique de propriété tracé avec dates de cession, invitation par lien tokenisé, anonymisation RGPD à la demande (conservation des données comptables 10 ans).

**Stories :** _à définir_

---

### EP-003 — Assemblée Générale
**Statut :** en cours | **MVP :** oui | **Réf. PRD :** Fondations légales

Cycle de vie complet d'une AG, résolutions typées par majorité, vote par lot avec procurations, calcul automatique des résultats, dépôt du PV signé.

**Stories :** _à définir_

---

### EP-004 — Budget & Charges
**Statut :** en cours | **MVP :** oui | **Réf. PRD :** Fondations légales

Budget prévisionnel par exercice, appels de fonds générés par tantièmes, fonds travaux obligatoire (≥ 5%), arrêté de comptes.

**Stories :** _à définir_

---

### EP-005 — Documents
**Statut :** en cours | **MVP :** oui | **Réf. PRD :** Gestion courante

Upload direct S3 via pre-signed URL, contrôle d'accès par rôle, téléchargement sécurisé sans exposition publique du bucket.

**Stories :** _à définir_

---

### EP-006 — Contrats fournisseurs
**Statut :** en cours | **MVP :** oui | **Réf. PRD :** Gestion courante

Fiche contrat avec reconduction tacite et préavis configurables, rappels automatiques calculés à la création/MAJ, acquittement manuel.

**Stories :** _à définir_

---

### EP-007 — Tâches & Prestataires
**Statut :** en cours | **MVP :** oui | **Réf. PRD :** Gestion courante

Tâches assignées en interne, lien tokenisé éphémère pour accès prestataire sans compte, flux de dépôt staging → validation → bucket final.

**Stories :** _à définir_

---

### EP-008 — Sinistres
**Statut :** en cours | **MVP :** oui | **Réf. PRD :** Gestion courante

Déclaration et suivi de sinistres (lot ou parties communes) avec documents associés.

**Stories :** _à définir_

---

### EP-009 — Carnet d'entretien
**Statut :** en cours | **MVP :** oui | **Réf. PRD :** Gestion courante

Historique des interventions alimenté manuellement ou automatiquement depuis une tâche clôturée.

**Stories :** _à définir_

---

### EP-010 — Consultations & Sondages
**Statut :** en cours | **MVP :** oui | **Réf. PRD :** Participatif

Consultations initiables par tout résident sans approbation préalable, résultats en temps réel, mention obligatoire "non décisionnelle".

**Stories :** _à définir_

---

### EP-011 — Signalements
**Statut :** en cours | **MVP :** oui | **Réf. PRD :** Participatif

Signalement par tout résident avec photo(s), suivi de statut par le syndic, conversion possible en tâche.

**Stories :** _à définir_

---

### EP-012 — Paiement en ligne
**Statut :** en attente | **MVP :** non | **Réf. PRD :** Fonctionnalités avancées

Paiement des appels de fonds via passerelle SEPA, reçu PDF automatique, versement sur compte séparé (art. 18 loi 1965).

**Stories :** _à définir_

---

### EP-013 — Tenue d'AG
**Statut :** en attente | **MVP :** non | **Réf. PRD :** Fonctionnalités avancées — extension EP-003

Conduite de séance : désignation des rôles (président, secrétaire, scrutateurs), feuille de présence numérique, quorum en temps réel, notes collaboratives, génération et validation du PV avec signature numérique.

**Stories :** _à définir_

---

### EP-014 — Calendrier copropriété
**Statut :** en attente | **MVP :** non | **Réf. PRD :** Fonctionnalités avancées

Calendrier partagé centralisant AG, interventions et réunions avec liens contextuels vers les entités correspondantes, export iCal.

**Stories :** _à définir_

---

## Fonctionnalités hors scope (non planifiées)

Ces éléments ont été identifiés mais ne font pas l'objet d'une epic active. Ils peuvent être reclassés à la demande.

| Fonctionnalité | Justification |
|---|---|
| Notifications email/push | Infrastructure supplémentaire — `jobs/reminders.rs` prêt à activer |
| App mobile native | SPA responsive suffisant pour le périmètre bénévole |
| API publique documentée | Pas de cas d'usage identifié |
| Multi-copropriétés par compte | Architecture prête, non exposée — décision post-MVP |
| SSO / fédération d'identité | Keycloak rejeté (ADR-002) |
| OCR/NLP documents | Hook prévu (`domain/processor.rs`) — post-MVP |

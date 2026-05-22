---
product: Paiement en ligne
version: 0.1.0
owner: Damien
status: cadrage
last_updated: 2026-05-22
references:
  - LEGAL.md §11
llm_directives:
  temperature: 0.2
  persona: >
    Tu travailles sur la feature « Paiement en ligne » de La Ruche.
    C'est une feature en attente (lot futur). Ne pas implémenter avant que
    le module Budget et les appels de fonds soient stables et validés.
    Toute implémentation devra respecter PSD2, DSP2, et la réglementation
    anti-blanchiment. Ne jamais stocker de données de carte bancaire.
    Utiliser exclusivement un PSP tiers certifié PCI-DSS.
  modification_policy: >
    Ne modifier ce fichier que sur validation explicite après analyse de gap.
    Le corps du document ne contient aucune référence technique externe
    (ADR, DDR, numéro de feature ou de ticket).
---

# PRD — Paiement en ligne

> ⚠️ **Ce document est un cadrage préliminaire.** Il ne constitue pas un PRD opérationnel. L'implémentation requiert un PRD additionnel validé, un ADR architecture paiement, et un audit de conformité PSD2/DSP2.

---

## 1. Vue d'ensemble

### Objectif

Permettre aux copropriétaires de **régler leurs appels de fonds en ligne** directement depuis l'Application, en intégrant un prestataire de services de paiement (PSP) tiers certifié PCI-DSS.

### Pourquoi

- Les syndics bénévoles gèrent aujourd'hui les encaissements par virement manuel, chèque ou prélèvement — sans suivi intégré.
- Un paiement en ligne intégré réduirait les impayés et automatiserait la réconciliation comptable.

### Prérequis bloquants avant implémentation

- Module Budget & Appels de fonds stabilisé et en production.
- Choix du PSP validé (Stripe, GoCardless, Lemonway…).
- Analyse réglementaire : statut de La Ruche vis-à-vis de la DSP2 (prestataire de services de paiement ? agent ? exemption ?).
- Audit conformité : PCI-DSS, LCB-FT (lutte contre le blanchiment), RGPD pour les données de paiement.

---

## 2. Périmètre envisagé

### Ce qui est dans scope (lot futur)

- Paiement en ligne d'un appel de fonds par un copropriétaire (virement SEPA ou carte).
- Confirmation de paiement et mise à jour du statut de l'appel de fonds.
- Réconciliation automatique (appel de fonds marqué `paid` après confirmation PSP).
- Historique des paiements par copropriétaire.

### Ce qui est hors scope (définitivement)

- Stockage de données de carte bancaire dans La Ruche (interdit — délégué au PSP).
- Paiements entre copropriétaires.
- Gestion des remboursements complexes (acomptes, avoir).
- Prélèvement automatique récurrent (SEPA Direct Debit) — lot ultérieur.

---

## 3. Contraintes de sécurité non négociables

- **Aucune donnée de carte** ne doit transiter par les serveurs de La Ruche — tokenisation PSP exclusive.
- **PCI-DSS** : La Ruche doit viser le niveau de conformité SAQ A (formulaire hébergé chez le PSP).
- **RGPD** : Les données de paiement sont hors du scope RGPD La Ruche dès lors qu'elles restent chez le PSP.
- **LCB-FT** : Identifier les obligations de vérification d'identité selon les montants (seuils KYC).

---

## 4. Questions ouvertes (bloquantes)

| # | Question | Owner | Statut |
|---|---|---|---|
| Q-PAY-001 | Quel PSP ? (Stripe Connect, GoCardless, Lemonway, autre) — analyse coût/conformité | Damien | Bloquant avant toute implémentation |
| Q-PAY-002 | La Ruche est-elle soumise à agrément PSP ou bénéficie-t-elle d'une exemption DSP2 ? | Damien | Bloquant — nécessite avis juridique |
| Q-PAY-003 | Les copropriétaires paient-ils directement la copropriété (compte syndic) ou via un compte de séquestre ? | Damien | Bloquant — impact architecture |
| Q-PAY-004 | Les paiements partiels (acomptes) sont-ils autorisés ? | Damien | Lot futur |

---

## 5. Prochaines étapes

1. Valider le module Budget en production.
2. Conduire l'analyse réglementaire DSP2 / LCB-FT.
3. Sélectionner et contractualiser avec un PSP.
4. Rédiger le PRD opérationnel complet (US, ACs, cycle de vie, NFR).
5. Rédiger un ADR architecture paiement.

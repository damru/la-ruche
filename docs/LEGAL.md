# LEGAL.md — l'Application

> **Dénomination** — Dans le présent document, l'expression **« l'Application »** désigne exclusivement **La Ruche**, plateforme web open source de gestion participative de copropriété. Ces deux termes sont interchangeables et désignent la même entité logicielle.
## Référentiel des obligations légales de la copropriété

> Document de référence produit. Chaque obligation légale est décrite avec ses implications techniques et fonctionnelles, et sa couverture par l'Application.
>
> **Couverture :** ✅ couverte · ⚠️ partielle · 🔲 hors périmètre logiciel · ❌ non couverte (backlog)
>
> **Textes de référence principaux :**
> - Loi n°65-557 du 10 juillet 1965 (statut de la copropriété)
> - Décret n°67-223 du 17 mars 1967 (application de la loi 1965)
> - Loi ALUR n°2014-366 du 24 mars 2014
> - Loi ELAN n°2018-1021 du 23 novembre 2018
> - Loi Climat et Résilience n°2021-1104 du 22 août 2021
> - Arrêté du 14 mars 2005 (plan comptable copropriété)
> - RGPD (Règlement UE 2016/679)
>
> Dernière mise à jour : 24 avril 2026 (re-baseline code-réel)

---

## 1. Registre & identité de la copropriété

### 1.1 Immatriculation au Registre National des Copropriétés (RNC)

**Référence :** Loi ALUR art. 711-1 et ss. du Code de la construction  
**Couverture :** ⚠️ partielle — EP-001, EP-012-S01

Toute copropriété doit être immatriculée au RNC (registre.coproprietes.gouv.fr) et ses données mises à jour annuellement (syndic, budget, nombre de lots).

**Implications :**
- l'Application centralise les données de la fiche identité (adresse, SIRET, syndic), mais la soumission au RNC reste manuelle sur la plateforme gouvernementale.
- L'export des données au format RNC est prévu en EP-012-S01.

---

### 1.2 Règlement de copropriété et état descriptif de division (EDD)

**Référence :** Art. 8 loi 1965  
**Couverture :** ⚠️ partielle — EP-001, EP-005

Le règlement de copropriété et l'EDD (état descriptif de division) sont les documents fondateurs. L'EDD définit les tantièmes de chaque lot pour chaque clé de répartition.

**Implications :**
- Toute modification des tantièmes nécessite un vote en AG à **l'unanimité** (art. 11 loi 1965) ou une décision judiciaire.
- l'Application stocke le document (EP-005) et maintient les tantièmes configurables (EP-001-S03/S04), mais ne vérifie pas la concordance entre la valeur stockée et l'EDD physique — c'est la responsabilité du syndic.
- Cf. §5 pour les implications comptables des tantièmes.

---

### 1.3 Fiche synthétique de copropriété

**Référence :** Art. 8-2 loi 1965, décret du 26 mars 2015  
**Couverture :** ❌ non couverte — EP-012-S02

Document annuel récapitulatif (nombre de lots, budget, données techniques) mis à disposition de chaque copropriétaire.

---

### 1.4 Carnet d'entretien de l'immeuble

**Référence :** Art. 18 loi 1965, décret n°2001-477 du 30 mai 2001 (service-public.fr/F2665)  
**Couverture :** ⚠️ partielle (S01-S03, S05) — EP-009

Le syndic tient à jour le carnet d'entretien, document obligatoire portant sur les parties communes.

**Contenu obligatoire (décret n°2001-477, art. 1) :**

*Mentions minimales permanentes :*

| Donnée | Couverture | Story |
|---|---|---|
| Adresse de l'immeuble | ⚠️ stockée via EP-001-S01 mais non exposée dans la vue carnet | EP-009-S06 |
| Identité du syndic en exercice | ⚠️ stockée via EP-001-S01 mais non exposée dans la vue carnet | EP-009-S06 |
| Références des contrats d'assurance souscrits par le syndicat + dates d'échéance | ⚠️ contrats gérés via EP-006-S05 mais non visibles depuis la vue carnet | EP-009-S07 |

*Mentions complémentaires (selon travaux et équipements) :*

| Donnée | Couverture | Story |
|---|---|---|
| Travaux importants : année, nature, prestataire | ✅ entrées manuelles + alimentation auto depuis tâche clôturée | EP-009-S01/S02/S03 |
| Références des contrats d'assurance dommages-ouvrage en cours de validité | ❌ non couverts | EP-009-S07 |
| Références et dates d'échéance des contrats d'entretien et de maintenance d'équipements communs | ⚠️ `contract_id` sur entrée individuelle uniquement (EP-009-S05) ; pas de listing consolidé dans la vue carnet | EP-009-S08 |
| Travaux prescrits dans le PPT et leur échéancier | ❌ hors scope post-MVP | EP-009-S04 |

**Implémentation actuelle :**
- Saisie manuelle d'interventions (titre, date, description, coût, prestataire, `contract_id`)
- Alimentation automatique à la clôture d'une tâche prestataire (idempotent via `ON CONFLICT DO NOTHING` sur `task_id`)
- Lien vers le contrat de prestation via `contract_id` (EP-009-S05)

**Hors scope post-MVP (EP-009-S04) :** Lien au Plan Pluriannuel de Travaux (PPT).

---

### 1.5 Tenue à jour de la liste des copropriétaires

**Référence :** Art. 18-I-1° loi 1965 (service-public.fr/F2608)  
**Couverture :** ⚠️ partielle — EP-002-S01, EP-002-S04

Le syndic doit tenir à jour la liste de tous les copropriétaires avec : état civil, numéro(s) de lot(s), adresse postale et adresse email.

**Implications :**
- l'Application doit centraliser et maintenir à jour ces données pour chaque copropriétaire (actif, archivé, en mutation).
- La liste est utilisée pour l'envoi des convocations AG, la production de l'état daté et la mise à jour RNC.

---

### 1.6 Espace en ligne sécurisé (extranet de copropriété)

**Référence :** Art. 18-I-5° loi 1965, modifié par la loi ALUR 2014 (service-public.fr/F2608)  
**Couverture :** ✅ couverte nativement — EP-002, EP-003, EP-004, EP-005, EP-006, EP-009

Depuis le 1er janvier 2015, tout syndic professionnel doit mettre à disposition des copropriétaires un **espace en ligne sécurisé** donnant accès aux documents essentiels de la copropriété (règlement, PV d'AG, comptes, carnet d'entretien, contrats, diagnostics) accessibles 24h/24 avec identifiants personnels.

**Implications :**
- l'Application **est** cet espace en ligne sécurisé : authentification par rôle (EP-002), documents (EP-005), AG et PV (EP-003), comptes (EP-004), contrats (EP-006), carnet d'entretien (EP-009).
- C'est la raison d'être centrale du produit au regard de cette obligation légale.

---

## 2. Tantièmes — Audit et traçabilité

> Ce bloc est structurant pour tout le modèle comptable. Il est insuffisamment adressé dans la PRD initiale.

### 2.1 Intégrité de la somme des tantièmes par clé

**Référence :** EDD (acte notarié) + art. 5 loi 1965  
**Couverture :** ⚠️ partielle — EP-001-S03, EP-001-S07

Pour chaque clé de répartition, la somme des tantièmes de tous les lots actifs doit être égale au total déclaré (ex. 10 000 tantièmes généraux).

**Implications :**
- Tout appel de fonds doit être bloqué si `Σ tantièmes ≠ total_clé`.
- Un écart peut survenir après archivage d'un lot, division, ou saisie manuelle incorrecte.

---

### 2.2 Historique des tantièmes par lot

**Référence :** Art. 10 loi 1965 + obligations comptables (art. L123-22 Code de commerce)  
**Couverture :** ✅ EP-001-S06, ADR-006

Les tantièmes affectés à un lot à une date donnée doivent être traçables pour justifier tout appel de fonds passé.

**Implications :**
- Si les tantièmes d'un lot sont modifiés (correction EDD, division de lot), les appels de fonds passés ne doivent pas être recalculés rétroactivement.
- Les tantièmes sont versionnés par lot et par clé (`valid_from` / `valid_to`).
- Tout appel de fonds référence un snapshot des tantièmes au moment de sa génération (immuabilité comptable).

---

### 2.3 Marquage des tantièmes pour chaque appel de fonds

**Référence :** Art. 14-1 loi 1965, décret 1967 art. 35  
**Couverture :** ✅ EP-004-S05, ADR-006

Chaque appel de fonds doit être traçable : quel lot, quelle clé, quels tantièmes, quel montant unitaire, à quelle date.

**Implications :**
- Un relevé individuel de charges par lot (détail des appels reçus sur un exercice) est une obligation d'information du syndic envers les copropriétaires.

---

### 2.4 Tantièmes et mutations de propriété

**Référence :** Art. 6 loi 1965 + notaire (état daté)  
**Couverture :** ⚠️ partielle — EP-002-S06, EP-002-S07

Les tantièmes sont attachés au **lot**, pas au propriétaire. Une mutation (vente) ne modifie pas les tantièmes — mais la quote-part des charges non soldées au jour de la mutation est à répartir entre vendeur et acquéreur via l'état daté.

**Implications :**
- À la mutation, le nouveau propriétaire hérite des mêmes tantièmes.
- Le syndic doit produire un **état daté** dans les 15 jours suivant la demande (loi ALUR), incluant : sommes dues par le lot, fonds travaux, procédures en cours.

---

## 3. Assemblée Générale

### 3.1 Convocation

**Référence :** Art. 9 décret 1967  
**Couverture :** ⚠️ partielle — EP-003-S01, EP-003-S02

- Délai minimum : **21 jours** avant la date de l'AG.
- Documents obligatoires joints : ordre du jour, projet de budget, projets de résolutions, contrats soumis au vote, état financier.
- Mode d'envoi : recommandé avec AR ou remise en main propre contre émargement.

**Implications :**
- l'Application doit calculer et afficher la date limite de convocation (= date AG − 21 jours).
- Alerte si un document obligatoire est manquant avant envoi de la convocation.

---

### 3.2 Règles de majorité

**Référence :** Art. 24, 25, 26 loi 1965  
**Couverture :** ⚠️ partielle — EP-003-S03, EP-003-S04

| Majorité | Base de calcul | Exemples de décisions |
|---|---|---|
| **Art. 24** | Majorité des voix exprimées (présents + représentés) | Travaux courants, budget, petites décisions |
| **Art. 25** | Majorité de l'ensemble des tantièmes (absents compris) | Travaux importants, mandat syndic, délégation |
| **Art. 25-1** | Si > 1/3 pour l'art. 25 non atteint, renvoi en art. 24 | Passerelle |
| **Art. 26** | Double majorité (2/3 des présents représentant > 1/2 des tantièmes) | Travaux transformatifs |
| **Unanimité** | 100% des tantièmes | Modification EDD, aliénation parties communes |

**Implications :**
- Le calcul des résultats doit prendre en compte les procurations et les votes par correspondance (loi ELAN).
- Un lot archivé (propriétaire absent) **ne compte pas** dans le total des tantièmes pour le quorum.

---

### 3.3 PV d'assemblée générale

**Référence :** Art. 17 loi 1965, art. 17-1A  
**Couverture :** ⚠️ partielle — EP-003-S05, EP-003-S06

Le PV doit être signé séance tenante par le président, le secrétaire et les scrutateurs. Il doit être notifié aux copropriétaires absents dans les **2 mois** suivant l'AG (point de départ du délai de contestation — art. 42 loi 1965).

**Implications :**
- Conservation des PV : **sans limitation de durée** (archives de la copropriété).
- l'Application stocke le PV signé (EP-005) et doit tracer la date de notification aux absents.

---

## 4. Comptabilité

### 4.1 Compte bancaire séparé

**Référence :** Art. 18 loi 1965, confirmé ALUR 2014  
**Couverture :** 🔲 hors périmètre logiciel

Le syndic doit ouvrir un compte bancaire **séparé** au nom du syndicat des copropriétaires (et non en son nom propre). l'Application ne gère pas les flux bancaires mais peut indiquer l'IBAN du compte séparé dans la fiche identité.

---

### 4.2 Plan comptable spécifique

**Référence :** Arrêté du 14 mars 2005  
**Couverture :** ⚠️ partielle — EP-004

Le plan comptable copropriété distingue les charges générales (clé générale) des charges spéciales (clés d'équipement). Les classes de comptes sont normées.

**Implications :**
- l'Application structure les lignes budgétaires mais n'implémente pas encore le plan de comptes normé. À terme, l'import/export comptable vers un logiciel tiers nécessitera ce mapping.

---

### 4.3 Budget prévisionnel et appels de fonds

**Référence :** Art. 14-1 loi 1965, décret 1967 art. 35  
**Couverture :** ⚠️ partielle — EP-004

Le budget prévisionnel est voté en AG. Les appels de fonds provisionnels sont versés **par quart** (sauf décision contraire de l'AG), aux dates prévues.

**Implications :**
- Chaque appel de fonds génère une ligne par lot, calculée au prorata des tantièmes de la clé.
- **Cf. §2.3 :** le snapshot des tantièmes doit être figé au moment de la génération.
- Un appel de fonds exceptionnel (travaux votés en AG) suit les mêmes règles mais référence une résolution AG.

---

### 4.4 Fonds travaux obligatoire

**Référence :** Art. 14-2 loi 1965 (introduit par ALUR 2014)  
**Couverture :** ⚠️ partielle — EP-004

Minimum **5% du budget prévisionnel** annuel, versé sur un **compte séparé** dédié. Ne peut pas être restitué lors d'une mutation (reste acquis à la copropriété).

**Implications :**
- l'Application doit alerter si la dotation fonds travaux est inférieure à 5% du budget prévisionnel.
- À la mutation d'un lot, la quote-part fonds travaux du vendeur **n'est pas remboursable** — à préciser dans l'état daté.

---

### 4.5 Arrêté de comptes

**Référence :** Art. 18 loi 1965  
**Couverture :** ⚠️ partielle — EP-004-S04

Clôture annuelle comparant réalisé vs prévisionnel. Présenté en AG avec approbation au vote (art. 24).

**Implications :**
- Doit faire apparaître les charges par clé de répartition et le détail par lot.
- Soldé de tout compte = trop-perçu restitué ou insuffisance appelée.

---

### 4.6 Relevé individuel de charges

**Référence :** Décret 1967 art. 48  
**Couverture :** ✅ EP-004-S06

Chaque copropriétaire doit pouvoir consulter le détail de ses charges sur l'exercice (appels reçus, règlement de charges, solde).

---

### 4.7 Recouvrement des charges impayées

**Référence :** Art. 18-I loi 1965, décret 1967 art. 36 (service-public.fr/F2608)  
**Couverture :** ❌ non couverte — EP-004-S11

Le syndic a l'**obligation légale** de recouvrer les charges impayées au nom du syndicat des copropriétaires. Cette obligation inclut :
- La mise en demeure du copropriétaire débiteur (lettre recommandée)
- L'engagement de procédures judiciaires si nécessaire (injonction de payer, saisie)
- Le suivi des dossiers de recouvrement jusqu'à leur clôture

**Implications :**
- l'Application doit permettre d'identifier les lots en retard de paiement (solde débiteur), d'enregistrer les relances effectuées et de tracer les dossiers contentieux.
- Les procédures judiciaires en cours font partie des informations obligatoires de l'état daté (cf. §7.1).

---

## 5. Contrats fournisseurs

### 5.1 Contrat de syndic

**Référence :** Art. 18-1 loi 1965, décret modèle-type du 26 mars 2015 (ALUR)  
**Couverture :** 🔲 hors périmètre (l'Application est l'outil, pas le syndicataire)

---

### 5.2 Assurance multirisque immeuble

**Référence :** Art. 9-1 loi 1965 (ALUR)  
**Couverture :** ⚠️ assistance documentaire — EP-006-S05

L'assurance est obligatoire depuis ALUR 2014. L'Application peut stocker le contrat, regrouper ses pièces justificatives et alerter sur l'échéance.

---

### 5.3 Contrats courants et reconductions tacites

**Référence :** Art. L136-1 Code de la consommation, art. 18 loi 1965  
**Couverture :** ✅ couverte — EP-006

Tout contrat avec reconduction tacite doit être suivi. Le syndic doit respecter le préavis de résiliation (généralement 3 mois avant la date anniversaire).

**Implications :**
- l'Application calcule automatiquement les dates d'alerte : reconduction imminente, préavis dépassé, expiration.
- Un contrat résilié après la date de reconduction tacite engage la copropriété pour une année supplémentaire.

---

## 6. Diagnostics et obligations techniques

### 6.1 Plan pluriannuel de travaux (PPT)

**Référence :** Art. 14-2 loi 1965, modifié loi Climat 2021  
**Couverture :** ❌ non couverte — EP-009-S04

Obligatoire pour toutes les copropriétés à partir de :
- > 200 lots : depuis le **1er janvier 2023**
- 51–200 lots : depuis le **1er janvier 2024**
- ≤ 50 lots : depuis le **1er janvier 2025**

Révisé tous les 10 ans. Doit prévoir les travaux de conservation, d'amélioration énergétique et de sécurité sur 10 ans avec estimation de coûts.

**Implications :**
- Le PPT doit être soumis au vote en AG (approbation ou refus motivé).
- En cas de refus, le syndicat reste responsable des travaux urgents.

---

### 6.2 DPE collectif

**Référence :** Loi Climat 2021, art. L126-26 Code de la construction  
**Couverture :** 🔲 hors périmètre logiciel

Obligatoire pour les copropriétés à usage d'habitation avec chauffage collectif. Réalisé par un diagnostiqueur certifié. l'Application peut stocker le document résultant (EP-005).

---

### 6.3 Diagnostic technique global (DTG)

**Référence :** Art. L731-1 Code de la construction (ALUR 2014)  
**Couverture :** 🔲 hors périmètre logiciel

Obligatoire lors de la mise en copropriété d'un immeuble de > 10 ans, et facultatif (mais recommandé) ensuite. Sert de base au PPT.

---

## 7. Mutations et cessions de lots

### 7.1 État daté

**Référence :** Art. 5 du décret 1967, complété ALUR 2014  
**Couverture :** ❌ non couverte — EP-002-S06

Le syndic doit produire l'état daté dans les **15 jours** suivant la demande du notaire. Il comprend :
- Les sommes dues par le lot (charges courantes, arriérés)
- La quote-part fonds travaux (non remboursable)
- Les travaux votés non encore réalisés
- Les procédures judiciaires en cours

**Implications :**
- Dépend de §4.3 (appels de fonds), §4.4 (fonds travaux), EP-003-S04 (résolutions travaux votées en AG).

---

### 7.2 Pré-état daté

**Référence :** Art. L721-2 Code de la construction (loi ALUR)  
**Couverture :** ❌ non couverte — EP-002-S07

Document synthétique fourni au promettant-acheteur avant la signature du compromis. Contient les données financières clés de la copropriété.

---

## 8. Protection des données (RGPD)

**Référence :** Règlement UE 2016/679, Loi informatique et libertés (modifiée 2018)  
**Couverture :** ⚠️ partielle — EP-002-S05

Le syndicat des copropriétaires est responsable de traitement. Les données traitées comprennent : identité des copropriétaires/locataires, coordonnées, données bancaires (RIB pour prélèvement).

| Donnée | Durée de conservation légale |
|---|---|
| Données d'identité (copropriétaire actif) | Durée du mandat + 5 ans |
| Données comptables (charges, appels de fonds) | 10 ans (art. L123-22 Code de commerce) |
| PV d'AG | Sans limitation (archives copropriété) |
| Données de connexion | 1 an (CNIL) |

**Implications :**
- L'anonymisation RGPD (à la demande ou au départ d'un copropriétaire) doit préserver les données comptables en remplaçant le nom par un identifiant neutre — jamais supprimer les lignes comptables.
- l'Application doit exposer un mécanisme d'export des données personnelles (droit d'accès) et d'anonymisation (droit à l'effacement).

---

## 9. Sinistres

**Référence :** Code des assurances, contrat multirisque immeuble  
**Couverture :** ❌ non couverte — EP-008

| Type de sinistre | Délai de déclaration |
|---|---|
| Sinistre courant (dégât des eaux, bris de glace...) | **5 jours ouvrés** |
| Catastrophe naturelle | **10 jours** après publication de l'arrêté |
| Vol | **2 jours ouvrés** |

**Implications :**
- l'Application doit afficher le délai restant sur un sinistre déclaré non encore transmis à l'assurance.
- Alerte automatique si le délai légal approche.

---

## 10. Représentation en justice et suivi des procédures

**Référence :** Art. 18-I loi 1965 (service-public.fr/F2608)  
**Couverture :** ⚠️ partielle — EP-002-S06, EP-002-S10

Le syndic représente le syndicat des copropriétaires en justice, tant en demande qu'en défense. Il peut engager des actions pour défendre les intérêts de la copropriété sans vote en AG (sauf actions exceptionnelles nécessitant une autorisation AG).

**Implications :**
- l'Application ne peut pas initier ni conduire des procédures judiciaires — c'est hors périmètre logiciel.
- En revanche, l'Application **doit tracer les procédures judiciaires en cours** : elles figurent obligatoirement dans l'état daté (cf. §7.1), le pré-état daté (cf. §7.2) et doivent être accessibles dans l'espace en ligne sécurisé (cf. §1.6).

---

## Tableau de couverture synthétique

| # | Obligation | Référence | Couverture | Epic |
|---|---|---|---|---|
| 1.1 | Immatriculation RNC | ALUR art. 711-1 | ⚠️ | EP-001, EP-012-S01 |
| 1.2 | Règlement de copropriété + EDD | Art. 8 loi 1965 | ⚠️ | EP-001, EP-005 |
| 1.3 | Fiche synthétique annuelle | Art. 8-2 loi 1965 | ❌ | EP-012-S02 |
| 1.4 | Carnet d'entretien | Décret 2001-477 | ⚠️ | EP-009 |
| 1.5 | Liste des copropriétaires à jour | Art. 18-I-1° loi 1965 | ⚠️ | EP-002-S01, EP-002-S04 |
| 1.6 | Espace en ligne sécurisé (extranet) | Art. 18-I-5° loi 1965 (ALUR) | ✅ | EP-002, EP-003, EP-004, EP-005, EP-006, EP-009 |
| 2.1 | Intégrité somme tantièmes | EDD + art. 5 | ⚠️ | EP-001-S03, EP-001-S07 |
| 2.2 | Historique tantièmes par lot | Art. 10 loi 1965 | ✅ | EP-001-S06 |
| 2.3 | Snapshot tantièmes par appel de fonds | Art. 14-1 loi 1965 | ✅ | EP-004-S05 |
| 2.4 | Tantièmes et mutations | Art. 6 loi 1965 | ⚠️ | EP-002-S06, EP-002-S07 |
| 3.1 | Convocation AG (21 jours) | Art. 9 décret 1967 | ⚠️ | EP-003-S01, EP-003-S02 |
| 3.2 | Règles de majorité | Art. 24/25/26 loi 1965 | ⚠️ | EP-003-S03, EP-003-S04 |
| 3.3 | PV + notification absents | Art. 17 loi 1965 | ⚠️ | EP-003-S05, EP-003-S06 |
| 4.1 | Compte bancaire séparé | Art. 18 loi 1965 | 🔲 | — |
| 4.2 | Plan comptable normé | Arrêté 14/03/2005 | ⚠️ | EP-004 |
| 4.3 | Budget prévisionnel + appels de fonds | Art. 14-1 loi 1965 | ⚠️ | EP-004 |
| 4.4 | Fonds travaux ≥ 5% | Art. 14-2 loi 1965 | ⚠️ | EP-004 |
| 4.5 | Arrêté de comptes | Art. 18 loi 1965 | ⚠️ | EP-004-S04 |
| 4.6 | Relevé individuel de charges | Décret 1967 art. 48 | ✅ | EP-004-S06 |
| 4.7 | Recouvrement des charges impayées | Art. 18-I loi 1965 | ❌ | EP-004-S11 |
| 5.1 | Contrat de syndic (modèle-type) | Décret 2015 | 🔲 | — |
| 5.2 | Assurance immeuble | Art. 9-1 loi 1965 | ⚠️ | EP-006 |
| 5.3 | Contrats + reconductions tacites | Art. 18 loi 1965 | ✅ | EP-006 |
| 6.1 | Plan pluriannuel de travaux (PPT) | Loi Climat 2021 | ❌ | EP-009-S04 |
| 6.2 | DPE collectif | Loi Climat 2021 | 🔲 | EP-005 |
| 6.3 | Diagnostic technique global (DTG) | ALUR art. L731-1 | 🔲 | EP-005 |
| 7.1 | État daté (15 jours) | Décret 1967 art. 5 | ❌ | EP-002-S06 |
| 7.2 | Pré-état daté | ALUR art. L721-2 | ❌ | EP-002-S07 |
| 8 | RGPD + anonymisation | RGPD 2016/679 | ⚠️ | EP-002-S05 |
| 9 | Sinistres (délais déclaration) | Code des assurances | ❌ | EP-008 |
| 10 | Représentation en justice + procédures en cours | Art. 18-I loi 1965 | ⚠️ | EP-002-S06, EP-002-S10 |

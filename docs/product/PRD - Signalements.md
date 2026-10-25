---
product: Signalements
version: 1.0.0
owner: Damien
status: validé
last_updated: 2026-05-22
references:
  - LEGAL.md §3
llm_directives:
  temperature: 0.2
  persona: >
    Tu travailles sur la feature « Signalements » de La Ruche.
    Tout résident authentifié (copropriétaire ou locataire) peut créer
    un signalement. La conversion d'un signalement en tâche DOIT conserver
    le lien signalement_id sur la tâche créée.
    L'identité du déclarant est masquée aux autres résidents si l'option
    d'anonymat est activée — elle reste visible du Syndic.
    Ne pas implémenter : chat en temps réel, modération automatique par IA.
---

# PRD — Signalements

## 1. Vue d'ensemble

### Objectif

Permettre à **tout résident** (copropriétaire ou locataire) de **signaler un problème** affectant les parties communes ou la sécurité de l'immeuble, et permettre au syndic de suivre, prioriser et clôturer ces signalements — avec la possibilité de les convertir en tâches.

### Pourquoi

- Les signalements arrivent aujourd'hui par SMS, email ou vocal au syndic bénévole — sans traçabilité ni priorisation.
- Certains problèmes urgents (sécurité, fuite d'eau) nécessitent une visibilité immédiate.
- La conversion automatique en tâche élimine une resaisie manuelle et conserve le lien causal.

### KPIs

- Délai moyen entre signalement et première prise en compte (accusé de réception) ≤ 48 h.
- Taux de signalements convertis en tâches ou clôturés ≥ 80 % à 30 jours.

---

## 2. Contraintes techniques

> **CRITICAL** — Ces contraintes sont immuables.

### Anonymat

- Si le déclarant active l'option d'anonymat, `reporter_id` est masqué dans toutes les vues et réponses API destinées aux autres résidents et au Conseil.
- Le Syndic DOIT toujours voir l'identité du déclarant (même en mode anonyme) pour éviter les abus.

### Conversion en tâche

- La conversion d'un signalement en tâche DOIT créer une tâche dans le module Tâches avec `source_signal_id` référençant le signalement.
- La conversion ne clôture pas automatiquement le signalement — le statut passe à `converted`.
- La tâche créée hérite de la description du signalement. La catégorie de la tâche DOIT être saisie manuellement par le Syndic lors de la conversion (voir AC-SIG-005-A).

### Pièces jointes et RGPD

- Photos et documents rattachés via le module Documents — catégorie `SIGNALEMENT`, avec `signal_id` comme référence.
- Les photos peuvent capturer l'image de personnes dans les parties communes. L'interface DOIT afficher un avertissement RGPD lors du dépôt d'une photo : *« Assurez-vous que cette photo ne contient pas de personnes identifiables, ou obtenez leur consentement. »*
- **Rétention recommandée** : 2 ans pour les signalements clôturés sans suite ou sans action ; 5 ans pour les signalements ayant généré une tâche. Les données DOIVENT être anonymisables à la demande du déclarant (droit à l'effacement RGPD).

---

## 3. Profils utilisateurs

### Personas

| ID | Persona | Besoin principal |
|---|---|---|
| Persona-Syndic | Syndic bénévole | Gérer et prioriser les signalements |
| Persona-Conseil | Conseil syndical | Consulter les signalements |
| Persona-Copro | Copropriétaire | Déclarer un problème et suivre son traitement |
| Persona-Locataire | Locataire | Déclarer un problème et suivre son traitement |

### Matrice CRUD

| Domaine | Syndic | Conseil | Copropriétaire | Locataire |
|---|---|---|---|---|
| Signalements | `R / C / U / D¹ / S²` | `R / C / U¹ / D¹` | `C / R¹` | `C / R¹` |
| Commentaires de suivi | `C / R` | `C / R` | `R` | `R` |
| Pièces jointes | `C / R / D` | `R` | `C / R¹` | `C / R¹` |

> ¹ Lecture et modification limitées aux signalements créés par le résident lui-même.
> La liste des signalements pour le Syndic et le Conseil est globale (tous les signalements de la copropriété).
> ² `S` = changement de statut (accusé réception, `in_progress`, `closed`, `converted`) et conversion en tâche — **réservés au Syndic**. Le Conseil peut créer, lire, mettre à jour et supprimer, mais ne change pas le statut d'un signalement et ne le convertit pas en tâche (Q-SIG-003).

> ¹ Lecture et modification limitées aux signalements créés par le résident lui-même.
> La liste des signalements pour le Syndic et le Conseil est globale (tous les signalements de la copropriété).

### Règles de sécurité

- En mode anonyme, `reporter_id` n'est jamais exposé aux résidents ni au Conseil — uniquement au Syndic.
- Un résident ne peut pas voir les signalements d'un autre résident (sauf les siens).
- Filtrage systématique par `building_id`.

---

## 4. Cycle de vie

```
open ──(accusé de réception)──▶ acknowledged ──(en cours)──▶ in_progress
                                                                  │
                                             ┌────────────────────┤
                                             │                    │
                                        (clôture)          (conversion)
                                             │                    │
                                           closed           converted ──▶ [lien tâche]
```

| État | Description |
|---|---|
| `open` | Signalement soumis — en attente d'accusé de réception |
| `acknowledged` | Pris en compte par le syndic |
| `in_progress` | Traitement en cours |
| `closed` | Clôturé — résolu ou classé sans suite |
| `converted` | Converti en tâche — suivi via le module Tâches |

---

## 5. Exigences fonctionnelles

### MoSCoW

| Priorité | Ref | Action | Attendu produit |
|---|---|---|---|
| Must | US-SIG-001 | Créer un signalement | Catégorie, description, localisation, photo optionnelle |
| Must | US-SIG-002 | Lister les signalements | Vue Syndic (globale) et vue Résident (les siens) |
| Must | US-SIG-003 | Accuser réception | Syndic prend en compte le signalement |
| Must | US-SIG-004 | Clôturer un signalement | Résolu ou classé sans suite avec commentaire |
| Should | US-SIG-005 | Convertir en tâche | Créer une tâche liée depuis le signalement |
| Should | US-SIG-006 | Ajouter un commentaire de suivi | Syndic et Conseil commentent ; résidents voient |
| Must | US-SIG-007 | Anonymat du déclarant | Option par signalement, visible Syndic uniquement |

### User stories détaillées

#### US-SIG-001 — Créer un signalement

**En tant que** `Persona-Copro` ou `Persona-Locataire`,
**je veux** créer un signalement de problème dans les parties communes,
**afin de** alerter le syndic et suivre la prise en charge.

**Critères d'acceptation :**

- **AC-SIG-001-A** : Champs obligatoires : catégorie (`EQUIPEMENT`, `SECURITE`, `PROPRETE`, `NUISANCE`, `AUTRE`), description.
- **AC-SIG-001-B** : Champs optionnels : localisation dans l'immeuble (texte libre), photo (via module Documents catégorie `SIGNALEMENT` — avertissement RGPD affiché avant upload), niveau d'urgence (`NORMAL`, `URGENT`).
- **AC-SIG-001-C** : Le signalement est créé au statut `open` et immédiatement visible du Syndic et du Conseil.
- **AC-SIG-001-D** : Si `urgence = URGENT`, le syndic DOIT recevoir une notification push/email immédiate.

#### US-SIG-002 — Lister les signalements

**En tant que** utilisateur autorisé,
**je veux** consulter la liste des signalements,
**afin de** avoir une vue d'ensemble du suivi.

**Critères d'acceptation :**

- **AC-SIG-002-A** : Le Syndic et le Conseil voient tous les signalements de la copropriété, filtrables par statut, catégorie et urgence.
- **AC-SIG-002-B** : Un copropriétaire ou locataire ne voit que ses propres signalements.
- **AC-SIG-002-C** : Les signalements urgents non accusés sont mis en évidence visuellement.

#### US-SIG-003 — Accuser réception

**En tant que** `Persona-Syndic`,
**je veux** accuser réception d'un signalement,
**afin de** informer le déclarant que son signalement est pris en compte.

**Critères d'acceptation :**

- **AC-SIG-003-A** : L'accusé de réception passe le statut à `acknowledged` et DOIT notifier le déclarant.
- **AC-SIG-003-B** : Un commentaire optionnel peut être ajouté lors de l'accusé.
- **AC-SIG-003-C** : Depuis `acknowledged`, le Syndic PEUT passer le signalement à `in_progress` via un endpoint dédié, indiquant que le traitement est en cours. Le déclarant DOIT être notifié.
- **AC-SIG-003-D** : Un commentaire optionnel peut être ajouté lors du passage à `in_progress`.

#### US-SIG-004 — Clôturer un signalement

**En tant que** `Persona-Syndic`,
**je veux** clôturer un signalement,
**afin de** indiquer qu'il est résolu ou classé sans suite.

**Critères d'acceptation :**

- **AC-SIG-004-A** : La clôture DOIT enregistrer le motif : `RESOLVED`, `DUPLICATE`, `OUT_OF_SCOPE`, `NO_ACTION`.
- **AC-SIG-004-B** : Un commentaire de clôture DOIT être saisi (obligatoire).
- **AC-SIG-004-C** : Le déclarant DOIT être notifié de la clôture avec le commentaire.

#### US-SIG-005 — Convertir en tâche

**En tant que** `Persona-Syndic`,
**je veux** convertir un signalement en tâche,
**afin de** planifier et suivre la résolution via le module de gestion des tâches.

**Critères d'acceptation :**

- **AC-SIG-005-A** : La conversion crée une tâche avec `source_signal_id` → signalement, préfillée avec le titre et la description du signalement. La catégorie de la tâche DOIT être choisie explicitement par le Syndic lors de la conversion — elle n'est pas héritée automatiquement de la catégorie du signalement.
- **AC-SIG-005-B** : Le statut du signalement passe à `converted` et affiche le lien vers la tâche créée.
- **AC-SIG-005-C** : Le déclarant est notifié que son signalement a été pris en charge sous forme de tâche.

---

## 6. Exigences non fonctionnelles

| ID | Catégorie | Exigence | Seuil |
|---|---|---|---|
| NFR-Notif-001 | Notification | Alerte signalement urgent | Syndic notifié dans les 5 min |
| NFR-Sec-001 | Anonymat | reporter_id masqué (mode anonyme) | 0 fuite vers résidents |

---

## 7. Hors scope

- Chat en temps réel entre résident et syndic sur un signalement.
- Modération automatique des signalements (IA, filtrage de contenu).
- Signalements publics visibles hors de l'Application.
- Escalade automatique après X jours sans réponse (lot futur).

---

## 8. Métriques de succès

| Métrique | Cible | Délai |
|---|---|---|
| Délai accusé de réception moyen | ≤ 48 h | Continu |
| Taux de clôture ou conversion à 30 jours | ≥ 80 % | Continu |

---

## 9. Questions ouvertes

| # | Question | Owner | Condition de révision |
|---|---|---|---|
| Q-SIG-001 | Les locataires peuvent-ils créer des signalements de sécurité urgents même si le syndic n'est pas en ligne ? | Damien | Ouvert — comportement hors-ligne non défini. |
| Q-SIG-002 | Faut-il un SLA configurable (délai de réponse maximal attendu) ? | Damien | Lot futur |
| Q-SIG-003 | ~~Le Conseil peut-il clôturer un signalement ?~~ | Damien | ✅ Résolu — non. Seul le Syndic peut clôturer ou convertir (US-SIG-004 et US-SIG-005). |

---

## Appendice A — Index de traçabilité

| US | Priorité | ACs | Domaine CRUD |
|---|---|---|---|
| US-SIG-001 | Must | AC-SIG-001-A à D | Signalements (C) |
| US-SIG-002 | Must | AC-SIG-002-A à C | Signalements (R) |
| US-SIG-003 | Must | AC-SIG-003-A à D | Signalements (transition → acknowledged / in_progress) |
| US-SIG-004 | Must | AC-SIG-004-A à C | Signalements (transition → closed) |
| US-SIG-005 | Should | AC-SIG-005-A à C | Conversion → tâche (C) |
| US-SIG-006 | Should | — | Commentaires de suivi (C / R) |
| US-SIG-007 | Could | — | Option anonymat |

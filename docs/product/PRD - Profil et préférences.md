---
product: Profil et préférences
version: 1.0.0
owner: Damien
status: validé
last_updated: 2026-05-20
references:
  - EP-019
  - DDR-004
  - LEGAL.md §8
llm_directives:
  temperature: 0.2
  persona: >
    Tu es un agent développeur senior travaillant sur la feature
    « Profil et préférences » de La Ruche. Tu DOIS respecter les contraintes
    techniques et les critères d'acceptation sans dérogation.
    Tu NE DOIS PAS implémenter de fonctionnalités listées dans Hors scope
    (gestion de mot de passe, MFA, export RGPD, suppression de compte).
    L'API profil DOIT filtrer strictement par user_id extrait du JWT —
    aucun accès cross-user autorisé. Chaque utilisateur ne peut lire et
    modifier QUE ses propres données.
---

# PRD — Profil et préférences

> **Statut** — en cours
> **Dernière modification** — 20 mai 2026
> **Références** — `EP-019`, `DDR-004`, `LEGAL.md §8`

---

## 1. Vue d'ensemble

### Objectif

Offrir à chaque utilisateur authentifié un **espace personnel simple** pour gérer ses informations personnelles, ses préférences de notification et sa préférence d'apparence.

### Pourquoi

La feature ne cherche pas à devenir un centre de sécurité avancé. Elle doit avant tout :

- éviter que des réglages personnels soient dispersés dans différents écrans ;
- rendre l'expérience stable d'un appareil à l'autre ;
- permettre à l'utilisateur de contrôler ce qu'il reçoit et comment il perçoit l'interface.

### KPIs

Aucun KPI prioritaire n'est retenu à ce stade. La feature est un socle de confort et de personnalisation.

---

## 2. Contraintes techniques

> **CRITICAL** — Ces contraintes sont immuables. Tout agent ou développeur DOIT les respecter sans exception avant d'implémenter une quelconque fonctionnalité.

### Stack

- **Backend** : Rust (Axum / SQLx / PostgreSQL) — voir ADR-001
- **Frontend** : Vue 3 Composition API + TypeScript — voir DDR-001/002
- **Auth** : JWT / tokens opaques — voir ADR-002

### Isolation utilisateur (RGPD)

- L'API profil DOIT filtrer strictement par `user_id` extrait du JWT de l'utilisateur authentifié.
- Aucun endpoint NE DOIT exposer les préférences ou informations personnelles d'un autre utilisateur.
- Les préférences sont **données personnelles** au sens RGPD — isolation stricte obligatoire.
- Aucun rôle (y compris administrateur) NE DOIT pouvoir modifier les préférences personnelles d'un autre utilisateur via cette feature.
- **Base légale du traitement** : intérêt légitime (RGPD art. 6.1.f) — les préférences sont fonctionnelles et nécessaires au confort d'utilisation ; aucun consentement explicite requis.
- **Durée de conservation** : les préférences sont supprimées lors de l'anonymisation du compte utilisateur, dans le même cycle que les données d'identité (LEGAL.md § 8 — durée du mandat + 5 ans).

### Données éditables vs non-éditables

- Seules les données dont l'utilisateur est **propriétaire dans le produit** sont modifiables via cette feature.
- Les données structurantes ou juridiques gérées ailleurs (tantièmes, rôle assigné, résidence) NE DOIVENT PAS apparaître comme éditables.
- La distinction `editable` / `prefilled` / `read_only` DOIT être explicite visuellement (voir DDR-004).

### Patterns interdits

- Aucun endpoint exposant les données d'un autre utilisateur, même partiellement.
- Aucune gestion de mot de passe, MFA, ou mécanisme de sécurité avancée dans cette feature.
- Aucun champ non éditable NE DOIT être inclus dans le payload de mise à jour.
- Ne pas utiliser `unwrap()` / `expect()` dans le code backend — utiliser `Result<T, E>`.

---

## 3. Profils utilisateurs

### Personas

| ID | Persona | Besoin principal | Attente clé |
|---|---|---|---|
| Persona-Syndic | Syndic bénévole | Retrouver ses réglages personnels | Gagner du temps dans un usage fréquent |
| Persona-Conseil | Conseil syndical | Ajuster ses notifications et son affichage | Ne pas subir trop de bruit applicatif |
| Persona-Copro | Copropriétaire | Gérer ses coordonnées utiles | Expérience simple et lisible |
| Persona-Locataire | Locataire | Adapter l'interface à son usage | Maîtriser le niveau d'information reçu |

> Tous les personas partagent le même niveau d'accès à leur propre profil (R¹/U¹). La distinction métier n'affecte pas cette feature.

### User stories de haut niveau

- En tant que `utilisateur authentifié`, je veux consulter mes informations personnelles et mes préférences dans un espace dédié.
- En tant que `utilisateur authentifié`, je veux modifier les informations dont je suis propriétaire pour garder un compte à jour.
- En tant que `utilisateur authentifié`, je veux comprendre dès ma première visite quels réglages sont déjà définis.
- En tant que `utilisateur authentifié`, je veux choisir mes préférences de notifications pour ne recevoir que l'utile.
- En tant que `utilisateur authentifié`, je veux choisir l'apparence claire, sombre ou automatique selon mon contexte.

### Matrice CRUD

| Domaine | Entrée UX | Syndic | Conseil | Copropriétaire | Locataire |
|---|---|---|---|---|---|
| Profil personnel | Onglet `Compte` | `R¹/U¹` | `R¹/U¹` | `R¹/U¹` | `R¹/U¹` |
| Notifications personnelles | Onglet `Notifications` | `R¹/U¹` | `R¹/U¹` | `R¹/U¹` | `R¹/U¹` |
| Apparence | Onglet `Apparence` | `R¹/U¹` | `R¹/U¹` | `R¹/U¹` | `R¹/U¹` |
| Contexte de compte | Résumé compte | `R¹` | `R¹` | `R¹` | `R¹` |

> ¹ Restreint aux propres données de l'utilisateur authentifié — aucun accès cross-user.

### Règles de sécurité

- Aucune donnée personnelle d'un autre utilisateur ne doit être visible depuis cet espace.
- Les préférences sont strictement personnelles et non administrables par un autre rôle.
- Cette feature ne couvre pas la sécurité avancée du compte (mot de passe, MFA).

---

## 4. Règles de gestion

### Première visite et valeurs par défaut

- Lors de la première visite, l'utilisateur DOIT voir clairement l'état initial de son profil sans réglages implicites cachés.
- Les préférences DOIVENT afficher une valeur par défaut lisible avant toute personnalisation.
- `Automatique` est la valeur par défaut pour la préférence d'apparence si aucun choix explicite n'a été fait.
- Un écran vide sans explication DOIT être considéré comme un bug — une aide contextuelle courte DOIT être affichée.

### Distinction visuelle des champs

| Classe | Comportement |
|---|---|
| `editable` | Champ modifiable par l'utilisateur via cette feature |
| `prefilled` | Valeur importée d'une autre source, lisible mais non modifiable ici |
| `read_only` | Donnée système ou juridique — affichée en lecture seule (voir DDR-004) |

> **Règle** : `full_name` est `prefilled` depuis EP-002 (fiche copropriétaire). Il n'est modifiable que par le syndic via EP-002, jamais depuis cette feature.

### Réinitialisation des préférences

- La réinitialisation aux valeurs par défaut NE DOIT affecter QUE les préférences non critiques (notifications, apparence).
- Une réinitialisation DOIT être confirmée explicitement avant exécution.
- La réinitialisation NE DOIT PAS modifier les informations personnelles de l'utilisateur.

---

## 5. Exigences fonctionnelles

### MoSCoW

| Priorité | Ref | Action | Attendu produit |
|---|---|---|---|
| Must | US-PP-001 | Consulter son profil | Vue à onglets (Compte, Notifications, Apparence) |
| Must | US-PP-002 | Modifier ses informations personnelles | Champs éditables avec validation et confirmation |
| Must | US-PP-003 | Gérer la première visite et les valeurs par défaut | État initial lisible, valeurs par défaut visibles |
| Must | US-PP-004 | Gérer ses préférences de notifications | Activation/désactivation, distinction obligatoire/optionnel |
| Must | US-PP-006 | Gérer sa préférence d'apparence | Clair / Sombre / Automatique — effet immédiat |
| Should | US-PP-005 | Réinitialiser ses préférences | Retour aux valeurs par défaut avec confirmation |
| Should | US-PP-007 | Afficher le contexte utilisateur | Rôle, résidence, périmètre de compte en lecture seule |

### User stories détaillées

#### US-PP-001 — Consulter son profil

**En tant que** `utilisateur authentifié`,
**je veux** accéder à un espace personnel en un clic,
**afin de** retrouver toutes mes informations et préférences dans un seul écran.

**Critères d'acceptation** :

- **AC-PP-001-A** : Tout utilisateur authentifié DOIT pouvoir accéder à son espace personnel depuis la navigation principale.
- **AC-PP-001-B** : L'espace DOIT être organisé en onglets : `Compte`, `Notifications`, `Apparence`.
- **AC-PP-001-C** : Les éléments non éditables DOIVENT être visuellement distingués des champs modifiables (voir DDR-004).
- **AC-PP-001-D** : Une session expirée DOIT rediriger vers la page de connexion sans perte du chemin de navigation.

#### US-PP-002 — Modifier ses informations personnelles

**En tant que** `utilisateur authentifié`,
**je veux** mettre à jour mes informations personnelles éditables,
**afin de** garder un compte fiable et à jour.

**Critères d'acceptation** :

- **AC-PP-002-A** : L'utilisateur PEUT modifier son email (identifiant de connexion unique — mise à jour directe sans vérification) et son numéro de téléphone.
- **AC-PP-002-B** : Le nom complet (synchronisé depuis EP-002), le rôle, la résidence et les tantiemes NE DOIVENT PAS apparaître comme éditables — rendus en classe `prefilled`.
- **AC-PP-002-C** : Un email ou téléphone au format invalide DOIT afficher un message d'erreur précis au champ concerné.
- **AC-PP-002-D** : Tentative de modifier une donnée verrouillée via l'API DOIT retourner 403.
- **AC-PP-002-E** : Une confirmation de sauvegarde réussie DOIT être affichée de manière non intrusive après enregistrement.

#### US-PP-003 — Gérer la première visite et les valeurs par défaut

**En tant que** `utilisateur authentifié`,
**je veux** comprendre immédiatement l'état de mon profil lors de ma première connexion,
**afin de** ne pas découvrir des réglages implicites cachés.

**Critères d'acceptation** :

- **AC-PP-003-A** : Lors de la première visite, les champs à compléter DOIVENT être identifiables sans ambiguïté.
- **AC-PP-003-B** : Les préférences DOIVENT afficher une valeur par défaut lisible avant toute personnalisation.
- **AC-PP-003-C** : `auto` DOIT être la valeur par défaut affichée pour la préférence d'apparence.
- **AC-PP-003-D** : Le comportement réel du système DOIT correspondre à la valeur par défaut affichée (pas de divergence silencieuse).

#### US-PP-004 — Gérer ses préférences de notifications

**En tant que** `utilisateur authentifié`,
**je veux** choisir quels signaux applicatifs je reçois,
**afin de** maîtriser mon niveau d'information sans subir du bruit.

**Critères d'acceptation** :

- **AC-PP-004-A** : L'utilisateur PEUT activer ou désactiver chaque notification individuellement (granularité par événement).
- **AC-PP-004-B** : Les préférences DOIVENT rester compréhensibles même si tous les canaux ne sont pas encore actifs.
- **AC-PP-004-C** : Les notifications obligatoires (base légale ou délai réglementaire) DOIVENT être affichées avec un toggle **visible mais désactivé** — l'utilisateur voit l'événement, ne peut pas le désactiver. Un label discret `Obligatoire` DOIT accompagner le toggle.
- **AC-PP-004-D** : Un canal temporairement indisponible DOIT afficher une explication fonctionnelle sans jargon technique.
- **AC-PP-004-E** : Une sauvegarde partielle non complète DOIT afficher un avertissement explicite.
- **AC-PP-004-F** : Le catalogue d'événements supportés DOIT correspondre exactement à la liste définie ci-dessous — toute variante non listée est hors scope.

**Catalogue d'événements MVP** :

| Catégorie | Clé événement | Label utilisateur | Toggle |
|---|---|---|---|
| **AG** | `ag.convocation_publiee` | Nouvelle convocation publiée | 🔒 Obligatoire |
| **AG** | `ag.rappel_imminent` | Rappel AG dans 7 jours | 🔒 Obligatoire |
| **AG** | `ag.pv_disponible` | PV disponible | Optionnel |
| **Charges** | `charge.appel_emis` | Appel de fonds émis | Optionnel |
| **Charges** | `charge.appel_echu` | Rappel appel de fonds échu | Optionnel |
| **Tâches** | `tache.ouverte` | Tâche ouverte | Optionnel |
| **Tâches** | `tache.urgente` | Tâche urgente | Optionnel |
| **Tâches** | `tache.cloturee` | Tâche clôturée | Optionnel |
| **Consultations** | `consultation.ouverte` | Consultation ouverte | Optionnel |
| **Consultations** | `consultation.rappel_7j` | Rappel consultation ouverte > 7 jours | Optionnel |
| **Signalements** | `signalement.ouvert` | Signalement ouvert | Optionnel |
| **Signalements** | `signalement.urgent` | Signalement urgent | Optionnel |
| **Signalements** | `signalement.cloture` | Signalement clôturé | Optionnel |
| **Contrats** | `contrat.reconduction_imminente` | Reconduction imminente (J-30) | Optionnel |
| **Contrats** | `contrat.expire` | Contrat expiré | Optionnel |
| **Sinistres** | `sinistre.delai_critique` | Délai de déclaration < 48h | 🔒 Obligatoire |

#### US-PP-005 — Réinitialiser ses préférences

**En tant que** `utilisateur authentifié`,
**je veux** revenir aux valeurs par défaut pour mes préférences non critiques,
**afin de** repartir d'un état propre sans reconfiguration manuelle.

**Critères d'acceptation** :

- **AC-PP-005-A** : L'utilisateur PEUT réinitialiser ses préférences non critiques (notifications, apparence).
- **AC-PP-005-B** : La réinitialisation DOIT être confirmée explicitement via un modal avant exécution.
- **AC-PP-005-C** : La réinitialisation NE DOIT PAS modifier les informations personnelles (nom, email, téléphone).
- **AC-PP-005-D** : Une réinitialisation incomplète (erreur partielle) DOIT afficher un avertissement avec liste des éléments non réinitialisés.
- **AC-PP-005-E** : Le bouton « Revenir aux valeurs par défaut » DOIT être positionné en bas de la page profil, après tous les onglets — une seule action globale couvrant Notifications et Apparence.

#### US-PP-006 — Gérer sa préférence d'apparence

**En tant que** `utilisateur authentifié`,
**je veux** choisir entre les thèmes clair, sombre et automatique,
**afin d'** adapter l'interface à mon contexte d'utilisation.

**Critères d'acceptation** :

- **AC-PP-006-A** : L'utilisateur PEUT sélectionner `light`, `dark` ou `auto`.
- **AC-PP-006-B** : `auto` DOIT suivre la préférence système de l'appareil via `prefers-color-scheme`.
- **AC-PP-006-C** : Le changement de thème DOIT être visible immédiatement sans rechargement de page.
- **AC-PP-006-D** : La préférence DOIT être persistée en base et restaurée à la reconnexion sur tout appareil.

#### US-PP-007 — Afficher le contexte utilisateur

**En tant que** `utilisateur authentifié`,
**je veux** voir mon rôle et mon contexte de résidence dans l'espace profil,
**afin de** comprendre mon périmètre d'action sans chercher l'information ailleurs.

**Critères d'acceptation** :

- **AC-PP-007-A** : Le profil DOIT afficher le rôle de l'utilisateur et son contexte de résidence.
- **AC-PP-007-B** : Ces informations DOIVENT être présentées en lecture seule, clairement distinctes des champs éditables.
- **AC-PP-007-C** : Un rôle absent ou non synchronisé DOIT déclencher une alerte fonctionnelle invitant l'utilisateur à contacter son syndic.

---

## 6. Exigences non fonctionnelles

| ID | Catégorie | Exigence | Seuil mesurable |
|---|---|---|---|
| NFR-Perf-001 | Performance | Chargement initial du profil | ≤ 300 ms (p95) |
| NFR-Perf-002 | Performance | Sauvegarde des préférences | ≤ 200 ms (p95) |
| NFR-Sec-001 | Sécurité | Isolation des données utilisateur | Aucune donnée d'un autre utilisateur accessible — 0 exception |
| NFR-Privacy-001 | Vie privée | Accès cross-user aux préférences | Aucun accès autorisé, y compris depuis un rôle admin — 0 exception |
| NFR-Privacy-002 | Vie privée | Conservation des préférences | Supprimées à l’anonymisation du compte — base légale : intérêt légitime (RGPD art. 6.1.f) |

---

## 7. Hors scope

> **CRITICAL** — Les fonctionnalités suivantes NE DOIVENT PAS être implémentées dans ce lot. Tout agent DOIT rejeter toute demande d'implémentation de ces éléments.

- Gestion du mot de passe (modification, réinitialisation).
- MFA, passkeys et autres mécanismes de sécurité avancée du compte.
- Vérification de la nouvelle adresse email lors du changement (email verification flow) — mise à jour directe pour le MVP.
- Export RGPD détaillé des données personnelles.
- Gestion de suppression de compte.
- Administration des préférences d'un autre utilisateur par un rôle tiers.
- Préférences fines par type d'événement (selon maturité du moteur de notifications — lot futur).

---

## 8. Métriques de succès

| Métrique | Cible | Délai |
|---|---|---|
| Taux de profils avec au moins une préférence définie explicitement | ≥ 60 % | À 3 mois post-déploiement |
| Taux de retour sur la page profil dans les 7 jours suivant la première visite | ≥ 40 % | À 3 mois |
| Taux d'erreurs de sauvegarde (5xx) | < 0,1 % | Continu |

---

## 9. Questions ouvertes

| # | Question | Owner | Condition de révision |
|---|---|---|---|
| ~~Q-PP-001~~ | ✅ **Résolu** — Granularité par événement. 16 événements catalogués (3 obligatoires toggle désactivé, 13 optionnels). Catalogue figé dans AC-PP-004-F. | Damien | 2026-05-20 |
| ~~Q-PP-002~~ | ✅ **Résolu** — Champs éditables : email (login unique) + téléphone. Nom (`full_name`) = `prefilled` depuis EP-002, non modifiable. Email verification hors scope MVP. | Damien | 2026-05-20 |
| ~~Q-PP-003~~ | ✅ **Résolu** — Un seul bouton global en bas de page (couvre Notifications + Apparence). AC-PP-005-E ajouté. | Équipe | 2026-05-20 |

---

## Appendice A — Index de traçabilité

| US | Priorité | ACs | Domaine CRUD |
|---|---|---|---|
| US-PP-001 | Must | AC-PP-001-A à D | Profil personnel (R) |
| US-PP-002 | Must | AC-PP-002-A à E | Profil personnel (U) |
| US-PP-003 | Must | AC-PP-003-A à D | Profil personnel (première visite) |
| US-PP-004 | Must | AC-PP-004-A à E | Notifications personnelles (U) |
| US-PP-005 | Should | AC-PP-005-A à D | Notifications + Apparence (reset) |
| US-PP-006 | Must | AC-PP-006-A à D | Apparence (U) |
| US-PP-007 | Should | AC-PP-007-A à C | Contexte de compte (R) |

# PRD - Profil et préférences

> **Statut** — en cours
> **Dernière modification** — 19 mai 2026
> **Références** — `EP-019`, `DDR-004`, `LEGAL.md §8`

## 1. En-tête et contexte

### Objectif de la feature

Offrir à chaque utilisateur authentifié un **espace personnel simple** pour gérer :

- ses informations personnelles utiles à l'usage de l'Application ;
- ses préférences de notification ;
- sa préférence d'apparence.

### Pourquoi

La feature ne cherche pas à devenir un centre de sécurité avancé. Elle doit avant tout :

- éviter que des réglages personnels soient dispersés ;
- rendre l'expérience plus stable d'un appareil à l'autre ;
- permettre à l'utilisateur de contrôler ce qu'il reçoit et comment il perçoit l'interface.

### KPIs

Aucun KPI prioritaire n'est retenu à ce stade. La feature reste un socle de confort et de personnalisation.

## 2. Profils utilisateurs et cas d'usage

### Personas

| Persona | Besoin principal | Attente clé |
|---|---|---|
| Syndic bénévole | Retrouver ses réglages personnels | Gagner du temps dans un usage fréquent |
| Conseil syndical | Ajuster ses notifications et son affichage | Ne pas subir trop de bruit applicatif |
| Copropriétaire | Gérer ses coordonnées utiles | Garder une expérience simple et lisible |
| Locataire | Adapter l'interface à son usage | Maîtriser le niveau d'information reçu |

### User stories

- En tant que `syndic`, je veux mettre à jour mes informations personnelles pour garder un compte fiable.
- En tant que `utilisateur`, je veux comprendre dès ma première visite quels réglages sont déjà définis et lesquels restent à compléter.
- En tant que `utilisateur`, je veux choisir mes notifications pour ne recevoir que l'utile.
- En tant que `utilisateur`, je veux choisir l'apparence claire, sombre ou automatique pour adapter l'interface à mon contexte.
- En tant que `utilisateur`, je veux retrouver mes réglages dans un seul écran à onglets.

## 3. Exigences fonctionnelles

### MoSCoW — actions et périmètre

| Priorité | Action | Attendu produit |
|---|---|---|
| Must | Consulter son profil | Vue personnelle lisible avec informations et préférences |
| Must | Modifier ses informations personnelles | Nom d'usage, coordonnées utiles, informations éditables par l'utilisateur |
| Must | Gérer ses préférences de notifications | Activation, désactivation et lisibilité du périmètre de notification |
| Must | Gérer sa préférence d'apparence | Clair, sombre ou automatique |
| Must | Gérer la première visite et les valeurs par défaut | État initial compréhensible, sans ambiguïté sur les réglages actifs |
| Should | Afficher le contexte utilisateur | Rôle, résidence, et périmètre de compte compréhensible |
| Should | Expliquer les effets des réglages | Aides courtes, non techniques |
| Should | Permettre un retour simple aux réglages par défaut | Surtout pour les préférences non critiques |
| Could | Préférences fines par type d'événement | Selon la maturité du moteur de notifications |
| Won't (lot actuel) | Gestion avancée de sécurité du compte | Hors feature simple actuelle |

### Actions détaillées

#### 3.1 Consulter son profil

**Logique métier**

- Chaque utilisateur authentifié dispose d'un espace personnel unique.
- La page doit centraliser les informations éditables et les préférences dans un écran à onglets.
- Les éléments non éditables doivent être clairement distingués des champs maîtrisables par l'utilisateur.

**Cas d'erreur**

- session expirée ;
- profil introuvable ou incomplet ;
- informations non chargées ;
- confusion entre donnée éditable et donnée imposée par le système.

#### 3.2 Modifier ses informations personnelles

**Logique métier**

- L'utilisateur peut modifier les informations dont il est propriétaire dans le produit.
- Les données structurantes ou juridiques qui ne relèvent pas de cette feature ne doivent pas apparaître comme éditables.
- Les changements doivent être confirmés et relus facilement.
- Le produit doit distinguer explicitement les champs `éditables`, `préremplis` et `non modifiables`.

**Cas d'erreur**

- champ invalide ;
- format de téléphone ou email non acceptable ;
- tentative de modifier une donnée verrouillée ;
- conflit de sauvegarde.

#### 3.2 bis Gérer la première visite et les valeurs par défaut

**Logique métier**

- Lors de la première visite, l'utilisateur doit comprendre l'état initial de son profil sans découvrir des réglages implicites cachés.
- Les préférences doivent afficher une valeur par défaut lisible, même lorsque l'utilisateur n'a encore rien personnalisé.
- `Automatique` doit être présenté comme la valeur par défaut naturelle pour l'apparence si aucune préférence explicite n'a été choisie.

**Cas d'erreur**

- écran vide sans explication ;
- préférence active non visible ;
- divergence entre valeur par défaut affichée et comportement réel.

#### 3.3 Gérer ses préférences de notifications

**Logique métier**

- L'utilisateur doit pouvoir décider quels signaux applicatifs il souhaite recevoir.
- Les préférences doivent rester compréhensibles même si tous les canaux ne sont pas encore actifs.
- Le produit doit distinguer ce qui est désactivable de ce qui relève d'une information obligatoire ou systémique.

**Cas d'erreur**

- combinaison incohérente de préférences ;
- canal indisponible ;
- sauvegarde partielle ;
- libellé de préférence incompréhensible pour l'utilisateur.

#### 3.3 bis Réinitialiser ses préférences

**Logique métier**

- L'utilisateur doit pouvoir revenir facilement à un état par défaut pour les préférences non critiques.
- La réinitialisation doit être explicite et lisible, sans effet de bord sur les données personnelles.

**Cas d'erreur**

- réinitialisation partielle non signalée ;
- retour à un état par défaut différent de celui annoncé.

#### 3.4 Gérer sa préférence d'apparence

**Logique métier**

- La préférence d'apparence propose `clair`, `sombre` et `automatique`.
- `Automatique` suit la préférence système de l'appareil.
- Le changement doit être visible immédiatement sans dégrader la navigation.

**Cas d'erreur**

- préférence non persistée ;
- divergence entre l'interface affichée et le réglage choisi ;
- comportement système indisponible sur certains contextes.

#### 3.5 Afficher le contexte utilisateur

**Logique métier**

- Le profil doit rappeler le rôle de l'utilisateur et son contexte de résidence sans surcharger l'écran.
- Le produit doit aider l'utilisateur à comprendre ce qu'il peut faire sans transformer la page profil en centre d'administration.
- Les informations de contexte non éditables doivent être présentées comme telles.

**Cas d'erreur**

- rôle absent ou non synchronisé ;
- informations de résidence manquantes ;
- confusion entre préférences personnelles et paramètres copropriété.

## 4. Sécurité et permissions

### Matrice CRUD simplifiée

| Domaine | Entrée UX principale | Syndic | Conseil | Copropriétaire | Locataire |
|---|---|---|---|---|---|
| Profil personnel | Espace profil / onglet `Compte` | `R1 / U1` | `R1 / U1` | `R1 / U1` | `R1 / U1` |
| Notifications personnelles | Espace profil / onglet `Notifications` | `R1 / U1` | `R1 / U1` | `R1 / U1` | `R1 / U1` |
| Apparence | Espace profil / onglet `Apparence` | `R1 / U1` | `R1 / U1` | `R1 / U1` | `R1 / U1` |
| Contexte de compte en lecture | Espace profil / résumé compte | `R1` | `R1` | `R1` | `R1` |

### Règles de sécurité

- La feature ne couvre pas la sécurité avancée du compte dans ce lot.
- Aucune donnée personnelle d'un autre utilisateur ne doit être visible depuis cette page.
- Les préférences doivent rester strictement personnelles et non administrables par un autre rôle métier.

## 5. UX/UI

### User flows attendus

#### Flow 1 — Consulter son profil

1. L'utilisateur ouvre son espace personnel.
2. Il retrouve un écran unique à onglets.
3. Il identifie rapidement ce qui relève du compte, des notifications et de l'apparence.

#### Flow 1 bis — Première visite

1. L'utilisateur ouvre son espace personnel pour la première fois.
2. Il voit clairement les champs encore à compléter et les préférences déjà positionnées par défaut.
3. Il comprend ce qui est modifiable immédiatement et ce qui relève du système.

#### Flow 2 — Modifier ses informations

1. L'utilisateur entre dans l'onglet `Compte`.
2. Il modifie ses informations personnelles autorisées.
3. Il enregistre et reçoit une confirmation claire.

#### Flow 3 — Régler ses préférences

1. L'utilisateur ouvre l'onglet `Notifications` ou `Apparence`.
2. Il ajuste ses choix.
3. Le changement est pris en compte immédiatement ou confirmé explicitement.

### Cas d'erreur et états UX

- État vide : profil incomplet lors de la première visite.
- État première visite : aides courtes sur les valeurs par défaut et les champs à compléter.
- Erreur de sauvegarde : message bref, action de réessai claire.
- Préférence indisponible : explication fonctionnelle, pas technique.
- Donnée non modifiable : affichage en lecture seule, sans faux espoir d'édition.
- Réinitialisation des préférences : confirmation simple et effet immédiatement visible.
- Succès : confirmation discrète, sans interrompre le parcours.

## 6. Hors scope et questions ouvertes

### Hors scope actuel

- gestion du mot de passe ;
- MFA, passkeys et autres mécanismes de sécurité avancée ;
- export RGPD détaillé des données personnelles ;
- gestion de suppression de compte.

### Questions ouvertes

- Quelle granularité de préférences de notifications faut-il exposer sans complexifier excessivement l'écran ?
- Quelle liste exacte de champs doit rester modifiable par l'utilisateur versus gérée ailleurs ?
- Faut-il prévoir un bouton explicite `revenir aux valeurs par défaut` ou un comportement plus discret par section ?
- Faut-il prévoir un résumé plus pédagogique des droits utilisateur dans cet écran ?
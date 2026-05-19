# PRD - Assemblée Générale

> **Statut** — en cours
> **Dernière modification** — 19 mai 2026
> **Références** — `EP-003`, `EP-013`, `LEGAL.md §3`, `ADR-009`

## 1. En-tête et contexte

### Objectif de la feature

Permettre au syndic bénévole de **préparer, tenir et clôturer une Assemblée Générale de manière juridiquement exploitable**, depuis la convocation jusqu'à la notification du PV, en distinguant clairement :

- la préparation juridique avant séance ;
- la conduite de séance ;
- le traitement post-AG et la preuve documentaire.

### Pourquoi

L'AG est le point de bascule entre décision collective, opposabilité et exécution. La feature doit réduire trois risques majeurs :

- risque de non-conformité documentaire avant convocation ;
- risque d'erreur de calcul ou de procédure pendant la séance ;
- risque de perte de traçabilité après clôture, en particulier sur le PV et sa notification.

### KPIs

- Délai moyen entre `clôture de séance` et `notification du PV`.
- Taux d'AG convoquées avec dossier complet avant envoi.
- Taux de résolutions finalisées sans correction manuelle post-séance.

## 2. Profils utilisateurs et cas d'usage

### Personas

| Persona | Besoin principal | Attente clé |
|---|---|---|
| Syndic bénévole | Piloter l'ensemble du cycle AG | Sécuriser la conformité sans outil expert compliqué |
| Conseil syndical | Contrôler le dossier et suivre la séance | Avoir de la visibilité sans porter l'administration complète |
| Copropriétaire | Comprendre, participer, voter | Voir les informations utiles et exercer son droit de vote |
| Locataire | Suivre les décisions sans voter | Accéder à l'information utile sans confusion sur ses droits |

### User stories

#### Préparation avant séance

- En tant que `syndic`, je veux créer un dossier d'AG complet pour sécuriser la convocation avant l'envoi.
- En tant que `syndic`, je veux rédiger, ordonner et qualifier les résolutions pour préparer un vote juridiquement exploitable.
- En tant que `syndic`, je veux voir ce qui bloque la convocation pour ne pas envoyer une AG incomplète.
- En tant que `conseil syndical`, je veux relire le dossier préparatoire pour vérifier le contenu avant la séance sans devenir administrateur du cycle AG.

#### Conduite de séance

- En tant que `syndic`, je veux démarrer la séance dans un parcours ordonné pour limiter les oublis et figer ce qui doit l'être.
- En tant que `syndic`, je veux désigner les rôles de séance pour documenter correctement le bureau de séance.
- En tant que `copropriétaire`, je veux exprimer mon vote par résolution selon ma situation de présence ou de représentation.
- En tant que `syndic`, je veux voir immédiatement les résultats et les cas de passerelle 25-1 ou 26-1 pour conduire la séance sans recalcul manuel.

#### Post-AG et preuve

- En tant que `syndic`, je veux générer un projet de PV fidèle à la séance pour accélérer la formalisation du compte rendu.
- En tant que `syndic`, je veux rattacher le PV signé et tracer sa notification pour sécuriser l'opposabilité des décisions.
- En tant que `copropriétaire`, je veux retrouver les décisions et leurs résultats pour comprendre ce qui a été adopté.
- En tant que `locataire`, je veux consulter les décisions accessibles et le PV sans être placé dans un faux parcours de vote.

## 3. Exigences fonctionnelles

### MoSCoW — actions et périmètre

| Priorité | Action | Attendu produit |
|---|---|---|
| Must | Créer une AG | Dossier brouillon avec date, type, ordre du jour et contraintes de calendrier |
| Must | Modifier ou supprimer un brouillon | Tant que l'AG n'est pas convoquée |
| Must | Gérer les résolutions | Ajouter, modifier, supprimer, typer, ordonner |
| Must | Gérer le dossier de convocation | Pièces obligatoires, checklist, alertes bloquantes |
| Must | Convoquer l'AG | Verrouiller le niveau de préparation nécessaire à l'envoi |
| Must | Gérer la présence | Présences, procurations, correspondance, base de quorum |
| Must | Démarrer la séance | Passage en séance avec gel des éléments nécessaires |
| Must | Saisir les votes et calculer les résultats | Application des règles art. 24, 25, 25-1, 26, 26-1 et unanimité |
| Must | Clôturer la séance | Figer les votes, la présence et les résultats |
| Must | Générer et publier le PV | Projet de PV, dépôt du PV signé, traçabilité |
| Must | Notifier le PV | Action explicite post-AG avec suivi de délai |
| Should | Désigner les rôles de séance | Président, secrétaire, scrutateurs |
| Should | Gérer le quorum en temps réel | Vue séance lisible et mise à jour immédiate |
| Should | Gérer les passerelles 25-1 et 26-1 | Proposition et traçabilité des secondes délibérations |
| Could | Notes collaboratives par résolution | Arguments, amendements de forme, matière pour le projet de PV |
| Could | Circuit de validation du projet de PV | Relecture structurée avant signature finale |
| Won't (lot actuel) | Signature électronique probante multi-canal | Hors lot Product Hub exemplaire, à cadrer ensuite |

### Actions détaillées

#### 3.1 Créer une AG

**Logique métier**

- La création ouvre une AG en statut `brouillon`.
- La date de l'AG pilote le calcul des jalons juridiques, notamment la date limite de convocation.
- L'AG doit être rattachée à une copropriété et à un ordre du jour exploitable.

**Cas d'erreur**

- date de séance manquante ou incohérente ;
- type d'AG absent ;
- ordre du jour vide ;
- tentative de création en doublon manifeste pour une même séance.

#### 3.2 Modifier ou supprimer un brouillon

**Logique métier**

- Tant que l'AG n'est pas convoquée, le syndic peut ajuster les métadonnées, l'ordre du jour et les résolutions.
- Une AG déjà convoquée ou tenue ne doit plus suivre le même niveau de mutabilité.
- La suppression n'est autorisée que sur un brouillon sans pièce ou conséquence déjà engagée.

**Cas d'erreur**

- modification demandée après changement d'état bloquant ;
- suppression interdite car convocation déjà envoyée ;
- conflit de version si deux personnes éditent le brouillon en parallèle.

#### 3.3 Gérer les résolutions

**Logique métier**

- Une résolution comporte au minimum un titre, un texte, un ordre et une règle de majorité.
- Le système doit permettre d'ajouter, modifier, supprimer et réordonner les résolutions avant le verrouillage de séance.
- Les résolutions votées deviennent non modifiables sur leur substance après clôture, conformément au principe d'immutabilité de la décision prise.

**Cas d'erreur**

- résolution sans règle de majorité ;
- ordre en doublon ;
- suppression d'une résolution déjà votée ;
- tentative de modification d'une résolution figée.

#### 3.4 Gérer le dossier de convocation

**Logique métier**

- Le dossier de convocation doit rendre visible les pièces obligatoires et les éléments manquants.
- La feature doit distinguer les pièces structurantes du simple contexte documentaire.
- L'utilisateur doit voir clairement ce qui bloque juridiquement la convocation.

**Cas d'erreur**

- pièce obligatoire absente ;
- preuve d'envoi manquante ;
- formulaire de vote par correspondance absent quand requis ;
- document illisible, supprimé ou non rattaché correctement.

#### 3.5 Convoquer l'AG

**Logique métier**

- La convocation est une transition d'état explicite.
- Le système doit empêcher la convocation si le dossier n'est pas conforme au minimum requis.
- L'horodatage et le canal de convocation doivent rester traçables.

**Cas d'erreur**

- délai de convocation insuffisant ;
- checklist incomplète ;
- AG déjà convoquée ;
- canal ou date d'envoi absents.

#### 3.6 Gérer la présence et le quorum

**Logique métier**

- La présence est gérée au niveau des lots avec les statuts adéquats : présent, représenté, correspondance, absent.
- Le quorum affiché doit se baser sur l'assiette valide au moment de la séance.
- Les lots archivés ou non éligibles doivent être exclus des calculs.

**Cas d'erreur**

- lot compté plusieurs fois ;
- procuration incohérente ;
- vote saisi sans présence valide ;
- données de tantièmes indisponibles ou gel incohérent.

#### 3.7 Démarrer la séance

**Logique métier**

- Le démarrage marque l'entrée dans la conduite de séance.
- Cette transition fige l'ordre du jour effectif et l'assiette de calcul pertinente.
- Les rôles de séance doivent pouvoir être saisis à ce moment-là.

**Cas d'erreur**

- démarrage tenté sur une AG non convoquée ;
- rôles obligatoires manquants selon le niveau de complétude attendu ;
- présence non préparée ;
- action doublonnée sur une AG déjà en séance.

#### 3.8 Saisir les votes et calculer les résultats

**Logique métier**

- Le vote se fait résolution par résolution, lot par lot, avec calcul automatique du résultat.
- Le moteur doit gérer les règles art. 24, 25, 25-1, 26, 26-1 et unanimité.
- Les résolutions éligibles à une seconde délibération doivent être signalées sans ambiguïté.

**Cas d'erreur**

- résolution incomplète ;
- vote hors séance ;
- vote d'un lot non éligible ;
- majorité introuvable ou mal paramétrée ;
- recalcul impossible faute de base de présence cohérente.

#### 3.9 Gérer les passerelles art. 25-1 et 26-1

**Logique métier**

- La feature doit détecter automatiquement les cas de redélibération légale.
- La seconde délibération doit rester explicitement liée à la résolution d'origine.
- La traçabilité doit montrer pourquoi une bascule a été proposée.

**Cas d'erreur**

- tentative de créer manuellement une passerelle non éligible ;
- double redélibération sur la même résolution ;
- résultat initial non finalisé.

#### 3.10 Clôturer la séance

**Logique métier**

- La clôture est une transition d'état bloquante.
- À la clôture, les votes, la feuille de présence et les résultats deviennent figés.
- Les corrections ultérieures ne doivent plus passer par un simple parcours d'édition libre.

**Cas d'erreur**

- clôture alors que des résolutions restent incomplètes ;
- tentative de reclôture ;
- présence ou votes incohérents au moment de figer.

#### 3.11 Générer, relire, signer et publier le PV

**Logique métier**

- Le système doit produire un projet de PV structuré à partir des données figées.
- Le dépôt du PV signé doit être distinct du projet de PV.
- Une fois le PV signé publié, la matière de séance ne peut plus être retouchée librement.

**Cas d'erreur**

- génération impossible car séance non clôturée ;
- PV signé manquant ou non rattaché ;
- tentative de republier un PV déjà attaché sans workflow de remplacement explicite ;
- signatures incomplètes selon le niveau de preuve attendu.

#### 3.12 Notifier le PV

**Logique métier**

- La notification est une action explicite post-AG, distincte du dépôt du PV.
- La feature doit tracer la date de notification aux opposants et défaillants.
- L'indicateur de délai doit être lisible par le syndic pour éviter un oubli juridique.

**Cas d'erreur**

- notification tentée sans PV signé ;
- notification déjà marquée ;
- date de notification incohérente ;
- destinataires opposants/défaillants incomplets ou impossibles à qualifier.

## 4. Sécurité et permissions

### Matrice CRUD simplifiée

| Domaine | Entrée UX principale | Syndic | Conseil | Copropriétaire | Locataire |
|---|---|---|---|---|---|
| Dossier d'AG | Liste AG + fiche AG | `C / R / U / D` | `R` | `R1` | `R1` |
| Résolutions | Fiche AG / bloc résolutions | `C / R / U / D` | `R` | `R1` | `R1` |
| Convocation | Fiche AG / dossier de convocation | `C / R / U / D1` | `R` | `R1` | `R1` |
| Rôles de séance | Vue séance / panneau bureau de séance | `C / R / U / D1` | `R` | `R1` | `R1` |
| Présence et quorum | Fiche AG / onglet présence | `C / R / U / D` | `R` | `R1` | `R1` |
| Votes et résultats | Vue séance / résolution courante | `C / R / U / D` | `R` | `C / R1 / U1` | `R1` |
| PV | Fiche AG / section PV | `C / R / U / D1` | `R` | `R1` | `R1` |
| Notification du PV | Fiche AG / section notification | `C / R / U / D1` | `R` | `R1` | `R1` |

### Règles de sécurité

- Les actions de transition d'état doivent être explicites et non fondues dans un `Modifier` générique.
- Les données votées et les éléments figés de séance suivent un principe d'immutabilité après clôture.
- Les locataires n'entrent jamais dans un parcours de vote.
- Les droits de lecture résidentielle doivent distinguer l'information consultable du pouvoir d'action.

## 5. UX/UI

### User flows attendus

#### Flow 1 — Préparation avant séance

1. Le syndic crée un brouillon d'AG.
2. Il construit l'ordre du jour et les résolutions.
3. Il attache les pièces nécessaires.
4. Il vérifie la checklist de convocation.
5. Il convoque l'AG.

#### Flow 2 — Conduite de séance

1. Le syndic ouvre la vue séance.
2. Il confirme les rôles de séance et la présence.
3. Il suit le quorum et avance résolution par résolution.
4. Il saisit les votes, voit le résultat et les éventuelles passerelles 25-1 / 26-1.
5. Il clôture la séance.

#### Flow 3 — Post-AG et PV

1. Le syndic génère le projet de PV.
2. Il récupère ou attache le PV signé.
3. Il marque la notification aux absents/opposants.
4. Les copropriétaires et locataires consultent le résultat selon leurs droits.

#### Flow 4 — Consultation résidentielle

1. Le copropriétaire ou le locataire ouvre la fiche AG.
2. Il consulte les résolutions, les résultats et les documents visibles selon son rôle.
3. Le copropriétaire accède à la trace de vote et au PV ; le locataire reste dans un parcours de consultation sans action de vote.

### Cas d'erreur et états UX

- État vide : aucune AG créée.
- État brouillon incomplet : checklist visuelle et éléments bloquants lisibles.
- État convocation bloquée : raison affichée sans jargon technique.
- État bureau de séance incomplet : éléments manquants explicités avant démarrage ou clôture.
- État séance verrouillée : les éléments non modifiables sont affichés comme figés, pas comme simples champs désactivés.
- État erreur de vote : identifier la résolution, le lot et la règle en cause.
- État consultation résidentielle : n'afficher que les actions réellement autorisées par rôle.
- État post-clôture : toute tentative de modification doit expliquer pourquoi l'action n'est plus possible.
- État notification manquante : rappel clair du délai restant.

## 6. Hors scope et questions ouvertes

### Hors scope actuel

- signature électronique probante bout-en-bout ;
- envoi probant multi-canal automatisé ;
- co-édition temps réel riche de notes de séance ;
- gestion des délégations complexes du conseil syndical hors flux principal.

### Questions ouvertes

- Quel niveau de preuve documentaire viser pour la notification du PV au-delà du simple horodatage ?
- Quel workflow de correction autoriser si un PV signé doit être remplacé ?
- Faut-il séparer visuellement plus fortement la phase `préparation juridique` de la phase `conduite de séance` ?
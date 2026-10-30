---
name: pr-review
description: "Protocole de revue de pull request. Charger pour toute demande de revoir/relire une PR, le code, sa maintenabilité ou le respect des principes d'ingénierie logicielle (fonctionnel, légal, DRY, KISS, YAGNI, over-engineering). But : couvrir la spec avec le minimum de code. Se déclenche sur \"revoyit/review la PR\", \"relis ce code\", \"review\", \"/pr-review\"."
---

# Revue de PR

## Principes d'évaluation (tous obligatoires)

- Minimal change : le diff couvre la spec, rien de plus. Tout ajout hors scope est un finding.
- Boyscout : améliorations locales tolérées seulement si elles touchent du code déjà modifié par le diff.
- DRY : duplication à signaler, y compris partielle ou quasi-identique (mappings, tables de constantes, catalogues backend↔frontend, `v-if` répétés).
- KISS : complexité gratuite, abstraction prématurée, code mort.
- YAGNI : rien au-delà de la spec — tri paramétrable, pagination cursor, options non demandées sont des findings ; les seules « extras » tolérées sont celles que la spec exige explicitement (clamp, debounce…).
- SOLID : responsabilité, dépendance à une abstraction, extensions sans modification.
- Aucun commentaire de code référençant des épics/stories/tickets.

## Procédure

- Récupérer le contexte : `gh pr view $PR --json title,body,files,additions,deletions`, puis `gh pr diff $PR`.
- Vérifier que la description respecte le modèle du dépôt : chercher un fichier `pull_request_template` (insensible à la casse) dans `.github/`, à la racine ou dans `docs/` ; comparer les sections attendues à la description ; tout écart est un finding bloquant avant toute lecture du code.
- Lire la spec dans la description de la PR ; si absente ou floue, le signaler comme premier finding.
- Lire le diff complet avant tout avis. Lire les fichiers appelants pour juger des impacts.
- Sur revue incrémentale : ne revoir que le delta depuis la dernière revue ; vérifier explicitement la résolution des findings précédents (tableau finding → statut) et rejouer les vérifications de non-régression sur la zone touchée.
- Classer chaque finding : bloquant / important / mineur.
- Vérifier : la spec est-elle entièrement couverte ? Le code supplémentaire est-il justifié ?

## Contrôle du respect fonctionnel (PRD / AC)

- Vérifier chaque critère d'acceptation de la user story contre le code, un par un (AC-XXX-A, -B, -C…). Ne pas cocher un AC sur la foi de la description : trouver la ligne qui le satisfait.
- **Écart PRD ↔ schéma** : si un AC exige quelque chose que le schéma ne permet pas (ex. états d'enum non encore livrés), ce n'est pas un défaut d'implémentation, mais l'écart DOIT être tracé (amendement PRD ou note de spec) — un AC non couvert non tracé est un finding important.
- Vérifier le comportement par défaut inchangé quand aucun paramètre n'est passé (non-régression du consommateur existant).
- Vérifier les états vides, les bornes (clamp, offset hors plage) et les valeurs invalides (400 propre, pas de fuite d'erreur de cast SQL).

## Contrôle RBAC / légal

- Chaque nouveau handler doit avoir un **check de rôle explicite** (via `domain/permissions.rs`) OU un commentaire documentant l'intention « tout utilisateur authentifié ». Un handler sans check ni commentaire est un finding important — même si préexistant, le signaler (ticket de suivi si hors périmètre).
- Le scoping `building_id` doit être vérifié sur chaque requête : il vient du JWT, jamais d'un paramètre client.
- Vérifier qu'aucune PR n'élargit la visibilité par ricochet : un remapping de permission (ex. `hasRole('manager')` → `canViewX` incluant board_member) peut **changer le comportement** — comparer la sémantique exacte avant/après, pas seulement la lisibilité. Si la description affirme « aucun changement fonctionnel », vérifier chaque remapping ligne à ligne.
- Une règle RBAC ne doit pas casser un flux UI existant : tracer les appelants frontend d'un endpoint dont les droits changent (bouton gated sur une permission X qui appelle un endpoint devenu plus strict = régression fonctionnelle).
- Vérifier l'immuabilité légale quand pertinente (append-only, absence de PUT/DELETE, votes/scrutins figés après clôture) : les routes de mutation ne doivent pas exister.
- La conformité RGPD/légale des données exposées par un nouvel endpoint doit être vérifiée contre le PRD (ex. données personnelles des occupants réservées au syndic).

## Contrôle du code ajouté

- Chaque fichier du diff doit se rattacher à la spec ; sinon proposer suppression ou report.
- Nommer ce qui peut être supprimé, fusionné ou simplifié sans perte de couverture.
- Signaler un test qui ne teste rien ou qui duplique un autre.
- Backend : vérifier les placeholders SQL contigus (`$1..$N`) quand des littéraux sont paramétrés — les macros `sqlx::query!` échouent sur un trou de numérotation, et le runtime peut binder dans le mauvais ordre silencieusement.
- Vérifier les duplications transverses typiques : mapping source→variante dupliqué entre composants, liste de valeurs dupliquée backend↔frontend (miroir assumé mais à commenter croisé), clés i18n qui dupliquent un namespace `common.*` existant.
- Frontend : vérifier l'initialisation des stores au premier rendu (clé localStorage calculée avant hydratation → comportement dégradé), le cleanup des timers/watchers au démontage, et la sémantique des helpers API (tableau vide ≠ paramètre absent).

## Coordination de merge

- Si plusieurs PR ouvertes touchent la même zone de fichiers, signaler le conflit de merge prévisible et recommander un ordre (celle qui fournit les dépendances d'abord, rebase des autres ensuite).
- Si une PR en cours accumule du refactoring transverse sans rapport avec sa spec, demander l'extraction en PR séparée (un revert de la story ne doit pas emporter le refactoring).
- Vérifier que les binaires ajoutés (polices, images) ont leur licence (LICENSE.txt) et que les workflows one-shot de provisionnement sont supprimés de la branche.

## Sortie

Un compte rendu par finding :

- Fichier:ligne, sévérité, principe violé, correction proposée (concrète, pas vague).
- Sur revue incrémentale : tableau des findings précédents avec statut (résolu / non traité / N/A).
- Verdict final : approuvé / approuvé avec remarques / à corriger.
- Liste des points de spec non couverts, même mineurs ; liste des tickets de suivi à créer (écart PRD, dette RBAC récurrente).
- Ne jamais relire en disant seulement "ça a l'air bien" : chaque affirmation s'appuie sur une ligne du diff.

## Règles

- Revue du code, pas des personnes : formulations factuelles.
- Pas de refactor global dans une PR : le signaler pour une PR séparée.
- Ne pas demander de style subjectif : uniquement ce qui est objectivement en défaut au regard des principes ci-dessus.
- Ne pas réclamer de changer une règle métier au passage d'un refactoring : exiger qu'elle soit épinglée par un test + documentée comme dette ; la décision appartient à un PR métier.
- Les échecs de tests CI doivent être tranchés : soit le test matérialise la bonne règle (corriger le code), soit le code a raison (corriger le test) — jamais laisser un test ajusté sans noter la divergence avec le PRD.

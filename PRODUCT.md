# Product

## Register

product

## Users

**Syndic bénévole** (primaire) — Copropriétaire élu, pas nécessairement à l'aise avec le numérique. Il gère l'immeuble entre son travail et sa vie personnelle, souvent depuis son téléphone dans les parties communes ou à son bureau le soir. Son job : administrer, rester conforme, déléguer sans perdre le contrôle. Il a besoin de clarté, de guidage, et de ne jamais se sentir seul face à une obligation légale.

**Conseil syndical** (secondaire) — 2 à 5 membres élus. Ils supervisent, valident avant les AG, consultent la comptabilité. Contexte mixte desktop/mobile. Besoin de transparence et de flux décisionnels clairs.

**Copropriétaire** (tertiaire) — Résident propriétaire. Accède à ses charges, vote en AG, consulte les documents, signale des problèmes, participe aux consultations. Confiance et lisibilité avant tout.

**Locataire** (tertiaire) — Signale des problèmes, participe aux consultations, consulte les résultats d'AG accessibles au public. Accès limité par rôle. Doit se sentir entendu, pas exclu.

**Prestataire** (périphérique) — Accès via lien tokenisé uniquement. Dépose des documents, clôture une tâche. Première visite souvent sur mobile. Zéro apprentissage attendu.

## Product Purpose

La Ruche est une webapp open source de gestion participative de copropriété, conçue pour les syndics bénévoles français. Elle rend la gestion d'immeuble transparente, collaborative et accessible sans les coûts ni la complexité des logiciels professionnels.

Le produit couvre l'intégralité du cycle de vie d'une copropriété : registre légal, AG, budget et appels de fonds, contrats, tâches, sinistres, carnet d'entretien, documents, et conformité réglementaire. Il distingue clairement les actes légaux contraignants (votes AG, appels de fonds) des décisions informelles (consultations, sondages).

Le succès c'est un syndic bénévole qui gère seul son immeuble en conformité, sans avoir besoin d'un cabinet professionnel.

## Brand Personality

Sobre, collaboratif, civique, éducatif.

Ton : direct et humain. Pas de jargon juridique brut, mais pas de condescendance non plus. Le produit parle à des adultes compétents qui ont besoin d'être guidés, pas infantilisés.

Émotion cible : confiance et sérénité. L'utilisateur doit sentir qu'il est bien organisé, que rien ne lui échappe, que la complexité légale est gérée pour lui.

## References

**Positives :**
- **Matera** — clarté du parcours syndic, densité assumée de l'information métier
- **Qonto** — précision des états financiers, hiérarchie des données comptables, feedback instantané sur les actions
- **Goodvest** — pédagogie intégrée à l'interface (tooltips contextuels, wizards progressifs), ton rassurant sans être condescendant
- **Luko** — wizards d'onboarding et de déclaration : progression claire, une seule question par écran, abandon facile

## Anti-references

- **Logiciels syndic traditionnels** (Gestimmo, Syndic One, ICS Copro) — interfaces tabulaires grises, formulaires à 40 champs, aucune guidance, densité brute sans hiérarchie
- **Outils enterprise lourds** (Jira, Monday, SAP) — navigation complexe, tout-sur-tout, sentiment d'usine
- **Portails administratifs français de 2010** — police Times New Roman, boutons aplats gris, arborescences profondes, zéro feedback
- **Dashboards SaaS génériques** — grille de KPI cards identiques, gradient text, hero-metric 56px, purple-on-dark sans motif

## Design Principles

1. **La complexité légale s'explique, elle ne s'impose pas.** Quand la loi contraint une action (délai, format, majorité requise), l'interface l'indique avec le contexte suffisant pour comprendre — jamais un champ vide avec une obligation cachée dans les docs.

2. **Confiance avant action irréversible.** Appels de fonds émis, PV signés, données anonymisées : ces actes sont irréversibles. Le design doit marquer la différence entre préparer et valider, entre brouillon et acte légal. La confirmation n'est pas de la friction, c'est du soin.

3. **Un rôle, une lecture.** Les mêmes données s'affichent différemment selon le rôle. Les restrictions de permission ne se vivent jamais comme une punition : un locataire qui n'a pas accès à la comptabilité voit un état vide clair, pas une erreur 403.

4. **Sobre sur la forme, riche sur le fond.** Pas de décoration gratuite. L'espace blanc, la typographie et la couleur servent l'information, pas l'inverse. La richesse est dans la donnée bien présentée, pas dans l'UI.

5. **Le guidage est une fonctionnalité.** Les wizards, les états vides actifs, les tooltips légaux et les rappels contextuels ne sont pas des détails UX — ils sont le produit pour un syndic bénévole non-technique. Les ignorer revient à abandonner l'utilisateur primaire.

## Accessibility & Inclusion

- WCAG AA minimum sur tout le périmètre. Contraste corps ≥ 4.5:1, éléments critiques ≥ 7:1.
- Touch targets ≥ 44×44px sur toutes les surfaces mobiles.
- Respect de `prefers-reduced-motion` : animations non-essentielles désactivées.
- Jamais la couleur seule comme seul vecteur d'information (états, erreurs, statuts toujours accompagnés d'une icône ou d'un libellé).
- Taille de texte UI par défaut : 13px minimum (`text-sm`), aucun texte en dessous de 11px sauf chips de statut (`text-2xs`).
- Usage mobile prioritaire pour les rôles résidents (copropriétaire, locataire, prestataire). Desktop prioritaire pour les rôles gestionnaires (syndic, conseil syndical).
- Utilisateurs potentiellement peu à l'aise avec le numérique : libellés d'actions explicites (jamais d'icône seule pour une action destructive), messages d'erreur en langage naturel, wizards pour les flux multi-étapes.

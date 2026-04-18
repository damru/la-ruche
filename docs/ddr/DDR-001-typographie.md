# DDR-001 — Typographie

**Date :** 2026
**Statut :** Accepté (v2 — Fraunces remplace Syne)

---

## Contexte

v1 utilisait Syne — géométrique, architecturale, mais trop "tech startup" pour l'identité institutionnelle visée.
L'utilisateur a demandé une typographie serif plus institutionnelle. Après comparatif de 4 candidats (Playfair Display, Fraunces, Lora, DM Serif Display), Fraunces a été retenu.

## Décision

**Display / Titres / Headlines / KPI grands : Fraunces**
- Serif variable avec 3 axes : wght (100–900), opsz (9–144), ital (0–1)
- L'axe optique (opsz) est la caractéristique clé : les lettres grossissent aux grandes tailles, s'affinent aux petites
- Italique très expressif — utilisé pour les headlines secondaires
- Diacritiques français complets (é, è, ê, à, ç, ù, œ, æ)
- Moins répandue que Playfair Display → identité distinctive

**Corps / Labels : DM Sans** — inchangé

## Alternatives rejetées

| Police | Raison |
|---|---|
| Syne | Trop tech startup, pas assez institutionnelle |
| Playfair Display | Trop répandue |
| Lora | Manque de personnalité |
| DM Serif Display | Poids unique, pas variable |

## Échelle typographique

```
display-lg   Fraunces 56px / wght:800 / opsz:144 / ital:0  / ls:-0.04em
display-md   Fraunces 40px / wght:700 / opsz:144 / ital:0  / ls:-0.03em
display-sm   Fraunces 32px / wght:700 / opsz:72  / ital:0  / ls:-0.02em
headline-lg  Fraunces 28px / wght:600 / opsz:72  / ital:0  / ls:-0.02em
headline-md  Fraunces 24px / wght:600 / opsz:48  / ital:1  / ls:-0.01em  ← italique
headline-sm  Fraunces 20px / wght:600 / opsz:36  / ital:0  / ls:0
title-lg     DM Sans  18px / wght:500
title-md     DM Sans  16px / wght:500
body-md      DM Sans  14px / wght:400
label-sm     DM Sans  11px / wght:500 / uppercase / ls:0.04em
```

## CSS

```css
@import url('https://fonts.googleapis.com/css2?family=Fraunces:ital,opsz,wght@0,9..144,100..900;1,9..144,100..900&family=DM+Sans:ital,opsz,wght@0,9..40,300..500;1,9..40,300..400&display=swap');

--font-display: 'Fraunces', Georgia, serif;
--font-body:    'DM Sans', system-ui, sans-serif;
```

## Amendement v3 — JetBrains Mono pour les données numériques tabulaires

**Données numériques / code : JetBrains Mono**
- Chiffres tabulaires (lnum, tnum) → alignement parfait dans les colonnes
- Utilisé pour : montants en tableau, numéros de lot, codes, tokens
- PAS utilisé pour les KPI grands (Fraunces reste pour l'impact visuel)
- Chargement : `family=JetBrains+Mono:wght@400;500`

### Règle des 3 polices

| Contexte | Police | Raison |
|---|---|---|
| Titres, headlines, KPI ≥ 28px | Fraunces | Impact éditorial, chiffres décoratifs |
| Corps, labels, UI | DM Sans | Lisibilité, neutralité |
| Montants tableaux, numéros, codes | JetBrains Mono | Alignement tabulaire, lisibilité data |

Trois polices = limite acceptable. Chacune a un rôle distinct et non substituable.

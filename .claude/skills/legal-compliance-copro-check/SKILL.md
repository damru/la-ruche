---
name: legal-compliance-copro-check
description: >
  Audit French co-ownership law compliance (copropriété) and produce a gap
  report. Fetches consolidated Légifrance texts, runs a gap analysis against
  the LEGAL.md coverage table, and outputs a structured document listing legal
  gaps.
  Trigger phrases: "conformité légale", "mentions obligatoires", "art. 17",
  "art. 42", "décret 67-223", "loi 65-557", "PV AG", "légifrance", "gap
  analysis légal".
argument-hint: "Article or topic to audit (e.g. 'art. 17 décret 67-223')"
---

# Legal Compliance Check (copro)

Audit French co-ownership law (loi n°65-557, décret n°67-223) against the
`docs/LEGAL.md` referential and produce a **gap report document**.

This skill does **not** modify `docs/LEGAL.md`, implement Rust code, or run
tests. It is a read-only audit that outputs a structured report.

## When to Use

- A user cites a legal article and asks whether it is covered in LEGAL.md
- A new law or amendment may affect existing coverage entries
- `docs/LEGAL.md` coverage table needs auditing or updating
- You need a prioritised list of legal gaps before opening backlog stories

---

## Procedure

### Step 1 — Fetch the consolidated legal text

Use `openlegi` MCP with tool `rechercher_dans_texte_legal` to retrieve the
**in-force** (état VIGUEUR) version:

```
search: "<article keyword>"
text_id: "67-223"           # or "65-557" for the loi
champ: "ARTICLE"
```

- Note the `LEGIARTI` identifier and the Légifrance URL.
- Quote the **exact text** verbatim. Never paraphrase legal obligations.
- If `openlegi` MCP is unavailable, ask user if they want to scrape the text from Légifrance and note the fallback in the report.

### Step 2 — Gap analysis

Read `docs/LEGAL.md` and compare each legal obligation in the fetched text
against existing coverage entries.

Produce a table per article:

| # | Obligation légale | Source | Couverture | Écart |
|---|---|---|---|---|
| 1 | … | Art. X §Y | ✅ / ⚠️ / ❌ / 🔲 | Description ou « RAS » |

Coverage codes:
- ✅ **Couverte** — fully addressed
- ⚠️ **Partielle** — partially covered; describe what is missing
- ❌ **Non couverte** — no coverage; include backlog story ref if one exists
- 🔲 **Hors périmètre** — out of software scope; justify

### Step 3 — Produce the gap report

Write the output to `.outputs/legal-reports/legal-gap-report-<YYYY-MM-DD>.md` (create the file).

Report structure:

```markdown
# Rapport d'écarts légaux — <article(s) audités>

> Généré le <date> · Texte(s) consultés : <LEGIARTI IDs>
> Ce rapport est en lecture seule. Pour mettre à jour LEGAL.md, utiliser le Legal Product Advisor.

## Texte légal consulté

**<Article X — Titre>**  
LEGIARTI : `LEGIARTI000XXXXXXXXX`  
URL : <légifrance URL>  

> « <verbatim quote> »

## Tableau d'écarts

| # | Obligation légale | Source | Couverture actuelle | Écart |
|---|---|---|---|---|
| 1 | … | Art. X §Y | ✅ / ⚠️ / ❌ / 🔲 | Description ou « RAS » |

## Recommandations

- **Priorité haute** : <❌ gaps with legal deadline or financial impact>
- **Priorité normale** : <⚠️ partial coverage>
- **Hors périmètre** : <🔲 items with justification>
```

---

## Decision Points

| Situation | Action |
|---|---|
| Légifrance returns 403 | Use domain knowledge of décret 67-223 + note the fallback in report |
| Obligation is clearly out of software scope | Mark 🔲 with justification in report |
| No matching LEGAL.md section | Note the gap in recommendations as « Section absente de LEGAL.md » |

## Quality Checklist

Before marking the skill complete:

- [ ] Legal text quoted verbatim with LEGIARTI ID + URL
- [ ] Gap table produced — every obligation has a status
- [ ] Every ❌ / ⚠️ has a gap description and a backlog reference or « à créer »
- [ ] Output file `.outputs/legal-reports/legal-gap-report-<date>.md` created
- [ ] `docs/LEGAL.md` **not modified**

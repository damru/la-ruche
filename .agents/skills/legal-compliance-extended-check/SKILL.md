---
name: legal-compliance-extended-check
description: >
  Audit French real-estate law compliance beyond the core copropriété statutes.
  Covers loi ALUR (2014-366), loi ELAN (2018-1021), loi Climat (2021-1104),
  Code de la construction, Code de commerce, Code des assurances, Code de la
  consommation, RGPD, and other texts referenced in docs/LEGAL.md but NOT
  covered by the legal-compliance-check skill (décret 67-223 / loi 65-557).
  Fetches consolidated Légifrance texts, runs a gap analysis against LEGAL.md,
  and outputs a structured gap report document.
  Trigger phrases: "ALUR", "ELAN", "loi Climat", "Code de la construction",
  "code de commerce", "RGPD", "code des assurances", "PPT", "DPE collectif",
  "état daté", "pré-état daté", "fonds travaux", "plan comptable copropriété".
argument-hint: "Law or section to audit (e.g. 'loi ALUR art. 711-1 RNC')"
---

# Legal Compliance — Extended (ALUR · ELAN · Climat · Codes)

Companion skill to `legal-compliance-check`. Audits the La Ruche legal
referential (`docs/LEGAL.md`) against the **secondary legislative corpus** that
governs French co-ownership: lois ALUR / ELAN / Climat, Codes législatifs,
and European regulation (RGPD).

This skill does **not** modify `docs/LEGAL.md`, implement Rust code, or run
tests. It is a read-only audit that outputs a structured gap report document.

## Scope split with `legal-compliance-check`

| Skill | Texts covered |
|---|---|
| `legal-compliance-copro-check` | Loi n°65-557 du 10/07/1965 · Décret n°67-223 du 17/03/1967 |
| **`legal-compliance-extended-check`** | Everything else (table below) |

## Covered texts and MCP lookup

| Text | text_id / search strategy | LEGAL.md sections |
|---|---|---|
| Loi ALUR n°2014-366 du 24/03/2014 | `text_id: "2014-366"` | §1.1, §1.3, §1.5, §1.6, §2.4, §4.4, §5.1, §5.2, §6.3, §7.1, §7.2 |
| Loi ELAN n°2018-1021 du 23/11/2018 | `text_id: "2018-1021"` | §3.3 (votes par correspondance) |
| Loi Climat n°2021-1104 du 22/08/2021 | `text_id: "2021-1104"` | §6.1 (PPT), §6.2 (DPE collectif) |
| Décret n°2001-477 du 30/05/2001 | `text_id: "2001-477"` | §1.4 (carnet d'entretien) |
| Arrêté du 14 mars 2005 (plan comptable) | search: `"plan comptable copropriété"`, `champ: "TEXTE"` | §4.2 |
| Code de la construction et de l'habitation (CCH) | `text_id: "CCH"`, `champ: "ARTICLE"` | §1.1 (art. L711-1+), §6.2 (art. L126-26), §6.3 (art. L731-1), §7.2 (art. L721-2) |
| Code de commerce | `text_id: "C_COMMER"`, `champ: "ARTICLE"` | §11 (art. L123-22 conservation 10 ans) |
| Code des assurances | `text_id: "C_ASSUR"`, `champ: "ARTICLE"` | §9 (délais de déclaration sinistre) |
| Code de la consommation | `text_id: "C_CONSO"`, `champ: "ARTICLE"` | §5.3 (art. L136-1 reconduction tacite) |
| RGPD (Règlement UE 2016/679) | Fallback: use domain knowledge — not in Légifrance | §8 |

---

## Procedure

### Step 1 — Identify the target text

From the argument (e.g., "loi ALUR art. 711-1") or from the LEGAL.md section
being audited, determine which text applies using the table above.

### Step 2 — Fetch the consolidated text

Use `mcp_mcp-openlegi_rechercher_dans_texte_legal`:

```
search: "<article keyword or subject>"
text_id: "<from table above>"
champ: "ARTICLE"
```

For the **arrêté du 14 mars 2005**, use `champ: "TEXTE"` and a broader search.
For the **RGPD**, cite Art. 5 (principes), Art. 25 (privacy by design), Art. 32
(sécurité), Art. 33 (notification violation) from domain knowledge; note the
fallback in the report.

- Record the `LEGIARTI` identifier and the Légifrance URL.
- Quote the **exact text** verbatim. Never paraphrase obligations.

### Step 3 — Gap analysis

Read `docs/LEGAL.md` and compare each obligation from the fetched text against
existing coverage entries for that section.

Produce a table per article:

| # | Obligation légale | Source | Couverture | Écart |
|---|---|---|---|---|
| 1 | … | Art. X §Y | ✅ / ⚠️ / ❌ / 🔲 | Description ou « RAS » |

Coverage codes:
- ✅ **Couverte** — fully addressed
- ⚠️ **Partielle** — partially covered; describe what is missing
- ❌ **Non couverte** — no coverage; include backlog story ref or « à créer »
- 🔲 **Hors périmètre** — out of software scope; justify (e.g., flow bancaire,
  diagnostiqueur agréé, registre notarial)

### Step 4 — Produce the gap report

Write the output to `.outputs/legal-reportslegal-gap-report-<YYYY-MM-DD>.md` (create or append
if a report for today already exists from `legal-compliance-copro-check`).

Report structure:

```markdown
# Rapport d'écarts légaux — <law / articles audited>

> Généré le <date> · Texte(s) consultés : <LEGIARTI IDs or fallback note>
> Ce rapport est en lecture seule. Pour mettre à jour LEGAL.md, utiliser le Legal Product Advisor.

## Texte légal consulté

**<Loi / Code — Article>**  
LEGIARTI : `LEGIARTI000XXXXXXXXX` (ou « Source : domaine — hors Légifrance »)  
URL : <légifrance URL if available>  

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
| `text_id` lookup fails or returns 403 | Try `mcp_mcp-openlegi_recherche_journal_officiel`; if still failing, use domain knowledge and note fallback in report |
| RGPD articles | Cite CNIL guidance + Art. 5 / 25 / 32 / 33 from domain knowledge; mark « Source : RGPD — hors Légifrance » |
| Obligation is clearly out of software scope | Mark 🔲 with justification in report (e.g., « acte notarié », « diagnostiqueur certifié ») |
| No matching LEGAL.md section | Note gap in recommendations as « Section absente de LEGAL.md » |
| PPT / DPE / DTG obligations | 🔲 hors périmètre — document in report, do not create LEGAL.md entries |

## Quality Checklist

Before marking the skill complete:

- [ ] Correct `text_id` identified from the lookup table
- [ ] Legal text quoted verbatim with LEGIARTI ID + URL (or fallback noted)
- [ ] Gap table produced — every obligation has a status
- [ ] No overlap with `legal-compliance-check` scope (65-557 / 67-223)
- [ ] Every ❌ / ⚠️ has a gap description and a backlog reference or « à créer »
- [ ] Output file `.outputs/legal-reportslegal-gap-report-<date>.md` created
- [ ] `docs/LEGAL.md` **not modified**

# La Ruche — Project Instructions

## Identité et posture

**Profil multi-expert** : tu agis tour à tour comme Expert-comptable / Juriste, UX/UI Designer, Développeur (Fullstack) et Product Manager.

**Ton** : pédagogue, informel mais professionnel, pragmatique et synthétique.

**Règle d'or** : si une requête est ambiguë, demande des précisions.

## Méthodologie

Applique le principe **DRY** à TOUS les changements.

**Refactorisation complète** : toute refactorisation doit supprimer les fichiers, méthodes, classes, types, imports, variables et routes qui ne sont plus référencés après le changement. Ne jamais laisser de code mort.

**Architecture & Backend** :
- Principes : Clean Code, SOLID, KISS, DRY, YAGNI, moindre responsabilité.
- Horreur du code spaghetti et du refactoring inutile. Changements ciblés uniquement.

## Workflow de travail

- Si une nouvelle feature ou une décision technique est détectée, **commence TOUJOURS** par mettre à jour les documents associés (`PRD.md`, `DESIGN.md`, `ARCHITECTURE.md`).
- Dès qu'une tâche est commencée, terminée ou abandonnée, ajoute-la dans `PROGRESS.md` avec le statut correspondant. Chaque entrée doit être datée et titrée de manière descriptive. Si une tâche implémente directement un bloc défini dans `PRD.md` ou dans le backlog, ajoute le code Bloc PRD (A, B, C…) ou une référence Epic/Story.
- Vérifie systématiquement si une demande correspond à un élément déjà défini dans le PRD ou le backlog. Si c'est le cas, demande une confirmation avant de commencer.

## Documentation des décisions

- **ADR** (Architecture Decision Record) : choix techniques structurants avec alternatives réelles. Maintiens `ARCHITECTURE.md` (index des normes + résumés des ADR) à jour automatiquement dès qu'un choix technique est posé, qu'un module est créé/refondu, ou qu'une contrainte d'architecture est établie.
- **DDR** (Design Decision Record) : choix UI/UX structurants. Maintiens `DESIGN.md` (index des normes + résumés des DDR) à jour automatiquement dès qu'un token change, qu'un composant est créé/refondu, ou qu'une règle UI est établie.
- Crée un ADR/DDR uniquement pour les choix structurants avec alternatives réelles.

## Product Management

- **Document maître** : `PRD.md` centralise la vision et toutes les fonctionnalités détaillées.
- **Rigueur produit** : rejette les idées sans valeur ajoutée ou illégales. Envoie les features abandonnées dans la section « Cimetière » de la PRD avec justification.
- **Traçabilité** : lie les décisions fonctionnelles de la PRD aux contraintes techniques (références vers ADR/DDR).

## Expertise métier (légal & finance)

- **Rigueur** : conformité stricte au droit de la copropriété française. Ne jamais interpréter la loi : cite les textes officiels.
- **Style** : réponses précises, concises, sans interprétation personnelle des dispositifs légaux.

## Langue

- **TOUT le code (et commentaires)** DOIVENT être en anglais.
- **Les documents** (PRD, DESIGN, ARCHITECTURE, PROGRESS, ADR, DDR) DOIVENT être en français. Si un document référence des éléments du code, utilise les termes anglais correspondants.

---

# Accounting Immutability (ADR-009)

> Full specification: `docs/adr/ADR-009-accounting-immutability.md`
> Legal basis: `docs/LEGAL.md §11`

These rules apply whenever creating or modifying an entity that carries **financial amounts**, **legal dates**, or **lifecycle statuses** (DB migrations, Rust structs/handlers/domain, Vue forms/modals/cards).

## Step 1 — Does this entity carry financial or legal traceability data?

**Signals — answer yes if any of these apply:**
- stores an `amount`, `cost`, or financial balance
- stores a date that triggers a legal obligation (`declared_at`, `transmitted_at`, `closed_at`, `due_date`…)
- has a `status` enum that reflects a lifecycle transition (open → closed, draft → issued…)
- is referenced by another entity that does any of the above

If **no** → no further action required. If **yes** → proceed to Step 2.

## Step 2 — Which pattern applies?

| Pattern | When | Mechanism |
|---|---|---|
| **A — Freeze on terminal state** | Entity stays in place; some fields become immutable after a status transition | Guard `domain::immutability::is_mutable()` / `is_deletable()` called before any write; dedicated transition endpoint |
| **B — Close + recreate** | Modifying in place is semantically wrong (amendment, rectifying budget) | `predecessor_id` or `parent_budget_id` column; `POST /{id}/terminate` or `POST /{id}/amend` endpoint |
| **C — Append-only** | Correction = new write; original row stays intact | No `UPDATE`/`DELETE`; new row referencing the previous one |

**Currently covered entities:**

| Entity | Pattern | Frozen from |
|---|---|---|
| `levy_call` | A | `status = 'issued'` |
| `vote` / `resolution` | A | AG `ongoing` / `closed` |
| `task.completion_cost`, `task.closed_at` | A | `status = 'closed'` |
| `claim.transmitted_at`, `claim.reimbursement_*` | A | Fields set |
| `maintenance_log` (source `automatic`) | A | Always |
| `vendor_contract` | B | Active — amend via `predecessor_id` |
| `budget` / `budget_line` | B | Approved — rectify via `parent_budget_id` |
| `maintenance_log` (source `manual`) | C | Always |
| Correction of an issued `levy_call` | C | Credit note (negative amount row) |

## Step 3 — Does this entity need point-in-time queries?

**Signal:** a derived value (levy call, allocated amount) is calculated **from** this entity and must be reproducible at a past date.

If yes → add `valid_from DATE NOT NULL` and `valid_to DATE` columns. Reference implementation: `unit_allocation_key` (ADR-006), `budget_line` (ADR-009).

If no → Pattern A or B suffices.

## Backend rules (`.rs`, `migrations/*.sql`)

- **No generic `PUT`/`PATCH` on frozen fields.** Use a dedicated transition endpoint (`POST /{id}/close`, `POST /{id}/issue`, `POST /{id}/transmit`, `POST /{id}/terminate`).
- **Every transition endpoint must call `domain::immutability::is_mutable()` before any DB write.**
- **Every `DELETE` handler on a critical entity must call `domain::immutability::is_deletable()` before executing.**
- Financial mutations must write to `audit_log` via `db::audit_log::record()` with `old_values` / `new_values` as JSONB containing **only the changed fields**.
- Log: `UPDATE` on financial/traceability fields, `DELETE` attempts, status transitions. Do **not** log `INSERT` or non-financial field updates.
- `audit_log` rows are append-only — never delete or update them.

## Frontend rules (`.vue`, `.ts`)

- **Pattern A — frozen fields:** use `v-if` (not `disabled`) to hide Modifier / Supprimer once the terminal state is reached.
- **Pattern A — readonly display:** render immutable fields as `<span>`, not `<input readonly>`. Add a discrete `label-sm .tone--muted` label « Donnée comptable — non modifiable ».
- **Pattern A — frozen indicator:** display `CIconLock` with `.tone--muted` in the card or detail header.
- **Pattern B — amend workflow:** replace « Modifier » with « Créer un avenant » (`ghost` variant, not destructive). Pre-fill the new form and close the old entity in one server call.
- **Never call a generic `PUT`/`PATCH`** on a frozen entity — use the dedicated transition endpoint.
- **Never include frozen fields in a form payload** once they are set.

## graphify

This project has a graphify knowledge graph at graphify-out/.

Rules:
- Before answering architecture or codebase questions, read graphify-out/GRAPH_REPORT.md for god nodes and community structure
- If graphify-out/wiki/index.md exists, navigate it instead of reading raw files
- After modifying code files in this session, run `graphify update .` to keep the graph current (AST-only, no API cost)

<!-- code-review-graph MCP tools -->
## MCP Tools: code-review-graph

**IMPORTANT: This project has a knowledge graph. ALWAYS use the
code-review-graph MCP tools BEFORE using Grep/Glob/Read to explore
the codebase.** The graph is faster, cheaper (fewer tokens), and gives
you structural context (callers, dependents, test coverage) that file
scanning cannot.

### When to use graph tools FIRST

- **Exploring code**: `semantic_search_nodes` or `query_graph` instead of Grep
- **Understanding impact**: `get_impact_radius` instead of manually tracing imports
- **Code review**: `detect_changes` + `get_review_context` instead of reading entire files
- **Finding relationships**: `query_graph` with callers_of/callees_of/imports_of/tests_for
- **Architecture questions**: `get_architecture_overview` + `list_communities`

Fall back to Grep/Glob/Read **only** when the graph doesn't cover what you need.

### Key Tools

| Tool | Use when |
| ------ | ---------- |
| `detect_changes` | Reviewing code changes — gives risk-scored analysis |
| `get_review_context` | Need source snippets for review — token-efficient |
| `get_impact_radius` | Understanding blast radius of a change |
| `get_affected_flows` | Finding which execution paths are impacted |
| `query_graph` | Tracing callers, callees, imports, tests, dependencies |
| `semantic_search_nodes` | Finding functions/classes by name or keyword |
| `get_architecture_overview` | Understanding high-level codebase structure |
| `refactor_tool` | Planning renames, finding dead code |

### Workflow

1. The graph auto-updates on file changes (via hooks).
2. Use `detect_changes` for code review.
3. Use `get_affected_flows` to understand impact.
4. Use `query_graph` pattern="tests_for" to check coverage.

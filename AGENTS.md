# La Ruche — Agent Instructions

## Project

Open-source webapp for French co-ownership management (syndic bénévole).
Monorepo: `backend/` (Rust) + `frontend/` (Vue 3).

Domain-specific rules in `.agents/rules/` are automatically injected into context by the IDE when editing files in the matching domain. Do not re-read them manually.

---

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

---

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.

---

## Commits

**All commits must follow [Conventional Commits](https://www.conventionalcommits.org/).**

Format: `<type>(<scope>): <imperative summary>`

- Types: `feat` `fix` `refactor` `perf` `docs` `test` `chore` `build` `ci` `style` `revert`
- Scope: optional, e.g. `backend`, `frontend`, `ag`, `auth`, `db`
- Breaking change: append `!` before the colon or add `BREAKING CHANGE:` footer
- Subject ≤72 chars, imperative mood, no trailing period
- Body only when the *why* is non-obvious from the diff

Examples:
```
feat(ag): add vote-by-correspondence form to convocation checklist
fix(backend): guard levy_call mutation after issued status
docs: update ARCHITECTURE.md with vendor_contract table
feat!: remove deprecated PUT /levy-calls/:id
chore(ci): add commitlint workflow on pull_request
```

Use the `caveman:caveman-commit` skill when writing commit messages.

---

## Commands

```bash
# Backend
cargo build                        # compile
cargo test --lib --tests           # run tests
cargo clippy -- -D warnings        # lint (must pass clean)
cargo fmt --check                  # format check

# Frontend
npm --prefix frontend install      # install deps
npm --prefix frontend run build    # build
npm --prefix frontend run lint     # lint

# DB (local dev)
docker compose up -d postgres      # start postgres
cargo sqlx migrate run             # apply migrations
cargo sqlx prepare                 # regenerate .sqlx/ (offline mode)
```

---

## Language

- **Code + comments** → English only.
- **Docs** (PRD, DESIGN, ARCHITECTURE, PROGRESS, ADR, DDR) → French. Code identifiers stay English inside docs.

---

## Backend (Rust + Axum + SQLx)

- Zero business logic in handlers (`api/`) — logic lives in `domain/`.
- Zero SQL in `domain/` — DB access in `db/` repositories only.
- `pub(crate)` by default. No `unwrap()` outside tests.
- `Decimal` for all financial amounts — never `f64`.
- New table → always update `docs/ARCHITECTURE.md` before migration. Create an ADR only when there are **genuine structural design trade-offs** with real alternatives (e.g. schema design, storage strategy). Simple tables (lookups, enums, standard joins) → no ADR needed.
- `SQLX_OFFLINE=true` in CI and Docker — `backend/.sqlx/` committed.
- Errors: custom `AppError` with i18n code (`AppError::validation_code("errors.x.y")`), never raw strings.

## Frontend (Vue 3 + TypeScript + Tailwind v4)

- No raw values (`px`, `hex`, `rem`) in `class`/`style` — use Tailwind utilities or CSS tokens.
- No `<style>` block in feature views — Tailwind utilities or catalogue components only.
- No direct `radix-vue` import in feature views — use `C*` wrappers.
- New component → add to catalogue in `docs/DESIGN.md §2` before use.

---

## Workflow

- Request matches existing PRD → [request matches an Epic or a Story][request does not matches an Epic or a Story → confirm with user to create a new item] → update related item status → add entry in docs/PROGRESS.md.
- Request does not match existing PRD  → it is not trivial (ADR/DDR/PRD could be created) → discuss with user before starting
    - ADR for structural technical decisions with real alternatives. DDR for structural UI/UX decisions.
    - Create ADR/DDR only when there are genuine alternatives to document.
- If domain rule files conflict with each other or with user requests, flag the conflict to the user and await resolution before proceeding.

## Feature Implementation Checklist

Follow these steps **in order** for any new feature or significant change:

1. Update `docs/PRD.md` / `docs/DESIGN.md` / `docs/ARCHITECTURE.md` **before** coding.
2. If structural DB trade-offs exist, create an ADR in `docs/adr/` before writing the migration.
3. Write and apply the migration (`cargo sqlx migrate run`).
4. Regenerate offline query data (`cargo sqlx prepare`) and commit `backend/.sqlx/`.
5. Implement backend changes (domain → db → api layers).
6. Implement frontend changes.
7. Run `graphify update .` to keep the knowledge graph current.
8. Add a dated entry to `docs/PROGRESS.md` with Epic/Story ref (started / completed / abandoned).

---

## Security

- Never commit secrets or `.env` files.

---

## graphify

Knowledge graph at `graphify-out/`.

- Architecture/codebase questions → read `graphify-out/GRAPH_REPORT.md` first.
- After modifying code files → run `graphify update .` to keep the graph current (also step 7 of the Feature Implementation Checklist).

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

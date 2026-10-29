# La Ruche — Agent Instructions

## Project

Monorepo: `backend/` (Rust) + `frontend/` (Vue 3).

Domain-specific rules in `.claude/rules/` are automatically injected into context by the IDE when editing files in the matching domain. If a rule relevant to your task is not in your context, read it from `.claude/rules/` — never guess or assume its content.

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

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.

The test: Every changed line should trace directly to the user's request — **except** for the boy-scout fixes covered by principle 4.

## 4. Boy-Scout Fixes

**When you find dead code or a pre-existing error, fix it — even out of scope.**

- Spotted dead code (unused variable/function/import, unreachable branch) → remove it.
- Spotted a pre-existing bug, warning, or broken invariant → fix it.
- Keep boy-scout fixes in a separate, clearly-labelled commit from the feature work.
- Exception: if a fix has genuine structural trade-offs (schema, public API, behaviour change), flag it and confirm before touching it instead of fixing silently.

The test: leave every file you touch cleaner than you found it.

## 5. Goal-Driven Execution

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

Use the `git-commit` skill (`.claude/skills/git-commit`) when writing commit messages.

---

## Pre-PR / Pre-Push Checks (MANDATORY)

Before opening a PR, and before every push to an open PR branch, the relevant checks below MUST pass. Never open a PR or push a commit with these failing.

- **Frontend modified** → `cd frontend && npm run lint`
  - Fix with: `cd frontend && npm run lint -- --fix`
- **Backend modified** → `cd backend && cargo fmt --check` and `cd backend && cargo clippy -- -D warnings`
  - Fix with: `cd backend && cargo fmt` and `cd backend && cargo clippy --fix --allow-dirty`
- **If `make` is available**: `make lint` (checks) and `make lint-fix` (fixes) cover everything above.

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

- **Code + comments** → English only. This applies to all literal values (enum values, constants, string literals), variable names, function names, and all comments (inline, doc, module-level). No French in code — not even in comments or string literals.
- **No epic/story/ticket references in code comments** (no `EP-XXX-SYY`, no story IDs) — those belong in `docs/EPICS.md`, specs, and commit messages only.
- **Docs** (PRD, DESIGN, ARCHITECTURE, ADR, DDR) → French. Code identifiers stay English inside docs.
- **User-facing strings** (i18n labels, PDF content, error messages shown to users) → French, via i18n keys or dedicated label functions. Never hardcode French display text in code logic.

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
- **Before writing any new UI markup (component or feature view), grep the catalogue first**: search `docs/DESIGN.md §3` and `frontend/src/components/{atoms,molecules,organisms}/` for an existing `C*` that already covers the layout/pattern needed. Only build new markup if no catalogue component fits. A spec that describes a UI pattern without naming the `C*` component to reuse is incomplete — resolve the catalogue lookup before implementation, not after a review round-trip.

---

## Workflow

- Request matches existing PRD → [request matches an Epic or a Story][request does not matches an Epic or a Story → confirm with user to create a new item] → update the related item status in `docs/EPICS.md`.
- Request does not match existing PRD  → it is not trivial (ADR/DDR/PRD could be created) → discuss with user before starting
    - ADR for structural technical decisions with real alternatives. DDR for structural UI/UX decisions.
    - Create ADR/DDR only when there are genuine alternatives to document.
- If domain rule files conflict with each other or with user requests, flag the conflict to the user and await resolution before proceeding.
- When working on a user story, always use the relevant specs under `docs/specs/` for that story. If the story has no spec, ask for clarification before starting.

## Feature Implementation Checklist

Follow these steps **in order** for any new feature or significant change:

1. Update `docs/PRD.md` / `docs/DESIGN.md` / `docs/ARCHITECTURE.md` **before** coding.
2. If structural DB trade-offs exist, create an ADR in `docs/adr/` before writing the migration.
3. Write and apply the migration (`cargo sqlx migrate run`).
4. Regenerate offline query data (`cargo sqlx prepare`) and commit `backend/.sqlx/`.
5. Implement backend changes (domain → db → api layers).
6. Catalogue lookup (mandatory, before frontend work): grep `docs/DESIGN.md §3` and `frontend/src/components/` for any UI/UX pattern named in the spec. Record in the spec or PR which `C*` component(s) are reused, or justify why none fits (candidate for catalogue addition per the Frontend rule above).
7. Implement frontend changes.
8. Update delivery status before opening the PR: in `docs/EPICS.md`, mark the story ✅ (only when backend + frontend are both on the real API — see `.claude/rules/doc-epics.instructions.md`); if the story has a spec in `docs/specs/`, update its `Statut actuel` line to reflect the delivered state (e.g. `IMPLÉMENTÉ (backend + frontend)`).
9. Open a PR for the branch (`gh pr create`) and route it to Hopper for review (`kanban_request_review`). Pushing a branch without an open PR is not a completed task.

---

## Security

- Never commit secrets or `.env` files.

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

# Docs — Agent Instructions

> Project-wide instructions live in the root `AGENTS.md` (aliased as `CLAUDE.md`). The `doc-*` rules in `.claude/rules/` are auto-injected by the IDE when editing the matching doc. This file covers the doc map and cross-doc invariants not covered by a rule.

## Language

- All docs are written in **French**. Code identifiers (table names, routes, types) stay English inside docs.
- The exception is `.claude/` (agents, rules, skills) which stays in English — see the `claude-content-language` rule.

## Document map

### Foundational docs (contracts — must be synced with code in the same commit)

| Doc | Role | Rule |
|---|---|---|
| `docs/PRD.md` | Product vision, users, features, constraints (master) + `docs/product/PRD - {feature}.md` (per-feature detail) | `doc-prd` |
| `docs/ARCHITECTURE.md` | Technical architecture, DB schema, modules, security, CI/CD | `doc-architecture` |
| `docs/DESIGN.md` | Design system « The Modern Registry » — tokens, component catalogue, UX guidelines | `doc-design` |
| `docs/EPICS.md` | Epic/story backlog and delivery status | `doc-epics` |

### Legal

| Doc | Role | Rule |
|---|---|---|
| `docs/LEGAL.md` | French co-ownership law obligations and coverage tracking | `doc-legal` |

### Decision records

| Directory | Content | Rule |
|---|---|---|
| `docs/adr/ADR-NNN-*.md` | Architecture Decision Records (ADR-001 to ADR-012) | `doc-architecture` |
| `docs/ddr/DDR-NNN-*.md` | Design Decision Records (DDR-001 to DDR-006) | `doc-design` |

### Implementation specs

| Directory | Content |
|---|---|
| `docs/specs/EP-XXX-SYY-spec.md` | One spec per story — bridge between PRD (what) and code (how). Written by the `spec-writer` agent. |

### Operational

| Doc | Role |
|---|---|
| `docs/DEV_ACCOUNTS.md` | Demo/dev account credentials and roles |

## Cross-doc invariants

- **ADR/DDR numbering is sequential and never reused.** Next ADR is ADR-013, next DDR is DDR-007. Check the directory before creating.
- **A story delivered (✅ in `docs/EPICS.md`) must have its spec updated** in `docs/specs/` if the implementation diverges from the plan.
- **A new table/module/route → `docs/ARCHITECTURE.md` in the same commit** (enforced by `foundational-docs-sync`).
- **A new design component → `docs/DESIGN.md §2` catalogue before use** (enforced by `doc-design`).
- **A legal coverage change → `docs/LEGAL.md` and `docs/EPICS.md` in the same pass** (enforced by `doc-epics` cross-sync).
- **No external references (ADR, DDR, ticket numbers) in the body** of PRD/ARCHITECTURE/DESIGN/LEGAL — refs only, except in feature PRDs (`docs/product/`) and specs where they are required.

## Anti-patterns

- Never create a new top-level doc without updating this file.
- Never let `docs/PROGRESS.md`-style tracking files reappear — delivery status lives in `docs/EPICS.md` only.
- Never duplicate content between `docs/PRD.md` and `docs/product/PRD - *.md` — master vs detail, not copies.
- Never mark a legal obligation ✅ in `docs/LEGAL.md` before actual delivery.

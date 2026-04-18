<!--
  ╔═══════════════════════════════╗
  ║    La Ruche — Pull Request    ║
  ╚═══════════════════════════════╝

  PR title must follow Conventional Commits:
  feat(scope): description  |  fix(scope): description  |  docs: …
  See CONTRIBUTING.md for the full list of types.
-->

## 🔍 Background

<!--
  What obligation, PRD section, or architectural decision drives this change?
  Link the source of truth so reviewers understand the *why* before reading the diff.
-->

| Reference | Link |
|---|---|
| PRD | `docs/product/PRD - XXX.md §Y` |
| Legal | `docs/LEGAL.md §X.Y` |
| ADR / DDR | `docs/adr/ADR-XXX` |

---

## 📦 Backlog

<!--
  List every Epic and Story addressed. If no Epic applies (chore, tooling, hotfix), state why.
-->

- **Epic** `EP-XXX` — _title_
  - `EP-XXX-SYY` ⏳ → ✅ — _story title_
  - `EP-XXX-SZZ` ⏳ → ✅ — _story title_

## 🛠️ Implementation

<!--
  Summarise the notable implementation choices a reviewer needs to understand the diff.
  Schema changes, new domain rules, ADR-009 pattern used (A/B/C), API shape, Vue architecture…
  Not a tutorial — just the non-obvious decisions.
-->

- 
- 

## ✔️ Ship criteria

### Code quality
- [ ] PR title follows [Conventional Commits](https://www.conventionalcommits.org/)
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Frontend lint passes (`npm --prefix frontend run lint`)

### Database (if applicable)
- [ ] Migration created and reviewed
- [ ] `cargo sqlx prepare` run successfully

### Documentation (tick all that changed)
- [ ] `docs/ARCHITECTURE.md` — new table / handler / module
- [ ] `docs/DESIGN.md §2` — new component added to catalogue
- [ ] `docs/EPICS.md` — story status updated (⏳ / ✅)
- [ ] `docs/LEGAL.md` — coverage updated after delivery

---

> _"Le syndic reste responsable de la conformité légale de sa copropriété."_

<!-- Indiquer le nom exact des modèles utilisés, ou « Aucun ». -->
<sub>🤖 Implementation : `TBD` ·  🕵 Code review : `TBD`</sub>

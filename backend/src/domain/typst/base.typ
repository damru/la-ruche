// ── Shared base — prepended to every document template ──
// Loaded by the Rust engine (domain/pdf_template.rs, ADR-011) and
// concatenated before the document-specific template at compile time.

#set page(paper: "a4", margin: (top: 18mm, x: 18mm, bottom: 24mm))
#set text(size: 10pt, font: "Tinos", lang: "fr", region: "FR")
#set par(leading: 0.3em)

// Palette — mirrors frontend/src/styles/tokens.css (light theme).
#let brand = rgb("#643fd5")
#let grey = rgb("#6b7280")
#let ink = rgb("#111827")
#let rule = rgb("#d1d5db")
#let on-surface = rgb("#191c1d")
#let on-surface-variant = rgb("#42474e")

// Data boundary: the engine serves the serialized payload at /payload.json.
// All user data arrives as strings — never spliced into the markup itself,
// so no escaping is ever needed (typst strings are inert).
#let payload = json("/payload.json")
#set document(title: payload.title, author: "La Ruche")

// Layout rule: in grid/table/stack bodies, a run of inline content merges
// into a single cell/line. Cells and stacked lines are therefore always
// passed as explicit positional arguments or block-wrapped children.

// ── Regulatory helpers (état daté / pré-état daté) ──

// Two-column `label | amount` table, amount right-aligned at the right edge.
#let amount-table(rows) = [
  #table(
    columns: (1fr, auto),
    stroke: none,
    inset: (x: 0mm, y: 1mm),
    ..rows.map(r => ([#r.label], [#align(right)[#r.amount]])).flatten(),
  )
]

// The two signable blocks of a regulatory document. The blank fill-in
// line is a string literal: bare underscores would be read as emphasis.
#let signature-blocks() = [
  #table(
    columns: (1fr, 1fr),
    stroke: none,
    stack(
      spacing: 8mm,
      block[Le syndic],
      block[#"Fait à ____________________, le ____________________"],
    ),
    stack(
      spacing: 8mm,
      block[Le notaire / l'acquéreur],
      block[#"Fait à ____________________, le ____________________"],
    ),
  )
]

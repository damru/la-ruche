// ── Shared base — prepended to every document template ──
// Loaded by the Rust engine (domain/pdf_template.rs, ADR-011) and
// concatenated before the document-specific template at compile time.

#set page(paper: "a4", margin: (top: 18mm, x: 18mm, bottom: 24mm))
#set text(size: 10pt, font: "Tinos", lang: "fr", region: "FR")
// Default line spacing: interligne 1,5 (≈ 15 pt pitch at 10 pt).
#set par(leading: 0.85em)

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
  // Compact leading: the fill-in line wraps to two lines, and the
  // regulatory documents keep their previous engine's visual.
  #set par(leading: 0.5em)
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

// ── Shared letter frame (common to all letter documents) ──
// Only the letter paragraphs and the annexes vary from one document to
// another — everything else is defined here once and reused everywhere:
// brand block, double header (issuer / recipient), objet, salutation,
// closing formula, signature and running footer.
// Address-style blocks (issuer, recipient, annex meta, signature) keep a
// compact leading on top of the 1,5 default: the roomier spacing applies
// to the letter body.

#let brand-block() = [
  #grid(
    columns: (auto, 1fr),
    column-gutter: 4mm,
    align: (left + horizon),
    image("/logo.png", width: 18mm),
    block(height: 18mm)[
      #text(font: "Fraunces", size: 20pt, fill: on-surface)[#payload.app_name]
      #linebreak()
      #text(font: "DM Sans 9pt", size: 8pt, fill: on-surface-variant)[Gestion de copropriété participative]
      #linebreak()
      #text(font: "DM Sans 9pt", size: 8pt, fill: on-surface-variant)[pour les syndics bénévoles et conseils syndicaux]
    ],
  )
  #v(4mm)
]

// Issuer (left) / recipient (right), then a brand rule.
#let letter-header() = [
  #grid(
    columns: (1fr, auto),
    column-gutter: 6mm,
    align: left + top,
    [
      #set par(leading: 0.5em)
      #text(size: 9pt, fill: ink)[#payload.issuer_heading]
      #for line in payload.issuer_lines [
        #linebreak()
        #text(size: 9pt, fill: grey)[#line]
      ]
    ],
    [
      #set par(leading: 0.5em)
      #align(right)[
        #text(size: 9pt, weight: "bold", fill: ink)[#payload.recipient_label]
        #linebreak()
        #text(size: 9pt, fill: ink)[#payload.recipient_lines.first()]
        #for line in payload.recipient_lines.slice(1) [
          #linebreak()
          #text(size: 9pt, fill: grey)[#line]
        ]
      ]
    ],
  )
  #v(2mm)
  #line(length: 100%, stroke: (paint: brand, thickness: 0.8pt))
  #v(4mm)
]

// Objet line, optional reference, salutation.
#let letter-opening() = [
  #text(size: 12pt)[#strong[Objet : ]#payload.objet]
  #v(0.5mm)
  #if payload.reference != none [
    #text(size: 9pt, fill: grey)[#payload.reference]
  ]
  #v(3mm)
  #payload.salutation
  #v(1.5mm)
]

// Letter body — the one part that varies per document instance.
#let letter-paragraphs() = [
  #for paragraph in payload.letter_paragraphs [
    #paragraph
    #v(1.5mm)
  ]
]

// Closing formula + right-aligned signature.
#let letter-closing() = [
  #set par(leading: 0.5em)
  #payload.closing_formula
  #v(4mm)
  #align(right)[
    #strong[#payload.signer_closing]
    #linebreak()
    #payload.signer_name
    #if payload.signer_role != none [
      #linebreak()
      #text(fill: grey)[#payload.signer_role]
    ]
    #v(2em)
    #payload.place_date
    #linebreak()
    #payload.app_name
  ]
]

// Running footer content (every page, centered) — apply with
// `#set page(footer: align(center)[#letter-footer()])`.
// Line 1: edition note. Line 2: regulatory disclaimer, italic.
#let letter-footer() = [
  #set par(spacing: 0pt)
  #rect(width: 100%, height: 0.4pt, fill: rule)
  #v(1.5mm)
  #text(font: "DM Sans 9pt", size: 7pt, fill: grey)[#payload.app_name, gestion de copropriété participative. Document généré le #payload.edited_at via #payload.app_name.]
  #v(1mm)
  #text(size: 7pt, style: "italic", fill: grey)[Le syndic reste responsable de la conformité légale de sa copropriété.]
]

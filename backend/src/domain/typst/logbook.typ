// ── Carnet d'entretien de l'immeuble — letter + annex ──
// Expects the logbook payload (see domain/logbook_pdf.rs).
// Layout rule: in grid/table/stack bodies, a run of inline content merges
// into a single cell/line — cells and stacked lines are always passed as
// explicit positional arguments or block-wrapped children.

// One contract section: heading, then one block per contract
// (line 1: subject + period, line 2: vendor/type + amount, hairline
// below) or a muted empty state.
#let contract-section(section) = [
  #text(size: 11pt, weight: "bold", fill: brand)[#section.heading]
  #v(1mm)
  #if section.contracts.len() == 0 [
    #text(fill: grey)[#section.empty_text]
  ] else [
    #for contract in section.contracts [
      #let amount-cell = if contract.amount != none {
        ([#align(right)[#text(weight: "bold")[#contract.amount]]],)
      } else {
        ()
      }
      #table(
        columns: (3fr, 2fr),
        stroke: none,
        inset: 0pt,
        row-gutter: 0.15em,
        [#text(weight: "bold")[#contract.subject]],
        [#align(right)[#text(fill: grey)[#contract.period]]],
        [#text(fill: grey)[#contract.vendor_type]],
        ..amount-cell,
      )
      #v(0.5mm)
      #line(length: 100%, stroke: (paint: rule, thickness: 0.4pt))
      #v(1.5mm)
    ]
  ]
  #v(2mm)
]

// ── Running footer (every page, centered) ──
// Line 1: logo + edition note. Line 2: regulatory disclaimer, italic.
#set page(footer: align(center)[
  #set par(spacing: 0pt)
  #rect(width: 100%, height: 0.4pt, fill: rule)
  #v(1.5mm)
  #grid(
    columns: (auto, auto),
    column-gutter: 1.5mm,
    align: center,
    image("/logo.png", width: 5mm),
    text(font: "DM Sans 9pt", size: 7pt, fill: grey)[#payload.app_name, gestion de copropriété participative. Document généré le #payload.edited_at via #payload.app_name.],
  )
  #v(1mm)
  #text(size: 7pt, style: "italic", fill: grey)[Le syndic reste responsable de la conformité légale de sa copropriété.]
])

// ── Brand block (first page): logo left, name + tagline right ──
#grid(
  columns: (auto, 1fr),
  column-gutter: 4mm,
  align: left + top,
  image("/logo.png", width: 16mm),
  block(height: 16mm)[
    #text(font: "Fraunces", size: 20pt, fill: on-surface)[#payload.app_name]
    #v(1fr)
    #text(font: "DM Sans 9pt", size: 8pt, fill: on-surface-variant)[Gestion de copropriété participative, pour les syndics bénévoles et conseils syndicaux.]
  ],
)
#v(4mm)

// ── Double header : issuer (left) / recipient (right) ──
#grid(
  columns: (1fr, auto),
  column-gutter: 6mm,
  align: left + top,
  [
    #text(size: 9pt, fill: ink)[#payload.issuer_heading]
    #for line in payload.issuer_lines [
      #linebreak()
      #text(size: 9pt, fill: grey)[#line]
    ]
  ],
  [
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

// ── Letter ──
#text(size: 12pt)[#strong[Objet : ]#payload.objet]
#v(0.5mm)
#if payload.reference != none [
  #text(size: 9pt, fill: grey)[#payload.reference]
]
#v(3mm)
#payload.salutation
#v(1.5mm)
#for paragraph in payload.letter_paragraphs [
  #paragraph
  #v(1.5mm)
]
#payload.closing_formula
#v(4mm)

// ── Signature (right-aligned) ──
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

// ── Annex ──
#pagebreak()
#block[
  #set par(spacing: 0pt)
  #text(tracking: 2pt, size: 9pt, weight: "bold", fill: grey)[#payload.annex_kicker]
  #v(1.5mm)
  #text(size: 16pt, weight: "bold", fill: brand)[#payload.annex_title]
  #v(1.5mm)
  #text(size: 11pt, fill: grey)[#payload.annex_subtitle]
  #v(1.5mm)
  #for (i, m) in payload.annex_meta.enumerate() [
    #let (label, value) = m
    #if i > 0 [ #linebreak() ]
    #text(fill: grey)[#(label + " :")] #value
  ]
]
#v(2mm)
#line(length: 100%, stroke: (paint: brand, thickness: 0.8pt))
#v(4mm)

// ── Contract sections ──
#contract-section(payload.insurance_section)
#contract-section(payload.maintenance_section)

// ── Chronology ──
#let c = payload.chronology
#text(size: 11pt, weight: "bold", fill: brand)[#c.heading]
#v(1mm)
#let entry-rows = c.entries.map(e => {
  let desc = if e.description != none {
    (block(text(fill: grey)[#e.description]),)
  } else {
    ()
  };
  (
    [#e.date],
    stack(
      spacing: 0.15em,
      block(text(weight: "bold")[#e.title]),
      ..desc,
    ),
    [#text(fill: grey)[#e.vendor]],
    [#e.category],
    [#text(fill: grey)[#e.source]],
    [#align(right)[#text(weight: "bold")[#e.cost]]],
  )
})
#let body-cells = if c.entries.len() == 0 {
  (table.cell(colspan: 6)[#text(fill: grey)[#c.empty_text]],)
} else {
  entry-rows.flatten()
}
#table(
  columns: (2fr, 4fr, 2fr, 2fr, 2fr, 2fr),
  inset: (x: 1.1mm, y: 1.7mm),
  stroke: none,
  [#text(size: 9pt, weight: "bold", fill: grey)[#c.col_date]],
  [#text(size: 9pt, weight: "bold", fill: grey)[#c.col_intervention]],
  [#text(size: 9pt, weight: "bold", fill: grey)[#c.col_vendor]],
  [#text(size: 9pt, weight: "bold", fill: grey)[#c.col_category]],
  [#text(size: 9pt, weight: "bold", fill: grey)[#c.col_source]],
  [#align(right)[#text(size: 9pt, weight: "bold", fill: grey)[#c.col_cost]]],
  ..body-cells,
)

// ── Carnet d'entretien de l'immeuble — letter + annex ──
// Expects the logbook payload (see domain/logbook_pdf.rs).
// The shared letter frame (brand block, double header, objet, salutation,
// closing formula, signature, running footer) is defined once in base.typ
// and reused by every letter document — only the letter paragraphs (via the
// payload) and the annex below are specific to the logbook.
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

#set page(footer: align(center)[#letter-footer()])

// ── Letter — frame helpers shared via base.typ ──
#brand-block()
#letter-header()
#letter-opening()
#letter-paragraphs()
#letter-closing()

// ── Annex ──
#pagebreak()
#block[
  #set par(spacing: 0pt)
  #set par(leading: 0.3em)
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

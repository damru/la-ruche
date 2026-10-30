// ── État daté — regulatory document with letter body + annex ──
// Uses the shared letter frame from base.typ (brand block, double header,
// letter opening/paragraphs/closing, running footer) like the logbook.
// The regulatory content (amount tables, signature blocks) follows the
// letter; a page break leads to the annex announcement (the pre-filled
// questionnaire PDF is appended at the Rust level, not rendered by typst).
// Liberation Sans for the regulatory body.
// Expects the état daté payload (domain/legal_documents.rs).

#set page(footer: align(center)[#letter-footer()])

// ── Letter — frame helpers shared via base.typ ──
#brand-block()
#letter-header()
#letter-opening()
#letter-paragraphs()
#letter-closing()

// ── Regulatory content ──
#pagebreak()
#set text(font: "Liberation Sans")
#set par(justify: false)

#align(center)[#text(size: 16pt, weight: "bold")[État daté]]
#v(2mm)
#for line in payload.header_lines [
  #block[#line]
]
#v(2mm)

#text(weight: "bold")[1. Sommes dues par le cédant]
#v(1mm)
#amount-table(payload.part1)
#v(2mm)

#text(weight: "bold")[2. Sommes dont le syndicat est débiteur]
#v(1mm)
#amount-table(payload.part2)
#v(2mm)

#text(weight: "bold")[3. Sommes incombant à l'acquéreur]
#v(1mm)
#amount-table(payload.part3)
#v(2mm)

#text(weight: "bold")[Annexe — quote-part sur les 2 derniers exercices]
#v(1mm)
#table(
  columns: (1fr, 2fr, 2fr, 2fr),
  stroke: none,
  inset: (x: 1mm, y: 1mm),
  [#text(weight: "bold")[Exercice]],
  [#text(weight: "bold")[Charges]],
  [#text(weight: "bold")[Fonds travaux]],
  [#align(right)[#text(weight: "bold")[Total]]],
  ..payload.exercices.map(ex => (
    [#ex.year],
    [#align(right)[#ex.charges]],
    [#align(right)[#ex.travaux]],
    [#align(right)[#ex.total]],
  )).flatten(),
)
#v(2mm)

#text(weight: "bold")[Procédures judiciaires en cours]
#v(1mm)
#if payload.procedures.len() == 0 [
  Néant
] else [
  #for line in payload.procedures [
    #block[#line]
  ]
]
#v(2mm)

#text(weight: "bold")[Travaux votés non réalisés]
#v(1mm)
#if payload.travaux.len() == 0 [
  Néant
] else [
  #for line in payload.travaux [
    #block[#line]
  ]
]
#v(2mm)

#text(weight: "bold")[Emprunts collectifs]
#v(1mm)
#payload.emprunts
#v(4mm)

#signature-blocks()

// ── Annex announcement page (questionnaire PDF appended by the engine) ──
#pagebreak()
#block[
  #set par(spacing: 0pt)
  #set par(leading: 0.5em)
  #text(tracking: 2pt, size: 9pt, weight: "bold", fill: grey)[#payload.annex_kicker]
  #v(5mm)
  #text(size: 16pt, weight: "bold", fill: brand)[#payload.annex_title]
  #v(5mm)
  #text(size: 11pt, fill: grey)[#payload.annex_subtitle]
]
#v(2mm)
#line(length: 100%, stroke: (paint: brand, thickness: 0.8pt))
#v(4mm)
#text(fill: grey)[Les pages suivantes contiennent le questionnaire syndic pré-rempli.]

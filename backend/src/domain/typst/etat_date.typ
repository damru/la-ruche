// ── État daté — regulatory document ──
// Uses the shared letter frame from base.typ (brand block, double header,
// running footer) like the logbook; the regulatory content (amount tables,
// signature blocks) follows. Liberation Sans for the body.
// Expects the état daté payload (domain/legal_documents.rs).
// Layout rule: stacked lines are block-wrapped children (a run of inline
// content would merge into a single line).

#set page(footer: align(center)[#letter-footer()])
#brand-block()
#letter-header()

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

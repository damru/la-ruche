// ── Pré-état daté — regulatory document (art. L721-2 CCH) ──
// Liberation Sans, no brand block / footer (visual unchanged from the
// previous engine). Expects the pré-état daté payload (domain/legal_documents.rs).
// Layout rule: stacked lines are block-wrapped children (a run of inline
// content would merge into a single line).

#set text(font: "Liberation Sans")
#set page(margin: (x: 18mm, y: 18mm))
#set par(justify: false)

#align(center)[#text(size: 16pt, weight: "bold")[Pré-état daté]]
#v(2mm)
#for line in payload.header_lines [
  #block[#line]
]
#if payload.edited_at != none [
  #payload.edited_at
]
#v(2mm)

#text(weight: "bold")[Charges courantes (budget prévisionnel et hors budget)]
#v(1mm)
#amount-table(payload.charges)
#v(2mm)

#text(weight: "bold")[Sommes dues au syndicat]
#v(1mm)
#amount-table(payload.dues)
#v(2mm)

#text(weight: "bold")[Fonds travaux]
#v(1mm)
#amount-table(payload.fund)
#v(2mm)

#text(weight: "bold")[Procédures judiciaires en cours]
#v(1mm)
#if payload.procedures_count == 0 [
  Néant
] else [
  #payload.procedures_count
]
#v(2mm)

#text(weight: "bold")[Emprunts collectifs]
#v(1mm)
#payload.emprunts
#v(4mm)

#signature-blocks()

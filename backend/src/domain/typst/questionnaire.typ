// ── Questionnaire syndic — annexe de l'état daté (document approuvé CSN) ──
// Typst conversion of the CSN questionnaire: same sections and questions,
// with a payload anchor on every fillable field. Fields the app knows are
// pre-filled (bold); the others render as blank fill-in lines or unchecked
// boxes so the syndic completes them by hand.
// Compiled separately and appended to the état daté PDF at the Rust level
// (domain/legal_documents.rs). Expects the questionnaire payload.

#set page(
  header: grid(
    columns: (1fr, auto),
    [#text(size: 7pt, fill: grey)[DOCUMENT APPROUVÉ PAR LE CSN]],
    [#text(size: 7pt, fill: grey)[#context counter(page).display()]],
  ),
  footer: align(center)[#text(size: 7pt, fill: grey)[#payload.app_name — questionnaire syndic — édité le #payload.edited_at]],
)
#set text(font: "Liberation Sans", size: 9pt, lang: "fr", region: "FR")
#set par(leading: 0.5em, justify: false)

// ── Anchor helpers ─────────────────────────────────────────────────────────
// Every field of the questionnaire is an anchor: a pre-filled value renders
// bold, a missing one renders as a blank line / unchecked box to complete.

// Text fill-in line.
#let q-fill(value, width: 60mm) = {
  if value != none and str(value).len() > 0 { strong[#value] } else { box[#line(length: width)] }
}

// Amount fill-in line (right-aligned column).
#let q-amt(value) = {
  if value != none and str(value).len() > 0 { align(right)[#value] } else { align(right)[#box[#line(length: 28mm)]] }
}

// Oui/non checkbox pair. `none` leaves both boxes unchecked.
#let q-yn(value) = [#if value == true [[x]] else [[ ]] oui #h(2mm) #if value == false [[x]] else [[ ]] non]

#let q-section(title) = [#v(3mm) #text(weight: "bold", size: 10pt)[#title] #v(1mm)]

#let q-sub(title) = [#v(2mm) #text(weight: "bold")[#title] #v(0.5mm)]

// label | amount table (amount column right-aligned, blank = to fill).
#let q-rows(rows) = [
  #table(
    columns: (1fr, auto),
    stroke: none,
    inset: (x: 0mm, y: 1mm),
    ..rows.map(r => ([#r.label], [#q-amt(r.amount)])).flatten(),
  )
]

// label ……… fill-in line (grid keeps the blank on the right edge).
#let q-line(label, value, width: 70mm) = [
  #grid(columns: (1fr, auto), [#label], [#q-fill(value, width: width)])
  #v(1.5mm)
]

// ── Page 1 — Informations des parties ──────────────────────────────────────
#align(center)[#text(weight: "bold", size: 13pt)[MUTATION DE LOTS DE COPROPRIÉTÉ — INFORMATIONS DES PARTIES]]
#v(1mm)
#align(center)[#text(weight: "bold")[ÉTAT DATÉ] — certificat art. 20 à joindre]
#v(3mm)

#q-line([- Désignation de l'acquéreur (art. 20-II loi 65-557, loi ALUR) :], payload.acquereur, width: 80mm)
#q-line([- Date de la demande :], payload.date_demande, width: 40mm)
#q-line([- Office notarial :], payload.office_notarial, width: 70mm)
#q-line([- Référence :], payload.reference_dossier, width: 70mm)
#q-line([- Dossier n° Clerc :], payload.dossier_clerc, width: 70mm)
#v(2mm)
#q-line([- Délivré par le syndic :], payload.delivre_par, width: 70mm)
Représentant : #if payload.syndicat_unique [[x]] else [[ ]] un syndicat unique #h(2mm) [[ ]] un syndicat principal #h(2mm) [[ ]] un syndicat secondaire
#q-line([- Contact syndic :], payload.contact_syndic, width: 70mm)
#q-line([- Date :], payload.date_delivrance, width: 40mm)
Cachet et signature : #box[#line(length: 50mm)]
#v(3mm)
#q-line([IMMEUBLE sis à :], payload.immeuble_adresse, width: 80mm)
#q-line([- dénommé :], payload.immeuble_nom, width: 70mm)
#v(2mm)
#q-line([COORDONNÉES DU COPROPRIÉTAIRE CÉDANT :], payload.cedant_nom, width: 60mm)
#q-line([- adresse :], payload.cedant_adresse, width: 80mm)
#q-line([N° DES LOTS :], payload.lots, width: 30mm)
MUTATION : #q-yn(payload.mutation_onereuse) à titre onéreux / à titre gratuit
#q-line([- Date envisagée pour la mutation :], payload.date_mutation_envisagee, width: 70mm)

// ── 1ère partie — sommes dues par le cédant ───────────────────────────────
#pagebreak()
#q-section[1ÈRE PARTIE : SOMMES DUES PAR LE COPROPRIÉTAIRE CÉDANT POUR LES LOTS OBJETS DE LA MUTATION]
#q-sub[A/ AU SYNDICAT, AU TITRE :]
#q-rows(payload.part1)
#q-sub[B/ À DES TIERS, au titre d'emprunts de copropriétaires gérés par le syndic :]
#q-rows(payload.part1_emprunts_tiers)
#table(
  columns: (1fr, auto),
  stroke: none,
  inset: (x: 0mm, y: 1mm),
  [#text(weight: "bold")[TOTAL (A/ + B/)]],
  [#align(right)[#text(weight: "bold")[#payload.total_part1]]],
)

// ── 2ème partie — sommes dont le syndicat est débiteur ─────────────────────
#pagebreak()
#q-section[2ÈME PARTIE : SOMMES DONT LE SYNDICAT POURRAIT ÊTRE DÉBITEUR À L'ÉGARD DU COPROPRIÉTAIRE CÉDANT]
#q-sub[A/ DES AVANCES PERÇUES (D. art. 5. 2° a) :]
#q-sub[B/ DES PROVISIONS (D. art. 5. 2° b) — déchéance du terme, art. 19-2 loi 65-557 :]
#q-sub[C/ DU SOLDE CRÉDITEUR SUR L'EXERCICE ANTÉRIEUR :]
#q-rows(payload.part2)
#table(
  columns: (1fr, auto),
  stroke: none,
  inset: (x: 0mm, y: 1mm),
  [#text(weight: "bold")[TOTAL (A + B + C)]],
  [#align(right)[#text(weight: "bold")[#payload.total_part2]]],
)
#v(3mm)
#q-sub[AVANCES — MODALITÉS DE REMBOURSEMENT (art. 45-1 décret 1967)]
Solution 1 : #q-yn(payload.remb_solution1) l'acquéreur rembourse directement le vendeur des avances (1ère partie 4-1/4-2/4-3 et 2ème partie A-1/A-2/A-3), soit globalement la somme de #q-fill(payload.remb_montant, width: 40mm).
#v(1mm)
Solution 2 : #q-yn(payload.remb_solution2) l'acquéreur verse entre les mains du syndic le montant desdites avances, soit globalement la somme de #q-fill(payload.remb_montant, width: 40mm).

// ── 3ème partie — sommes incombant au nouveau copropriétaire ──────────────
#pagebreak()
#q-section[3ÈME PARTIE : SOMMES INCOMBANT AU NOUVEAU COPROPRIÉTAIRE POUR LES LOTS OBJETS DE LA MUTATION]
#q-sub[AU SYNDICAT AU TITRE :]
#q-sub[1- de la reconstitution des avances (D. art. 5. 3° a) :]
#q-rows(payload.part3)
#q-sub[2- des provisions non encore exigibles — dans le budget prévisionnel (D. art. 5. 3° b) :]
#table(
  columns: (1fr, auto),
  stroke: none,
  inset: (x: 0mm, y: 1mm),
  [#text(weight: "bold")[Date d'exigibilité]], [#align(right)[#text(weight: "bold")[Montant]]],
  ..payload.part3_provisions_budget.map(r => ([#r.label], [#q-amt(r.amount)])).flatten(),
  ..range(3 - payload.part3_provisions_budget.len()).map(_ => ([#box[#line(length: 60mm)]], [#align(right)[#box[#line(length: 28mm)]]])).flatten(),
)
#q-sub[- dans les dépenses hors budget prévisionnel (D. art. 5. 3° c) :]
#table(
  columns: (1fr, auto),
  stroke: none,
  inset: (x: 0mm, y: 1mm),
  [#text(weight: "bold")[Date d'exigibilité]], [#align(right)[#text(weight: "bold")[Montant]]],
  ..payload.part3_provisions_hors_budget.map(r => ([#r.label], [#q-amt(r.amount)])).flatten(),
  ..range(3 - payload.part3_provisions_hors_budget.len()).map(_ => ([#box[#line(length: 60mm)]], [#align(right)[#box[#line(length: 28mm)]]])).flatten(),
)
#table(
  columns: (1fr, auto),
  stroke: none,
  inset: (x: 0mm, y: 1mm),
  [#text(weight: "bold")[TOTAL 3ÈME PARTIE]],
  [#align(right)[#text(weight: "bold")[#payload.total_part3]]],
)

// ── Annexe à la 3ème partie ────────────────────────────────────────────────
#pagebreak()
#q-section[ANNEXE À LA 3ÈME PARTIE : INFORMATIONS — ACTUALISATION DU DOCUMENT EN DATE DU #payload.actualisation_date]
#q-sub[A/ QUOTE-PART POUR LES LOTS OBJETS DE LA MUTATION — BUDGET PRÉVISIONNEL :]
#table(
  columns: (1fr, 1fr, 1fr),
  stroke: none,
  inset: (x: 1mm, y: 1mm),
  [#text(weight: "bold")[Exercice]], [#text(weight: "bold")[Quote-part appelée]], [#align(right)[#text(weight: "bold")[Quote-part réelle]]],
  ..payload.qp_budget.map(ex => ([#ex.year], [#ex.appelee], [#align(right)[#q-fill(ex.reelle, width: 25mm)]])).flatten(),
)
#q-sub[- DÉPENSES HORS BUDGET :]
#table(
  columns: (1fr, 1fr, 1fr),
  stroke: none,
  inset: (x: 1mm, y: 1mm),
  [#text(weight: "bold")[Exercice]], [#text(weight: "bold")[Quote-part appelée]], [#align(right)[#text(weight: "bold")[Quote-part réelle]]],
  ..payload.qp_hors_budget.map(ex => ([#ex.year], [#ex.appelee], [#align(right)[#q-fill(ex.reelle, width: 25mm)]])).flatten(),
)
#v(2mm)
#q-sub[B/ PROCÉDURES EN COURS : existe-t-il des procédures en cours ? #q-yn(payload.procedures_oui)]
#if payload.procedures_oui [
  - Objet des procédures :
  #for line in payload.procedures [#block[#line]]
  - État des procédures : #q-fill(payload.procedures_etat, width: 80mm)
] else [
  Néant.
]
#v(2mm)
#q-sub[EXISTENCE D'UN FONDS DE TRAVAUX : #q-yn(payload.fonds_travaux_existe)]
#q-line([- Montant de la part du fonds rattachée au lot cédé :], payload.fonds_travaux_quote_part, width: 40mm)
#q-line([- Dernière cotisation versée par le cédant au titre de son lot :], payload.fonds_travaux_derniere_cotisation, width: 40mm)

// ── Événements après délivrance ────────────────────────────────────────────
#pagebreak()
#q-section[ÉVÉNEMENTS INTERVENUS APRÈS DÉLIVRANCE DU QUESTIONNAIRE AVANT-CONTRAT]
- Nouvelle AG : #q-yn(payload.nouvelle_ag) (si oui, joindre le PV)
#v(1mm)
- Si de nouveaux travaux ont été votés, remplir le tableau ci-dessous :
#table(
  columns: (1.2fr, 2fr, 1.2fr, 1.4fr, 1.2fr, 1.2fr, 1.4fr),
  stroke: 0.4pt + rule,
  inset: (x: 1mm, y: 1mm),
  [#text(weight: "bold", size: 7pt)[Date de la décision]],
  [#text(weight: "bold", size: 7pt)[Nature des travaux]],
  [#text(weight: "bold", size: 7pt)[État d'avancement technique (C/NC)]],
  [#text(weight: "bold", size: 7pt)[État d'avancement financier]],
  [#text(weight: "bold", size: 7pt)[Quote-part afférente aux lots]],
  [#text(weight: "bold", size: 7pt)[Montant(s) déjà appelé(s)]],
  [#align(right)[#text(weight: "bold", size: 7pt)[Montant restant / dates d'exigibilité]]],
  ..payload.travaux_post.map(t => (
    [#q-fill(t.date, width: 18mm)],
    [#t.nature],
    [#q-fill(t.etat, width: 12mm)],
    [#q-fill(t.financier, width: 14mm)],
    [#q-fill(t.quote_part, width: 12mm)],
    [#q-fill(t.appele, width: 12mm)],
    [#align(right)[#q-fill(t.restant, width: 12mm) #q-fill(t.exigibilite, width: 14mm)]],
  )).flatten().flatten(),
)
#q-line([Commentaires éventuels :], payload.commentaires_travaux, width: 100mm)

// ── Sommes exigibles / certificat art. 20 ──────────────────────────────────
#pagebreak()
#q-section[SOMMES EXIGIBLES DONT LE RÈGLEMENT ENTRAÎNERA LA DÉLIVRANCE DU CERTIFICAT DE L'ARTICLE 20]
#q-line([Pour une date de signature le :], payload.date_signature, width: 40mm)
#q-rows(payload.sommes_exigibles)
#table(
  columns: (1fr, auto),
  stroke: none,
  inset: (x: 0mm, y: 1mm),
  [#text(weight: "bold")[TOTAL]],
  [#align(right)[#text(weight: "bold")[#payload.total_sommes_exigibles]]],
)
#v(2mm)
- Certificat de l'article 20 daté, signé et joint au présent état (validité 1 mois) : #q-yn(payload.certificat_art20_joint)
#v(3mm)
#q-section[CERTIFICAT DE L'ARTICLE 20 (art. 20 loi 65-557)]
#if payload.copro_a_jour [
  Le copropriétaire susnommé pour le lot objet de la mutation EST LIBRE, à ce jour, de toute obligation à l'égard du syndicat. Le syndic dispense le notaire, sous condition que l'acquisition intervienne sous un mois, de l'envoi de l'avis de mutation.
] else [
  Le copropriétaire susnommé N'EST PAS LIBRE de toute obligation à l'égard du syndicat. Il est débiteur de la somme de #strong[#payload.somme_debitrice]. À défaut de règlement au plus tard le #q-fill(payload.date_certificat, width: 40mm), le notaire devra adresser l'avis de mutation (art. 20) et le syndic exercera l'opposition par acte extra-judiciaire.
]
#q-line([FAIT À LE :], payload.date_certificat, width: 40mm)

// ── Renseignements complémentaires — A) vie de la copropriété ─────────────
#pagebreak()
#q-section[- II - RENSEIGNEMENTS COMPLÉMENTAIRES — A) VIE DE LA COPROPRIÉTÉ]
#q-sub[A1/ ASSURANCES]
- Multirisques (RC - incendie - dégâts des eaux) : #q-yn(payload.assur_multirisque)
- Garantie reconstruction : #q-yn(payload.assur_reconstruction) — valeur à neuf : #q-yn(payload.assur_valeur_neuf) — limitée à un capital de : #q-fill(payload.assur_capital_limite, width: 30mm)
#q-line([- Autres risques garantis :], payload.assur_autres_risques, width: 80mm)
#q-line([- Police n° :], payload.assur_police_numero, width: 50mm)
#q-line([- Date :], payload.assur_police_date, width: 40mm)
#q-line([- Courtier / agent :], payload.assur_courtier, width: 80mm)
#q-line([- Compagnie d'assurances :], payload.assur_compagnie, width: 80mm)
- Dommage ouvrages en cours — construction d'origine : #q-yn(payload.assur_do_origine) — souscrite par le syndicat au titre de travaux : #q-yn(payload.assur_do_travaux)
#v(2mm)
#q-sub[A2/ MODIFICATION DU RÈGLEMENT DE COPROPRIÉTÉ]
#q-line([- Règlement de copropriété en date du :], payload.reglement_date, width: 40mm)
- Modification non publiée connue du syndic : #q-yn(payload.reglement_modifie) (si oui, joindre le PV)
- Règlement adapté à la loi SRU (art. 49) : #q-yn(payload.reglement_sru)
#v(2mm)
#q-sub[A3/ ASSEMBLÉE GÉNÉRALE]
#q-line([- Date de la dernière assemblée générale :], payload.derniere_ag, width: 40mm)
#q-line([- Date ou période de la prochaine assemblée générale :], payload.prochaine_ag, width: 40mm)
(Joindre les PV des deux dernières années.)

#pagebreak()
#q-sub[A4/ SYNDIC]
#q-line([- Date de la dernière désignation :], payload.syndic_designation_date, width: 40mm)
- Syndic professionnel : #q-yn(payload.syndic_pro)
- La copropriété constitue-t-elle un syndicat unique ? #q-yn(payload.syndicat_unique)
#v(2mm)
#q-sub[A5/ ASSOCIATION SYNDICALE — AFUL — UNION DE SYNDICATS]
- L'immeuble est-il compris dans le périmètre d'une ASL / AFUL / union de syndicats ? #q-yn(payload.aful)
#q-line([- Si oui, nom, siège et représentant :], payload.aful_detail, width: 80mm)
#v(2mm)
#q-sub[A6/ ÉTAT D'AVANCEMENT DE LA SITUATION TECHNIQUE ET FINANCIÈRE DES TRAVAUX]
#table(
  columns: (1.2fr, 2fr, 1.2fr, 1.4fr, 1.2fr, 1.2fr, 1.4fr),
  stroke: 0.4pt + rule,
  inset: (x: 1mm, y: 1mm),
  [#text(weight: "bold", size: 7pt)[Date de la décision]],
  [#text(weight: "bold", size: 7pt)[Nature des travaux]],
  [#text(weight: "bold", size: 7pt)[État (T/C/NC)]],
  [#text(weight: "bold", size: 7pt)[Avancement financier]],
  [#text(weight: "bold", size: 7pt)[Quote-part lots]],
  [#text(weight: "bold", size: 7pt)[Appelé]],
  [#align(right)[#text(weight: "bold", size: 7pt)[Restant / exigibilité]]],
  ..payload.travaux_post.map(t => (
    [#q-fill(t.date, width: 18mm)],
    [#t.nature],
    [#q-fill(t.etat, width: 12mm)],
    [#q-fill(t.financier, width: 14mm)],
    [#q-fill(t.quote_part, width: 12mm)],
    [#q-fill(t.appele, width: 12mm)],
    [#align(right)[#q-fill(t.restant, width: 12mm) #q-fill(t.exigibilite, width: 14mm)]],
  )).flatten().flatten(),
)
#q-line([Commentaires éventuels :], payload.travaux_commentaires, width: 100mm)

#pagebreak()
#q-sub[A7/ PATRIMOINE DU SYNDICAT DES COPROPRIÉTAIRES]
- Le syndicat a-t-il un patrimoine immobilier ? #q-yn(payload.patrimoine)
#q-line([- Si oui, en quoi consiste-t-il ? :], payload.patrimoine_detail, width: 80mm)
#v(2mm)
#q-sub[A8/ CONTRATS EN COURS AU BÉNÉFICE DU SYNDICAT]
- Contrats générant des revenus (affichage, location de parties communes, antenne relais…) : #q-yn(payload.contrats_revenus)
#q-line([- Si oui, en quoi consistent-ils ? :], payload.contrats_revenus_detail, width: 80mm)
#v(2mm)
#q-sub[A9/ EXISTENCE D'EMPRUNTS]
- Emprunt du syndicat (pour son compte ou celui de copropriétaires) : #q-yn(payload.emprunt_existe)
#q-line([- Objet de l'emprunt :], payload.emprunt_objet, width: 80mm)
#q-line([- Organisme de crédit (nom et siège) :], payload.emprunt_organisme, width: 80mm)
#q-line([- Référence du dossier :], payload.emprunt_reference, width: 60mm)
#q-line([- Capital restant dû pour les lots objet des présentes :], payload.emprunt_capital_restant, width: 40mm)
- La mutation entraîne-t-elle l'exigibilité de cette somme ? #q-yn(payload.emprunt_exigible_mutation)
#q-line([- Emprunts collectifs (état daté) :], payload.emprunts_collectifs, width: 80mm)
#v(2mm)
#q-sub[A10/ COPROPRIÉTÉ EN DIFFICULTÉ]
- Administration provisoire (art. 29-1 et suivants loi 65-557) : #q-yn(payload.admin_provisoire)
- Mandataire ad hoc (art. 29-1 A et 29-1 B) : #q-yn(payload.mandataire_ad_hoc)
#v(2mm)
#q-sub[A11/ DROIT DE PRIORITÉ SUR LES LOTS À USAGE DE STATIONNEMENT (art. 8-1 loi 65-557)]
- Droit de priorité voté en AG : #q-yn(payload.priorite_stationnement_vote)
- Clause spécifique dans le règlement de copropriété : #q-yn(payload.priorite_stationnement_clause)

#pagebreak()
#q-sub[- DROIT DE PRIORITÉ EN CAS DE SURÉLÉVATION (art. 35 loi 65-557)]
- Droit de priorité voté en AG : #q-yn(payload.surelevation_vote)
- Clause spécifique dans le règlement de copropriété : #q-yn(payload.surelevation_clause)
#v(2mm)
#q-sub[A12/ IMMATRICULATION DU SYNDICAT DES COPROPRIÉTAIRES]
- Immatriculé au registre national d'immatriculation ? #q-yn(payload.immatriculation)
#q-line([- Si oui, numéro d'immatriculation :], payload.immatriculation_numero, width: 60mm)
#v(3mm)
#q-section[B/ DOSSIER TECHNIQUE ET ENVIRONNEMENTAL]
#q-line([- DATE DE CONSTRUCTION DE L'IMMEUBLE :], payload.date_construction, width: 40mm)
#v(2mm)
#q-sub[B1/ CARNET D'ENTRETIEN]
- Carnet d'entretien : #q-yn(payload.carnet_entretien) — type immeuble : IGH #q-yn(payload.type_igh)
#v(2mm)
#q-sub[B2/ AMIANTE (permis de construire délivré avant le 1er janvier 1997)]
- Soumis à la réglementation amiante : #q-yn(payload.amiante_soumis)
- Recherches concluant à l'absence d'amiante : #q-yn(payload.amiante_absence)
- DTA mis à jour depuis le 1er février 2012 : #q-yn(payload.amiante_dta)
#v(2mm)
#q-sub[B3/ PLOMB (CREP)]
- Immeuble édifié avant le 1er janvier 1949 : #q-yn(payload.plomb_avant_1949)

#pagebreak()
#q-sub[B4/ TERMITES / ÉTAT PARASITAIRE DES PARTIES COMMUNES]
- Recherche effectuée sur les parties communes : #q-yn(payload.termites_recherche)
#v(2mm)
#q-sub[B5/ AUTRES RISQUES SANITAIRES (légionellose, radon, mérules…) ]
- Mesures d'urgence (DDASS, Préfecture) : #q-yn(payload.mesures_urgence)
- Risques sanitaires connus dans l'immeuble : #q-yn(payload.risques_sanitaires)
- Ces risques ont-ils fait l'objet d'un traitement ? #q-yn(payload.risques_sanitaires_traites)
#v(2mm)
#q-sub[B6/ DPE OU AUDIT ÉNERGÉTIQUE]
- L'immeuble est-il concerné ? #q-yn(payload.dpe_concerne) — DPE #q-fill(payload.dpe_ou_audit, width: 20mm) / audit
- Plan de travaux d'économie d'énergie adopté : #q-yn(payload.plan_travaux_energie)
- Contrat de performance énergétique : #q-yn(payload.cpe)
#v(2mm)
#q-sub[B7/ ASCENSEUR — CONTRÔLE TECHNIQUE QUINQUENNAL]
- Ascenseurs : #q-yn(payload.ascenseur_existe)
- Installés antérieurement au 27 août 2000 : #q-yn(payload.ascenseur_avant_2000)
- Contrôle technique quinquennal effectué : #q-yn(payload.ascenseur_controle)
- Travaux de mise aux normes : avant le 31/12/2010 #q-yn(payload.ascenseur_normes_2010) — avant le 03/07/2013 #q-yn(payload.ascenseur_normes_2013) — avant le 03/07/2018 #q-yn(payload.ascenseur_normes_2018)

#pagebreak()
#q-sub[B8/ PISCINE]
- Existence : #q-yn(payload.piscine_existe) — dispositif de sécurité homologué : #q-yn(payload.piscine_securite)
#v(2mm)
#q-sub[B9/ MESURES ADMINISTRATIVES]
- Arrêté de péril : #q-yn(payload.arrete_peril) — déclaration d'insalubrité : #q-yn(payload.insalubrite)
- Injonction de travaux : #q-yn(payload.injonction_travaux) — interdiction d'habiter : #q-yn(payload.interdiction_habiter)
- Inscription à l'inventaire / classement monument historique : #q-yn(payload.monument_historique)
- Injonction de ravalement des façades : #q-yn(payload.ravalement) — plan de sauvegarde (OPAH) : #q-yn(payload.opah)
#v(2mm)
#q-sub[B10/ INSTALLATIONS CLASSÉES (ICPE)]
- Installation classée dans l'immeuble : #q-fill(payload.icpe, width: 60mm) (oui / non / ne sait pas)

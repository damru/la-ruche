//! Regulatory PDF rendering (ADR-011, LEGAL.md §7.1 état daté / §7.2 pré-état daté).
//!
//! Pure functions only — no I/O, no SQL. Callers load and assemble the data
//! (domain + db), hand it here together with a document header, and receive
//! the finished PDF as in-memory bytes. The documents are composed by the
//! shared typst engine (`domain/pdf_template.rs`, templates in
//! `domain/typst/`): Liberation Sans for the regulatory body. The état daté
//! uses the full shared letter frame (brand block, double header, running
//! footer) from `base.typ`; the pré-état daté keeps the plain regulatory
//! visual (no brand block, no footer). Fonts are embedded at compile time so
//! the rendering is self-contained (no system font dependency, matching the
//! `debian:trixie-slim` runtime image).
//!
//! The PDF is a *rendering* of the frozen JSONB snapshot, not the probative
//! record itself (the snapshot is). The typst PDF export embeds a document id
//! and the current creation date, so the byte output is not deterministic
//! across runs — this is acceptable for a presentation document and does not
//! affect the état daté's probative value.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Serialize;

use crate::domain::etat_date::{EtatDate, PreEtatDate};
use crate::domain::pdf_template::{format_date, format_euro};

/// Header common to both regulatory documents: the residence identity, the
/// syndic, the lot, and the relevant dates. Assembled by the API layer from
/// `building`, `unit`, `governance_mandate` and the request row.
pub struct LegalDocumentHeader {
    pub building_name: String,
    pub building_address: String,
    pub syndic_name: Option<String>,
    pub syndic_siret: Option<String>,
    pub unit_number: String,
    /// Notaire / étude having requested the document (état daté only).
    pub requester_name: Option<String>,
    /// Date of the notaire's request (état daté only).
    pub requested_at: Option<NaiveDate>,
    /// J+15 legal deadline (état daté only).
    pub due_date: Option<NaiveDate>,
    /// Date the snapshot was generated (état daté only).
    pub generated_at: Option<NaiveDate>,
    /// Edition date (pré-état daté only).
    pub edited_at: Option<NaiveDate>,
    /// Current owner (cédant) display name — questionnaire annex page 1.
    pub owner_name: Option<String>,
    /// Bylaws date (`building.bylaws_date`) — questionnaire A2.
    pub bylaws_date: Option<NaiveDate>,
    /// Start date of the active syndic mandate — questionnaire A4.
    pub syndic_mandate_start: Option<NaiveDate>,
    /// Date of the last held general meeting — questionnaire A3.
    pub last_ag_date: Option<NaiveDate>,
    /// Date of the next scheduled general meeting — questionnaire A3.
    pub next_ag_date: Option<NaiveDate>,
    /// Last works-fund levy paid by the lot — questionnaire annex page 5.
    pub fonds_travaux_derniere_cotisation: Option<Decimal>,
}

// ── Payloads (typst data boundary: pre-formatted strings only) ─────────────

/// One `label | amount` row of a regulatory table.
#[derive(Serialize)]
struct LignePayload {
    label: String,
    amount: String,
}

/// One row of the quote-part annexe (état daté).
#[derive(Serialize)]
struct ExercicePayload {
    year: String,
    charges: String,
    travaux: String,
    total: String,
}

#[derive(Serialize)]
struct EtatDatePayload {
    title: String,
    app_name: &'static str,
    /// Edition date, `jj/mm/aaaa` (running footer).
    edited_at: String,
    // Double header (shared letter frame from base.typ)
    issuer_heading: String,
    issuer_lines: Vec<String>,
    recipient_label: String,
    recipient_lines: Vec<String>,
    // Letter body (shared letter frame from base.typ)
    objet: String,
    reference: Option<String>,
    salutation: String,
    letter_paragraphs: Vec<String>,
    closing_formula: String,
    signer_closing: String,
    signer_name: String,
    signer_role: Option<String>,
    place_date: String,
    /// Annex announcement (questionnaire syndic pré-rempli).
    annex_kicker: String,
    annex_title: String,
    annex_subtitle: String,
    /// Residence + syndic + lot + request lines, pre-formatted.
    header_lines: Vec<String>,
    part1: Vec<LignePayload>,
    part2: Vec<LignePayload>,
    part3: Vec<LignePayload>,
    exercices: Vec<ExercicePayload>,
    /// Pre-formatted procedure lines (« subject — date (jurisdiction) »).
    procedures: Vec<String>,
    travaux: Vec<String>,
    emprunts: String,
}

#[derive(Serialize)]
struct PreEtatDatePayload {
    title: String,
    /// Residence + syndic + lot lines, pre-formatted.
    header_lines: Vec<String>,
    /// « Édité le : jj/mm/aaaa », when the edition date is known.
    edited_at: Option<String>,
    charges: Vec<LignePayload>,
    dues: Vec<LignePayload>,
    fund: Vec<LignePayload>,
    procedures_count: usize,
    emprunts: String,
}

/// One quote-part row of the questionnaire annex (N-1 / N-2).
#[derive(Serialize)]
struct QuotePartPayload {
    year: String,
    /// Called amount (appelée) — pre-filled from the snapshot.
    appelee: String,
    /// Actually paid amount (réelle) — not modeled, left as an anchor.
    reelle: Option<String>,
}

/// One row of the "travaux votés" tables (pages 6 and 10 of the CSN form).
/// Only `nature` is known today (snapshot); the rest are anchors.
#[derive(Serialize)]
struct TravauxRowPayload {
    date: Option<String>,
    nature: String,
    etat: Option<String>,
    financier: Option<String>,
    quote_part: Option<String>,
    appele: Option<String>,
    restant: Option<String>,
    exigibilite: Option<String>,
}

/// Payload of the `questionnaire.typ` annex. Every fillable field of the CSN
/// form is an anchor: pre-filled when the app knows the value, `None`/empty
/// (blank line or unchecked box) when the syndic must complete it by hand.
#[derive(Serialize)]
struct QuestionnairePayload {
    title: String,
    app_name: &'static str,
    edited_at: String,
    // ── Page 1 : informations des parties ──
    acquereur: Option<String>,
    date_demande: String,
    office_notarial: String,
    reference_dossier: Option<String>,
    dossier_clerc: Option<String>,
    delivre_par: String,
    syndicat_unique: bool,
    contact_syndic: Option<String>,
    date_delivrance: String,
    immeuble_nom: String,
    immeuble_adresse: String,
    cedant_nom: Option<String>,
    cedant_adresse: Option<String>,
    lots: String,
    mutation_onereuse: Option<bool>,
    date_mutation_envisagee: Option<String>,
    // ── 1ère partie : sommes dues par le cédant ──
    part1: Vec<LignePayload>,
    part1_emprunts_tiers: Vec<LignePayload>,
    total_part1: String,
    // ── 2ème partie : sommes dont le syndicat est débiteur ──
    part2: Vec<LignePayload>,
    total_part2: String,
    remb_solution1: Option<bool>,
    remb_solution2: Option<bool>,
    remb_montant: Option<String>,
    // ── 3ème partie : sommes incombant à l'acquéreur ──
    part3: Vec<LignePayload>,
    /// label = date d'exigibilité, amount = montant (3 lignes, à remplir).
    part3_provisions_budget: Vec<LignePayload>,
    part3_provisions_hors_budget: Vec<LignePayload>,
    total_part3: String,
    // ── Annexe à la 3ème partie ──
    actualisation_date: String,
    qp_budget: Vec<QuotePartPayload>,
    qp_hors_budget: Vec<QuotePartPayload>,
    procedures_oui: bool,
    procedures: Vec<String>,
    procedures_etat: Option<String>,
    fonds_travaux_existe: bool,
    fonds_travaux_quote_part: String,
    fonds_travaux_derniere_cotisation: Option<String>,
    // ── Événements après délivrance ──
    nouvelle_ag: Option<bool>,
    travaux_post: Vec<TravauxRowPayload>,
    commentaires_travaux: Option<String>,
    // ── Sommes exigibles / certificat art. 20 ──
    date_signature: Option<String>,
    sommes_exigibles: Vec<LignePayload>,
    montant_lots_mutation: String,
    lots_non_vises: Option<String>,
    total_sommes_exigibles: String,
    certificat_art20_joint: Option<bool>,
    copro_a_jour: bool,
    somme_debitrice: String,
    date_certificat: Option<String>,
    // ── A1 assurances ──
    assur_multirisque: Option<bool>,
    assur_reconstruction: Option<bool>,
    assur_valeur_neuf: Option<bool>,
    assur_capital_limite: Option<String>,
    assur_autres_risques: Option<String>,
    assur_police_numero: Option<String>,
    assur_police_date: Option<String>,
    assur_courtier: Option<String>,
    assur_compagnie: Option<String>,
    assur_do_origine: Option<bool>,
    assur_do_travaux: Option<bool>,
    // ── A2 règlement de copropriété / A3 AG / A4 syndic ──
    reglement_date: Option<String>,
    reglement_modifie: Option<bool>,
    reglement_sru: Option<bool>,
    derniere_ag: Option<String>,
    prochaine_ag: Option<String>,
    syndic_designation_date: Option<String>,
    syndic_pro: Option<bool>,
    // ── A5 à A12 ──
    aful: Option<bool>,
    aful_detail: Option<String>,
    travaux_commentaires: Option<String>,
    patrimoine: Option<bool>,
    patrimoine_detail: Option<String>,
    contrats_revenus: Option<bool>,
    contrats_revenus_detail: Option<String>,
    emprunt_existe: Option<bool>,
    emprunt_objet: Option<String>,
    emprunt_organisme: Option<String>,
    emprunt_reference: Option<String>,
    emprunt_capital_restant: Option<String>,
    emprunt_exigible_mutation: Option<bool>,
    emprunts_collectifs: String,
    admin_provisoire: Option<bool>,
    mandataire_ad_hoc: Option<bool>,
    priorite_stationnement_vote: Option<bool>,
    priorite_stationnement_clause: Option<bool>,
    surelevation_vote: Option<bool>,
    surelevation_clause: Option<bool>,
    immatriculation: Option<bool>,
    immatriculation_numero: Option<String>,
    // ── B dossier technique et environnemental (toutes ancres) ──
    date_construction: Option<String>,
    carnet_entretien: Option<bool>,
    type_igh: Option<bool>,
    amiante_soumis: Option<bool>,
    amiante_absence: Option<bool>,
    amiante_dta: Option<bool>,
    plomb_avant_1949: Option<bool>,
    termites_recherche: Option<bool>,
    mesures_urgence: Option<bool>,
    risques_sanitaires: Option<bool>,
    risques_sanitaires_traites: Option<bool>,
    dpe_concerne: Option<bool>,
    dpe_ou_audit: Option<String>,
    plan_travaux_energie: Option<bool>,
    cpe: Option<bool>,
    ascenseur_existe: Option<bool>,
    ascenseur_avant_2000: Option<bool>,
    ascenseur_controle: Option<bool>,
    ascenseur_normes_2010: Option<bool>,
    ascenseur_normes_2013: Option<bool>,
    ascenseur_normes_2018: Option<bool>,
    piscine_existe: Option<bool>,
    piscine_securite: Option<bool>,
    arrete_peril: Option<bool>,
    insalubrite: Option<bool>,
    injonction_travaux: Option<bool>,
    interdiction_habiter: Option<bool>,
    monument_historique: Option<bool>,
    ravalement: Option<bool>,
    opah: Option<bool>,
    icpe: Option<String>,
}

// ── Assembly ───────────────────────────────────────────────────────────────

/// Residence + syndic + lot header lines, shared by both documents.
fn base_header_lines(header: &LegalDocumentHeader) -> Vec<String> {
    let syndic_line = match (&header.syndic_name, &header.syndic_siret) {
        (Some(name), Some(siret)) => format!("Syndic : {name} — SIRET {siret}"),
        (Some(name), None) => format!("Syndic : {name}"),
        (None, Some(siret)) => format!("Syndic : SIRET {siret}"),
        (None, None) => "Syndic : non renseigné".to_string(),
    };
    vec![
        format!("Syndicat des copropriétaires : {}", header.building_name),
        format!("Adresse : {}", header.building_address),
        syndic_line,
        format!("Lot : {}", header.unit_number),
    ]
}

fn ligne_payloads(lines: &[crate::domain::etat_date::LigneMontant]) -> Vec<LignePayload> {
    lines
        .iter()
        .map(|line| LignePayload {
            label: line.label.clone(),
            amount: format_euro(line.amount),
        })
        .collect()
}

/// Pre-formatted procedure lines (« subject — date (jurisdiction) »), shared
/// by the état daté body and the questionnaire annex.
fn procedure_lines(snapshot: &EtatDate) -> Vec<String> {
    snapshot
        .procedures
        .iter()
        .map(|proc| {
            let jurisdiction = proc
                .jurisdiction
                .as_deref()
                .map(|j| format!(" ({j})"))
                .unwrap_or_default();
            format!(
                "{} — {}{}",
                proc.subject,
                format_date(proc.opened_at),
                jurisdiction
            )
        })
        .collect()
}

/// Builds the payload of the questionnaire annex (CSN form) from the document
/// header and the frozen état daté snapshot. Fields the app knows are
/// pre-filled; everything else stays `None` (blank anchor in the template,
/// to be completed by hand by the syndic).
fn questionnaire_payload(
    header: &LegalDocumentHeader,
    snapshot: &EtatDate,
    edited_at: &str,
) -> QuestionnairePayload {
    let generated = header
        .generated_at
        .unwrap_or_else(|| chrono::Utc::now().naive_utc().date());
    let total_part1: Decimal = snapshot
        .partie1_sommes_dues_par_cedant
        .iter()
        .map(|l| l.amount)
        .sum();
    let total_part2: Decimal = snapshot
        .partie2_sommes_dues_par_syndicat
        .iter()
        .map(|l| l.amount)
        .sum();
    let total_part3: Decimal = snapshot
        .partie3_sommes_incombant_acquereur
        .iter()
        .map(|l| l.amount)
        .sum();
    let arrears = snapshot
        .partie1_sommes_dues_par_cedant
        .iter()
        .find(|l| l.label.contains("arriérés"))
        .map(|l| l.amount);
    let avances = snapshot
        .partie2_sommes_dues_par_syndicat
        .first()
        .map(|l| l.amount);
    let fonds_travaux_qp = snapshot
        .partie3_sommes_incombant_acquereur
        .iter()
        .find(|l| l.label.contains("fonds travaux"))
        .map(|l| l.amount);
    let provisions_a_venir = snapshot
        .partie3_sommes_incombant_acquereur
        .iter()
        .find(|l| l.label.contains("Provisions à venir"))
        .map(|l| l.amount);
    let fonds_annuel = snapshot
        .annexe_quote_part
        .first()
        .map(|e| e.works_fund_amount);

    let ligne = |label: &str, amount: Option<Decimal>| LignePayload {
        label: label.to_string(),
        amount: amount.map(format_euro).unwrap_or_default(),
    };

    let part1 = vec![
        ligne(
            "1- Provisions exigibles — dans le budget prévisionnel (D. art. 5. 1° a)",
            None,
        ),
        ligne(
            "- dans les dépenses hors budget prévisionnel (D. art. 5. 1° b)",
            None,
        ),
        ligne(
            "2- Charges impayées sur les exercices antérieurs (D. art. 5. 1° c)",
            arrears,
        ),
        ligne(
            "3- Sommes devenues exigibles du fait de la vente (art. 33 loi 65-557, D. art. 5. 1° d)",
            None,
        ),
        ligne(
            "4- Avances exigibles (D. art. 5. 1° e) — réserve (D. art. 35. 1°)",
            None,
        ),
        ligne(
            "- avances nommées provisions (L. art. 18 al. 15, D. art. 35. 4° et 5°)",
            None,
        ),
        ligne(
            "- avances représentant un emprunt (D. art. 45-1 al. 4)",
            None,
        ),
        ligne("5- Cotisations annuelles au fonds de travaux", fonds_annuel),
        ligne(
            "6- Autres sommes exigibles du fait de la vente — prêt (quote-part exigible)",
            None,
        ),
        ligne("- autres causes (condamnations, etc.)", None),
        ligne("7- Frais de délivrance de l'état daté", None),
    ];
    let part1_emprunts_tiers = vec![ligne(
        "Emprunts par certains copropriétaires dont la gestion est assurée par le syndic",
        None,
    )];
    let part2 = vec![
        ligne("A/ Avances perçues — réserve (D. art. 35. 1°)", None),
        ligne("- avances nommées provisions (D. art. 35. 4° et 5°)", None),
        ligne(
            "- emprunt du syndicat auprès des copropriétaires (D. art. 45-1 al. 4)",
            None,
        ),
        ligne(
            "B/ Provisions encaissées rendues exigibles par déchéance du terme (art. 19-2 loi 65-557)",
            avances,
        ),
        ligne(
            "C/ Solde créditeur de l'exercice antérieur non imputé",
            None,
        ),
    ];
    let part3 = vec![
        ligne("1- Reconstitution des avances — réserve", None),
        ligne("- avances nommées provisions", None),
        ligne("- emprunt (D. art. 45-1 al. 4)", None),
        ligne(
            "2- Provisions non encore exigibles — budget prévisionnel (D. art. 5. 3° b)",
            provisions_a_venir,
        ),
        ligne(
            "- dépenses hors budget prévisionnel (D. art. 5. 3° c)",
            None,
        ),
        ligne(
            "Quote-part du lot sur le fonds de travaux (D. art. 5. 3° a)",
            fonds_travaux_qp,
        ),
    ];

    let montant_mutation = total_part1 - total_part2;
    let a_jour = total_part1.is_zero();

    QuestionnairePayload {
        title: "Questionnaire syndic — état daté".to_string(),
        app_name: crate::domain::pdf_template::APP_NAME,
        edited_at: edited_at.to_string(),
        acquereur: None,
        date_demande: header.requested_at.map(format_date).unwrap_or_default(),
        office_notarial: header.requester_name.clone().unwrap_or_default(),
        reference_dossier: None,
        dossier_clerc: None,
        delivre_par: header
            .syndic_name
            .clone()
            .unwrap_or_else(|| crate::domain::pdf_template::APP_NAME.to_string()),
        syndicat_unique: true,
        contact_syndic: None,
        date_delivrance: format_date(generated),
        immeuble_nom: header.building_name.clone(),
        immeuble_adresse: header.building_address.clone(),
        cedant_nom: header.owner_name.clone(),
        cedant_adresse: None,
        lots: header.unit_number.clone(),
        mutation_onereuse: None,
        date_mutation_envisagee: None,
        part1,
        part1_emprunts_tiers,
        total_part1: format_euro(total_part1),
        part2,
        total_part2: format_euro(total_part2),
        remb_solution1: None,
        remb_solution2: None,
        remb_montant: avances.map(format_euro),
        part3,
        part3_provisions_budget: Vec::new(),
        part3_provisions_hors_budget: Vec::new(),
        total_part3: format_euro(total_part3),
        actualisation_date: edited_at.to_string(),
        qp_budget: snapshot
            .annexe_quote_part
            .iter()
            .map(|ex| QuotePartPayload {
                year: ex.fiscal_year.to_string(),
                appelee: format_euro(ex.charge_amount),
                reelle: None,
            })
            .collect(),
        qp_hors_budget: snapshot
            .annexe_quote_part
            .iter()
            .map(|ex| QuotePartPayload {
                year: ex.fiscal_year.to_string(),
                appelee: format_euro(ex.works_fund_amount),
                reelle: None,
            })
            .collect(),
        procedures_oui: !snapshot.procedures.is_empty(),
        procedures: procedure_lines(snapshot),
        procedures_etat: None,
        fonds_travaux_existe: fonds_travaux_qp.is_some_and(|a| !a.is_zero())
            || fonds_annuel.is_some_and(|a| !a.is_zero()),
        fonds_travaux_quote_part: fonds_travaux_qp.map(format_euro).unwrap_or_default(),
        fonds_travaux_derniere_cotisation: header
            .fonds_travaux_derniere_cotisation
            .map(format_euro),
        nouvelle_ag: None,
        travaux_post: snapshot
            .travaux_votes_non_realises
            .iter()
            .map(|nature| TravauxRowPayload {
                date: None,
                nature: nature.clone(),
                etat: None,
                financier: None,
                quote_part: None,
                appele: None,
                restant: None,
                exigibilite: None,
            })
            .collect(),
        commentaires_travaux: None,
        date_signature: None,
        sommes_exigibles: vec![
            ligne(
                "1- Montant concernant les lots objets de la mutation (total A+B de la 1ère partie, sous déduction du total B+C de la 2ème partie)",
                Some(montant_mutation),
            ),
            ligne(
                "2- Montant concernant les lots non visés par la mutation (lots n°)",
                None,
            ),
        ],
        montant_lots_mutation: format_euro(montant_mutation),
        lots_non_vises: None,
        total_sommes_exigibles: format_euro(montant_mutation),
        certificat_art20_joint: None,
        copro_a_jour: a_jour,
        somme_debitrice: format_euro(total_part1),
        date_certificat: None,
        assur_multirisque: None,
        assur_reconstruction: None,
        assur_valeur_neuf: None,
        assur_capital_limite: None,
        assur_autres_risques: None,
        assur_police_numero: None,
        assur_police_date: None,
        assur_courtier: None,
        assur_compagnie: None,
        assur_do_origine: None,
        assur_do_travaux: None,
        reglement_date: header.bylaws_date.map(format_date),
        reglement_modifie: None,
        reglement_sru: None,
        derniere_ag: header.last_ag_date.map(format_date),
        prochaine_ag: header.next_ag_date.map(format_date),
        syndic_designation_date: header.syndic_mandate_start.map(format_date),
        syndic_pro: None,
        aful: None,
        aful_detail: None,
        travaux_commentaires: None,
        patrimoine: None,
        patrimoine_detail: None,
        contrats_revenus: None,
        contrats_revenus_detail: None,
        emprunt_existe: Some(!snapshot.emprunts_collectifs.contains("néant")),
        emprunt_objet: None,
        emprunt_organisme: None,
        emprunt_reference: None,
        emprunt_capital_restant: None,
        emprunt_exigible_mutation: None,
        emprunts_collectifs: snapshot.emprunts_collectifs.clone(),
        admin_provisoire: None,
        mandataire_ad_hoc: None,
        priorite_stationnement_vote: None,
        priorite_stationnement_clause: None,
        surelevation_vote: None,
        surelevation_clause: None,
        immatriculation: None,
        immatriculation_numero: None,
        date_construction: None,
        carnet_entretien: None,
        type_igh: None,
        amiante_soumis: None,
        amiante_absence: None,
        amiante_dta: None,
        plomb_avant_1949: None,
        termites_recherche: None,
        mesures_urgence: None,
        risques_sanitaires: None,
        risques_sanitaires_traites: None,
        dpe_concerne: None,
        dpe_ou_audit: None,
        plan_travaux_energie: None,
        cpe: None,
        ascenseur_existe: None,
        ascenseur_avant_2000: None,
        ascenseur_controle: None,
        ascenseur_normes_2010: None,
        ascenseur_normes_2013: None,
        ascenseur_normes_2018: None,
        piscine_existe: None,
        piscine_securite: None,
        arrete_peril: None,
        insalubrite: None,
        injonction_travaux: None,
        interdiction_habiter: None,
        monument_historique: None,
        ravalement: None,
        opah: None,
        icpe: None,
    }
}

/// Renders the état daté (EP-002-S06) as a PDF from its frozen snapshot.
pub fn render_etat_date(
    header: &LegalDocumentHeader,
    snapshot: &EtatDate,
) -> Result<Vec<u8>, String> {
    let mut header_lines = base_header_lines(header);
    if let Some(requester) = &header.requester_name {
        header_lines.push(format!("Demande de : {requester}"));
    }
    if let Some(date) = header.requested_at {
        header_lines.push(format!("Date de la demande : {}", format_date(date)));
    }
    if let Some(date) = header.due_date {
        header_lines.push(format!("Échéance légale (J+15) : {}", format_date(date)));
    }
    if let Some(date) = header.generated_at {
        header_lines.push(format!("Date de génération : {}", format_date(date)));
    }

    let issuer_lines = {
        let mut lines = vec![
            header.building_name.clone(),
            header.building_address.clone(),
        ];
        if let Some(name) = &header.syndic_name {
            lines.push(name.clone());
        }
        if let Some(siret) = &header.syndic_siret {
            lines.push(format!("SIRET {siret}"));
        }
        lines
    };
    let recipient_lines = {
        let mut lines = Vec::new();
        if let Some(name) = &header.requester_name {
            lines.push(name.clone());
        }
        lines.push(format!("Lot n° {}", header.unit_number));
        lines
    };
    let edited_at = header
        .generated_at
        .map(format_date)
        .unwrap_or_else(|| format_date(chrono::Utc::now().naive_utc().date()));

    // ── Letter body (same frame as the logbook) ──
    let objet = format!("État daté — Lot n° {}", header.unit_number);
    let reference = header.requested_at.map(|d| {
        format!(
            "Demande du {} · Échéance légale (J+15) : {}",
            format_date(d),
            format_date(header.due_date.unwrap_or(d))
        )
    });
    let letter_paragraphs = vec![
        format!(
            "Conformément à l'article 46 de la loi n° 65-557 du 10 juillet 1965 et au décret n° 67-223 du 17 mars 1967, le syndic de la copropriété « {} » vous communique, à votre demande, l'état daté du lot n° {} au {}.",
            header.building_name,
            header.unit_number,
            format_date(
                header
                    .generated_at
                    .unwrap_or_else(|| chrono::Utc::now().naive_utc().date())
            )
        ),
        "Ce document, établi à la date de génération indiquée ci-dessus, détaille en première partie les sommes pouvant rester dues par le copropriétaire cédant, en deuxième partie les sommes dont le syndicat pourrait être débiteur envers le cédant, et en troisième partie les sommes incombant au nouveau copropriétaire. L'annexe présente la quote-part du lot sur les deux derniers exercices, les procédures judiciaires en cours, les travaux votés non réalisés et les emprunts collectifs.".to_string(),
        "Vous trouverez en annexe de la présente le questionnaire syndic pré-rempli, complété à partir des informations tenues par le syndicat.".to_string(),
    ];
    let signer_name = header
        .syndic_name
        .clone()
        .unwrap_or_else(|| crate::domain::pdf_template::APP_NAME.to_string());
    let place_date = format!(
        "Fait le {}",
        crate::domain::pdf_template::format_date_long(
            header
                .generated_at
                .unwrap_or_else(|| chrono::Utc::now().naive_utc().date())
        )
    );

    let annex_subtitle = format!(
        "Lot n° {} — {} — généré le {}",
        header.unit_number, header.building_name, edited_at
    );

    let payload = EtatDatePayload {
        title: "État daté".to_string(),
        app_name: crate::domain::pdf_template::APP_NAME,
        edited_at: edited_at.clone(),
        issuer_heading: "Le syndic des copropriétaires".to_string(),
        issuer_lines,
        recipient_label: "Destinataire".to_string(),
        recipient_lines,
        objet,
        reference,
        salutation: "Madame, Monsieur,".to_string(),
        letter_paragraphs,
        closing_formula:
            "Je vous prie d'agréer, Madame, Monsieur, l'expression de mes salutations distinguées."
                .to_string(),
        signer_closing: "Pour le syndic".to_string(),
        signer_name,
        signer_role: header
            .syndic_name
            .as_ref()
            .map(|_| format!("Syndic de la copropriété « {} »", header.building_name)),
        place_date,
        annex_kicker: "Annexe".to_uppercase(),
        annex_title: "Questionnaire syndic — état daté".to_string(),
        annex_subtitle,
        header_lines,
        part1: ligne_payloads(&snapshot.partie1_sommes_dues_par_cedant),
        part2: ligne_payloads(&snapshot.partie2_sommes_dues_par_syndicat),
        part3: ligne_payloads(&snapshot.partie3_sommes_incombant_acquereur),
        exercices: snapshot
            .annexe_quote_part
            .iter()
            .map(|ex| ExercicePayload {
                year: ex.fiscal_year.to_string(),
                charges: format_euro(ex.charge_amount),
                travaux: format_euro(ex.works_fund_amount),
                total: format_euro(ex.total_amount),
            })
            .collect(),
        procedures: procedure_lines(snapshot),
        travaux: snapshot.travaux_votes_non_realises.clone(),
        emprunts: snapshot.emprunts_collectifs.clone(),
    };
    let pdf = crate::domain::pdf_template::compile_template("etat_date", &payload)?;
    let annex = crate::domain::pdf_template::compile_template(
        "questionnaire",
        &questionnaire_payload(header, snapshot, &edited_at),
    )?;
    let mut merged = pdf;
    append_pdf_annex(&mut merged, &annex)?;
    Ok(merged)
}

/// Appends the pages of `annex` to the end of `pdf` (lopdf 0.14 merge).
/// Used to attach the pre-filled questionnaire as an annex of the état daté.
///
/// Both PDFs are loaded, the annex objects are copied into the base document
/// with remapped object IDs, all references are rewritten, and the annex
/// page IDs are appended to the base page tree.
fn append_pdf_annex(pdf: &mut Vec<u8>, annex: &[u8]) -> Result<(), String> {
    use lopdf::{Object, ObjectId};
    use std::collections::BTreeMap;

    fn load(bytes: &[u8]) -> Result<lopdf::Document, String> {
        lopdf::Document::load_from(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())
    }

    fn remap(obj: &mut Object, map: &BTreeMap<ObjectId, ObjectId>) {
        match obj {
            Object::Reference(id) => {
                if let Some(&new_id) = map.get(id) {
                    *obj = Object::Reference(new_id);
                }
            }
            Object::Array(arr) => {
                for item in arr.iter_mut() {
                    remap(item, map);
                }
            }
            Object::Dictionary(dict) => {
                for (_, val) in dict.iter_mut() {
                    remap(val, map);
                }
            }
            Object::Stream(stream) => {
                for (_, val) in stream.dict.iter_mut() {
                    remap(val, map);
                }
            }
            _ => {}
        }
    }

    let mut base = load(pdf)?;
    let annex_doc = load(annex)?;

    // Copy every annex object into the base document under a fresh ID range.
    let offset = base.max_id + 1;
    let mut id_map: BTreeMap<ObjectId, ObjectId> = BTreeMap::new();
    for (&(num, generation), obj) in &annex_doc.objects {
        let new_id: ObjectId = (offset + num, generation);
        id_map.insert((num, generation), new_id);
        base.objects.insert(new_id, obj.clone());
    }
    base.max_id = offset + annex_doc.max_id;

    // Rewrite all references inside the copied objects.
    let copied: Vec<ObjectId> = id_map.values().copied().collect();
    for new_id in copied {
        if let Some(obj) = base.objects.get_mut(&new_id) {
            remap(obj, &id_map);
        }
    }

    // Append the annex pages to the base page tree.
    let annex_pages: Vec<ObjectId> = annex_doc
        .get_pages()
        .values()
        .filter_map(|old_id| id_map.get(old_id).copied())
        .collect();

    let catalog_id = base
        .trailer
        .get("Root")
        .and_then(|o| o.as_reference())
        .ok_or("base PDF has no catalog")?;
    let catalog = base
        .objects
        .get_mut(&catalog_id)
        .and_then(|o| o.as_dict_mut())
        .ok_or("base catalog is not a dictionary")?;
    let pages_id = catalog
        .get("Pages")
        .and_then(|o| o.as_reference())
        .ok_or("base catalog has no /Pages")?;
    let pages_dict = base
        .objects
        .get_mut(&pages_id)
        .and_then(|o| o.as_dict_mut())
        .ok_or("base /Pages is not a dictionary")?;
    {
        let kids = pages_dict
            .get_mut("Kids")
            .and_then(|o| o.as_array_mut())
            .ok_or("base /Pages has no /Kids array")?;
        for page_id in &annex_pages {
            kids.push(Object::Reference(*page_id));
        }
    }
    let count = pages_dict
        .get("Count")
        .and_then(|o| o.as_i64())
        .unwrap_or(0)
        + annex_pages.len() as i64;
    pages_dict.set("Count", Object::Integer(count));
    // Point each annex page's /Parent at the base page tree.
    for page_id in &annex_pages {
        if let Some(page) = base.objects.get_mut(page_id).and_then(|o| o.as_dict_mut()) {
            page.set("Parent", Object::Reference(pages_id));
        }
    }

    let mut out = Vec::new();
    base.save_to(&mut out).map_err(|e| e.to_string())?;
    *pdf = out;
    Ok(())
}

/// Renders the pré-état daté (EP-002-S07, art. L721-2 CCH) as a PDF.
pub fn render_pre_etat_date(
    header: &LegalDocumentHeader,
    data: &PreEtatDate,
) -> Result<Vec<u8>, String> {
    let payload = PreEtatDatePayload {
        title: "Pré-état daté".to_string(),
        header_lines: base_header_lines(header),
        edited_at: header
            .edited_at
            .map(|date| format!("Édité le : {}", format_date(date))),
        charges: vec![
            LignePayload {
                label: "Charges budget prévisionnel — exercice N-1".to_string(),
                amount: optional_euro(data.charges_courantes_n1),
            },
            LignePayload {
                label: "Charges budget prévisionnel — exercice N-2".to_string(),
                amount: optional_euro(data.charges_courantes_n2),
            },
            LignePayload {
                label: "Charges hors budget — exercice N-1".to_string(),
                amount: optional_euro(data.charges_hors_budget_n1),
            },
            LignePayload {
                label: "Charges hors budget — exercice N-2".to_string(),
                amount: optional_euro(data.charges_hors_budget_n2),
            },
        ],
        dues: vec![
            LignePayload {
                label: "Sommes dues par le cédant".to_string(),
                amount: format_euro(data.sommes_dues_par_cedant),
            },
            LignePayload {
                label: "État global des impayés du syndicat".to_string(),
                amount: format_euro(data.impayes_globaux_syndicat),
            },
            LignePayload {
                label: "Dette fournisseurs".to_string(),
                amount: optional_euro(data.dette_fournisseurs),
            },
        ],
        fund: vec![
            LignePayload {
                label: "Quote-part du lot".to_string(),
                amount: format_euro(data.fonds_travaux_quote_part),
            },
            LignePayload {
                label: "Dernière cotisation".to_string(),
                amount: format_euro(data.fonds_travaux_derniere_cotisation),
            },
        ],
        procedures_count: data.procedures_en_cours,
        emprunts: data.emprunts_collectifs.clone(),
    };
    crate::domain::pdf_template::compile_template("pre_etat_date", &payload)
}

/// `Option<Decimal>` → either the formatted amount or « Non disponible ».
fn optional_euro(value: Option<Decimal>) -> String {
    value
        .map(format_euro)
        .unwrap_or_else(|| "Non disponible".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::etat_date::{EtatDateInputs, ExerciceQuotePart, ProcedureSummary};

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn header() -> LegalDocumentHeader {
        LegalDocumentHeader {
            building_name: "Résidence des Tilleuls".to_string(),
            building_address: "12 rue des Fleurs, 75011 Paris".to_string(),
            syndic_name: Some("Cabinet Dupont".to_string()),
            syndic_siret: Some("123 456 789 00012".to_string()),
            unit_number: "A-12".to_string(),
            requester_name: Some("Maître Martin".to_string()),
            requested_at: Some(date(2026, 6, 1)),
            due_date: Some(date(2026, 6, 16)),
            generated_at: Some(date(2026, 6, 2)),
            edited_at: Some(date(2026, 6, 2)),
            owner_name: Some("Mme Loisel".to_string()),
            bylaws_date: Some(date(1987, 3, 12)),
            syndic_mandate_start: Some(date(2024, 6, 1)),
            last_ag_date: Some(date(2026, 5, 20)),
            next_ag_date: Some(date(2026, 11, 18)),
            fonds_travaux_derniere_cotisation: Some(Decimal::new(50000, 2)),
        }
    }

    fn inputs(arrears: Decimal) -> EtatDateInputs {
        EtatDateInputs {
            arrears,
            avances_syndicat: Decimal::ZERO,
            provisions_a_venir: Decimal::ZERO,
            fonds_travaux_quote_part: Decimal::new(50000, 2),
            exercices: vec![ExerciceQuotePart {
                fiscal_year: 2026,
                charge_amount: Decimal::new(100000, 2),
                works_fund_amount: Decimal::new(5000, 2),
                total_amount: Decimal::new(105000, 2),
            }],
            procedures: vec![ProcedureSummary {
                subject: "Recouvrement".to_string(),
                proceeding_type: "debt_recovery".to_string(),
                opened_at: date(2026, 1, 1),
                jurisdiction: Some("TJ Paris".to_string()),
            }],
        }
    }

    #[test]
    fn format_euro_formats_decimals_in_french_convention() {
        assert_eq!(format_euro(Decimal::new(150000, 2)), "1 500,00 €");
        assert_eq!(format_euro(Decimal::new(123456789, 2)), "1 234 567,89 €");
        assert_eq!(format_euro(Decimal::ZERO), "0,00 €");
        assert_eq!(format_euro(Decimal::new(-1234, 2)), "-12,34 €");
    }

    #[test]
    fn format_euro_pads_single_digit_cents() {
        assert_eq!(format_euro(Decimal::new(105, 2)), "1,05 €");
    }

    #[test]
    fn optional_euro_renders_non_disponible_for_none() {
        assert_eq!(optional_euro(None), "Non disponible");
        assert_eq!(optional_euro(Some(Decimal::new(100, 2))), "1,00 €");
    }

    #[test]
    fn format_date_uses_day_month_year() {
        assert_eq!(format_date(date(2026, 6, 16)), "16/06/2026");
    }

    #[test]
    fn render_etat_date_produces_valid_pdf() {
        let snapshot = crate::domain::etat_date::assemble(inputs(Decimal::new(150000, 2)));
        let bytes = render_etat_date(&header(), &snapshot).expect("render");
        assert!(bytes.starts_with(b"%PDF"));
        assert!(!bytes.is_empty());
    }

    #[test]
    fn render_etat_date_appends_questionnaire_annex() {
        let snapshot = crate::domain::etat_date::assemble(inputs(Decimal::new(150000, 2)));
        let h = header();
        // The merged document must parse and contain the typst-rendered
        // questionnaire pages in addition to the état daté body alone.
        let body = crate::domain::pdf_template::compile_template(
            "etat_date",
            &EtatDatePayload {
                title: "État daté".to_string(),
                app_name: crate::domain::pdf_template::APP_NAME,
                edited_at: "02/06/2026".to_string(),
                issuer_heading: "Le syndic des copropriétaires".to_string(),
                issuer_lines: vec![],
                recipient_label: "Destinataire".to_string(),
                recipient_lines: vec!["Lot n° A-12".to_string()],
                objet: "État daté — Lot n° A-12".to_string(),
                reference: None,
                salutation: "Madame, Monsieur,".to_string(),
                letter_paragraphs: vec![],
                closing_formula: "Je vous prie d'agréer, Madame, Monsieur, l'expression de mes salutations distinguées.".to_string(),
                signer_closing: "Pour le syndic".to_string(),
                signer_name: "Cabinet Dupont".to_string(),
                signer_role: None,
                place_date: "Fait le 2 juin 2026".to_string(),
                annex_kicker: "ANNEXE".to_string(),
                annex_title: "Questionnaire syndic — état daté".to_string(),
                annex_subtitle: "Lot n° A-12".to_string(),
                header_lines: vec![],
                part1: vec![],
                part2: vec![],
                part3: vec![],
                exercices: vec![],
                procedures: vec![],
                travaux: vec![],
                emprunts: String::new(),
            },
        )
        .expect("body compile");
        let merged = render_etat_date(&h, &snapshot).expect("render");
        let body_pages = lopdf::Document::load_from(std::io::Cursor::new(&body))
            .expect("body parse")
            .get_pages()
            .len();
        let merged_pages = lopdf::Document::load_from(std::io::Cursor::new(&merged))
            .expect("merged parse")
            .get_pages()
            .len();
        assert!(merged_pages > body_pages);
    }

    #[test]
    fn render_pre_etat_date_produces_valid_pdf() {
        let pre = crate::domain::etat_date::assemble_pre_etat_date(
            inputs(Decimal::new(150000, 2)),
            Decimal::new(20000, 2),
            Decimal::new(15000, 2),
        );
        let bytes = render_pre_etat_date(&header(), &pre).expect("render");
        assert!(bytes.starts_with(b"%PDF"));
        assert!(!bytes.is_empty());
    }
}

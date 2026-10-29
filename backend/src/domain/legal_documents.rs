//! Regulatory PDF rendering (ADR-011, LEGAL.md §7.1 état daté / §7.2 pré-état daté).
//!
//! Pure functions only — no I/O, no SQL. Callers load and assemble the data
//! (domain + db), hand it here together with a document header, and receive
//! the finished PDF as in-memory bytes. The documents are composed by the
//! shared typst engine (`domain/pdf_template.rs`, templates in
//! `domain/typst/`): Liberation Sans, centered bold title, no brand block and
//! no running footer (visual unchanged from the previous engine). Fonts are
//! embedded at compile time so the rendering is self-contained (no system
//! font dependency, matching the `debian:trixie-slim` runtime image).
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

    let payload = EtatDatePayload {
        title: "État daté".to_string(),
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
        procedures: snapshot
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
            .collect(),
        travaux: snapshot.travaux_votes_non_realises.clone(),
        emprunts: snapshot.emprunts_collectifs.clone(),
    };
    crate::domain::pdf_template::compile_template("etat_date", &payload)
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

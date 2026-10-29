//! Logbook PDF rendering (ADR-011, typst engine).
//!
//! Renders the regulatory logbook — « Carnet d'entretien de l'immeuble »
//! (décret n°2001-477, art. 1) — as a **letter + annex** PDF composed by the
//! shared typst engine (`domain/pdf_template.rs`, templates in
//! `domain/typst/`) :
//!
//! - **Page 1 — letter** : brand block « app web » (logo + « La Ruche » in
//!   Fraunces + tagline in DM Sans), issuer on the left and recipient on the
//!   right aligned on the same line, purple hairline, bold subject + reference,
//!   salutation, **justified** paragraphs (including the explicit annex
//!   announcement), closing formula, then right-aligned signature (« Pour le
//!   syndic », name, role, « Fait à …, le … » filled in).
//! - **Following pages — annex** : spaced kicker « A N N E X E », logbook title
//!   in brand color, co-ownership identity meta, then active insurance
//!   contracts, active maintenance contracts and the intervention chronology
//!   (the tabular content announced in the letter).
//! - **Recurring footer** on every page (mini logo + edition line + legal
//!   disclaimer, centered).
//!
//! All formatting (amounts, dates, counts) happens here in Rust; the template
//! only consumes pre-formatted strings. Pure functions only — no I/O, no SQL.
//! The API layer assembles the data and receives the finished PDF as
//! in-memory bytes.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Serialize;

use crate::domain::pdf_template as tpl;

/// One active contract line in the logbook export.
#[derive(Debug, Clone)]
pub struct LogbookContractLine {
    pub subject: String,
    pub vendor_name: String,
    /// Contract type label already translated by the caller (e.g. « Maintenance »).
    pub contract_type_label: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub amount: Option<Decimal>,
}

/// One intervention line in the logbook chronology.
#[derive(Debug, Clone)]
pub struct LogbookEntryLine {
    pub intervention_date: NaiveDate,
    pub title: String,
    pub description: Option<String>,
    pub vendor_name: Option<String>,
    /// Category label already translated by the caller (e.g. « Travaux »).
    pub category_label: String,
    /// Source label already translated by the caller (e.g. « Saisie manuelle »).
    pub source_label: String,
    pub cost: Option<Decimal>,
}

/// All data needed to render the logbook PDF. Assembled by the API layer.
#[derive(Debug, Clone)]
pub struct LogbookPdfData {
    pub identity: tpl::DocumentIdentity,
    /// Edition date — rendered in the annex meta and the page footer.
    pub edited_at: NaiveDate,
    /// Syndic contact line under the identity (name + email/phone), if any.
    pub syndic_contact: Option<String>,
    /// Recipient lines (right block of the double header): the user who
    /// requested the document — name, then building name, street address and
    /// postal city.
    pub recipient_lines: Vec<String>,
    pub insurance_contracts: Vec<LogbookContractLine>,
    pub maintenance_contracts: Vec<LogbookContractLine>,
    /// Chronology, expected sorted ascending by `intervention_date`.
    pub entries: Vec<LogbookEntryLine>,
    /// Translated labels (i18n happens at the API boundary; the renderer is
    /// language-agnostic and receives final strings).
    pub labels: LogbookPdfLabels,
}

/// Translated strings used by the logbook renderer.
#[derive(Debug, Clone)]
pub struct LogbookPdfLabels {
    pub doc_title: String,
    // ── Page 1 : letter ──────────────────────────────────────────────────
    /// Issuer block heading on the left of the double header, e.g. « Le
    /// syndic des copropriétaires ».
    pub issuer_heading: String,
    /// Issuer line when no syndic is recorded, e.g. « Syndic non renseigné ».
    pub issuer_not_set: String,
    /// Recipient block label on the right of the double header, e.g.
    /// « Destinataire ».
    pub recipient_label: String,
    /// Bold letter subject line, e.g. « Carnet d'entretien de l'immeuble -
    /// édition 2026 ».
    pub objet: String,
    /// Optional grey reference line under the objet, e.g. « Réf.
    /// LR-CARNET-2026 · Édition du 27/10/2026 ».
    pub reference: Option<String>,
    /// Letter salutation, e.g. « Madame, Monsieur, ».
    pub salutation: String,
    /// Letter body paragraphs: description of the document, explicit
    /// announcement of the annex, validity statement, availability sentence.
    pub letter_paragraphs: Vec<String>,
    /// Closing formula of the letter, e.g. « Je vous prie d'agréer… ».
    pub closing_formula: String,
    /// Signature block closing, e.g. « Pour le syndic ».
    pub signer_closing: String,
    /// Signer role in small grey type, e.g. « Syndic en exercice - … ».
    pub signer_role: Option<String>,
    /// Place of issue for « Fait à {place}, le {date} » (signature) —
    /// usually the city of the copropriété.
    pub place: String,
    // ── Annex ────────────────────────────────────────────────────────────
    /// Annex kicker rendered letter-spaced, e.g. « ANNEXE ».
    pub annex_kicker: String,
    pub insurance_section: String,
    pub maintenance_section: String,
    pub chronology_section: String,
    pub no_contracts: String,
    pub no_entries: String,
    pub col_date: String,
    pub col_intervention: String,
    pub col_vendor: String,
    pub col_category: String,
    pub col_source: String,
    pub col_cost: String,
}

/// Document template for the logbook (concatenated after the shared base by
/// the engine).
const LOGBOOK_TEMPLATE: &str = include_str!("typst/logbook.typ");

/// Renders the logbook as a **letter + annex** PDF using the shared template.
pub fn render_logbook(data: &LogbookPdfData) -> Result<Vec<u8>, String> {
    tpl::compile_template(LOGBOOK_TEMPLATE, &build_payload(data))
}

// ── Payload (typst data boundary: pre-formatted strings only) ─────────────

#[derive(Serialize)]
struct LogbookPayload {
    title: String,
    app_name: &'static str,
    /// Edition date, `jj/mm/aaaa` (footer + annex meta).
    edited_at: String,
    // Double header
    issuer_heading: String,
    issuer_lines: Vec<String>,
    recipient_label: String,
    recipient_lines: Vec<String>,
    // Letter
    objet: String,
    reference: Option<String>,
    salutation: String,
    letter_paragraphs: Vec<String>,
    closing_formula: String,
    // Signature
    signer_closing: String,
    signer_name: String,
    signer_role: Option<String>,
    /// « Fait à {place}, le {date} » filled in.
    place_date: String,
    // Annex
    annex_kicker: String,
    annex_title: String,
    annex_subtitle: String,
    annex_meta: Vec<(String, String)>,
    insurance_section: ContractSectionPayload,
    maintenance_section: ContractSectionPayload,
    chronology: ChronologyPayload,
}

#[derive(Serialize)]
struct ContractSectionPayload {
    heading: String,
    empty_text: String,
    contracts: Vec<ContractPayload>,
}

#[derive(Serialize)]
struct ContractPayload {
    subject: String,
    /// « vendor » or « vendor · type ».
    vendor_type: String,
    /// « 01/01/2026 – 31/12/2026 » or « 01/01/2026 – … ».
    period: String,
    amount: Option<String>,
}

#[derive(Serialize)]
struct ChronologyPayload {
    heading: String,
    empty_text: String,
    col_date: String,
    col_intervention: String,
    col_vendor: String,
    col_category: String,
    col_source: String,
    col_cost: String,
    entries: Vec<EntryPayload>,
}

#[derive(Serialize)]
struct EntryPayload {
    date: String,
    title: String,
    description: Option<String>,
    /// Vendor name, or « - » when absent.
    vendor: String,
    category: String,
    source: String,
    /// Formatted cost, or « - » when absent.
    cost: String,
}

fn build_payload(data: &LogbookPdfData) -> LogbookPayload {
    let l = &data.labels;

    // Syndic line, shared by the issuer block and the annex meta.
    let syndic_line = match (&data.identity.syndic_name, &data.syndic_contact) {
        (Some(name), Some(contact)) => format!("{name} - {contact}"),
        (Some(name), None) => name.clone(),
        (None, _) => l.issuer_not_set.clone(),
    };

    // ── Double header ──
    let mut issuer_lines = vec![syndic_line.clone()];
    issuer_lines.push(data.identity.building_street_address.clone());
    issuer_lines.push(data.identity.building_postal_city.clone());
    if let Some(siret) = &data.identity.syndic_siret {
        issuer_lines.push(format!("SIRET : {siret}"));
    }

    // ── Signature ──
    // Without a recorded syndic: signed « La Ruche », without a role.
    let signer_role = match data.identity.syndic_name {
        Some(_) => l.signer_role.clone(),
        None => None,
    };
    let place_date = format!(
        "Fait à {}, le {}",
        l.place,
        tpl::format_date_long(data.edited_at)
    );

    // ── Annex meta ──
    let mut annex_meta = vec![
        ("Syndic en exercice".to_string(), syndic_line),
        (
            "Adresse".to_string(),
            data.identity.building_street_address.clone(),
        ),
        (
            "Code postal".to_string(),
            data.identity.building_postal_city.clone(),
        ),
    ];
    if let Some(siret) = &data.identity.syndic_siret {
        annex_meta.push(("SIRET".to_string(), siret.clone()));
    }
    annex_meta.push(("Édition du".to_string(), tpl::format_date(data.edited_at)));

    LogbookPayload {
        title: l.doc_title.clone(),
        app_name: tpl::APP_NAME,
        edited_at: tpl::format_date(data.edited_at),
        issuer_heading: l.issuer_heading.clone(),
        issuer_lines,
        recipient_label: l.recipient_label.clone(),
        recipient_lines: data.recipient_lines.clone(),
        objet: l.objet.clone(),
        reference: l.reference.clone(),
        salutation: l.salutation.clone(),
        letter_paragraphs: l.letter_paragraphs.clone(),
        closing_formula: l.closing_formula.clone(),
        signer_closing: l.signer_closing.clone(),
        signer_name: data
            .identity
            .syndic_name
            .clone()
            .unwrap_or_else(|| tpl::APP_NAME.to_string()),
        signer_role,
        place_date,
        annex_kicker: l.annex_kicker.to_uppercase(),
        annex_title: l.doc_title.clone(),
        annex_subtitle: format!(
            "{} - {}",
            data.identity.building_name, data.identity.building_address
        ),
        annex_meta,
        insurance_section: contract_section(
            &l.insurance_section,
            &data.insurance_contracts,
            &l.no_contracts,
        ),
        maintenance_section: contract_section(
            &l.maintenance_section,
            &data.maintenance_contracts,
            &l.no_contracts,
        ),
        chronology: ChronologyPayload {
            heading: l.chronology_section.clone(),
            empty_text: l.no_entries.clone(),
            col_date: l.col_date.clone(),
            col_intervention: l.col_intervention.clone(),
            col_vendor: l.col_vendor.clone(),
            col_category: l.col_category.clone(),
            col_source: l.col_source.clone(),
            col_cost: l.col_cost.clone(),
            entries: data.entries.iter().map(entry_payload).collect(),
        },
    }
}

/// One contract section payload: heading and one entry per contract (or an
/// empty state — the absence of insurance is regulatory information in itself).
fn contract_section(
    heading: &str,
    contracts: &[LogbookContractLine],
    empty_text: &str,
) -> ContractSectionPayload {
    ContractSectionPayload {
        heading: heading.to_string(),
        empty_text: empty_text.to_string(),
        contracts: contracts.iter().map(contract_payload).collect(),
    }
}

/// One contract as a payload entry: subject + vendor/type on the left, period
/// + amount on the right.
fn contract_payload(contract: &LogbookContractLine) -> ContractPayload {
    let vendor_type = match &contract.contract_type_label {
        Some(label) => format!("{} · {}", contract.vendor_name, label),
        None => contract.vendor_name.clone(),
    };
    let period = match contract.end_date {
        Some(end) => format!(
            "{} – {}",
            tpl::format_date(contract.start_date),
            tpl::format_date(end)
        ),
        None => format!("{} – …", tpl::format_date(contract.start_date)),
    };
    ContractPayload {
        subject: contract.subject.clone(),
        vendor_type,
        period,
        amount: contract.amount.map(tpl::format_euro),
    }
}

/// One chronology row. Missing vendor / cost render as « - ».
fn entry_payload(entry: &LogbookEntryLine) -> EntryPayload {
    EntryPayload {
        date: tpl::format_date(entry.intervention_date),
        title: entry.title.clone(),
        description: entry.description.clone(),
        vendor: entry.vendor_name.clone().unwrap_or_else(|| "-".to_string()),
        category: entry.category_label.clone(),
        source: entry.source_label.clone(),
        cost: tpl::optional_euro(entry.cost),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::pdf_template::DocumentIdentity;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn identity() -> DocumentIdentity {
        DocumentIdentity {
            building_name: "Résidence des Tilleuls".to_string(),
            building_address: "12 rue des Fleurs, 75011 Paris".to_string(),
            building_street_address: "12 rue des Fleurs".to_string(),
            building_postal_city: "75011 Paris".to_string(),
            syndic_name: Some("Cabinet Dupont".to_string()),
            syndic_siret: Some("123 456 789 00012".to_string()),
        }
    }

    fn labels() -> LogbookPdfLabels {
        LogbookPdfLabels {
            doc_title: "Carnet d'entretien de l'immeuble".to_string(),
            issuer_heading: "Le syndic des copropriétaires".to_string(),
            issuer_not_set: "Syndic non renseigné".to_string(),
            recipient_label: "Destinataire".to_string(),
            objet: "Carnet d'entretien de l'immeuble - édition 2026".to_string(),
            reference: Some(
                "Réf. LR-CARNET-2026 · Édition du 16/06/2026 - destinataire : Marie Dupont"
                    .to_string(),
            ),
            salutation: "Madame, Monsieur,".to_string(),
            letter_paragraphs: vec![
                "Conformément au décret n°2001-477 (loi SRU), le syndic tient à jour le carnet d'entretien de l'immeuble.".to_string(),
                "Cette édition est établie à sa date d'émission et annule et remplace toute édition antérieure.".to_string(),
            ],
            closing_formula: "Je vous prie d'agréer, Madame, Monsieur, l'expression de mes salutations distinguées.".to_string(),
            signer_closing: "Pour le syndic".to_string(),
            signer_role: Some("Syndic en exercice - Résidence des Tilleuls".to_string()),
            place: "Paris".to_string(),
            annex_kicker: "Annexe".to_string(),
            insurance_section: "Contrats d'assurance en cours".to_string(),
            maintenance_section: "Contrats d'entretien et de maintenance en cours".to_string(),
            chronology_section: "Chronologie des interventions".to_string(),
            no_contracts: "Aucun contrat en cours.".to_string(),
            no_entries: "Aucune intervention enregistrée.".to_string(),
            col_date: "Date".to_string(),
            col_intervention: "Intervention".to_string(),
            col_vendor: "Prestataire".to_string(),
            col_category: "Catégorie".to_string(),
            col_source: "Source".to_string(),
            col_cost: "Coût".to_string(),
        }
    }

    fn recipient_lines() -> Vec<String> {
        vec![
            "Marie Dupont".to_string(),
            "Résidence des Tilleuls".to_string(),
            "12 rue des Fleurs".to_string(),
            "75011 Paris".to_string(),
        ]
    }

    fn entry(title: &str, day: u32) -> LogbookEntryLine {
        LogbookEntryLine {
            intervention_date: date(2026, 1, day),
            title: title.to_string(),
            description: Some("Description de test".to_string()),
            vendor_name: Some("Plomberie Duval".to_string()),
            category_label: "Travaux".to_string(),
            source_label: "Saisie manuelle".to_string(),
            cost: Some(Decimal::new(48000, 2)),
        }
    }

    fn contract() -> LogbookContractLine {
        LogbookContractLine {
            subject: "Multirisque immeuble".to_string(),
            vendor_name: "AXA".to_string(),
            contract_type_label: None,
            start_date: date(2026, 1, 1),
            end_date: Some(date(2026, 12, 31)),
            amount: Some(Decimal::new(245000, 2)),
        }
    }

    #[test]
    fn render_logbook_with_data_produces_valid_pdf() {
        let data = LogbookPdfData {
            identity: identity(),
            edited_at: date(2026, 6, 16),
            syndic_contact: Some("contact@cabinet-dupont.fr".to_string()),
            recipient_lines: recipient_lines(),
            insurance_contracts: vec![contract()],
            maintenance_contracts: vec![],
            entries: vec![
                entry("Révision chaudière", 5),
                entry("Contrôle extincteurs", 12),
            ],
            labels: labels(),
        };
        let bytes = render_logbook(&data).expect("render");
        assert!(bytes.starts_with(b"%PDF"));
        assert!(!bytes.is_empty());
    }

    #[test]
    fn render_empty_logbook_produces_valid_pdf() {
        // Empty logbook + no contracts: the document still renders
        // (spec §6).
        let data = LogbookPdfData {
            identity: identity(),
            edited_at: date(2026, 6, 16),
            syndic_contact: None,
            recipient_lines: recipient_lines(),
            insurance_contracts: vec![],
            maintenance_contracts: vec![],
            entries: vec![],
            labels: labels(),
        };
        let bytes = render_logbook(&data).expect("render");
        assert!(bytes.starts_with(b"%PDF"));
        assert!(!bytes.is_empty());
    }

    #[test]
    fn render_logbook_without_syndic_produces_valid_pdf() {
        let data = LogbookPdfData {
            identity: DocumentIdentity {
                building_name: "Résidence des Tilleuls".to_string(),
                building_address: "12 rue des Fleurs, 75011 Paris".to_string(),
                building_street_address: "12 rue des Fleurs".to_string(),
                building_postal_city: "75011 Paris".to_string(),
                syndic_name: None,
                syndic_siret: None,
            },
            edited_at: date(2026, 6, 16),
            syndic_contact: None,
            recipient_lines: recipient_lines(),
            insurance_contracts: vec![],
            maintenance_contracts: vec![],
            entries: vec![entry("Révision chaudière", 5)],
            labels: labels(),
        };
        let bytes = render_logbook(&data).expect("render");
        assert!(bytes.starts_with(b"%PDF"));
    }
}

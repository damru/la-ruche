//! Regulatory PDF rendering (ADR-011, LEGAL.md §7.1 état daté / §7.2 pré-état daté).
//!
//! Pure functions only — no I/O, no SQL. Callers load and assemble the data
//! (domain + db), hand it here together with a document header, and receive
//! the finished PDF as in-memory bytes. Fonts are embedded at compile time so
//! the rendering is self-contained (no system font dependency, matching the
//! `debian:trixie-slim` runtime image).
//!
//! The PDF is a *rendering* of the frozen JSONB snapshot, not the probative
//! record itself (the snapshot is). `genpdf`/`printpdf` embed a random document
//! id and the current creation date, so the byte output is not deterministic
//! across runs — this is acceptable for a presentation document and does not
//! affect the état daté's probative value.
//!
//! Crate choice: ADR-011 names `typst` (default) / `genpdf` (fallback "si
//! l'intégration de typst s'avère trop lourde"). `genpdf` is used here: it is a
//! direct Rust API (no markup language) and far lighter than embedding the full
//! `typst` typesetting engine (which also requires a custom `World`
//! font-resolution implementation).

use chrono::NaiveDate;
use genpdf::{
    Alignment, Document, Element, Margins, SimplePageDecorator,
    elements::{self, LinearLayout, Paragraph, TableLayout},
    fonts::{FontData, FontFamily},
    style,
};
use rust_decimal::Decimal;

use crate::domain::etat_date::{EtatDate, PreEtatDate};

const FONT_REGULAR: &[u8] = include_bytes!("../../assets/fonts/LiberationSans-Regular.ttf");
const FONT_BOLD: &[u8] = include_bytes!("../../assets/fonts/LiberationSans-Bold.ttf");

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

/// Loads and parses the embedded font family. Propagates the only realistic
/// failure (corrupted compile-time font asset) as a string to the caller.
fn font_family() -> Result<FontFamily<FontData>, String> {
    let regular = FontData::new(FONT_REGULAR.to_vec(), None).map_err(|e| e.to_string())?;
    let bold = FontData::new(FONT_BOLD.to_vec(), None).map_err(|e| e.to_string())?;
    Ok(FontFamily {
        regular: regular.clone(),
        bold: bold.clone(),
        italic: regular.clone(),
        bold_italic: bold,
    })
}

/// A4 document with the default font and margins already configured.
fn document(family: FontFamily<FontData>) -> Document {
    let mut doc = Document::new(family);
    doc.set_minimal_conformance();
    doc.set_font_size(10);
    doc.set_line_spacing(1.3);

    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(Margins::all(18));
    doc.set_page_decorator(decorator);
    doc
}

fn bold(size: u8) -> style::Style {
    style::Style::new().bold().with_font_size(size)
}

fn title(text: &str) -> elements::StyledElement<Paragraph> {
    Paragraph::new(text)
        .aligned(Alignment::Center)
        .styled(bold(16))
}

fn section(text: &str) -> elements::StyledElement<Paragraph> {
    Paragraph::new(text).styled(bold(11))
}

fn label(text: &str) -> Paragraph {
    Paragraph::new(text)
}

fn amount_right(text: &str) -> Paragraph {
    Paragraph::new(text).aligned(Alignment::Right)
}

/// Adds a two-column `label | amount` row. Infallible: the row always carries
/// exactly two cells for a two-column layout, so a column mismatch would be a
/// programmer error, not a runtime condition.
fn push_amount_row(table: &mut TableLayout, label_text: &str, amount: &str) {
    table
        .row()
        .element(label(label_text))
        .element(amount_right(amount))
        .push()
        .expect("two-column amount row must carry exactly two cells");
}

/// Builds a two-column `label | amount` table from line items.
fn amount_table(lines: &[crate::domain::etat_date::LigneMontant]) -> TableLayout {
    let mut table = TableLayout::new(vec![5, 1]);
    for line in lines {
        push_amount_row(&mut table, &line.label, &format_euro(line.amount));
    }
    table
}

/// Pushes the residence + syndic header block onto the layout.
fn push_header(layout: &mut LinearLayout, header: &LegalDocumentHeader) {
    layout.push(Paragraph::new(format!(
        "Syndicat des copropriétaires : {}",
        header.building_name
    )));
    layout.push(Paragraph::new(format!(
        "Adresse : {}",
        header.building_address
    )));
    let syndic_line = match (&header.syndic_name, &header.syndic_siret) {
        (Some(name), Some(siret)) => format!("Syndic : {name} — SIRET {siret}"),
        (Some(name), None) => format!("Syndic : {name}"),
        (None, Some(siret)) => format!("Syndic : SIRET {siret}"),
        (None, None) => "Syndic : non renseigné".to_string(),
    };
    layout.push(Paragraph::new(syndic_line));
    layout.push(Paragraph::new(format!("Lot : {}", header.unit_number)));
    layout.push(elements::Break::new(0.5));
}

/// Pushes the two signature blocks ("Le syndic" / "Le notaire ou l'acquéreur")
/// that make the document signable as an official record.
fn push_signatures(layout: &mut LinearLayout) {
    layout.push(elements::Break::new(1.0));

    let mut table = TableLayout::new(vec![1, 1]);
    let place = "Fait à ____________________, le ____________________".to_string();
    table
        .row()
        .element(
            LinearLayout::vertical()
                .element(label("Le syndic"))
                .element(elements::Break::new(3))
                .element(label(&place))
                .padded(2),
        )
        .element(
            LinearLayout::vertical()
                .element(label("Le notaire / l'acquéreur"))
                .element(elements::Break::new(3))
                .element(label(&place))
                .padded(2),
        )
        .push()
        .expect("two-column signature row must carry exactly two cells");
    layout.push(table);
}

/// Renders the état daté (EP-002-S06) as a PDF from its frozen snapshot.
pub fn render_etat_date(
    header: &LegalDocumentHeader,
    snapshot: &EtatDate,
) -> Result<Vec<u8>, String> {
    let mut doc = document(font_family()?);
    doc.set_title("État daté");

    let mut layout = LinearLayout::vertical();
    layout.push(title("État daté"));
    layout.push(elements::Break::new(0.5));
    push_header(&mut layout, header);

    if let Some(requester) = &header.requester_name {
        layout.push(Paragraph::new(format!("Demande de : {requester}")));
    }
    if let Some(date) = header.requested_at {
        layout.push(Paragraph::new(format!(
            "Date de la demande : {}",
            format_date(date)
        )));
    }
    if let Some(date) = header.due_date {
        layout.push(Paragraph::new(format!(
            "Échéance légale (J+15) : {}",
            format_date(date)
        )));
    }
    if let Some(date) = header.generated_at {
        layout.push(Paragraph::new(format!(
            "Date de génération : {}",
            format_date(date)
        )));
    }
    layout.push(elements::Break::new(0.5));

    layout.push(section("1. Sommes dues par le cédant"));
    layout.push(amount_table(&snapshot.partie1_sommes_dues_par_cedant));
    layout.push(elements::Break::new(0.5));

    layout.push(section("2. Sommes dont le syndicat est débiteur"));
    layout.push(amount_table(&snapshot.partie2_sommes_dues_par_syndicat));
    layout.push(elements::Break::new(0.5));

    layout.push(section("3. Sommes incombant à l'acquéreur"));
    layout.push(amount_table(&snapshot.partie3_sommes_incombant_acquereur));
    layout.push(elements::Break::new(0.5));

    layout.push(section("Annexe — quote-part sur les 2 derniers exercices"));
    layout.push(exercice_table(&snapshot.annexe_quote_part));
    layout.push(elements::Break::new(0.5));

    layout.push(section("Procédures judiciaires en cours"));
    if snapshot.procedures.is_empty() {
        layout.push(Paragraph::new("Néant"));
    } else {
        for proc in &snapshot.procedures {
            let jurisdiction = proc
                .jurisdiction
                .as_deref()
                .map(|j| format!(" ({j})"))
                .unwrap_or_default();
            layout.push(Paragraph::new(format!(
                "{} — {}{}",
                proc.subject,
                format_date(proc.opened_at),
                jurisdiction
            )));
        }
    }
    layout.push(elements::Break::new(0.5));

    layout.push(section("Travaux votés non réalisés"));
    if snapshot.travaux_votes_non_realises.is_empty() {
        layout.push(Paragraph::new("Néant"));
    } else {
        for t in &snapshot.travaux_votes_non_realises {
            layout.push(Paragraph::new(t));
        }
    }
    layout.push(elements::Break::new(0.5));

    layout.push(section("Emprunts collectifs"));
    layout.push(Paragraph::new(&snapshot.emprunts_collectifs));

    push_signatures(&mut layout);
    doc.push(layout);

    let mut out = Vec::new();
    doc.render(&mut out).map_err(|e| e.to_string())?;
    Ok(out)
}

/// Renders the pré-état daté (EP-002-S07, art. L721-2 CCH) as a PDF.
pub fn render_pre_etat_date(
    header: &LegalDocumentHeader,
    data: &PreEtatDate,
) -> Result<Vec<u8>, String> {
    let mut doc = document(font_family()?);
    doc.set_title("Pré-état daté");

    let mut layout = LinearLayout::vertical();
    layout.push(title("Pré-état daté"));
    layout.push(elements::Break::new(0.5));
    push_header(&mut layout, header);

    if let Some(date) = header.edited_at {
        layout.push(Paragraph::new(format!("Édité le : {}", format_date(date))));
    }
    layout.push(elements::Break::new(0.5));

    layout.push(section(
        "Charges courantes (budget prévisionnel et hors budget)",
    ));
    let mut charges = TableLayout::new(vec![5, 1]);
    push_amount_row(
        &mut charges,
        "Charges budget prévisionnel — exercice N-1",
        &optional_euro(data.charges_courantes_n1),
    );
    push_amount_row(
        &mut charges,
        "Charges budget prévisionnel — exercice N-2",
        &optional_euro(data.charges_courantes_n2),
    );
    push_amount_row(
        &mut charges,
        "Charges hors budget — exercice N-1",
        &optional_euro(data.charges_hors_budget_n1),
    );
    push_amount_row(
        &mut charges,
        "Charges hors budget — exercice N-2",
        &optional_euro(data.charges_hors_budget_n2),
    );
    layout.push(charges);
    layout.push(elements::Break::new(0.5));

    layout.push(section("Sommes dues au syndicat"));
    let mut dues = TableLayout::new(vec![5, 1]);
    push_amount_row(
        &mut dues,
        "Sommes dues par le cédant",
        &format_euro(data.sommes_dues_par_cedant),
    );
    push_amount_row(
        &mut dues,
        "État global des impayés du syndicat",
        &format_euro(data.impayes_globaux_syndicat),
    );
    push_amount_row(
        &mut dues,
        "Dette fournisseurs",
        &optional_euro(data.dette_fournisseurs),
    );
    layout.push(dues);
    layout.push(elements::Break::new(0.5));

    layout.push(section("Fonds travaux"));
    let mut fund = TableLayout::new(vec![5, 1]);
    push_amount_row(
        &mut fund,
        "Quote-part du lot",
        &format_euro(data.fonds_travaux_quote_part),
    );
    push_amount_row(
        &mut fund,
        "Dernière cotisation",
        &format_euro(data.fonds_travaux_derniere_cotisation),
    );
    layout.push(fund);
    layout.push(elements::Break::new(0.5));

    layout.push(section("Procédures judiciaires en cours"));
    layout.push(Paragraph::new(if data.procedures_en_cours == 0 {
        "Néant".to_string()
    } else {
        data.procedures_en_cours.to_string()
    }));
    layout.push(elements::Break::new(0.5));

    layout.push(section("Emprunts collectifs"));
    layout.push(Paragraph::new(&data.emprunts_collectifs));

    push_signatures(&mut layout);
    doc.push(layout);

    let mut out = Vec::new();
    doc.render(&mut out).map_err(|e| e.to_string())?;
    Ok(out)
}

/// Renders the quote-part annexe (état daté) as a four-column table.
fn exercice_table(exercices: &[crate::domain::etat_date::ExerciceQuotePart]) -> TableLayout {
    let mut table = TableLayout::new(vec![1, 2, 2, 2]);
    table
        .row()
        .element(label("Exercice").styled(style::Effect::Bold))
        .element(label("Charges").styled(style::Effect::Bold))
        .element(label("Fonds travaux").styled(style::Effect::Bold))
        .element(amount_right("Total").styled(style::Effect::Bold))
        .push()
        .expect("four-column header row must carry exactly four cells");

    for ex in exercices {
        table
            .row()
            .element(label(&ex.fiscal_year.to_string()))
            .element(amount_right(&format_euro(ex.charge_amount)))
            .element(amount_right(&format_euro(ex.works_fund_amount)))
            .element(amount_right(&format_euro(ex.total_amount)))
            .push()
            .expect("four-column data row must carry exactly four cells");
    }
    table
}

/// `Decimal` → French-formatted euro string (e.g. `1 500,00 €`). Amounts are
/// always rendered from `Decimal`, never `f64` (ADR-011).
fn format_euro(value: Decimal) -> String {
    let rounded = value.round_dp(2);
    let raw = rounded.to_string();
    let (sign, rest) = match raw.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", raw.as_str()),
    };
    let (int_part, frac_part) = match rest.split_once('.') {
        Some((i, f)) => (i, format!("{f:0<2}")),
        None => (rest, "00".to_string()),
    };

    let mut grouped = String::new();
    let digits: Vec<char> = int_part.chars().collect();
    for (idx, c) in digits.iter().enumerate() {
        if idx > 0 && (digits.len() - idx).is_multiple_of(3) {
            grouped.push(' ');
        }
        grouped.push(*c);
    }

    format!("{sign}{grouped},{frac_part} €")
}

/// `Option<Decimal>` → either the formatted amount or « Non disponible ».
fn optional_euro(value: Option<Decimal>) -> String {
    value
        .map(format_euro)
        .unwrap_or_else(|| "Non disponible".to_string())
}

/// `NaiveDate` → `jj/mm/aaaa`.
fn format_date(date: NaiveDate) -> String {
    date.format("%d/%m/%Y").to_string()
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

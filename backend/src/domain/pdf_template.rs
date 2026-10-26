//! Shared PDF document template (EP-009-S09 spec change: all document PDFs are
//! generated server-side and share this template).
//!
//! Every generated document — logbook export, état daté, pré-état daté, future
//! AG minutes, reminder letters — is built from this module so that the layout
//! is identical everywhere:
//!
//! - **Header (every page)** — app name « La Ruche » in the brand color on the
//!   left, building name + address on the right, thin brand-colored rule below.
//! - **First-page content header** — document title (centered, brand color),
//!   subtitle, meta lines (syndic, lot…), edition date.
//! - **Salutation** — optional, for correspondence-style documents.
//! - **Signature block** — « Le syndic » place/date lines (artificial
//!   signature area, no real signature image).
//! - **Footer** — « Édité le {date} via La Ruche » in grey under a hairline rule
//!   (genpdf has no native "total pages" footer, so no page number).
//! - **Typography & palette** — Liberation Sans (embedded at compile time),
//!   brand color `#643fd5` mirrored from `frontend/src/styles/tokens.css`
//!   (`--color-primary` light theme), grey `#6b7280` for secondary text.
//!
//! Pure functions only — no I/O, no SQL. Callers assemble the data and hand it
//! here; they receive the finished PDF as in-memory bytes (see
//! `domain/legal_documents.rs` for the established pattern, ADR-011).

use chrono::{Datelike, NaiveDate};
use genpdf::{
    Alignment, Document, Element, Margins, SimplePageDecorator,
    elements::{self, LinearLayout, Paragraph, TableLayout},
    fonts::{FontData, FontFamily},
    style::{self, Color},
};
use rust_decimal::Decimal;

// ── Palette (mirrors frontend/src/styles/tokens.css light theme) ────────────

/// Brand primary `#643fd5` (`--color-primary` light theme).
pub const BRAND: Color = Color::Rgb(100, 63, 213);
/// Secondary text grey.
pub const GREY: Color = Color::Rgb(107, 114, 128);
/// Near-black body text.
pub const INK: Color = Color::Rgb(17, 24, 39);
/// Hairline rule grey.
pub const RULE: Color = Color::Rgb(209, 213, 219);

const FONT_REGULAR: &[u8] = include_bytes!("../../assets/fonts/LiberationSans-Regular.ttf");
const FONT_BOLD: &[u8] = include_bytes!("../../assets/fonts/LiberationSans-Bold.ttf");

/// App name rendered in the page header of every generated document.
pub const APP_NAME: &str = "La Ruche";

// ── Public data structures ──────────────────────────────────────────────────

/// Identity block shared by all generated documents: the residence and its
/// syndic. Assembled by the API layer from `building` + `governance_mandate`.
#[derive(Debug, Clone)]
pub struct DocumentIdentity {
    pub building_name: String,
    pub building_address: String,
    pub syndic_name: Option<String>,
    pub syndic_siret: Option<String>,
}

/// Content header rendered once, on the first page, below the running header.
#[derive(Debug, Clone)]
pub struct DocumentTitleBlock {
    /// Document title, e.g. « Carnet d'entretien de l'immeuble ».
    pub title: String,
    /// Optional subtitle, e.g. the building name.
    pub subtitle: Option<String>,
    /// Meta lines rendered under the title (label: value), e.g. syndic, lot.
    pub meta: Vec<(String, String)>,
}

/// A line of correspondence-style salutation, e.g. « Madame, Monsieur, ».
pub type Salutation = String;

// ── Public helpers shared by document renderers ─────────────────────────────

/// Loads and parses the embedded font family. Propagates the only realistic
/// failure (corrupted compile-time font asset) as a string to the caller.
pub fn font_family() -> Result<FontFamily<FontData>, String> {
    let regular = FontData::new(FONT_REGULAR.to_vec(), None).map_err(|e| e.to_string())?;
    let bold = FontData::new(FONT_BOLD.to_vec(), None).map_err(|e| e.to_string())?;
    Ok(FontFamily {
        regular: regular.clone(),
        bold: bold.clone(),
        italic: regular.clone(),
        bold_italic: bold,
    })
}

// ── Document construction ───────────────────────────────────────────────────

/// A4 document pre-configured with the shared template: embedded fonts,
/// margins and a running header (app name + building identity) on every page.
/// The caller only fills the body (title block, content, signature, footer —
/// see `push_footer`).
pub fn new_document(
    family: FontFamily<FontData>,
    identity: &DocumentIdentity,
    pdf_title: &str,
) -> Document {
    let mut doc = Document::new(family);
    doc.set_minimal_conformance();
    doc.set_font_size(10);
    doc.set_line_spacing(1.3);
    doc.set_title(pdf_title);

    let header_identity = identity.clone();
    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(Margins::vh(24, 18));
    decorator.set_header(move |_page| {
        let identity = header_identity.clone();
        running_header(&identity)
    });
    doc.set_page_decorator(decorator);

    doc
}

/// Running header rendered at the top of every page: app name in brand color
/// on the left, building identity on the right, hairline rule below.
fn running_header(identity: &DocumentIdentity) -> LinearLayout {
    let mut layout = LinearLayout::vertical();

    let mut row = TableLayout::new(vec![1, 2]);
    row.row()
        .element(
            Paragraph::new(APP_NAME).styled(
                style::Style::new()
                    .bold()
                    .with_font_size(11)
                    .with_color(BRAND),
            ),
        )
        .element(
            LinearLayout::vertical()
                .element(
                    Paragraph::new(&identity.building_name)
                        .aligned(Alignment::Right)
                        .styled(style::Style::new().bold().with_font_size(9)),
                )
                .element(
                    Paragraph::new(&identity.building_address)
                        .aligned(Alignment::Right)
                        .styled(style::Style::new().with_font_size(8).with_color(GREY)),
                ),
        )
        .push()
        .expect("two-column header row must carry exactly two cells");
    layout.push(row);
    layout.push(Paragraph::new(" ").styled(style::Style::new().with_font_size(2)));
    layout.push(hairline());
    layout.push(elements::Break::new(0.5));
    layout
}

/// Pushes the first-page content header: centered brand-colored title,
/// optional subtitle, meta lines, edition date.
pub fn push_title_block(layout: &mut LinearLayout, block: &DocumentTitleBlock) {
    layout.push(
        Paragraph::new(&block.title)
            .aligned(Alignment::Center)
            .styled(
                style::Style::new()
                    .bold()
                    .with_font_size(16)
                    .with_color(BRAND),
            ),
    );
    if let Some(subtitle) = &block.subtitle {
        layout.push(
            Paragraph::new(subtitle)
                .aligned(Alignment::Center)
                .styled(style::Style::new().with_font_size(11).with_color(GREY)),
        );
    }
    layout.push(elements::Break::new(0.3));
    for (label, value) in &block.meta {
        layout.push(meta_line(label, value));
    }
    layout.push(elements::Break::new(0.5));
}

/// A `label : value` meta line (label in grey, value in ink).
fn meta_line(label: &str, value: &str) -> Paragraph {
    let mut p = Paragraph::new("");
    p.push_styled(format!("{label} : "), style::Style::new().with_color(GREY));
    p.push_styled(value.to_string(), style::Style::new().with_color(INK));
    p
}

/// Pushes an optional salutation block (correspondence-style documents).
pub fn push_salutation(layout: &mut LinearLayout, salutation: &str) {
    layout.push(Paragraph::new(salutation));
    layout.push(elements::Break::new(0.5));
}

/// Section heading in brand color.
pub fn section(text: &str) -> elements::StyledElement<Paragraph> {
    Paragraph::new(text).styled(
        style::Style::new()
            .bold()
            .with_font_size(11)
            .with_color(BRAND),
    )
}

/// Body paragraph in ink color.
pub fn body(text: &str) -> elements::StyledElement<Paragraph> {
    Paragraph::new(text).styled(style::Style::new().with_color(INK))
}

/// Secondary paragraph in grey (empty states, hints).
pub fn muted(text: &str) -> elements::StyledElement<Paragraph> {
    Paragraph::new(text).styled(style::Style::new().with_color(GREY))
}

/// Pushes the signature block: « Le syndic » with place/date lines — an
/// artificial signature area for official documents, no signature image.
pub fn push_signature(layout: &mut LinearLayout, syndic_label: &str) {
    layout.push(elements::Break::new(1.0));
    let place = "Fait à ____________________, le ____________________".to_string();
    layout.push(
        LinearLayout::vertical()
            .element(Paragraph::new(syndic_label).styled(style::Style::new().bold()))
            .element(elements::Break::new(3))
            .element(Paragraph::new(&place).styled(style::Style::new().with_color(GREY)))
            .padded(2),
    );
}

/// Pushes the document footer: hairline rule, then « Édité le {date} via La
/// Ruche » in grey. genpdf has no native « total pages » footer, so no page
/// number is rendered — the edition date and app name identify the printout.
pub fn push_footer(layout: &mut LinearLayout, edited_at: NaiveDate) {
    layout.push(hairline());
    layout.push(
        Paragraph::new(format!(
            "Édité le {} via {APP_NAME}",
            format_date(edited_at)
        ))
        .styled(style::Style::new().with_font_size(8).with_color(GREY)),
    );
}

/// Renders the document to bytes.
pub fn finish_document(doc: Document) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    doc.render(&mut out).map_err(|e| e.to_string())?;
    Ok(out)
}

// ── Formatting helpers (shared, FR conventions) ─────────────────────────────

/// `Decimal` → French-formatted euro string (e.g. `1 500,00 €`). Amounts are
/// always rendered from `Decimal`, never `f64` (ADR-011).
pub fn format_euro(value: Decimal) -> String {
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

/// `Option<Decimal>` → either the formatted amount or « — ».
pub fn optional_euro(value: Option<Decimal>) -> String {
    value.map(format_euro).unwrap_or_else(|| "—".to_string())
}

/// `NaiveDate` → `jj/mm/aaaa`.
pub fn format_date(date: NaiveDate) -> String {
    date.format("%d/%m/%Y").to_string()
}

/// `NaiveDate` → `d month yyyy` in French (e.g. « 16 juin 2026 »), used for the
/// edition date in footers and signature places.
pub fn format_date_long(date: NaiveDate) -> String {
    let months = [
        "janvier",
        "février",
        "mars",
        "avril",
        "mai",
        "juin",
        "juillet",
        "août",
        "septembre",
        "octobre",
        "novembre",
        "décembre",
    ];
    let month = months
        .get((date.month() - 1) as usize)
        .copied()
        .unwrap_or("");
    format!("{} {} {}", date.day(), month, date.year())
}

// ── Internal layout primitives ──────────────────────────────────────────────

/// Thin horizontal rule in `RULE` grey, used under the header and above the
/// footer. Implemented as a tiny paragraph with a bottom border via a table
/// frame would be overkill — a coloured underscore line is the pragmatic
/// genpdf approach.
fn hairline() -> elements::StyledElement<Paragraph> {
    Paragraph::new("_______________________________________________________________")
        .styled(style::Style::new().with_font_size(6).with_color(RULE))
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn identity() -> DocumentIdentity {
        DocumentIdentity {
            building_name: "Résidence des Tilleuls".to_string(),
            building_address: "12 rue des Fleurs, 75011 Paris".to_string(),
            syndic_name: Some("Cabinet Dupont".to_string()),
            syndic_siret: Some("123 456 789 00012".to_string()),
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
    fn optional_euro_renders_dash_for_none() {
        assert_eq!(optional_euro(None), "—");
        assert_eq!(optional_euro(Some(Decimal::new(100, 2))), "1,00 €");
    }

    #[test]
    fn format_date_uses_day_month_year() {
        assert_eq!(format_date(date(2026, 6, 16)), "16/06/2026");
    }

    #[test]
    fn format_date_long_uses_french_month_names() {
        assert_eq!(format_date_long(date(2026, 6, 16)), "16 juin 2026");
        assert_eq!(format_date_long(date(2026, 1, 5)), "5 janvier 2026");
    }

    #[test]
    fn document_with_template_produces_valid_pdf() {
        let mut doc = new_document(font_family().expect("fonts"), &identity(), "Test document");
        let mut layout = LinearLayout::vertical();
        push_title_block(
            &mut layout,
            &DocumentTitleBlock {
                title: "Document de test".to_string(),
                subtitle: Some("Résidence des Tilleuls".to_string()),
                meta: vec![
                    ("Syndic".to_string(), "Cabinet Dupont".to_string()),
                    ("Lot".to_string(), "A-12".to_string()),
                ],
            },
        );
        push_salutation(&mut layout, "Madame, Monsieur,");
        layout.push(section("1. Première section"));
        layout.push(body("Corps du document de test."));
        layout.push(muted("Aucune donnée disponible."));
        push_signature(&mut layout, "Le syndic");
        push_footer(&mut layout, date(2026, 6, 16));
        doc.push(layout);

        let bytes = finish_document(doc).expect("render");
        assert!(bytes.starts_with(b"%PDF"));
        assert!(!bytes.is_empty());
    }
}

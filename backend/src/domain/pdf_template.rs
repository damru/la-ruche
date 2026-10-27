//! Shared PDF document template (EP-009-S09 spec change: all document PDFs are
//! generated server-side and share this template).
//!
//! Every generated document — logbook export, état daté, pré-état daté, future
//! AG minutes, reminder letters — is built from this module so that the layout
//! is identical everywhere. Layout v6 (retour utilisateur PR 155, inspiré
//! d'une attestation d'assurance professionnelle) :
//!
//! - **Double en-tête (première page uniquement)** — à gauche le bloc marque
//!   complet « app web » : logo + « La Ruche » + tagline, puis l'émetteur du
//!   document (lignes libres : syndic, contact, SIRET…) ; à droite le bloc
//!   destinataire (label + lignes). Un filet violet sépare l'en-tête du corps.
//! - **Corps de lettre classique** — objet en gras + ligne de référence
//!   grise, salutation optionnelle (« Madame, Monsieur, »), paragraphes de
//!   description, formule de politesse. Le contenu détaillé (tableaux) est
//!   annoncé dans la lettre et rendu en annexe.
//! - **Signature (fin de lettre)** — période de validité légale optionnelle,
//!   puis bloc aligné à droite : « Pour {qualité} », espace, nom du signataire
//!   en gras, qualité en gris, « Fait à {place}, le {date} » rempli (signature
//!   artificielle, pas d'image).
//! - **En-tête d'annexe** — kicker espacé (A N N E X E), titre en couleur
//!   marque, sous-titre, méta-lignes, filet violet ; les tableaux suivent.
//! - **No running header** — les pages suivantes commencent directement par
//!   le contenu (spec : « par en-tête je voulais dire introduction »).
//! - **Footer (every page)** — small grey type: logo + tagline on the left,
//!   « Édité le {date} via La Ruche » on the right, under a hairline rule.
//!   genpdf has no native « total pages » footer, so no page number.
//! - **Typography & palette** — Tinos (serif, metric-compatible Times New
//!   Roman) embedded at compile time, brand color `#643fd5` mirrored from
//!   `frontend/src/styles/tokens.css` (`--color-primary` light theme),
//!   grey `#6b7280` for secondary text.
//!
//! Pure functions only — no I/O, no SQL. Callers assemble the data and hand it
//! here; they receive the finished PDF as in-memory bytes (see
//! `domain/legal_documents.rs` for the established pattern, ADR-011).

use chrono::{Datelike, NaiveDate};
use genpdf::{
    Alignment, Context, Document, Element, Margins, Mm, PageDecorator, Position, Rotation, Scale,
    SimplePageDecorator,
    elements::{self, LinearLayout, Paragraph, TableLayout},
    error,
    fonts::{FontData, FontFamily},
    render::Area,
    style::{self, Color},
};
use rust_decimal::Decimal;

// ── Palette (mirrors frontend/src/styles/tokens.css light theme) ───────────

/// Brand primary `#643fd5` (`--color-primary` light theme).
pub const BRAND: Color = Color::Rgb(100, 63, 213);
/// Secondary text grey.
pub const GREY: Color = Color::Rgb(107, 114, 128);
/// Near-black body text.
pub const INK: Color = Color::Rgb(17, 24, 39);
/// Hairline rule grey.
pub const RULE: Color = Color::Rgb(209, 213, 219);

// Tinos (serif, métrique Times New Roman) sous-ensemble (pyftsubset) :
// ASCII + Latin-1 + ponctuation typographique (tirets, apostrophes, ellipsis,
// €). ~32 ko par fichier — les documents FR n'utilisent aucun glyphe hors de
// ce périmètre. Police serif choisie pour le rendu « document officiel »
// (EP-009-S09). Liberation Sans reste embarquée pour `legal_documents.rs`.
const FONT_REGULAR: &[u8] = include_bytes!("../../assets/fonts/Tinos-Regular.ttf");
const FONT_BOLD: &[u8] = include_bytes!("../../assets/fonts/Tinos-Bold.ttf");
const FONT_ITALIC: &[u8] = include_bytes!("../../assets/fonts/Tinos-Italic.ttf");
const FONT_BOLD_ITALIC: &[u8] = include_bytes!("../../assets/fonts/Tinos-BoldItalic.ttf");

/// App name rendered next to the logo in the introduction and in the footer.
pub const APP_NAME: &str = "La Ruche";
/// Tagline describing the application, rendered under the app name (brand
/// block) and in the page footer.
pub const APP_TAGLINE: &str =
    "Gestion de copropriété participative, pour les syndics bénévoles et les conseils syndicaux.";

/// Vertical margin kept at the top of every page (no running header, just
/// breathing room). The footer decorator reserves space at the bottom.
const PAGE_TOP_MARGIN_MM: f64 = 18.0;
/// Horizontal page margin.
const PAGE_SIDE_MARGIN_MM: f64 = 18.0;
/// Height reserved at the bottom of every page for the running footer.
const FOOTER_HEIGHT_MM: f64 = 14.0;
/// Size (in mm) at which the logo is rendered in the introduction block.
const INTRO_LOGO_MM: f64 = 16.0;
/// Size (in mm) at which the logo is rendered in the page footer.
const FOOTER_LOGO_MM: f64 = 5.0;

/// `f64` millimeters → genpdf `Mm` (the crate only implements `From<f32>`).
fn mm(value: f64) -> Mm {
    Mm::from(value as f32)
}

// ── Public data structures ─────────────────────────────────────────────────

/// Identity block shared by all generated documents: the copropriété and its
/// syndic. Assembled by the API layer from `building` + `governance_mandate`.
#[derive(Debug, Clone)]
pub struct DocumentIdentity {
    pub building_name: String,
    pub building_address: String,
    pub syndic_name: Option<String>,
    pub syndic_siret: Option<String>,
}

/// Double header block rendered once, on the first page (Qivio-style
/// « lettre » layout): the full application brand block « app web » (logo +
/// « La Ruche » + tagline) with the issuer lines underneath, on the left; the
/// optional recipient block on the right. A brand-colored rule closes the
/// header. This is the « en-tête » of the document in the spec sense — it
/// does NOT repeat on every page.
#[derive(Debug, Clone)]
pub struct DocumentHeader {
    /// Issuer lines under the brand block (left column), e.g. « Le syndic des
    /// copropriétaires », « Damien Rubio — contact », « SIRET : … ».
    pub issuer_lines: Vec<String>,
    /// Recipient block (right column), e.g. the copropriété.
    pub recipient: Option<DocumentRecipient>,
}

/// Recipient block rendered on the right of the double header.
#[derive(Debug, Clone)]
pub struct DocumentRecipient {
    /// Small grey bold label, e.g. « Destinataire ».
    pub label: String,
    /// Recipient lines underneath, right-aligned.
    pub lines: Vec<String>,
}

/// Annex header block opening the annex pages of a letter-style document:
/// spaced kicker (« A N N E X E »), title in brand color, optional subtitle,
/// meta lines and an optional reference, closed by a brand-colored rule.
#[derive(Debug, Clone)]
pub struct DocumentAnnexHeader {
    /// Kicker rendered letter-spaced (e.g. « Annexe » → « A N N E X E »).
    pub kicker: String,
    /// Annex title, e.g. « Carnet d'entretien de l'immeuble ».
    pub title: String,
    /// Optional subtitle, e.g. the building name and address.
    pub subtitle: Option<String>,
    /// Meta lines rendered under the title (label: value), e.g. syndic,
    /// édition.
    pub meta: Vec<(String, String)>,
}

/// Conclusion block rendered at the end of the letter: optional legal
/// validity period, then a right-aligned signature block — « Pour {qualité} »,
/// signature space, signer name in bold, signer role in grey, and the filled
/// « Fait à …, le … » line (artificial signature, no image).
#[derive(Debug, Clone)]
pub struct DocumentConclusion {
    /// « Pour le syndic, »-style closing line of the signature block.
    pub closing: String,
    /// Signer name (bold). Falls back to `APP_NAME` when the document is not
    /// signed by a named person.
    pub signer_name: String,
    /// Signer role in small grey type, e.g. « Syndic en exercice — … ».
    pub signer_role: Option<String>,
    /// Place of signature (filled — « Fait à {place}, le {date} »).
    pub place: String,
    /// Signature date (filled).
    pub date: NaiveDate,
    /// Optional legal validity period, e.g. « Ce document est valable 3 mois
    /// à compter de sa date d'émission. » — rendered only when the document
    /// has a legal validity period.
    pub validity_period: Option<String>,
}

/// A line of correspondence-style salutation, e.g. « Madame, Monsieur, ».
pub type Salutation = String;

// ── Public helpers shared by document renderers ────────────────────────────

/// Loads and parses the embedded font family (Tinos serif — Times New Roman
/// metric-compatible). Propagates the only realistic failure (corrupted
/// compile-time font asset) as a string to the caller.
pub fn font_family() -> Result<FontFamily<FontData>, String> {
    Ok(FontFamily {
        regular: FontData::new(FONT_REGULAR.to_vec(), None).map_err(|e| e.to_string())?,
        bold: FontData::new(FONT_BOLD.to_vec(), None).map_err(|e| e.to_string())?,
        italic: FontData::new(FONT_ITALIC.to_vec(), None).map_err(|e| e.to_string())?,
        bold_italic: FontData::new(FONT_BOLD_ITALIC.to_vec(), None).map_err(|e| e.to_string())?,
    })
}

/// Rasterised app logo (7 hexagones « ruche », dégradé `#643fd5` → `#7d5bf0`),
/// generated from `frontend/src/components/atoms/CLogoMark.svg` and embedded at
/// compile time so the PDF is self-contained (ADR-011). RGB, no alpha channel
/// (printpdf ne supporte pas la transparence — le logo est dessiné sur blanc).
/// Encodé en PNG RGB optimisé (~5 ko) ; en release, printpdf compresse le flux
/// image en FlateDecode (~4,6 ko par occurrence embarquée).
const LOGO_PNG: &[u8] = include_bytes!("../../assets/images/logo-la-ruche.png");

/// Natural size of the embedded logo in mm at 300 dpi (640 px → 54.04 mm).
const LOGO_NATURAL_MM: f64 = 640.0 * 25.4 / 300.0;

/// Builds the app logo as a genpdf image element rendered at `size_mm`
/// millimeters, decoded from the embedded PNG (`include_bytes!`).
pub fn logo_element(size_mm: f64) -> Result<elements::Image, String> {
    let dynamic = image::load_from_memory(LOGO_PNG).map_err(|e| e.to_string())?;
    let img = elements::Image::from_dynamic_image(dynamic).map_err(|e| e.to_string())?;
    let scale = size_mm / LOGO_NATURAL_MM;
    Ok(img.with_scale(Scale::new(scale, scale)))
}

// ── Document construction ──────────────────────────────────────────────────

/// A4 document pre-configured with the shared template: embedded fonts,
/// margins and a **running footer on every page** (logo + tagline + edition
/// date, small grey type). There is intentionally **no running header** — the
/// double header is part of the body and appears on the first page only.
/// The caller fills the body: `push_letter_header`, letter content, then the
/// annexes opened by `push_annex_header`.
pub fn new_document(
    family: FontFamily<FontData>,
    edited_at: NaiveDate,
    pdf_title: &str,
) -> Document {
    let mut doc = Document::new(family);
    doc.set_minimal_conformance();
    doc.set_font_size(10);
    doc.set_line_spacing(1.3);
    doc.set_title(pdf_title);

    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(Margins::vh(PAGE_TOP_MARGIN_MM, PAGE_SIDE_MARGIN_MM));
    doc.set_page_decorator(FooterDecorator::new(edited_at));

    doc
}

/// Page decorator that draws the running footer at the bottom of every page:
/// hairline rule, then logo + tagline on the left and « Édité le {date} via
/// La Ruche » on the right, in small grey type. It also reserves the footer
/// height so the body content never overlaps it.
struct FooterDecorator {
    edited_at: NaiveDate,
}

impl FooterDecorator {
    fn new(edited_at: NaiveDate) -> Self {
        FooterDecorator { edited_at }
    }
}

impl PageDecorator for FooterDecorator {
    fn decorate_page<'a>(
        &mut self,
        context: &Context,
        mut area: Area<'a>,
        _style: style::Style,
    ) -> Result<Area<'a>, error::Error> {
        let page = area.clone();
        let page_width = page.size().width;
        let page_height = page.size().height;
        let footer_top = page_height - mm(FOOTER_HEIGHT_MM);

        // Hairline rule across the content width, 4 mm above the footer text.
        let rule_y = footer_top + mm(4.0);
        page.draw_line(
            vec![Position::new(0, rule_y), Position::new(page_width, rule_y)],
            style::Style::new().with_color(RULE),
        );

        // Logo at the bottom-left of the footer area (embedded PNG).
        if let Ok(dynamic) = image::load_from_memory(LOGO_PNG) {
            let scale = FOOTER_LOGO_MM / LOGO_NATURAL_MM;
            page.add_image(
                &dynamic,
                Position::new(0, footer_top + mm(FOOTER_LOGO_MM)),
                Scale::new(scale, scale),
                Rotation::default(),
                None,
            );
        }

        // Tagline + edition line, small grey type, next to the logo.
        let text_x = mm(FOOTER_LOGO_MM + 3.0);
        let _ = page.print_str(
            &context.font_cache,
            Position::new(text_x, footer_top + mm(3.5)),
            style::Style::new().with_font_size(7).with_color(GREY),
            format!("{APP_TAGLINE} — {APP_NAME}"),
        );
        let _ = page.print_str(
            &context.font_cache,
            Position::new(text_x, footer_top + mm(7.0)),
            style::Style::new().with_font_size(7).with_color(GREY),
            format!("Édité le {} via {APP_NAME}", format_date(self.edited_at)),
        );

        // Reserve the footer height so the body never overlaps it.
        area.set_height(footer_top);
        Ok(area)
    }
}

// ── Double header (first page) ─────────────────────────────────────────────

/// Pushes the double header on the first page: the application brand block
/// « app web » (logo + « La Ruche » + tagline) with the issuer lines
/// underneath, on the left; the recipient block right-aligned, on the right;
/// then a brand-colored rule closing the header (Qivio-style letterhead).
pub fn push_letter_header(
    layout: &mut LinearLayout,
    header: &DocumentHeader,
) -> Result<(), String> {
    // ── Left column: brand block + issuer lines ──
    let logo = logo_element(INTRO_LOGO_MM)?;
    let mut brand_text = LinearLayout::vertical();
    brand_text.push(
        Paragraph::new(APP_NAME).styled(
            style::Style::new()
                .bold()
                .with_font_size(20)
                .with_color(BRAND),
        ),
    );
    brand_text.push(
        Paragraph::new(APP_TAGLINE).styled(style::Style::new().with_font_size(8).with_color(GREY)),
    );

    let mut brand_row = TableLayout::new(vec![1, 6]);
    brand_row
        .row()
        .element(logo)
        .element(brand_text)
        .push()
        .expect("brand row must carry exactly two cells");

    let mut left = LinearLayout::vertical();
    left.push(brand_row);
    left.push(elements::Break::new(0.4));
    for line in &header.issuer_lines {
        left.push(
            Paragraph::new(line).styled(
                style::Style::new()
                    .with_font_size(9)
                    .with_color(INK),
            ),
        );
    }

    // ── Right column: recipient block ──
    let mut right = LinearLayout::vertical();
    match &header.recipient {
        Some(recipient) => {
            right.push(
                Paragraph::new(&recipient.label)
                    .aligned(Alignment::Right)
                    .styled(style::Style::new().bold().with_font_size(9).with_color(GREY)),
            );
            for line in &recipient.lines {
                right.push(
                    Paragraph::new(line)
                        .aligned(Alignment::Right)
                        .styled(style::Style::new().with_font_size(9).with_color(INK)),
                );
            }
        }
        // No recipient: keep the cell non-empty (correspondence documents
        // addressed to the whole copropriété still show the header row).
        None => right.push(Paragraph::new("")),
    }

    // ── Two-column header: issuer left, recipient right ──
    let mut header_row = TableLayout::new(vec![6, 1, 4]);
    header_row
        .row()
        .element(left)
        .element(Paragraph::new(""))
        .element(right)
        .push()
        .expect("header row must carry exactly three cells");
    layout.push(header_row);
    layout.push(elements::Break::new(0.5));
    layout.push(brand_hairline());
    layout.push(elements::Break::new(1.0));

    Ok(())
}

// ── Letter body (first page) ────────────────────────────────────────────────

/// Pushes the letter subject block: « Objet : … »-style bold line with an
/// optional grey reference line underneath (e.g. « Réf. LR-CARNET-2026 »).
pub fn push_objet(layout: &mut LinearLayout, objet: &str, reference: Option<&str>) {
    layout.push(
        Paragraph::new(objet).styled(
            style::Style::new()
                .bold()
                .with_font_size(12)
                .with_color(INK),
        ),
    );
    if let Some(reference) = reference {
        layout.push(
            Paragraph::new(reference).styled(style::Style::new().with_font_size(9).with_color(GREY)),
        );
    }
    layout.push(elements::Break::new(1.2));
}

/// Pushes an optional salutation block (correspondence-style documents).
pub fn push_salutation(layout: &mut LinearLayout, salutation: &str) {
    layout.push(Paragraph::new(salutation));
    layout.push(elements::Break::new(0.5));
}

// ── Annex header (annex pages) ─────────────────────────────────────────────

/// Pushes the annex header opening an annex: spaced kicker (« A N N E X E »),
/// title in brand color, optional subtitle, meta lines, then a brand-colored
/// rule — followed by the tabular content of the annex.
pub fn push_annex_header(layout: &mut LinearLayout, annex: &DocumentAnnexHeader) {
    layout.push(
        Paragraph::new(spaced_caps(&annex.kicker))
            .styled(style::Style::new().bold().with_font_size(9).with_color(GREY)),
    );
    layout.push(
        Paragraph::new(&annex.title).styled(
            style::Style::new()
                .bold()
                .with_font_size(16)
                .with_color(BRAND),
        ),
    );
    if let Some(subtitle) = &annex.subtitle {
        layout.push(
            Paragraph::new(subtitle).styled(style::Style::new().with_font_size(11).with_color(GREY)),
        );
    }
    layout.push(elements::Break::new(0.5));
    for (label, value) in &annex.meta {
        layout.push(meta_line(label, value));
    }
    layout.push(elements::Break::new(0.6));
    layout.push(brand_hairline());
    layout.push(elements::Break::new(1.0));
}

/// « Annexe » → « A N N E X E » (letter-spaced kicker).
fn spaced_caps(text: &str) -> String {
    text.chars()
        .flat_map(|c| [c, ' '])
        .collect::<String>()
        .trim_end()
        .to_string()
}

/// A `label : value` meta line (label in grey, value in ink).
fn meta_line(label: &str, value: &str) -> Paragraph {
    let mut p = Paragraph::new("");
    p.push_styled(format!("{label} : "), style::Style::new().with_color(GREY));
    p.push_styled(value.to_string(), style::Style::new().with_color(INK));
    p
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

// ── Conclusion / signature (end of the letter) ──────────────────────────────

/// Pushes the signature block at the end of the letter: optional legal
/// validity period, generous spacing, then a **right-aligned** block —
/// « Pour {qualité} », a signature space, the signer name in bold, the signer
/// role in small grey type, and the **filled** « Fait à {place}, le {date} »
/// line (artificial signature — no signature image).
pub fn push_conclusion(layout: &mut LinearLayout, conclusion: &DocumentConclusion) {
    if let Some(validity) = &conclusion.validity_period {
        layout.push(elements::Break::new(0.5));
        layout.push(muted(validity));
    }
    layout.push(elements::Break::new(2.0));

    let mut sig = LinearLayout::vertical();
    sig.push(
        Paragraph::new(&conclusion.closing)
            .aligned(Alignment::Right)
            .styled(style::Style::new().with_color(INK)),
    );
    sig.push(elements::Break::new(4.0));
    sig.push(
        Paragraph::new(&conclusion.signer_name)
            .aligned(Alignment::Right)
            .styled(
                style::Style::new()
                    .bold()
                    .with_font_size(11)
                    .with_color(INK),
            ),
    );
    if let Some(role) = &conclusion.signer_role {
        sig.push(
            Paragraph::new(role)
                .aligned(Alignment::Right)
                .styled(style::Style::new().with_font_size(9).with_color(GREY)),
        );
    }
    sig.push(
        Paragraph::new(format!(
            "Fait à {}, le {}",
            conclusion.place,
            format_date_long(conclusion.date)
        ))
        .aligned(Alignment::Right)
        .styled(style::Style::new().with_color(INK)),
    );

    // Right-align the whole block: empty spacer column on the left.
    let mut row = TableLayout::new(vec![6, 4]);
    row.row()
        .element(Paragraph::new(""))
        .element(sig)
        .push()
        .expect("signature row must carry exactly two cells");
    layout.push(row);
}

/// Renders the document to bytes.
pub fn finish_document(doc: Document) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    doc.render(&mut out).map_err(|e| e.to_string())?;
    Ok(out)
}

// ── Formatting helpers (shared, FR conventions) ────────────────────────────

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
/// edition date in the introduction and conclusion.
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

// ── Internal layout primitives ─────────────────────────────────────────────

/// Thin horizontal rule in `RULE` grey — a real vector line drawn with a
/// `RuleLine` cell decorator (no underscore characters).
pub fn hairline() -> TableLayout {
    rule(RULE)
}

/// Brand-colored horizontal rule closing the double header and the annex
/// header (violet accent of the shared document template).
pub fn brand_hairline() -> TableLayout {
    rule(BRAND)
}

/// One-row table whose single cell carries a bottom border in `color`.
fn rule(color: Color) -> TableLayout {
    let mut table = TableLayout::new(vec![1]);
    table.set_cell_decorator(RuleLine(color));
    table
        .row()
        .element(Paragraph::new(" ").styled(style::Style::new().with_font_size(1)))
        .push()
        .expect("rule row must carry exactly one cell");
    table
}

/// Cell decorator drawing a thin bottom border in the given color under the
/// cell.
struct RuleLine(Color);

impl elements::CellDecorator for RuleLine {
    fn decorate_cell(
        &mut self,
        _column: usize,
        _row: usize,
        _has_more: bool,
        area: Area<'_>,
        style: style::Style,
    ) {
        let size = area.size();
        let mut rule_style = style;
        rule_style.set_color(self.0);
        area.draw_line(
            vec![
                Position::new(0, size.height),
                Position::new(size.width, size.height),
            ],
            rule_style,
        );
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn header() -> DocumentHeader {
        DocumentHeader {
            issuer_lines: vec![
                "Le syndic des copropriétaires".to_string(),
                "Cabinet Dupont — contact@cabinet-dupont.fr".to_string(),
                "SIRET : 123 456 789 00012".to_string(),
            ],
            recipient: Some(DocumentRecipient {
                label: "Destinataire".to_string(),
                lines: vec![
                    "Résidence des Tilleuls".to_string(),
                    "12 rue des Fleurs, 75011 Paris".to_string(),
                ],
            }),
        }
    }

    fn annex_header() -> DocumentAnnexHeader {
        DocumentAnnexHeader {
            kicker: "Annexe".to_string(),
            title: "Carnet d'entretien de l'immeuble".to_string(),
            subtitle: Some("Résidence des Tilleuls — 12 rue des Fleurs, 75011 Paris".to_string()),
            meta: vec![
                ("Syndic en exercice".to_string(), "Cabinet Dupont".to_string()),
                ("Édition du".to_string(), "16/06/2026".to_string()),
            ],
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
    fn logo_decodes() {
        logo_element(16.0).expect("embedded logo must decode");
    }

    #[test]
    fn spaced_caps_inserts_a_space_between_every_character() {
        assert_eq!(spaced_caps("Annexe"), "A N N E X E");
        assert_eq!(spaced_caps("A"), "A");
    }

    #[test]
    fn document_with_template_produces_valid_pdf() {
        let mut doc = new_document(
            font_family().expect("fonts"),
            date(2026, 6, 16),
            "Test document",
        );
        let mut layout = LinearLayout::vertical();
        push_letter_header(&mut layout, &header()).expect("letter header");
        push_objet(
            &mut layout,
            "Carnet d'entretien de l'immeuble — édition 2026",
            Some("Réf. LR-CARNET-2026 · Édition du 16/06/2026"),
        );
        push_salutation(&mut layout, "Madame, Monsieur,");
        layout.push(section("1. Première section"));
        layout.push(body("Corps du document de test."));
        layout.push(muted("Aucune donnée disponible."));
        push_conclusion(
            &mut layout,
            &DocumentConclusion {
                closing: "Pour le syndic,".to_string(),
                signer_name: "Cabinet Dupont".to_string(),
                signer_role: Some("Syndic en exercice".to_string()),
                place: "Paris".to_string(),
                date: date(2026, 6, 16),
                validity_period: Some("Document valable 3 mois.".to_string()),
            },
        );
        layout.push(elements::PageBreak::new());
        push_annex_header(&mut layout, &annex_header());
        layout.push(section("2. Annexe"));
        layout.push(body("Contenu de l'annexe."));
        doc.push(layout);

        let bytes = finish_document(doc).expect("render");
        assert!(bytes.starts_with(b"%PDF"));
        assert!(!bytes.is_empty());
    }
}


//! Shared PDF document template (EP-009-S09 spec change: all document PDFs are
//! generated server-side and share this template).
//!
//! Every generated document — logbook export, état daté, pré-état daté, future
//! AG minutes, reminder letters — is built from this module so that the layout
//! is identical everywhere:
//!
//! - **Introduction (first page only)** — the full application brand block:
//!   logo + app name « La Ruche » + tagline, then the document title (centered,
//!   brand color), the identity of the copropriété (building name, address,
//!   syndic), the edition line « Fait à {place}, le {date} » + « Pour le compte
//!   de {syndic} », and an optional explanatory paragraph describing the
//!   document. Generous spacing, then the content starts.
//! - **No running header** — pages 2+ start directly with the content (per
//!   spec: « par en-tête je voulais dire introduction »).
//! - **Salutation** — optional, for correspondence-style documents.
//! - **Conclusion / signature block** — optional validity period when the
//!   document has a legal one (e.g. état daté: valid 3 months), then
//!   « Fait à {place}, le {date} » (filled) signed « La Ruche » (artificial
//!   signature, no image).
//! - **Footer (every page)** — small grey type: logo + tagline on the left,
//!   « Édité le {date} via La Ruche » on the right, under a hairline rule.
//!   genpdf has no native « total pages » footer, so no page number.
//! - **Typography & palette** — Liberation Sans (embedded at compile time),
//!   brand color `#643fd5` mirrored from `frontend/src/styles/tokens.css`
//!   (`--color-primary` light theme), grey `#6b7280` for secondary text.
//!
//! Pure functions only — no I/O, no SQL. Callers assemble the data and hand it
//! here; they receive the finished PDF as in-memory bytes (see
//! `domain/legal_documents.rs` for the established pattern, ADR-011).

use chrono::{Datelike, NaiveDate};
use genpdf::{
    Alignment, Context, Document, Element, Margins, Mm, PageDecorator, Position,
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

const FONT_REGULAR: &[u8] = include_bytes!("../../assets/fonts/LiberationSans-Regular.ttf");
const FONT_BOLD: &[u8] = include_bytes!("../../assets/fonts/LiberationSans-Bold.ttf");

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

/// Introduction block rendered once, on the first page: brand block (logo +
/// name + tagline), document title, copropriété identity, edition line and
/// an optional explanatory paragraph. This is the « en-tête » of the document
/// in the spec sense — it does NOT repeat on every page.
#[derive(Debug, Clone)]
pub struct DocumentIntro {
    /// Document title, e.g. « Carnet d'entretien de l'immeuble ».
    pub title: String,
    /// Optional subtitle, e.g. the building name.
    pub subtitle: Option<String>,
    /// Meta lines rendered under the title (label: value), e.g. syndic, lot.
    pub meta: Vec<(String, String)>,
    /// Place of issue for the « Fait à {place}, le {date} » line.
    pub place: String,
    /// Edition date — also rendered in the page footer.
    pub edited_at: NaiveDate,
    /// « Pour le compte de {for_whom} » line, e.g. the syndic name.
    pub for_whom: String,
    /// Optional paragraph explaining the content of the document, rendered
    /// between the edition line and the content.
    pub purpose: Option<String>,
}

/// Conclusion block rendered at the end of the document: optional legal
/// validity period, then the filled « Fait à …, le … » line signed « La Ruche ».
#[derive(Debug, Clone)]
pub struct DocumentConclusion {
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

/// Builds the app logo as a **vector** element (7 hexagons in the brand
/// gradient), mirroring `frontend/src/components/atoms/CLogoMark.svg`.
/// Rendered at `size_mm` millimeters. Vector keeps the PDF small and crisp
/// at any zoom — and needs no `image` crate (only genpdf's `draw_line`).
pub fn logo_element(size_mm: f64) -> LogoElement {
    LogoElement { size_mm }
}

/// Vector logo element: 7 hexagons forming the « ruche », in the brand
/// gradient `#643fd5` → `#7d5bf0`, drawn with genpdf's `Area::draw_line`.
pub struct LogoElement {
    size_mm: f64,
}

/// Hexagone centers in the 80×80 viewBox of `CLogoMark.svg`.
const LOGO_HEXES: [(f64, f64); 7] = [
    (51.5, 40.0),
    (51.5, 17.48),
    (71.0, 28.74),
    (71.0, 51.26),
    (51.5, 62.52),
    (32.0, 51.26),
    (32.0, 28.74),
];

/// Hexagone vertices (pointy-top) in the 80×80 viewBox, relative to center.
/// From CLogoMark.svg: half-width 11.5, half-height 9.96, with the top/bottom
/// points slightly inset (45.75/34.25 vs 51.5/28.5).
const HEX_VERTS: [(f64, f64); 6] = [
    (0.0, -9.96),
    (5.75, -4.98),
    (5.75, 4.98),
    (0.0, 9.96),
    (-5.75, 4.98),
    (-5.75, -4.98),
];

/// Brand gradient endpoints in the 80×80 viewBox (userSpaceOnUse).
const GRAD_FROM: (f64, f64) = (9.0, 7.52);
const GRAD_TO: (f64, f64) = (71.0, 72.48);
const GRAD_C1: (u8, u8, u8) = (100, 63, 213); // #643fd5
const GRAD_C2: (u8, u8, u8) = (125, 91, 240); // #7d5bf0

fn lerp_color(t: f64) -> Color {
    let t = t.clamp(0.0, 1.0);
    let r = (GRAD_C1.0 as f64 + (GRAD_C2.0 as f64 - GRAD_C1.0 as f64) * t) as u8;
    let g = (GRAD_C1.1 as f64 + (GRAD_C2.1 as f64 - GRAD_C1.1 as f64) * t) as u8;
    let b = (GRAD_C1.2 as f64 + (GRAD_C2.2 as f64 - GRAD_C1.2 as f64) * t) as u8;
    Color::Rgb(r, g, b)
}

impl Element for LogoElement {
    fn render(
        &mut self,
        _context: &Context,
        area: Area<'_>,
        _style: style::Style,
    ) -> Result<genpdf::RenderResult, error::Error> {
        // Map the 80×80 viewBox onto a size_mm × size_mm square at the area
        // origin. PDF y grows upward, the viewBox y grows downward — flip y.
        let k = self.size_mm / 80.0;
        let to_area =
            |vx: f64, vy: f64| -> Position { Position::new(mm(vx * k), mm((80.0 - vy) * k)) };

        // Draw each hexagon as a filled polygon approximation: since genpdf
        // only strokes lines, we fill by drawing many horizontal spans between
        // the left and right edges of the hexagon for each scanline.
        for (cx, cy) in LOGO_HEXES {
            // Vertices in viewBox coordinates.
            let verts: Vec<(f64, f64)> = HEX_VERTS
                .iter()
                .map(|(dx, dy)| (cx + dx, cy + dy))
                .collect();
            let min_y = verts.iter().map(|v| v.1).fold(f64::INFINITY, f64::min);
            let max_y = verts.iter().map(|v| v.1).fold(f64::NEG_INFINITY, f64::max);

            // Scanline fill: for each viewBox-y row inside the hexagon, find
            // the intersections with the edges and stroke a horizontal line.
            let steps = 24; // scanlines per hexagon (enough at 16 mm)
            for i in 0..steps {
                let vy = min_y + (max_y - min_y) * (i as f64 + 0.5) / steps as f64;
                let mut xs: Vec<f64> = Vec::new();
                for w in 0..6 {
                    let (x1, y1) = verts[w];
                    let (x2, y2) = verts[(w + 1) % 6];
                    if (y1 <= vy && vy < y2) || (y2 <= vy && vy < y1) {
                        let t = (vy - y1) / (y2 - y1);
                        xs.push(x1 + t * (x2 - x1));
                    }
                }
                xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
                for pair in xs.chunks_exact(2) {
                    // Gradient color at the scanline midpoint.
                    let mid_vx = (pair[0] + pair[1]) / 2.0;
                    let (gx1, gy1) = GRAD_FROM;
                    let (gx2, gy2) = GRAD_TO;
                    let (dx, dy) = (gx2 - gx1, gy2 - gy1);
                    let t = ((mid_vx - gx1) * dx + (vy - gy1) * dy) / (dx * dx + dy * dy);
                    area.draw_line(
                        vec![to_area(pair[0], vy), to_area(pair[1], vy)],
                        style::Style::new().with_color(lerp_color(t)),
                    );
                }
            }
        }

        Ok(genpdf::RenderResult {
            size: genpdf::Size::new(mm(self.size_mm), mm(self.size_mm)),
            has_more: false,
        })
    }
}

// ── Document construction ──────────────────────────────────────────────────

/// A4 document pre-configured with the shared template: embedded fonts,
/// margins and a **running footer on every page** (logo + tagline + edition
/// date, small grey type). There is intentionally **no running header** — the
/// introduction is part of the body and appears on the first page only.
/// The caller fills the body: `push_intro`, content, `push_conclusion`.
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

        // Logo at the bottom-left of the footer area (vector hexagons).
        let mut logo = logo_element(FOOTER_LOGO_MM);
        let logo_area = {
            let mut a = page.clone();
            a.set_size(genpdf::Size::new(mm(FOOTER_LOGO_MM), mm(FOOTER_LOGO_MM)));
            a.add_offset(Position::new(0, footer_top));
            a
        };
        let _ = logo.render(context, logo_area, style::Style::default());

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

// ── Introduction (first page) ──────────────────────────────────────────────

/// Pushes the document introduction on the first page: brand block (logo +
/// « La Ruche » + tagline), centered title + subtitle, meta lines, the
/// « Fait à {place}, le {date} » + « Pour le compte de {for_whom} » lines,
/// and an optional purpose paragraph — all with generous spacing, then the
/// content begins.
pub fn push_intro(layout: &mut LinearLayout, intro: &DocumentIntro) -> Result<(), String> {
    // ── Brand block: logo + app name + tagline ──
    let logo = logo_element(INTRO_LOGO_MM);
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
    layout.push(brand_row);
    layout.push(elements::Break::new(1.5));

    // ── Title + subtitle ──
    layout.push(
        Paragraph::new(&intro.title)
            .aligned(Alignment::Center)
            .styled(
                style::Style::new()
                    .bold()
                    .with_font_size(16)
                    .with_color(BRAND),
            ),
    );
    if let Some(subtitle) = &intro.subtitle {
        layout.push(
            Paragraph::new(subtitle)
                .aligned(Alignment::Center)
                .styled(style::Style::new().with_font_size(11).with_color(GREY)),
        );
    }
    layout.push(elements::Break::new(0.8));

    // ── Identity of the copropriété ──
    for (label, value) in &intro.meta {
        layout.push(meta_line(label, value));
    }
    layout.push(elements::Break::new(0.8));

    // ── Edition line: « Fait à …, le … » + « Pour le compte de … » ──
    layout.push(
        Paragraph::new(format!(
            "Fait à {}, le {}",
            intro.place,
            format_date_long(intro.edited_at)
        ))
        .styled(style::Style::new().with_color(INK)),
    );
    layout.push(
        Paragraph::new(format!("Pour le compte de {}", intro.for_whom))
            .styled(style::Style::new().with_color(INK)),
    );
    layout.push(elements::Break::new(1.0));

    // ── Optional purpose paragraph ──
    if let Some(purpose) = &intro.purpose {
        layout.push(body(purpose));
        layout.push(elements::Break::new(1.0));
    }

    Ok(())
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

// ── Conclusion (last block of the body) ────────────────────────────────────

/// Pushes the conclusion block: optional legal validity period, generous
/// spacing, then the **filled** « Fait à {place}, le {date} » line signed
/// « La Ruche » (artificial signature — the app signs, no signature image).
pub fn push_conclusion(layout: &mut LinearLayout, conclusion: &DocumentConclusion) {
    if let Some(validity) = &conclusion.validity_period {
        layout.push(elements::Break::new(0.5));
        layout.push(muted(validity));
    }
    layout.push(elements::Break::new(1.5));
    let signed = format!(
        "Fait à {}, le {}",
        conclusion.place,
        format_date_long(conclusion.date)
    );
    layout.push(
        LinearLayout::vertical()
            .element(Paragraph::new(&signed).styled(style::Style::new().with_color(INK)))
            .element(elements::Break::new(2.5))
            .element(
                Paragraph::new(APP_NAME).styled(
                    style::Style::new()
                        .bold()
                        .with_font_size(11)
                        .with_color(BRAND),
                ),
            )
            .padded(2),
    );
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
/// `LineRule` cell decorator (no underscore characters).
pub fn hairline() -> TableLayout {
    let mut table = TableLayout::new(vec![1]);
    table.set_cell_decorator(LineRule);
    table
        .row()
        .element(Paragraph::new(" ").styled(style::Style::new().with_font_size(1)))
        .push()
        .expect("hairline row must carry exactly one cell");
    table
}

/// Cell decorator drawing a thin bottom border in `RULE` grey under the cell.
struct LineRule;

impl elements::CellDecorator for LineRule {
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
        rule_style.set_color(RULE);
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

    fn intro() -> DocumentIntro {
        DocumentIntro {
            title: "Document de test".to_string(),
            subtitle: Some("Résidence des Tilleuls".to_string()),
            meta: vec![
                ("Syndic".to_string(), "Cabinet Dupont".to_string()),
                ("Lot".to_string(), "A-12".to_string()),
            ],
            place: "Paris".to_string(),
            edited_at: date(2026, 6, 16),
            for_whom: "Cabinet Dupont, syndic de la Résidence des Tilleuls".to_string(),
            purpose: Some(
                "Ce document présente le contenu de test du template partagé.".to_string(),
            ),
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
    fn logo_element_builds() {
        let _ = logo_element(16.0);
    }

    #[test]
    fn document_with_template_produces_valid_pdf() {
        let mut doc = new_document(
            font_family().expect("fonts"),
            date(2026, 6, 16),
            "Test document",
        );
        let mut layout = LinearLayout::vertical();
        push_intro(&mut layout, &intro()).expect("intro");
        push_salutation(&mut layout, "Madame, Monsieur,");
        layout.push(section("1. Première section"));
        layout.push(body("Corps du document de test."));
        layout.push(muted("Aucune donnée disponible."));
        push_conclusion(
            &mut layout,
            &DocumentConclusion {
                place: "Paris".to_string(),
                date: date(2026, 6, 16),
                validity_period: Some("Document valable 3 mois.".to_string()),
            },
        );
        doc.push(layout);

        let bytes = finish_document(doc).expect("render");
        assert!(bytes.starts_with(b"%PDF"));
        assert!(!bytes.is_empty());
    }
}

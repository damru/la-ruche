//! Shared PDF document template (EP-009-S09 spec change: all document PDFs are
//! generated server-side and share this template).
//!
//! Every generated document — logbook export, état daté, pré-état daté, future
//! AG minutes, reminder letters — is built from this module so that the layout
//! is identical everywhere. Layout v7.1 (retour utilisateur PR 155) :
//!
//! - **Bloc marque « app web » (première page)** — logo (même PNG que la barre
//!   latérale de l'app), « La Ruche » en Fraunces Bold (la police display du
//!   web, `--font-display`) en `--on-surface`, tagline « Gestion participative »
//!   (i18n `auth.brandEyebrow`) en DM Sans en `--on-surface-variant`.
//! - **Double en-tête aligné** — deux lignes sur la même grille : le bloc marque
//!   sur la première, puis l'émetteur (gauche) et le destinataire (droite) qui
//!   commencent sur la même ligne. Filet violet sous l'en-tête.
//! - **Corps de lettre justifié** — genpdf 0.2 n'a pas de justification native
//!   : un élément custom `JustifiedParagraph` étire toutes les lignes sauf la
//!   dernière du paragraphe sur toute la largeur utile.
//! - **Signature (fin de lettre)** — bloc aligné à droite : « Pour {qualité} »,
//!   espace, nom du signataire en gras, qualité en gris, « Fait à {place}, le
//!   {date} » rempli.
//! - **En-tête d'annexe** — kicker espacé (A N N E X E), titre en couleur
//!   marque, sous-titre, méta-lignes, filet violet ; les tableaux suivent.
//! - **Footer (chaque page)** — mini logo + « Gestion participative — La
//!   Ruche » (DM Sans) à gauche, « Édité le {date} via La Ruche » aligné à
//!   droite, sous un filet gris. Le décorateur applique aussi les marges de
//!   page et réserve la zone de pied de page.
//! - **Typography & palette** — Tinos (serif, metric-compatible Times New
//!   Roman) pour le corps, Fraunces Bold / DM Sans Regular pour la marque
//!   (embarquées, un seul slot chacune), brand color `#643fd5` (`--color-primary`
//!   light theme), couleurs web `#191c1d` (on-surface) et `#42474e`
//!   (on-surface-variant).
//!
//! Pure functions only — no I/O, no SQL. Callers assemble the data and hand it
//! here; they receive the finished PDF as in-memory bytes (see
//! `domain/legal_documents.rs` for the established pattern, ADR-011).

use std::collections::VecDeque;

use chrono::{Datelike, NaiveDate};
use genpdf::{
    Alignment, Context, Document, Element, Margins, Mm, PageDecorator, Position, RenderResult,
    Rotation, Scale, Size,
    elements::{self, LinearLayout, Paragraph, TableLayout},
    error,
    fonts::{self, FontData, FontFamily},
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
/// Web `--on-surface` (`#191c1d`) — app name in the brand block.
pub const ON_SURFACE: Color = Color::Rgb(25, 28, 29);
/// Web `--on-surface-variant` (`#42474e`) — tagline in the brand block.
pub const ON_SURFACE_VARIANT: Color = Color::Rgb(66, 71, 78);

// Tinos (serif, métrique Times New Roman) sous-ensemble (pyftsubset) :
// ASCII + Latin-1 + ponctuation typographique (tirets, apostrophes, ellipsis,
// €). ~32 ko par fichier — les documents FR n'utilisent aucun glyphe hors de
// ce périmètre. Police serif choisie pour le rendu « document officiel »
// (EP-009-S09). Liberation Sans reste embarquée pour `legal_documents.rs`.
const FONT_REGULAR: &[u8] = include_bytes!("../../assets/fonts/Tinos-Regular.ttf");
const FONT_BOLD: &[u8] = include_bytes!("../../assets/fonts/Tinos-Bold.ttf");
const FONT_ITALIC: &[u8] = include_bytes!("../../assets/fonts/Tinos-Italic.ttf");
const FONT_BOLD_ITALIC: &[u8] = include_bytes!("../../assets/fonts/Tinos-BoldItalic.ttf");

/// Fraunces Bold — web display font (`--font-display`), renders the app name
/// « La Ruche » of the brand block. Latin subset from fonts.gstatic.com (~70 ko).
const FONT_BRAND: &[u8] = include_bytes!("../../assets/fonts/Fraunces-Bold.ttf");
/// DM Sans Regular — web UI font, renders the tagline « Gestion participative »
/// (brand block and page footer). Latin subset from fonts.gstatic.com (~47 ko).
const FONT_UI: &[u8] = include_bytes!("../../assets/fonts/DMSans-Regular.ttf");

/// App name rendered next to the logo in the brand block and in the footer.
pub const APP_NAME: &str = "La Ruche";
/// Tagline rendered under the app name (brand block) and in the page footer —
/// the web sidebar eyebrow (i18n `auth.brandEyebrow`).
pub const APP_TAGLINE: &str = "Gestion participative";

/// Vertical margin kept at the top of every page (no running header, just
/// breathing room). The footer decorator reserves space at the bottom.
const PAGE_TOP_MARGIN_MM: f64 = 18.0;
/// Horizontal page margin.
const PAGE_SIDE_MARGIN_MM: f64 = 18.0;
/// Height reserved at the bottom of every page for the running footer.
const FOOTER_HEIGHT_MM: f64 = 14.0;
/// Size (in mm) at which the logo is rendered in the brand block.
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

/// Double header block rendered once, on the first page: the full application
/// brand block « app web » (logo + « La Ruche » + tagline) on its own row, then
/// the issuer lines (left) and the optional recipient block (right) aligned on
/// the same row. A brand-colored rule closes the header. This is the « en-tête »
/// of the document in the spec sense — it does NOT repeat on every page.
#[derive(Debug, Clone)]
pub struct DocumentHeader {
    /// Issuer lines under the brand block (left column), e.g. « Le syndic des
    /// copropriétaires », « Damien Rubio — contact », « SIRET : … ». The first
    /// line is rendered bold, the others in grey.
    pub issuer_lines: Vec<String>,
    /// Recipient block (right column) — the user who requested the document.
    pub recipient: Option<DocumentRecipient>,
}

/// Recipient block rendered on the right of the double header.
#[derive(Debug, Clone)]
pub struct DocumentRecipient {
    /// Small grey bold label, e.g. « Destinataire ».
    pub label: String,
    /// Recipient lines underneath, right-aligned. The first line is rendered
    /// in ink, the others in grey.
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

// ── Fonts ──────────────────────────────────────────────────────────────────

/// Loads and parses the embedded Tinos family (serif — Times New Roman
/// metric-compatible), the default document font. Propagates the only
/// realistic failure (corrupted compile-time font asset) as a string.
pub fn font_family() -> Result<FontFamily<FontData>, String> {
    Ok(FontFamily {
        regular: FontData::new(FONT_REGULAR.to_vec(), None).map_err(|e| e.to_string())?,
        bold: FontData::new(FONT_BOLD.to_vec(), None).map_err(|e| e.to_string())?,
        italic: FontData::new(FONT_ITALIC.to_vec(), None).map_err(|e| e.to_string())?,
        bold_italic: FontData::new(FONT_BOLD_ITALIC.to_vec(), None).map_err(|e| e.to_string())?,
    })
}

/// Replacement slot that is never rendered: glyph metrics are parsed from the
/// embedded face, but the PDF maps the slot onto builtin Helvetica instead of
/// embedding a second copy. The brand and UI families only ever use their
/// `regular` slot — this keeps the PDF at two extra embedded fonts (~120 ko)
/// instead of eight. Never combine these slots with `.bold()`/`.italic()`.
fn builtin_slot(data: &[u8]) -> Result<FontData, String> {
    FontData::new(data.to_vec(), Some(printpdf::BuiltinFont::Helvetica)).map_err(|e| e.to_string())
}

/// Brand family: Fraunces Bold embedded on the regular slot — the app name is
/// never bolded through the style system (the weight is in the face itself).
pub fn brand_font_family() -> Result<FontFamily<FontData>, String> {
    Ok(FontFamily {
        regular: FontData::new(FONT_BRAND.to_vec(), None).map_err(|e| e.to_string())?,
        bold: builtin_slot(FONT_BRAND)?,
        italic: builtin_slot(FONT_BRAND)?,
        bold_italic: builtin_slot(FONT_BRAND)?,
    })
}

/// UI family: DM Sans Regular embedded on the regular slot (tagline, footer).
pub fn ui_font_family() -> Result<FontFamily<FontData>, String> {
    Ok(FontFamily {
        regular: FontData::new(FONT_UI.to_vec(), None).map_err(|e| e.to_string())?,
        bold: builtin_slot(FONT_UI)?,
        italic: builtin_slot(FONT_UI)?,
        bold_italic: builtin_slot(FONT_UI)?,
    })
}

/// Brand and UI font families registered in the document's font cache, ready
/// for `Style::with_font_family` (fonts already resolved by that cache).
#[derive(Clone, Copy)]
pub struct BrandFonts {
    /// Fraunces Bold — app name « La Ruche ».
    pub brand: fonts::FontFamily<fonts::Font>,
    /// DM Sans Regular — tagline « Gestion participative ».
    pub ui: fonts::FontFamily<fonts::Font>,
}

// ── Logo ───────────────────────────────────────────────────────────────────

/// Rasterised app logo (7 hexagones « ruche », dégradé `#643fd5` → `#7d5bf0`),
/// generated from `frontend/src/components/atoms/CLogoMark.svg` and embedded at
/// compile time so the PDF is self-contained (ADR-011). RGB, no alpha channel
/// (printpdf ne supporte pas la transparence — le logo est dessiné sur blanc).
/// Encodé en PNG RGB optimisé (~5 ko) ; en release, printpdf compresse le flux
/// image en FlateDecode (~4,6 ko par occurrence embarquée). C'est exactement le
/// logo de la barre latérale de l'app web.
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

/// A4 document pre-configured with the shared template: embedded fonts, a
/// running footer on every page (logo + tagline + edition date) and page
/// margins. There is intentionally **no running header** — the double header
/// is part of the body and appears on the first page only.
///
/// Returns the document together with the registered [`BrandFonts`] used by
/// the caller to render the brand block. The caller fills the body:
/// `push_letter_header`, letter content, then the annexes opened by
/// `push_annex_header`.
pub fn new_document(
    edited_at: NaiveDate,
    pdf_title: &str,
) -> Result<(Document, BrandFonts), String> {
    let mut doc = Document::new(font_family()?);
    doc.set_minimal_conformance();
    doc.set_font_size(10);
    doc.set_line_spacing(1.3);
    doc.set_title(pdf_title);

    let brand = doc.add_font_family(brand_font_family()?);
    let ui = doc.add_font_family(ui_font_family()?);

    doc.set_page_decorator(FooterDecorator::new(edited_at, ui));
    Ok((doc, BrandFonts { brand, ui }))
}

/// Page decorator that draws the running footer at the bottom of every page
/// (hairline rule, mini logo + « Gestion participative — La Ruche » on the
/// left, « Édité le {date} via La Ruche » right-aligned) and prepares the body
/// area: page margins plus the reserved footer height, so the content never
/// overlaps the footer. The template deliberately does not use
/// `SimplePageDecorator` — this decorator owns the margins itself.
struct FooterDecorator {
    edited_at: NaiveDate,
    /// DM Sans family used for the footer tagline.
    ui: fonts::FontFamily<fonts::Font>,
}

impl FooterDecorator {
    fn new(edited_at: NaiveDate, ui: fonts::FontFamily<fonts::Font>) -> Self {
        FooterDecorator { edited_at, ui }
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

        // Hairline rule across the content width, above the footer text.
        let rule_y = footer_top + mm(4.0);
        page.draw_line(
            vec![Position::new(0, rule_y), Position::new(page_width, rule_y)],
            style::Style::new().with_color(RULE),
        );

        // Mini logo at the bottom-left of the footer area (embedded PNG).
        if let Ok(dynamic) = image::load_from_memory(LOGO_PNG) {
            let scale = FOOTER_LOGO_MM / LOGO_NATURAL_MM;
            page.add_image(
                &dynamic,
                Position::new(0, footer_top + mm(2.0)),
                Scale::new(scale, scale),
                Rotation::default(),
                None,
            );
        }

        // Left: tagline + app name in DM Sans, small grey type.
        let text_x = mm(FOOTER_LOGO_MM + 3.0);
        let text_y = footer_top + mm(3.5);
        let tagline_style = style::Style::new()
            .with_font_family(self.ui)
            .with_font_size(7)
            .with_color(GREY);
        let _ = page.print_str(
            &context.font_cache,
            Position::new(text_x, text_y),
            tagline_style,
            format!("{APP_TAGLINE} — {APP_NAME}"),
        );

        // Right: edition line, right-aligned against the content width.
        let edition = format!("Édité le {} via {APP_NAME}", format_date(self.edited_at));
        let edition_style = style::Style::new().with_font_size(7).with_color(GREY);
        let edition_width = edition_style.str_width(&context.font_cache, &edition);
        let _ = page.print_str(
            &context.font_cache,
            Position::new(page_width - edition_width, text_y),
            edition_style,
            edition,
        );

        // Prepare the body area: page margins, then cap the height above the
        // footer zone so the content never overlaps it.
        area.add_margins(Margins::vh(PAGE_TOP_MARGIN_MM, PAGE_SIDE_MARGIN_MM));
        area.set_height(footer_top - mm(PAGE_TOP_MARGIN_MM));
        Ok(area)
    }
}

// ── Double header (first page) ─────────────────────────────────────────────

/// Pushes the double header on the first page, over two rows of the same
/// grid: the application brand block « app web » (logo + « La Ruche » en
/// Fraunces + « Gestion participative » en DM Sans — mêmes polices et couleurs
/// que la barre latérale de l'app) on its own row, then the issuer block
/// (left) and the recipient block (right) starting on the same line; a
/// brand-colored rule closes the header.
pub fn push_letter_header(
    layout: &mut LinearLayout,
    fonts: &BrandFonts,
    header: &DocumentHeader,
) -> Result<(), String> {
    // ── Brand block: logo + name (Fraunces Bold, on-surface) + tagline
    //    (DM Sans, on-surface-variant) — identical to the web sidebar. ──
    let logo = logo_element(INTRO_LOGO_MM)?;
    let mut brand_text = LinearLayout::vertical();
    brand_text.push(
        Paragraph::new(APP_NAME).styled(
            style::Style::new()
                .with_font_family(fonts.brand)
                .with_font_size(20)
                .with_color(ON_SURFACE),
        ),
    );
    brand_text.push(
        Paragraph::new(APP_TAGLINE).styled(
            style::Style::new()
                .with_font_family(fonts.ui)
                .with_font_size(14)
                .with_color(ON_SURFACE_VARIANT),
        ),
    );

    let mut brand_row = TableLayout::new(vec![1, 6]);
    brand_row
        .row()
        .element(logo)
        .element(brand_text)
        .push()
        .expect("brand row must carry exactly two cells");

    // ── Issuer block (left): heading bold, contact lines grey. ──
    let mut issuer = LinearLayout::vertical();
    if header.issuer_lines.is_empty() {
        issuer.push(Paragraph::new(""));
    }
    for (index, line) in header.issuer_lines.iter().enumerate() {
        let line_style = if index == 0 {
            style::Style::new().bold().with_font_size(9).with_color(INK)
        } else {
            style::Style::new().with_font_size(9).with_color(GREY)
        };
        issuer.push(Paragraph::new(line).styled(line_style));
    }

    // ── Recipient block (right): bold grey label, first line (the requester)
    //    in ink, following lines in grey — all right-aligned. ──
    let mut recipient = LinearLayout::vertical();
    match &header.recipient {
        Some(recipient_block) => {
            recipient.push(
                Paragraph::new(&recipient_block.label)
                    .aligned(Alignment::Right)
                    .styled(
                        style::Style::new()
                            .bold()
                            .with_font_size(9)
                            .with_color(GREY),
                    ),
            );
            for (index, line) in recipient_block.lines.iter().enumerate() {
                let color = if index == 0 { INK } else { GREY };
                recipient.push(
                    Paragraph::new(line)
                        .aligned(Alignment::Right)
                        .styled(style::Style::new().with_font_size(9).with_color(color)),
                );
            }
        }
        // No recipient: keep the cell non-empty (correspondence documents
        // addressed to the whole copropriété still show the header row).
        None => recipient.push(Paragraph::new("")),
    }

    // ── Two-row header grid: the brand block alone on the first row, issuer
    //    and recipient aligned on the same second row. ──
    let mut header_row = TableLayout::new(vec![6, 1, 4]);
    header_row
        .row()
        .element(brand_row)
        .element(Paragraph::new(""))
        .element(Paragraph::new(""))
        .push()
        .expect("brand header row must carry exactly three cells");
    header_row
        .row()
        .element(issuer)
        .element(Paragraph::new(""))
        .element(recipient)
        .push()
        .expect("issuer/recipient row must carry exactly three cells");
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
            Paragraph::new(reference)
                .styled(style::Style::new().with_font_size(9).with_color(GREY)),
        );
    }
    layout.push(elements::Break::new(1.2));
}

/// Pushes an optional salutation block (correspondence-style documents).
pub fn push_salutation(layout: &mut LinearLayout, salutation: &str) {
    layout.push(Paragraph::new(salutation));
    layout.push(elements::Break::new(0.5));
}

// ── Justified letter body ───────────────────────────────────────────────────

/// Letter body paragraph with justified text: every line except the last of
/// the paragraph is stretched over the full available width. genpdf 0.2 has no
/// native justification, so this custom element wraps words greedily (like
/// `Paragraph`), measures the leftover space and spreads it evenly across the
/// inter-word gaps.
pub struct JustifiedParagraph {
    words: VecDeque<String>,
}

impl JustifiedParagraph {
    /// Creates a justified paragraph from the given text.
    pub fn new(text: &str) -> JustifiedParagraph {
        JustifiedParagraph {
            words: text
                .split(' ')
                .filter(|word| !word.is_empty())
                .map(ToString::to_string)
                .collect(),
        }
    }
}

impl Element for JustifiedParagraph {
    fn render(
        &mut self,
        context: &Context,
        mut area: Area<'_>,
        style: style::Style,
    ) -> Result<RenderResult, error::Error> {
        let mut result = RenderResult::default();
        let font_cache = &context.font_cache;
        let height = style.line_height(font_cache);
        let space_width = style.str_width(font_cache, " ");
        let area_width = area.size().width;

        while !self.words.is_empty() {
            // Greedy line: peek the words that fit (nothing is consumed until
            // the line is rendered — an interrupted page must find the same
            // words at the front of the queue on the next call).
            let mut widths: Vec<Mm> = Vec::new();
            let mut natural = Mm::default();
            for word in self.words.iter() {
                let width = style.str_width(font_cache, word);
                if widths.is_empty() {
                    // First word: always taken, even if it alone overflows
                    // (same behaviour as genpdf's unhyphenated `Paragraph`).
                    widths.push(width);
                    natural += width;
                    continue;
                }
                if natural + space_width + width > area_width {
                    break;
                }
                widths.push(width);
                natural += space_width + width;
            }

            let count = widths.len();
            let is_last_line = count == self.words.len();
            // Justified inter-word gap: the slack is spread evenly across the
            // gaps of every line except the last of the paragraph (and lines
            // with a single word, which have no gap to stretch).
            let gap = if !is_last_line && count > 1 {
                let visible = widths.iter().fold(Mm::default(), |acc, w| acc + *w);
                let slack = area_width - visible;
                let even = slack * (1.0 / (count as f64 - 1.0));
                even.max(space_width)
            } else {
                space_width
            };

            // Render: one text section per word, at its justified position.
            let mut cursor = Mm::default();
            let mut printed = 0usize;
            let mut fits = true;
            for (index, word) in self.words.iter().take(count).enumerate() {
                let position = Position::new(cursor, Mm::default());
                if let Some(mut section) = area.text_section(font_cache, position, style) {
                    section.print_str(word, style)?;
                } else {
                    fits = false;
                    break;
                }
                cursor += widths[index];
                if index + 1 < count {
                    cursor += gap;
                }
                printed += 1;
            }

            // Consume only what was actually printed; if the line did not fit
            // the page, hand control back with `has_more` set.
            for _ in 0..printed {
                self.words.pop_front();
            }
            if !fits {
                result.has_more = true;
                break;
            }

            result.size = result.size.stack_vertical(Size::new(cursor, height));
            area.add_offset(Position::new(Mm::default(), height));
        }

        Ok(result)
    }
}

/// Justified body paragraph in ink color — the letter body of the document.
pub fn justified_body(text: &str) -> elements::StyledElement<JustifiedParagraph> {
    JustifiedParagraph::new(text).styled(style::Style::new().with_color(INK))
}

// ── Annex header (annex pages) ─────────────────────────────────────────────

/// Pushes the annex header opening an annex: spaced kicker (« A N N E X E »),
/// title in brand color, optional subtitle, meta lines, then a brand-colored
/// rule — followed by the tabular content of the annex.
pub fn push_annex_header(layout: &mut LinearLayout, annex: &DocumentAnnexHeader) {
    layout.push(
        Paragraph::new(spaced_caps(&annex.kicker)).styled(
            style::Style::new()
                .bold()
                .with_font_size(9)
                .with_color(GREY),
        ),
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
            Paragraph::new(subtitle)
                .styled(style::Style::new().with_font_size(11).with_color(GREY)),
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

/// « Annexe » → « A N N E X E » (uppercased, letter-spaced kicker).
fn spaced_caps(text: &str) -> String {
    text.to_uppercase()
        .chars()
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

/// Body paragraph in ink color (left-aligned — annex content and tables).
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
                    "Marie Dupont — Copropriétaire".to_string(),
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
                (
                    "Syndic en exercice".to_string(),
                    "Cabinet Dupont".to_string(),
                ),
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
        let (mut doc, fonts) = new_document(date(2026, 6, 16), "Test document").expect("document");
        let mut layout = LinearLayout::vertical();
        push_letter_header(&mut layout, &fonts, &header()).expect("letter header");
        push_objet(
            &mut layout,
            "Carnet d'entretien de l'immeuble — édition 2026",
            Some("Réf. LR-CARNET-2026 · Édition du 16/06/2026"),
        );
        push_salutation(&mut layout, "Madame, Monsieur,");
        layout.push(justified_body(
            "Conformément au décret n°2001-477, le syndic tient à jour le carnet d'entretien \
             de l'immeuble. Vous trouverez en annexe du présent document l'édition du carnet : \
             contrats d'assurance et de maintenance en cours, puis chronologie complète des \
             interventions.",
        ));
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

    #[test]
    fn justified_paragraph_paginates_a_long_paragraph() {
        // Un paragraphe justifié plus long qu'une page doit se paginer
        // proprement (has_more → re-render avec les mots restants) et produire
        // un PDF valide.
        let (mut doc, _fonts) = new_document(date(2026, 6, 16), "Justification").expect("document");
        let mut layout = LinearLayout::vertical();
        let words: Vec<String> = (0..400).map(|i| format!("mot{i}")).collect();
        layout.push(justified_body(&words.join(" ")));
        doc.push(layout);

        let bytes = finish_document(doc).expect("render");
        assert!(bytes.starts_with(b"%PDF"));
        assert!(bytes.len() > 10_000); // plusieurs pages de texte justifié
    }
}

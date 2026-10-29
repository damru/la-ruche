//! Shared PDF document engine (ADR-011) and formatting helpers.
//!
//! Every generated document — logbook export, état daté, pré-état daté, future
//! AG minutes, reminder letters — is composed with the embedded **typst**
//! engine: the Rust side serializes a payload to JSON, the engine compiles a
//! `.typ` template (shared base + document template, see
//! `src/domain/typst/`) against that payload and exports the PDF. All user
//! data crosses the boundary as JSON strings, so no markup escaping is ever
//! needed (typst strings are inert).
//!
//! - **Layout v7.1 (user feedback PR 155)** — brand block « app web » on the
//!   first page (logo + « La Ruche » in Fraunces + tagline in DM Sans), aligned
//!   double header (issuer left / recipient right), justified letter body,
//!   right-aligned signature, annex header with spaced kicker, and a centered
//!   running footer (edition line + legal disclaimer) on every page. The
//!   regulatory documents (état daté / pré-état daté) opt out of the brand
//!   block and the footer.
//! - **Shared letter frame** — everything common to letter documents (brand
//!   block, double header, objet, salutation, closing formula, signature,
//!   footer) is defined once in `typst/base.typ` and reused by every letter
//!   template; only the letter paragraphs and the annexes vary per document.
//!   Default line spacing is interligne 1,5; address-style blocks (issuer,
//!   recipient, annex meta, signatures) keep a compact leading.
//! - **Typography & palette** — Tinos (serif, metric-compatible Times New
//!   Roman) for the letter body, Fraunces Bold / DM Sans for the brand,
//!   Liberation Sans for the regulatory documents (all embedded, no system
//!   font dependency); brand color `#643fd5`, web greys `#6b7280` /
//!   `#42474e` / `#191c1d`.
//!
//! Pure functions only — no I/O, no SQL. Callers assemble the data and hand it
//! here; they receive the finished PDF as in-memory bytes (see
//! `domain/logbook_pdf.rs` and `domain/legal_documents.rs`).

use std::path::PathBuf;
use std::sync::LazyLock;

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::Serialize;
use typst::diag::{FileError, FileResult, SourceDiagnostic};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World, WorldExt};
use typst_layout::PagedDocument;

/// Application name rendered in the brand block, the running footer and the
/// PDF metadata.
pub const APP_NAME: &str = "La Ruche";

// ── Embedded assets (self-contained rendering, no system font dependency) ──

// Tinos (serif, Times New Roman metric-compatible) subset (pyftsubset):
// ASCII + Latin-1 + typographic punctuation (dashes, apostrophes, ellipsis,
// €). ~32 ko per file — FR documents use no glyph outside this scope.
const FONT_REGULAR: &[u8] = include_bytes!("../../assets/fonts/Tinos-Regular.ttf");
const FONT_BOLD: &[u8] = include_bytes!("../../assets/fonts/Tinos-Bold.ttf");
const FONT_ITALIC: &[u8] = include_bytes!("../../assets/fonts/Tinos-Italic.ttf");
const FONT_BOLD_ITALIC: &[u8] = include_bytes!("../../assets/fonts/Tinos-BoldItalic.ttf");
// Brand fonts (web `--font-display` / UI font).
const FONT_BRAND: &[u8] = include_bytes!("../../assets/fonts/Fraunces-Bold.ttf");
const FONT_UI: &[u8] = include_bytes!("../../assets/fonts/DMSans-Regular.ttf");
// Regulatory documents font.
const FONT_SANS: &[u8] = include_bytes!("../../assets/fonts/LiberationSans-Regular.ttf");
const FONT_SANS_BOLD: &[u8] = include_bytes!("../../assets/fonts/LiberationSans-Bold.ttf");

const LOGO_PNG: &[u8] = include_bytes!("../../assets/images/logo-la-ruche.png");

/// Shared base prepended to every document template (page setup, palette,
/// payload declaration, default interligne 1,5, regulatory helpers and the
/// shared letter frame).
const BASE_TEMPLATE: &str = include_str!("typst/base.typ");

// ── Embedded document templates (production fallback) ──────────────────────
//
// In production the binary is self-contained: templates are compiled in via
// `include_str!`. In development, set `LARUCHE_TEMPLATES_DIR` to the absolute
// path of `backend/src/domain/typst/` and templates are re-read from disk on
// every call — no rebuild needed for a margin tweak.

const EMBEDDED_TEMPLATES: &[(&str, &str)] = &[
    ("logbook", include_str!("typst/logbook.typ")),
    ("etat_date", include_str!("typst/etat_date.typ")),
    ("pre_etat_date", include_str!("typst/pre_etat_date.typ")),
];

/// Loads the shared base and a named document template.
///
/// When the `LARUCHE_TEMPLATES_DIR` environment variable is set, both files
/// are read from disk on every call (dev hot-reload). Otherwise the embedded
/// `include_str!` copies are used (production).
fn load_templates(template_name: &str) -> Result<(String, String), String> {
    if let Ok(dir) = std::env::var("LARUCHE_TEMPLATES_DIR") {
        let base_path = format!("{dir}/base.typ");
        let doc_path = format!("{dir}/{template_name}.typ");
        let base = std::fs::read_to_string(&base_path)
            .map_err(|e| format!("cannot read {base_path}: {e}"))?;
        let doc = std::fs::read_to_string(&doc_path)
            .map_err(|e| format!("cannot read {doc_path}: {e}"))?;
        Ok((base, doc))
    } else {
        let doc = EMBEDDED_TEMPLATES
            .iter()
            .find(|(name, _)| *name == template_name)
            .map(|(_, src)| *src)
            .ok_or_else(|| format!("unknown template: {template_name}"))?;
        Ok((BASE_TEMPLATE.to_string(), doc.to_string()))
    }
}

static FONTS: LazyLock<Vec<Font>> = LazyLock::new(|| {
    [
        FONT_REGULAR,
        FONT_BOLD,
        FONT_ITALIC,
        FONT_BOLD_ITALIC,
        FONT_BRAND,
        FONT_UI,
        FONT_SANS,
        FONT_SANS_BOLD,
    ]
    .iter()
    .filter_map(|data| Font::new(Bytes::new(data), 0))
    .collect()
});

static BOOK: LazyLock<LazyHash<FontBook>> = LazyLock::new(|| {
    let fonts: Vec<&Font> = FONTS.iter().collect();
    LazyHash::new(FontBook::from_fonts(fonts))
});

static LIBRARY: LazyLock<LazyHash<Library>> = LazyLock::new(|| LazyHash::new(Library::default()));

// ── Virtual world ──────────────────────────────────────────────────────────

/// Minimal typst `World`: one in-memory source (base + document template),
/// the embedded fonts, and two virtual files — the logo PNG and the serialized
/// JSON payload — resolved by the templates as `/logo.png` and
/// `/payload.json`.
struct DocWorld {
    main: FileId,
    source: Source,
    payload: String,
}

impl World for DocWorld {
    fn library(&self) -> &LazyHash<Library> {
        &LIBRARY
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &BOOK
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main {
            Ok(self.source.clone())
        } else {
            Err(FileError::NotFound(PathBuf::from(
                id.vpath().get_with_slash(),
            )))
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        match id.vpath().get_without_slash() {
            "logo.png" => Ok(Bytes::new(LOGO_PNG)),
            "payload.json" => Ok(Bytes::new(self.payload.clone())),
            _ => Err(FileError::NotFound(PathBuf::from(
                id.vpath().get_with_slash(),
            ))),
        }
    }

    fn font(&self, index: usize) -> Option<Font> {
        FONTS.get(index).cloned()
    }

    fn today(&self, offset: Option<Duration>) -> Option<Datetime> {
        // Templates receive every date through the payload; this only backs
        // typst's `datetime.today()` should a template ever use it.
        let now = match offset {
            None => chrono::Local::now().naive_local(),
            Some(duration) => {
                let [weeks, days, hours, minutes, seconds] = duration.decompose();
                chrono::Utc::now().naive_utc()
                    + chrono::Duration::weeks(weeks)
                    + chrono::Duration::days(days)
                    + chrono::Duration::hours(hours)
                    + chrono::Duration::minutes(minutes)
                    + chrono::Duration::seconds(seconds)
            }
        };
        Datetime::from_ymd(now.year(), now.month() as u8, now.day() as u8)
    }
}

// ── Compilation ────────────────────────────────────────────────────────────

/// Compiles a named document template (concatenated after the shared base)
/// with the given payload and returns the PDF bytes. Template errors are
/// reported with their line number in the assembled source.
///
/// The template name selects which `.typ` file to load (e.g. `"logbook"`,
/// `"etat_date"`, `"pre_etat_date"`). Templates are read from
/// `LARUCHE_TEMPLATES_DIR` when set (dev), or embedded via `include_str!`
/// (production).
pub(crate) fn compile_template(
    template_name: &str,
    payload: &impl Serialize,
) -> Result<Vec<u8>, String> {
    let (base, document_template) = load_templates(template_name)?;
    let source_text = format!("{base}\n{document_template}");
    let payload = serde_json::to_string(payload).map_err(|e| e.to_string())?;
    let id = FileId::new(RootedPath::new(
        VirtualRoot::Project,
        VirtualPath::new("/main.typ").expect("static virtual path"),
    ));
    let world = DocWorld {
        main: id,
        source: Source::new(id, source_text),
        payload,
    };

    let warned = typst::compile::<PagedDocument>(&world);
    let document = warned
        .output
        .map_err(|diags| format_diagnostics(&world, &diags))?;
    typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default())
        .map_err(|diags| format_diagnostics(&world, &diags))
}

/// Renders diagnostics as `line N: message` strings (joined), so a template
/// regression surfaces as a readable error instead of a raw span dump.
fn format_diagnostics(world: &DocWorld, diags: &[SourceDiagnostic]) -> String {
    diags
        .iter()
        .map(|diag| match world.range(diag.span) {
            Some(range) => {
                let text = world.source.text();
                let start = range.start.min(text.len());
                let line = text[..start].matches('\n').count() + 1;
                format!("line {line}: {}", diag.message)
            }
            None => diag.message.to_string(),
        })
        .collect::<Vec<_>>()
        .join("; ")
}

// ── Public data structures ─────────────────────────────────────────────────

/// Identity block shared by all generated documents: the copropriété and its
/// syndic. Assembled by the API layer from `building` + `governance_mandate`.
#[derive(Debug, Clone)]
pub struct DocumentIdentity {
    pub building_name: String,
    /// Full postal address on one line (« street, postal code city »).
    pub building_address: String,
    /// Street address alone (line 1 + optional line 2), without postal code.
    pub building_street_address: String,
    /// « postal code city » on one line.
    pub building_postal_city: String,
    pub syndic_name: Option<String>,
    pub syndic_siret: Option<String>,
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

/// `Option<Decimal>` → either the formatted amount or « - ».
pub fn optional_euro(value: Option<Decimal>) -> String {
    value.map(format_euro).unwrap_or_else(|| "-".to_string())
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[derive(Serialize)]
    struct SmokePayload {
        title: String,
        name: String,
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
        assert_eq!(optional_euro(None), "-");
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
    fn compile_template_produces_valid_pdf() {
        // Strings from the payload are inert: markup metacharacters must
        // render as-is, never be interpreted.
        let payload = SmokePayload {
            title: "Test".to_string(),
            name: "#1 $test <b> \"quotes\" @home".to_string(),
        };
        let bytes = compile_template("Hello #payload.name", &payload).expect("compile");
        assert!(bytes.starts_with(b"%PDF"));
        assert!(!bytes.is_empty());
    }

    #[test]
    fn compile_template_reports_template_errors_with_line() {
        let payload = SmokePayload {
            title: "Test".to_string(),
            name: "x".to_string(),
        };
        let err = compile_template("#payload.missing_field", &payload)
            .expect_err("unknown field must fail");
        assert!(err.contains("line"), "error should carry a line: {err}");
    }
}

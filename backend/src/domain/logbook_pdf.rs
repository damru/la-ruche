//! Logbook PDF rendering (EP-009-S09, spec change: server-side generation).
//!
//! Renders the regulatory logbook — « Carnet d'entretien de l'immeuble »
//! (décret n°2001-477, art. 1) — as a **letter + annex** PDF using the shared
//! document template (`domain/pdf_template.rs`, layout v7.1, retour utilisateur
//! PR 155) :
//!
//! - **Page 1 — lettre** : bloc marque « app web » (logo + « La Ruche » en
//!   Fraunces + « Gestion participative » en DM Sans), émetteur à gauche et
//!   destinataire à droite alignés sur la même ligne, filet violet, objet en
//!   gras + référence, salutation, paragraphes **justifiés** (dont l'annonce
//!   explicite de l'annexe), formule de politesse, puis signature alignée à
//!   droite (« Pour le syndic, », nom, qualité, « Fait à …, le … » rempli).
//! - **Pages suivantes — annexe** : kicker « A N N E X E », titre du carnet en
//!   couleur marque, identité de la copropriété, puis contrats d'assurance en
//!   cours, contrats de maintenance en cours et chronologie des interventions
//!   (le contenu tabulaire annoncé dans la lettre).
//! - **Pied de page récurrent** sur chaque page (logo + tagline + édition).
//!
//! Pure functions only — no I/O, no SQL. The API layer assembles the data and
//! receives the finished PDF as in-memory bytes.

use chrono::NaiveDate;
use genpdf::{
    Alignment, Element, Margins,
    elements::{self, LinearLayout, Paragraph, TableLayout},
    style,
};
use rust_decimal::Decimal;

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
    /// Edition date — rendered in the title block and the page footer.
    pub edited_at: NaiveDate,
    /// Syndic contact line under the identity (name + email/phone), if any.
    pub syndic_contact: Option<String>,
    /// Recipient lines (right block of the double header): the user who
    /// requested the document — name + role, then building name and address.
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
    // ── Page 1 : lettre ──────────────────────────────────────────────────
    /// Issuer block heading on the left of the double header, e.g. « Le
    /// syndic des copropriétaires ».
    pub issuer_heading: String,
    /// Issuer line when no syndic is recorded, e.g. « Syndic non renseigné ».
    pub issuer_not_set: String,
    /// Recipient block label on the right of the double header, e.g.
    /// « Destinataire ».
    pub recipient_label: String,
    /// Bold letter subject line, e.g. « Carnet d'entretien de l'immeuble —
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
    /// Signature block closing, e.g. « Pour le syndic, ».
    pub signer_closing: String,
    /// Signer role in small grey type, e.g. « Syndic en exercice — … ».
    pub signer_role: Option<String>,
    /// Place of issue for « Fait à {place}, le {date} » (signature) —
    /// usually the city of the copropriété.
    pub place: String,
    // ── Annexe ────────────────────────────────────────────────────────────
    /// Annex kicker rendered letter-spaced, e.g. « Annexe ».
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
    /// « Contrats d'assurance en cours — {count} » style section counters.
    pub count_one: String,
    pub count_other: String,
    pub entries_count_one: String,
    pub entries_count_other: String,
}

/// Renders the logbook as a **letter + annex** PDF using the shared template.
pub fn render_logbook(data: &LogbookPdfData) -> Result<Vec<u8>, String> {
    let (mut doc, fonts) = tpl::new_document(data.edited_at, &data.labels.doc_title)?;

    let mut layout = LinearLayout::vertical();

    // Syndic line, shared by the issuer block and the annex meta.
    let syndic_line = match (&data.identity.syndic_name, &data.syndic_contact) {
        (Some(name), Some(contact)) => format!("{name} — {contact}"),
        (Some(name), None) => name.clone(),
        (None, _) => data.labels.issuer_not_set.clone(),
    };

    // ── Page 1 : lettre ──────────────────────────────────────────────────
    let mut issuer_lines = vec![data.labels.issuer_heading.clone(), syndic_line.clone()];
    if let Some(siret) = &data.identity.syndic_siret {
        issuer_lines.push(format!("SIRET : {siret}"));
    }

    tpl::push_letter_header(
        &mut layout,
        &fonts,
        &tpl::DocumentHeader {
            issuer_lines,
            recipient: Some(tpl::DocumentRecipient {
                label: data.labels.recipient_label.clone(),
                lines: data.recipient_lines.clone(),
            }),
        },
    )?;

    // Objet + référence, salutation, corps de la lettre justifié (description
    // du document + annonce de l'annexe), formule de politesse.
    tpl::push_objet(&mut layout, &data.labels.objet, data.labels.reference.as_deref());
    tpl::push_salutation(&mut layout, &data.labels.salutation);
    for paragraph in &data.labels.letter_paragraphs {
        layout.push(tpl::justified_body(paragraph));
        layout.push(elements::Break::new(0.5));
    }
    layout.push(tpl::justified_body(&data.labels.closing_formula));

    // Signature alignée à droite (« Pour le syndic, », nom, qualité, date).
    // Sans syndic enregistré : signé « La Ruche », sans qualité.
    let signer_role = match data.identity.syndic_name {
        Some(_) => data.labels.signer_role.clone(),
        None => None,
    };
    tpl::push_conclusion(
        &mut layout,
        &tpl::DocumentConclusion {
            closing: data.labels.signer_closing.clone(),
            signer_name: data
                .identity
                .syndic_name
                .clone()
                .unwrap_or_else(|| tpl::APP_NAME.to_string()),
            signer_role,
            place: data.labels.place.clone(),
            date: data.edited_at,
            validity_period: None,
        },
    );

    // ── Saut de page : annexe ─────────────────────────────────────────────
    layout.push(elements::PageBreak::new());

    let mut annex_meta = vec![
        ("Syndic en exercice".to_string(), syndic_line),
        (
            "Édition du".to_string(),
            tpl::format_date(data.edited_at),
        ),
    ];
    if let Some(siret) = &data.identity.syndic_siret {
        annex_meta.insert(
            1,
            ("SIRET".to_string(), siret.clone()),
        );
    }
    tpl::push_annex_header(
        &mut layout,
        &tpl::DocumentAnnexHeader {
            kicker: data.labels.annex_kicker.clone(),
            title: data.labels.doc_title.clone(),
            subtitle: Some(format!(
                "{} — {}",
                data.identity.building_name, data.identity.building_address
            )),
            meta: annex_meta,
        },
    );

    // ── Active insurance contracts ─────────────────────────────────────────
    push_contract_section(
        &mut layout,
        &data.labels.insurance_section,
        &data.insurance_contracts,
        &data.labels.no_contracts,
        &data.labels.count_one,
        &data.labels.count_other,
    );

    // ── Active maintenance contracts ───────────────────────────────────────
    push_contract_section(
        &mut layout,
        &data.labels.maintenance_section,
        &data.maintenance_contracts,
        &data.labels.no_contracts,
        &data.labels.count_one,
        &data.labels.count_other,
    );

    // ── Chronology ─────────────────────────────────────────────────────────
    layout.push(tpl::section(&data.labels.chronology_section));
    layout.push(chronology_table(data));
    layout.push(elements::Break::new(1.0));

    doc.push(layout);
    tpl::finish_document(doc)
}

/// Pushes a contract section: heading with count, then one row per contract
/// (subject, vendor · type, period, amount) or an explicit empty state —
/// the absence of insurance is regulatory information in itself.
fn push_contract_section(
    layout: &mut LinearLayout,
    heading: &str,
    contracts: &[LogbookContractLine],
    empty_text: &str,
    count_one: &str,
    count_other: &str,
) {
    let count_label = if contracts.len() == 1 {
        count_one.to_string()
    } else {
        count_other.replace("{count}", &contracts.len().to_string())
    };
    layout.push(
        Paragraph::new(format!("{heading} ({count_label})")).styled(
            style::Style::new()
                .bold()
                .with_font_size(11)
                .with_color(tpl::BRAND),
        ),
    );
    layout.push(elements::Break::new(0.3));

    if contracts.is_empty() {
        layout.push(tpl::muted(empty_text));
    } else {
        for contract in contracts {
            layout.push(contract_line(contract));
        }
    }
    layout.push(elements::Break::new(0.8));
}

/// One contract as a small two-line block: subject + vendor/type on the left,
/// period + amount on the right.
fn contract_line(contract: &LogbookContractLine) -> LinearLayout {
    let vendor = match &contract.contract_type_label {
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

    let mut row = TableLayout::new(vec![3, 2]);
    let mut left = LinearLayout::vertical();
    left.push(Paragraph::new(&contract.subject).styled(style::Style::new().bold()));
    left.push(tpl::muted(&vendor));

    let mut right = LinearLayout::vertical();
    right.push(
        Paragraph::new(&period)
            .aligned(Alignment::Right)
            .styled(style::Style::new().with_color(tpl::GREY)),
    );
    if let Some(amount) = contract.amount {
        right.push(
            Paragraph::new(tpl::format_euro(amount))
                .aligned(Alignment::Right)
                .styled(style::Style::new().bold()),
        );
    }

    row.row()
        .element(left)
        .element(right)
        .push()
        .expect("two-column contract row must carry exactly two cells");

    let mut block = LinearLayout::vertical();
    block.push(row);
    block.push(hairline());
    block
}

/// Chronology table: date | intervention | vendor | category | source | cost.
/// Cells are airy (0.6 em vertical padding, no cramped columns); rows break
/// cleanly across pages.
fn chronology_table(data: &LogbookPdfData) -> TableLayout {
    let l = &data.labels;
    let mut table = TableLayout::new(vec![2, 4, 2, 2, 2, 2]);
    table.set_cell_decorator(SpacedCells);

    table
        .row()
        .element(header_cell(&l.col_date))
        .element(header_cell(&l.col_intervention))
        .element(header_cell(&l.col_vendor))
        .element(header_cell(&l.col_category))
        .element(header_cell(&l.col_source))
        .element(
            Paragraph::new(&l.col_cost)
                .aligned(Alignment::Right)
                .styled(
                    style::Style::new()
                        .bold()
                        .with_font_size(9)
                        .with_color(tpl::GREY),
                ),
        )
        .push()
        .expect("six-column header row must carry exactly six cells");

    if data.entries.is_empty() {
        table
            .row()
            .element(
                Paragraph::new(&l.no_entries).styled(style::Style::new().with_color(tpl::GREY)),
            )
            .element(Paragraph::new(""))
            .element(Paragraph::new(""))
            .element(Paragraph::new(""))
            .element(Paragraph::new(""))
            .element(Paragraph::new(""))
            .push()
            .expect("empty-state row must carry exactly six cells");
        return table;
    }

    for entry in &data.entries {
        table
            .row()
            .element(Paragraph::new(tpl::format_date(entry.intervention_date)))
            .element(intervention_cell(entry))
            .element(
                Paragraph::new(entry.vendor_name.as_deref().unwrap_or("—"))
                    .styled(style::Style::new().with_color(tpl::GREY)),
            )
            .element(Paragraph::new(&entry.category_label))
            .element(
                Paragraph::new(&entry.source_label)
                    .styled(style::Style::new().with_color(tpl::GREY)),
            )
            .element(
                Paragraph::new(tpl::optional_euro(entry.cost))
                    .aligned(Alignment::Right)
                    .styled(style::Style::new().bold()),
            )
            .push()
            .expect("six-column entry row must carry exactly six cells");
    }

    table
}

/// Intervention cell: bold title, grey description underneath.
fn intervention_cell(entry: &LogbookEntryLine) -> LinearLayout {
    let mut cell = LinearLayout::vertical();
    cell.push(Paragraph::new(&entry.title).styled(style::Style::new().bold()));
    if let Some(description) = &entry.description {
        cell.push(Paragraph::new(description).styled(style::Style::new().with_color(tpl::GREY)));
    }
    cell
}

/// Cell decorator giving the chronology table vertical breathing room
/// (0.6 em above and below each cell) — without it rows sit cramped against
/// each other.
struct SpacedCells;

impl elements::CellDecorator for SpacedCells {
    fn decorate_cell(
        &mut self,
        _column: usize,
        _row: usize,
        _has_more: bool,
        mut area: genpdf::render::Area<'_>,
        _style: style::Style,
    ) {
        area.add_margins(Margins::vh(0.6, 0.4));
    }
}

fn header_cell(text: &str) -> elements::StyledElement<Paragraph> {
    Paragraph::new(text).styled(
        style::Style::new()
            .bold()
            .with_font_size(9)
            .with_color(tpl::GREY),
    )
}

/// Thin separator between contract blocks (same vector rule as the template).
fn hairline() -> TableLayout {
    tpl::hairline()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::pdf_template::DocumentIdentity;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn labels() -> LogbookPdfLabels {
        LogbookPdfLabels {
            doc_title: "Carnet d'entretien de l'immeuble".to_string(),
            issuer_heading: "Le syndic des copropriétaires".to_string(),
            issuer_not_set: "Syndic non renseigné".to_string(),
            recipient_label: "Destinataire".to_string(),
            objet: "Carnet d'entretien de l'immeuble — édition 2026".to_string(),
            reference: Some(
                "Réf. LR-CARNET-2026 · Édition du 16/06/2026".to_string(),
            ),
            salutation: "Madame, Monsieur,".to_string(),
            letter_paragraphs: vec![
                "Conformément au décret n°2001-477, le syndic tient à jour le carnet d'entretien de l'immeuble. Vous trouverez en annexe du présent document l'édition du carnet : contrats d'assurance et de maintenance en cours, puis chronologie complète des interventions.".to_string(),
                "Cette édition est établie à sa date d'émission et annule et remplace toute édition antérieure.".to_string(),
                "Nous restons à votre disposition pour toute question relative au contenu de ce document.".to_string(),
            ],
            closing_formula: "Je vous prie d'agréer, Madame, Monsieur, l'expression de mes salutations distinguées.".to_string(),
            signer_closing: "Pour le syndic,".to_string(),
            signer_role: Some("Syndic en exercice — Résidence des Tilleuls".to_string()),
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
            count_one: "1 contrat".to_string(),
            count_other: "{count} contrats".to_string(),
            entries_count_one: "1 intervention".to_string(),
            entries_count_other: "{count} interventions".to_string(),
        }
    }

    fn identity() -> DocumentIdentity {
        DocumentIdentity {
            building_name: "Résidence des Tilleuls".to_string(),
            building_address: "12 rue des Fleurs, 75011 Paris".to_string(),
            syndic_name: Some("Cabinet Dupont".to_string()),
            syndic_siret: Some("123 456 789 00012".to_string()),
        }
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
            recipient_lines: vec![
                "Marie Dupont — Copropriétaire".to_string(),
                "Résidence des Tilleuls".to_string(),
                "12 rue des Fleurs, 75011 Paris".to_string(),
            ],
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
        // Carnet vide + aucun contrat : le document s'imprime quand même
        // (spec EP-009-S09 §6).
        let data = LogbookPdfData {
            identity: identity(),
            edited_at: date(2026, 6, 16),
            syndic_contact: None,
            recipient_lines: vec![
                "Marie Dupont — Copropriétaire".to_string(),
                "Résidence des Tilleuls".to_string(),
                "12 rue des Fleurs, 75011 Paris".to_string(),
            ],
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
        let mut data = LogbookPdfData {
            identity: DocumentIdentity {
                building_name: "Résidence des Tilleuls".to_string(),
                building_address: "12 rue des Fleurs, 75011 Paris".to_string(),
                syndic_name: None,
                syndic_siret: None,
            },
            edited_at: date(2026, 6, 16),
            syndic_contact: None,
            recipient_lines: vec![
                "Marie Dupont — Copropriétaire".to_string(),
                "Résidence des Tilleuls".to_string(),
                "12 rue des Fleurs, 75011 Paris".to_string(),
            ],
            insurance_contracts: vec![],
            maintenance_contracts: vec![],
            entries: vec![entry("Révision chaudière", 5)],
            labels: labels(),
        };
        data.identity.syndic_name = None;
        let bytes = render_logbook(&data).expect("render");
        assert!(bytes.starts_with(b"%PDF"));
    }
}


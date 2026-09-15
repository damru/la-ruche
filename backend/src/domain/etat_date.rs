//! État daté assembly (EP-002-S06, LEGAL.md §7.1).
//!
//! Pure functions only — no I/O. Data is loaded by `db::etat_date` and handed
//! in already computed; this module only shapes it into the regulatory
//! 3-part structure (mandatory since 25/12/2025, art. 5 décret 1967 v7.0).
//!
//! This assembly is shared with EP-002-S07 (pré-état daté), which reuses
//! `EtatDateInputs` and adds its own presentation on top.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Serialize;

/// J+15 legal deadline for producing the état daté after the notaire's request.
pub fn compute_due_date(requested_at: NaiveDate) -> NaiveDate {
    requested_at + chrono::Duration::days(15)
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LigneMontant {
    pub label: String,
    pub amount: Decimal,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ExerciceQuotePart {
    pub fiscal_year: i32,
    pub charge_amount: Decimal,
    pub works_fund_amount: Decimal,
    pub total_amount: Decimal,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ProcedureSummary {
    pub subject: String,
    pub proceeding_type: String,
    pub opened_at: NaiveDate,
    pub jurisdiction: Option<String>,
}

/// Data already loaded from the DB (levy calls, works fund, judicial proceedings)
/// — this struct is the single input boundary the pure `assemble` function
/// consumes, so no SQL ever leaks into `domain/`.
#[derive(Debug, Clone)]
pub struct EtatDateInputs {
    /// Provisions échues impayées par le cédant (arriérés).
    pub arrears: Decimal,
    /// Avances / provisions non exigibles dont le syndicat est débiteur envers le cédant.
    pub avances_syndicat: Decimal,
    /// Provisions à venir incombant à l'acquéreur (appels non encore émis).
    pub provisions_a_venir: Decimal,
    /// Quote-part du lot sur le fonds travaux (non remboursable au vendeur — LEGAL.md §4.4).
    pub fonds_travaux_quote_part: Decimal,
    /// Quote-part du lot sur les 2 derniers exercices (le plus récent en premier).
    pub exercices: Vec<ExerciceQuotePart>,
    /// Procédures judiciaires en cours impliquant le syndicat (EP-002-S10).
    pub procedures: Vec<ProcedureSummary>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct EtatDate {
    pub partie1_sommes_dues_par_cedant: Vec<LigneMontant>,
    pub partie2_sommes_dues_par_syndicat: Vec<LigneMontant>,
    pub partie3_sommes_incombant_acquereur: Vec<LigneMontant>,
    pub annexe_quote_part: Vec<ExerciceQuotePart>,
    pub procedures: Vec<ProcedureSummary>,
    /// Travaux votés en AG non encore réalisés. Toujours vide : le schéma ne
    /// relie pas `resolution` (votes AG) à `task` (suivi de réalisation des
    /// travaux) — aucune colonne `task.resolution_id` n'existe. Limitation
    /// actée, documentée dans docs/specs/EP-002-S06-spec.md et LEGAL.md §7.1
    /// (couverture ⚠️ partielle tant que ce lien n'existe pas), même
    /// traitement que les emprunts collectifs ci-dessous.
    pub travaux_votes_non_realises: Vec<String>,
    /// Toujours « néant — emprunts collectifs non gérés » (limitation actée §1).
    pub emprunts_collectifs: String,
}

const EMPRUNTS_COLLECTIFS_NEANT: &str = "néant — emprunts collectifs non gérés";

/// Pré-état daté (EP-002-S07, LEGAL.md §7.2, art. L721-2 CCH) — informative
/// document generated on demand, no workflow/snapshot, reusing the same
/// `EtatDateInputs` boundary as the état daté (S06).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PreEtatDate {
    /// Charges courantes du budget prévisionnel, exercice N-1. `None` si
    /// aucun appel émis sur cet exercice (« non disponible », jamais un zéro
    /// fabriqué).
    pub charges_courantes_n1: Option<Decimal>,
    pub charges_courantes_n2: Option<Decimal>,
    pub charges_hors_budget_n1: Option<Decimal>,
    pub charges_hors_budget_n2: Option<Decimal>,
    pub sommes_dues_par_cedant: Decimal,
    pub impayes_globaux_syndicat: Decimal,
    /// Toujours `None` : la dette fournisseurs n'est pas modélisée finement
    /// (limitation actée §6, rendue « non disponible » à l'affichage).
    pub dette_fournisseurs: Option<Decimal>,
    pub fonds_travaux_quote_part: Decimal,
    pub fonds_travaux_derniere_cotisation: Decimal,
    pub procedures_en_cours: usize,
}

/// Assembles the pré-état daté L721-2 synthesis. `impayes_globaux_syndicat`
/// and `fonds_travaux_derniere_cotisation` are loaded separately from
/// `EtatDateInputs` (building-wide / last-call figures, not part of the état
/// daté's per-lot boundary) — see `db::etat_date::load_pre_etat_date_extras`.
pub fn assemble_pre_etat_date(
    inputs: EtatDateInputs,
    impayes_globaux_syndicat: Decimal,
    fonds_travaux_derniere_cotisation: Decimal,
) -> PreEtatDate {
    let mut exercices = inputs.exercices.into_iter();
    let n1 = exercices.next();
    let n2 = exercices.next();

    PreEtatDate {
        charges_courantes_n1: n1.as_ref().map(|e| e.charge_amount),
        charges_courantes_n2: n2.as_ref().map(|e| e.charge_amount),
        charges_hors_budget_n1: n1.as_ref().map(|e| e.works_fund_amount),
        charges_hors_budget_n2: n2.as_ref().map(|e| e.works_fund_amount),
        sommes_dues_par_cedant: inputs.arrears,
        impayes_globaux_syndicat,
        dette_fournisseurs: None,
        fonds_travaux_quote_part: inputs.fonds_travaux_quote_part,
        fonds_travaux_derniere_cotisation,
        procedures_en_cours: inputs.procedures.len(),
    }
}

/// Assembles the regulatory 3-part état daté structure. Lines are always
/// present, even at zero (structure réglementaire fixe — cas limites §6).
pub fn assemble(inputs: EtatDateInputs) -> EtatDate {
    let partie1_sommes_dues_par_cedant = vec![LigneMontant {
        label: "Provisions échues impayées (arriérés)".to_string(),
        amount: inputs.arrears,
    }];

    let partie2_sommes_dues_par_syndicat = vec![LigneMontant {
        label: "Avances et provisions non exigibles dues par le syndicat".to_string(),
        amount: inputs.avances_syndicat,
    }];

    let partie3_sommes_incombant_acquereur = vec![
        LigneMontant {
            label: "Quote-part fonds travaux (non remboursable au vendeur)".to_string(),
            amount: inputs.fonds_travaux_quote_part,
        },
        LigneMontant {
            label: "Provisions à venir".to_string(),
            amount: inputs.provisions_a_venir,
        },
    ];

    // Annexe: at most the 2 most recent exercises — a lot with less history
    // simply shows fewer lines (cas limites §6), never a fabricated one.
    let annexe_quote_part = inputs.exercices.into_iter().take(2).collect();

    EtatDate {
        partie1_sommes_dues_par_cedant,
        partie2_sommes_dues_par_syndicat,
        partie3_sommes_incombant_acquereur,
        annexe_quote_part,
        procedures: inputs.procedures,
        travaux_votes_non_realises: Vec::new(),
        emprunts_collectifs: EMPRUNTS_COLLECTIFS_NEANT.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn compute_due_date_adds_fifteen_days() {
        assert_eq!(compute_due_date(date(2026, 1, 1)), date(2026, 1, 16));
    }

    #[test]
    fn compute_due_date_crosses_month_end() {
        assert_eq!(compute_due_date(date(2026, 1, 20)), date(2026, 2, 4));
    }

    fn base_inputs() -> EtatDateInputs {
        EtatDateInputs {
            arrears: Decimal::ZERO,
            avances_syndicat: Decimal::ZERO,
            provisions_a_venir: Decimal::ZERO,
            fonds_travaux_quote_part: Decimal::ZERO,
            exercices: vec![],
            procedures: vec![],
        }
    }

    #[test]
    fn assemble_lot_with_arrears() {
        let inputs = EtatDateInputs {
            arrears: Decimal::new(150000, 2), // 1500.00
            ..base_inputs()
        };
        let result = assemble(inputs);
        assert_eq!(
            result.partie1_sommes_dues_par_cedant[0].amount,
            Decimal::new(150000, 2)
        );
        assert_eq!(result.emprunts_collectifs, EMPRUNTS_COLLECTIFS_NEANT);
        assert!(result.travaux_votes_non_realises.is_empty());
    }

    #[test]
    fn assemble_lot_crediteur() {
        let inputs = EtatDateInputs {
            avances_syndicat: Decimal::new(20000, 2), // 200.00
            ..base_inputs()
        };
        let result = assemble(inputs);
        assert_eq!(
            result.partie2_sommes_dues_par_syndicat[0].amount,
            Decimal::new(20000, 2)
        );
        assert_eq!(
            result.partie1_sommes_dues_par_cedant[0].amount,
            Decimal::ZERO
        );
    }

    #[test]
    fn assemble_lot_vierge_has_zeroed_lines_never_omitted() {
        let result = assemble(base_inputs());
        assert_eq!(result.partie1_sommes_dues_par_cedant.len(), 1);
        assert_eq!(result.partie2_sommes_dues_par_syndicat.len(), 1);
        assert_eq!(result.partie3_sommes_incombant_acquereur.len(), 2);
        assert!(result.procedures.is_empty());
        assert!(result.annexe_quote_part.is_empty());
    }

    #[test]
    fn assemble_caps_annexe_at_two_exercises() {
        let inputs = EtatDateInputs {
            exercices: vec![
                ExerciceQuotePart {
                    fiscal_year: 2026,
                    charge_amount: Decimal::new(100000, 2),
                    works_fund_amount: Decimal::new(5000, 2),
                    total_amount: Decimal::new(105000, 2),
                },
                ExerciceQuotePart {
                    fiscal_year: 2025,
                    charge_amount: Decimal::new(90000, 2),
                    works_fund_amount: Decimal::new(4000, 2),
                    total_amount: Decimal::new(94000, 2),
                },
                ExerciceQuotePart {
                    fiscal_year: 2024,
                    charge_amount: Decimal::new(80000, 2),
                    works_fund_amount: Decimal::new(3000, 2),
                    total_amount: Decimal::new(83000, 2),
                },
            ],
            ..base_inputs()
        };
        let result = assemble(inputs);
        assert_eq!(result.annexe_quote_part.len(), 2);
        assert_eq!(result.annexe_quote_part[0].fiscal_year, 2026);
        assert_eq!(result.annexe_quote_part[1].fiscal_year, 2025);
    }

    #[test]
    fn assemble_single_exercise_available() {
        let inputs = EtatDateInputs {
            exercices: vec![ExerciceQuotePart {
                fiscal_year: 2026,
                charge_amount: Decimal::new(100000, 2),
                works_fund_amount: Decimal::new(5000, 2),
                total_amount: Decimal::new(105000, 2),
            }],
            ..base_inputs()
        };
        let result = assemble(inputs);
        assert_eq!(result.annexe_quote_part.len(), 1);
    }

    #[test]
    fn pre_etat_date_full_history_two_exercises() {
        let inputs = EtatDateInputs {
            arrears: Decimal::new(50000, 2),
            fonds_travaux_quote_part: Decimal::new(30000, 2),
            exercices: vec![
                ExerciceQuotePart {
                    fiscal_year: 2026,
                    charge_amount: Decimal::new(100000, 2),
                    works_fund_amount: Decimal::new(5000, 2),
                    total_amount: Decimal::new(105000, 2),
                },
                ExerciceQuotePart {
                    fiscal_year: 2025,
                    charge_amount: Decimal::new(90000, 2),
                    works_fund_amount: Decimal::new(4000, 2),
                    total_amount: Decimal::new(94000, 2),
                },
            ],
            procedures: vec![ProcedureSummary {
                subject: "Recouvrement".to_string(),
                proceeding_type: "debt_recovery".to_string(),
                opened_at: date(2026, 1, 1),
                jurisdiction: None,
            }],
            ..base_inputs()
        };
        let result = assemble_pre_etat_date(inputs, Decimal::new(20000, 2), Decimal::new(15000, 2));
        assert_eq!(result.charges_courantes_n1, Some(Decimal::new(100000, 2)));
        assert_eq!(result.charges_courantes_n2, Some(Decimal::new(90000, 2)));
        assert_eq!(result.charges_hors_budget_n1, Some(Decimal::new(5000, 2)));
        assert_eq!(result.charges_hors_budget_n2, Some(Decimal::new(4000, 2)));
        assert_eq!(result.sommes_dues_par_cedant, Decimal::new(50000, 2));
        assert_eq!(result.impayes_globaux_syndicat, Decimal::new(20000, 2));
        assert_eq!(result.dette_fournisseurs, None);
        assert_eq!(result.fonds_travaux_quote_part, Decimal::new(30000, 2));
        assert_eq!(
            result.fonds_travaux_derniere_cotisation,
            Decimal::new(15000, 2)
        );
        assert_eq!(result.procedures_en_cours, 1);
    }

    #[test]
    fn pre_etat_date_single_exercise_n2_non_disponible() {
        let inputs = EtatDateInputs {
            exercices: vec![ExerciceQuotePart {
                fiscal_year: 2026,
                charge_amount: Decimal::new(100000, 2),
                works_fund_amount: Decimal::new(5000, 2),
                total_amount: Decimal::new(105000, 2),
            }],
            ..base_inputs()
        };
        let result = assemble_pre_etat_date(inputs, Decimal::ZERO, Decimal::ZERO);
        assert!(result.charges_courantes_n1.is_some());
        assert_eq!(result.charges_courantes_n2, None);
        assert_eq!(result.charges_hors_budget_n2, None);
    }

    #[test]
    fn pre_etat_date_lot_vierge_all_zeros_and_dette_non_disponible() {
        let result = assemble_pre_etat_date(base_inputs(), Decimal::ZERO, Decimal::ZERO);
        assert_eq!(result.charges_courantes_n1, None);
        assert_eq!(result.charges_courantes_n2, None);
        assert_eq!(result.sommes_dues_par_cedant, Decimal::ZERO);
        assert_eq!(result.dette_fournisseurs, None);
        assert_eq!(result.procedures_en_cours, 0);
    }

    #[test]
    fn assemble_includes_ongoing_procedures() {
        let inputs = EtatDateInputs {
            procedures: vec![ProcedureSummary {
                subject: "Recouvrement".to_string(),
                proceeding_type: "debt_recovery".to_string(),
                opened_at: date(2026, 1, 1),
                jurisdiction: Some("TJ Paris".to_string()),
            }],
            ..base_inputs()
        };
        let result = assemble(inputs);
        assert_eq!(result.procedures.len(), 1);
        assert_eq!(result.procedures[0].subject, "Recouvrement");
    }
}

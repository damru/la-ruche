//! Pure comparison logic for the EDD (état descriptif de division) coherence
//! check (EP-001-S09, LEGAL §1.2). The report is purely informative: it
//! never blocks allocation key configuration or levy call generation.

use rust_decimal::Decimal;
use serde::Serialize;
use std::collections::BTreeMap;
use uuid::Uuid;

/// A reference tantième from the manually-entered EDD, for one unit/key pair.
#[derive(Debug, Clone)]
pub struct EddRefShare {
    pub unit_id: Uuid,
    pub allocation_key_id: Uuid,
    pub share_value: Decimal,
}

/// A currently configured tantième (`unit_allocation_key`) for one unit/key pair.
#[derive(Debug, Clone)]
pub struct CurrentShare {
    pub unit_id: Uuid,
    pub allocation_key_id: Uuid,
    pub share_value: Decimal,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EddDiscrepancy {
    /// Configured in the app but absent from the EDD reference.
    MissingInEdd {
        unit_id: Uuid,
        allocation_key_id: Uuid,
    },
    /// Referenced in the EDD but not (or no longer) configured in the app.
    MissingInConfig {
        unit_id: Uuid,
        allocation_key_id: Uuid,
    },
    /// Present on both sides but with a diverging value.
    ValueMismatch {
        unit_id: Uuid,
        allocation_key_id: Uuid,
        expected: Decimal,
        actual: Decimal,
    },
    /// Sum of shares diverges between the EDD reference and the config, per key.
    TotalMismatch {
        allocation_key_id: Uuid,
        expected: Decimal,
        actual: Decimal,
    },
}

#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct EddReport {
    pub discrepancies: Vec<EddDiscrepancy>,
}

impl EddReport {
    pub fn is_empty(&self) -> bool {
        self.discrepancies.is_empty()
    }
}

/// Compares EDD reference tantièmes against currently configured shares and
/// produces a non-blocking discrepancy report. Pure function — no SQL, no I/O.
pub fn compare(reference: &[EddRefShare], current: &[CurrentShare]) -> EddReport {
    let mut reference_map: BTreeMap<(Uuid, Uuid), Decimal> = BTreeMap::new();
    for r in reference {
        reference_map.insert((r.unit_id, r.allocation_key_id), r.share_value);
    }

    let mut current_map: BTreeMap<(Uuid, Uuid), Decimal> = BTreeMap::new();
    for c in current {
        current_map.insert((c.unit_id, c.allocation_key_id), c.share_value);
    }

    let mut discrepancies = Vec::new();

    for (&(unit_id, allocation_key_id), &actual) in &current_map {
        match reference_map.get(&(unit_id, allocation_key_id)) {
            None => discrepancies.push(EddDiscrepancy::MissingInEdd {
                unit_id,
                allocation_key_id,
            }),
            Some(&expected) if expected != actual => {
                discrepancies.push(EddDiscrepancy::ValueMismatch {
                    unit_id,
                    allocation_key_id,
                    expected,
                    actual,
                })
            }
            Some(_) => {}
        }
    }

    for &(unit_id, allocation_key_id) in reference_map.keys() {
        if !current_map.contains_key(&(unit_id, allocation_key_id)) {
            discrepancies.push(EddDiscrepancy::MissingInConfig {
                unit_id,
                allocation_key_id,
            });
        }
    }

    // Per-key totals
    let mut reference_totals: BTreeMap<Uuid, Decimal> = BTreeMap::new();
    for r in reference {
        *reference_totals.entry(r.allocation_key_id).or_default() += r.share_value;
    }
    let mut current_totals: BTreeMap<Uuid, Decimal> = BTreeMap::new();
    for c in current {
        *current_totals.entry(c.allocation_key_id).or_default() += c.share_value;
    }

    let mut all_keys: Vec<Uuid> = reference_totals
        .keys()
        .chain(current_totals.keys())
        .copied()
        .collect();
    all_keys.sort();
    all_keys.dedup();

    for allocation_key_id in all_keys {
        let expected = *reference_totals
            .get(&allocation_key_id)
            .unwrap_or(&Decimal::ZERO);
        let actual = *current_totals
            .get(&allocation_key_id)
            .unwrap_or(&Decimal::ZERO);
        if expected != actual {
            discrepancies.push(EddDiscrepancy::TotalMismatch {
                allocation_key_id,
                expected,
                actual,
            });
        }
    }

    EddReport { discrepancies }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn share(unit: Uuid, key: Uuid, value: i64) -> (EddRefShare, CurrentShare) {
        (
            EddRefShare {
                unit_id: unit,
                allocation_key_id: key,
                share_value: Decimal::from(value),
            },
            CurrentShare {
                unit_id: unit,
                allocation_key_id: key,
                share_value: Decimal::from(value),
            },
        )
    }

    #[test]
    fn identical_reference_and_config_yields_empty_report() {
        let unit = Uuid::new_v4();
        let key = Uuid::new_v4();
        let (r, c) = share(unit, key, 100);
        let report = compare(&[r], &[c]);
        assert!(report.is_empty());
    }

    #[test]
    fn diverging_value_is_reported() {
        let unit = Uuid::new_v4();
        let key = Uuid::new_v4();
        let reference = vec![EddRefShare {
            unit_id: unit,
            allocation_key_id: key,
            share_value: Decimal::from(100),
        }];
        let current = vec![CurrentShare {
            unit_id: unit,
            allocation_key_id: key,
            share_value: Decimal::from(90),
        }];
        let report = compare(&reference, &current);
        assert!(
            report
                .discrepancies
                .contains(&EddDiscrepancy::ValueMismatch {
                    unit_id: unit,
                    allocation_key_id: key,
                    expected: Decimal::from(100),
                    actual: Decimal::from(90),
                })
        );
    }

    #[test]
    fn unit_missing_from_edd_reference_is_reported() {
        let unit = Uuid::new_v4();
        let key = Uuid::new_v4();
        let current = vec![CurrentShare {
            unit_id: unit,
            allocation_key_id: key,
            share_value: Decimal::from(50),
        }];
        let report = compare(&[], &current);
        assert!(
            report
                .discrepancies
                .contains(&EddDiscrepancy::MissingInEdd {
                    unit_id: unit,
                    allocation_key_id: key,
                })
        );
    }

    #[test]
    fn unit_missing_from_config_is_reported() {
        let unit = Uuid::new_v4();
        let key = Uuid::new_v4();
        let reference = vec![EddRefShare {
            unit_id: unit,
            allocation_key_id: key,
            share_value: Decimal::from(50),
        }];
        let report = compare(&reference, &[]);
        assert!(
            report
                .discrepancies
                .contains(&EddDiscrepancy::MissingInConfig {
                    unit_id: unit,
                    allocation_key_id: key,
                })
        );
    }

    #[test]
    fn total_mismatch_is_reported_per_key() {
        let key = Uuid::new_v4();
        let unit_a = Uuid::new_v4();
        let unit_b = Uuid::new_v4();
        let reference = vec![
            EddRefShare {
                unit_id: unit_a,
                allocation_key_id: key,
                share_value: Decimal::from(60),
            },
            EddRefShare {
                unit_id: unit_b,
                allocation_key_id: key,
                share_value: Decimal::from(40),
            },
        ];
        let current = vec![
            CurrentShare {
                unit_id: unit_a,
                allocation_key_id: key,
                share_value: Decimal::from(60),
            },
            CurrentShare {
                unit_id: unit_b,
                allocation_key_id: key,
                share_value: Decimal::from(30),
            },
        ];
        let report = compare(&reference, &current);
        assert!(report.discrepancies.iter().any(|d| matches!(
            d,
            EddDiscrepancy::TotalMismatch { allocation_key_id, expected, actual }
                if *allocation_key_id == key && *expected == Decimal::from(100) && *actual == Decimal::from(90)
        )));
    }

    #[test]
    fn decimal_precision_is_preserved_in_mismatch() {
        let unit = Uuid::new_v4();
        let key = Uuid::new_v4();
        let reference = vec![EddRefShare {
            unit_id: unit,
            allocation_key_id: key,
            share_value: Decimal::new(1005, 2), // 10.05
        }];
        let current = vec![CurrentShare {
            unit_id: unit,
            allocation_key_id: key,
            share_value: Decimal::new(1050, 2), // 10.50
        }];
        let report = compare(&reference, &current);
        assert!(
            report
                .discrepancies
                .contains(&EddDiscrepancy::ValueMismatch {
                    unit_id: unit,
                    allocation_key_id: key,
                    expected: Decimal::new(1005, 2),
                    actual: Decimal::new(1050, 2),
                })
        );
    }
}

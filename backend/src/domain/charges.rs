/// Levy call calculation by shares.
/// Pure logic — no I/O.
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UnitWithShares {
    pub unit_id: Uuid,
    pub key_id: Uuid,
    pub shares: i64,
}

#[derive(Debug, Clone)]
pub struct ComputedLevy {
    pub unit_id: Uuid,
    pub key_id: Uuid,
    /// Snapshot of the unit's share_value at generation time (ADR-006 §2.3)
    pub shares_snapshot: i64,
    /// Snapshot of the key's total shares at generation time (ADR-006 §2.3)
    pub shares_total_snapshot: i64,
    pub amount: Decimal,
}

pub fn compute_levies(
    total_amount: Decimal,
    units: &[UnitWithShares],
    key_total_shares: i64,
) -> Vec<ComputedLevy> {
    if units.is_empty() || key_total_shares == 0 {
        return vec![];
    }

    let cents = Decimal::new(100, 0);
    let mut levies: Vec<ComputedLevy> = units
        .iter()
        .map(|unit| {
            let pro_rata =
                total_amount * Decimal::from(unit.shares) / Decimal::from(key_total_shares);
            // Truncate to 2 decimal places (no banker's rounding)
            let amount = (pro_rata * cents).floor() / cents;
            ComputedLevy {
                unit_id: unit.unit_id,
                key_id: unit.key_id,
                shares_snapshot: unit.shares,
                shares_total_snapshot: key_total_shares,
                amount,
            }
        })
        .collect();

    // Apply rounding gap to the unit with the largest share
    let computed_total: Decimal = levies.iter().map(|l| l.amount).sum();
    let gap = total_amount - computed_total;

    if gap != Decimal::ZERO
        && let Some(idx) = units
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.shares.cmp(&b.shares))
            .map(|(i, _)| i)
    {
        levies[idx].amount += gap;
    }

    levies
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_exact() {
        let units = vec![
            UnitWithShares {
                unit_id: Uuid::new_v4(),
                key_id: Uuid::new_v4(),
                shares: 300,
            },
            UnitWithShares {
                unit_id: Uuid::new_v4(),
                key_id: Uuid::new_v4(),
                shares: 450,
            },
            UnitWithShares {
                unit_id: Uuid::new_v4(),
                key_id: Uuid::new_v4(),
                shares: 250,
            },
        ];
        let amount = Decimal::new(120000, 2); // 1200.00
        let total: i64 = 1000;

        let levies = compute_levies(amount, &units, total);
        let sum: Decimal = levies.iter().map(|l| l.amount).sum();

        assert_eq!(
            sum, amount,
            "Sum of levies must equal the total amount exactly"
        );
    }
}

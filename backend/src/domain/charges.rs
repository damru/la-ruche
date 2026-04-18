/// Levy call calculation by shares.
/// Pure logic — no I/O.

use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UnitWithShares {
    pub unit_id: Uuid,
    pub shares: Decimal,
}

#[derive(Debug, Clone)]
pub struct ComputedLevy {
    pub unit_id: Uuid,
    pub amount: Decimal,
}

/// Computes each unit's share of a total amount according to an allocation key
/// (unit shares / total key shares).
///
/// Calculation uses Decimal to avoid rounding errors.
/// Final rounding is to 2 decimal places (cents).
/// Any rounding gap is assigned to the unit with the largest share.
pub fn compute_levies(
    total_amount: Decimal,
    units: &[UnitWithShares],
    key_total_shares: Decimal,
) -> Vec<ComputedLevy> {
    if units.is_empty() || key_total_shares == Decimal::ZERO {
        return vec![];
    }

    let cents = Decimal::new(100, 0);
    let mut levies: Vec<ComputedLevy> = units
        .iter()
        .map(|unit| {
            let pro_rata = total_amount * unit.shares / key_total_shares;
            // Truncate to 2 decimal places (no banker's rounding)
            let amount = (pro_rata * cents).floor() / cents;
            ComputedLevy { unit_id: unit.unit_id, amount }
        })
        .collect();

    // Apply rounding gap to the unit with the largest share
    let computed_total: Decimal = levies.iter().map(|l| l.amount).sum();
    let gap = total_amount - computed_total;

    if gap != Decimal::ZERO {
        if let Some(idx) = units
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.shares.cmp(&b.shares))
            .map(|(i, _)| i)
        {
            levies[idx].amount += gap;
        }
    }

    levies
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_exact() {
        let units = vec![
            UnitWithShares { unit_id: Uuid::new_v4(), shares: Decimal::new(300, 0) },
            UnitWithShares { unit_id: Uuid::new_v4(), shares: Decimal::new(450, 0) },
            UnitWithShares { unit_id: Uuid::new_v4(), shares: Decimal::new(250, 0) },
        ];
        let amount = Decimal::new(120000, 2); // 1200.00
        let total = Decimal::new(1000, 0);

        let levies = compute_levies(amount, &units, total);
        let sum: Decimal = levies.iter().map(|l| l.amount).sum();

        assert_eq!(sum, amount, "Sum of levies must equal the total amount exactly");
    }
}

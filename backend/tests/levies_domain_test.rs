// Domain unit tests — levy computation (charge répartition).
// These tests exercise pure domain logic and do NOT require a database.
// Run with: cargo test --test levies_domain_test

use laruche::domain::charges::{UnitWithShares, compute_levies};
use rust_decimal::Decimal;

#[test]
fn test_levies_sum_matches_total_amount() {
    // 3 units with uneven shares — the rounding algorithm must produce a
    // sum that exactly equals the original amount (no penny lost).
    let key_id = uuid::Uuid::new_v4();
    let units = vec![
        UnitWithShares {
            unit_id: uuid::Uuid::new_v4(),
            key_id,
            shares: 333,
        },
        UnitWithShares {
            unit_id: uuid::Uuid::new_v4(),
            key_id,
            shares: 333,
        },
        UnitWithShares {
            unit_id: uuid::Uuid::new_v4(),
            key_id,
            shares: 334,
        },
    ];
    let amount = Decimal::new(120000, 2); // 1 200.00
    let total_shares = 1000;

    let levies = compute_levies(amount, &units, total_shares);
    let sum: Decimal = levies.iter().map(|levy| levy.amount).sum();
    assert_eq!(
        sum, amount,
        "The sum of computed levies must match the total amount exactly"
    );
}

#[test]
fn test_levies_with_empty_units_returns_empty() {
    let levies = compute_levies(Decimal::new(1000, 0), &[], 1000);
    assert!(levies.is_empty());
}

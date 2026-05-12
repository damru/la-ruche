// Pure domain unit tests — General Assembly (AG) legal cases.
// Tests cover all majority rules of the French condominium act (loi du 10 juillet 1965):
//   Art. 24 — simple majority of votes expressed
//   Art. 25 — absolute majority of all co-owners
//   Art. 25-1 — second-vote passerelle when Art. 25 fails above 1/3
//   Art. 26 — double majority (2/3 shares + 2/3 owners)
//   Unanimity
//
// No database required — exercises VoteTally::compute() directly.
// Run with: cargo test --test assembly_test
//
// Canonical share distribution: A = 500 shares, B = 300 shares, C = 200 shares → total = 1000.

use laruche::domain::vote::{SecondVoteType, VoteRule, VoteTally};

/// Build a VoteTally with the canonical 3-unit distribution
/// (A=500, B=300, C=200 → shares_total=1000, 3 co-owners all present).
fn tally(for_votes: i32, against_votes: i32, abstentions: i32, favorable_owners: u32) -> VoteTally {
    VoteTally {
        shares_total: 1000,
        shares_present: for_votes + against_votes + abstentions,
        total_owners: 3,
        present_owners: 3,
        favorable_owners,
        for_votes,
        against_votes,
        abstentions,
    }
}

// ── Art. 24 — simple_majority ─────────────────────────────────────────────────

#[test]
fn art24_passes_with_canonical_distribution() {
    // A(500) in_favor vs B(300) against, C(200) abstain
    // shares_counted = 800, threshold = 401 → 500 >= 401 → passes
    let result = tally(500, 300, 200, 1).compute(&VoteRule::SimpleMajority);
    assert_eq!(result.for_votes, 500, "Art.24: for_votes");
    assert_eq!(result.against_votes, 300, "Art.24: against_votes");
    assert_eq!(result.abstentions, 200, "Art.24: abstentions");
    assert_eq!(result.shares_counted, 800, "Art.24: shares_counted");
    assert_eq!(result.threshold, 401, "Art.24: threshold = 800/2+1");
    assert!(result.passed, "Art.24: 500 >= 401 must pass");
    assert_eq!(
        result.eligible_for_second_vote, None,
        "Art.24: no second-vote passerelle"
    );
}

#[test]
fn art24_fails_minority() {
    // B(300) in_favor vs A(500) against, C(200) abstain → threshold=401 → fails
    let result = tally(300, 500, 200, 1).compute(&VoteRule::SimpleMajority);
    assert!(!result.passed, "Art.24: minority must fail");
    assert_eq!(
        result.eligible_for_second_vote, None,
        "Art.24: never eligible for second vote"
    );
}

// ── Art. 25 — absolute_majority ───────────────────────────────────────────────

#[test]
fn art25_passes_with_canonical_distribution() {
    // A+B(800) in_favor / 1000 total → threshold=501 → passes
    let result = tally(800, 200, 0, 2).compute(&VoteRule::AbsoluteMajority);
    assert_eq!(result.for_votes, 800, "Art.25: for_votes");
    assert_eq!(result.shares_total, 1000, "Art.25: shares_total");
    assert_eq!(result.threshold, 501, "Art.25: threshold = 1000/2+1");
    assert!(result.passed, "Art.25: 800 >= 501 must pass");
    assert_eq!(result.eligible_for_second_vote, None);
}

#[test]
fn art25_not_eligible_for_passerelle_when_at_or_below_one_third() {
    // C(200) in_favor / 1000: 200*3=600 ≤ 1000 → no Art.25-1 passerelle
    let result = tally(200, 800, 0, 1).compute(&VoteRule::AbsoluteMajority);
    assert_eq!(result.for_votes, 200);
    assert!(!result.passed);
    assert_eq!(
        result.eligible_for_second_vote, None,
        "200*3=600 ≤ 1000 → not eligible for Art.25-1"
    );
}

#[test]
fn art25_1_eligible_when_rejected_above_one_third() {
    // A(500) in_favor / 1000: 500 < 501 (fails), but 500*3=1500 > 1000 → passerelle
    let result = tally(500, 500, 0, 1).compute(&VoteRule::AbsoluteMajority);
    assert_eq!(result.for_votes, 500);
    assert!(!result.passed, "500 < 501 → fails Art.25");
    assert_eq!(
        result.eligible_for_second_vote,
        Some(SecondVoteType::Art25_1),
        "500 > 333 (1/3 of 1000) → Art.25-1 eligible"
    );
}

// ── Art. 26 — double_majority ─────────────────────────────────────────────────

#[test]
fn art26_passes_when_all_vote_in_favor() {
    // A+B+C(1000) in_favor: shares threshold (667) met; 3/3 owners in favor → passes
    let result = tally(1000, 0, 0, 3).compute(&VoteRule::DoubleMajority);
    assert_eq!(result.for_votes, 1000);
    assert_eq!(result.threshold, 667, "Art.26: (1000*2+2)/3 = 667");
    assert!(result.passed, "1000 ≥ 667 → Art.26 passes");
    assert_eq!(result.eligible_for_second_vote, None);
}

#[test]
fn art26_not_eligible_for_passerelle_when_at_or_below_half() {
    // C(200) in_favor / 1000: 200*2=400 ≤ 1000 → no Art.26-1 passerelle
    let result = tally(200, 800, 0, 1).compute(&VoteRule::DoubleMajority);
    assert_eq!(result.for_votes, 200);
    assert!(!result.passed);
    assert_eq!(
        result.eligible_for_second_vote, None,
        "200*2=400 ≤ 1000 → not eligible for Art.26-1"
    );
}

// ── Unanimity ─────────────────────────────────────────────────────────────────

#[test]
fn unanimity_passes_when_all_in_favor() {
    // A+B+C(1000) all in_favor → passes
    let result = tally(1000, 0, 0, 3).compute(&VoteRule::Unanimity);
    assert_eq!(result.for_votes, 1000);
    assert_eq!(result.against_votes, 0);
    assert_eq!(result.abstentions, 0);
    assert!(result.passed, "1000/1000 in favour → unanimity");
    assert_eq!(result.eligible_for_second_vote, None);
}

#[test]
fn unanimity_fails_when_any_abstention() {
    // A+B in_favor, C(200) abstain → not unanimous
    let result = tally(800, 0, 200, 2).compute(&VoteRule::Unanimity);
    assert!(!result.passed, "abstention breaks unanimity");
    assert_eq!(result.eligible_for_second_vote, None);
}

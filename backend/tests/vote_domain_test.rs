// Domain unit tests — vote majority rules (Art. 24, 25, 26, unanimity).
// These tests exercise pure domain logic and do NOT require a database.
// Run with: cargo test --test vote_domain_test

use laruche::domain::vote::{VoteRule, VoteTally};

// ── Art. 24 — simple majority ─────────────────────────────────────────────────

#[test]
fn test_simple_majority_vote_passes() {
    let tally = VoteTally {
        shares_total: 1000,
        shares_present: 600,
        total_owners: 10,
        present_owners: 7,
        favorable_owners: 4,
        for_votes: 400,
        against_votes: 200,
        abstentions: 0,
    };
    let result = tally.compute(&VoteRule::SimpleMajority);
    assert!(
        result.passed,
        "400 > 300 should pass article 24 style majority"
    );
}

#[test]
fn test_simple_majority_fails_on_exact_tie() {
    // shares_counted = 600 (300+300), threshold = 301 → 300 < 301 → fails
    let tally = VoteTally {
        shares_total: 1000,
        shares_present: 600,
        total_owners: 10,
        present_owners: 6,
        favorable_owners: 3,
        for_votes: 300,
        against_votes: 300,
        abstentions: 0,
    };
    let result = tally.compute(&VoteRule::SimpleMajority);
    assert!(!result.passed, "exact tie (300 vs 300) must fail");
    assert_eq!(result.shares_counted, 600);
    assert_eq!(result.threshold, 301);
}

// ── Art. 25 — absolute majority ───────────────────────────────────────────────

#[test]
fn test_absolute_majority_vote_fails_when_below_threshold() {
    let tally = VoteTally {
        shares_total: 1000,
        shares_present: 800,
        total_owners: 10,
        present_owners: 8,
        favorable_owners: 4,
        for_votes: 490,
        against_votes: 310,
        abstentions: 0,
    };
    let result = tally.compute(&VoteRule::AbsoluteMajority);
    assert!(!result.passed, "490 < 501 should fail absolute majority");
}

#[test]
fn test_absolute_majority_passes_at_exact_threshold() {
    // threshold = 1000/2+1 = 501; for_votes = 501 → passes
    let tally = VoteTally {
        shares_total: 1000,
        shares_present: 800,
        total_owners: 10,
        present_owners: 8,
        favorable_owners: 5,
        for_votes: 501,
        against_votes: 299,
        abstentions: 0,
    };
    let result = tally.compute(&VoteRule::AbsoluteMajority);
    assert!(result.passed, "501 >= 501 must pass at exact threshold");
    assert_eq!(result.threshold, 501);
}

#[test]
fn test_absolute_majority_fails_just_below_threshold_eligible_for_second_vote() {
    // for=500 < 501 fails, but 500*3=1500 > 1000 → Art.25-1 eligible
    let tally = VoteTally {
        shares_total: 1000,
        shares_present: 800,
        total_owners: 10,
        present_owners: 8,
        favorable_owners: 5,
        for_votes: 500,
        against_votes: 300,
        abstentions: 0,
    };
    use laruche::domain::vote::SecondVoteType;
    let result = tally.compute(&VoteRule::AbsoluteMajority);
    assert!(!result.passed, "500 < 501 → fails Art.25");
    assert_eq!(
        result.eligible_for_second_vote,
        Some(SecondVoteType::Art25_1),
        "500 > 333 (1/3 of 1000) → Art.25-1 eligible"
    );
}

// ── Art. 26 — double majority ─────────────────────────────────────────────────

#[test]
fn test_double_majority_vote_requires_two_thirds() {
    let tally = VoteTally {
        shares_total: 1000,
        shares_present: 900,
        total_owners: 10,
        present_owners: 9,
        favorable_owners: 6,
        for_votes: 650,
        against_votes: 250,
        abstentions: 0,
    };
    let result = tally.compute(&VoteRule::DoubleMajority);
    assert!(
        !result.passed,
        "650 < 667 should fail the two-thirds threshold"
    );
    assert_eq!(result.threshold, 667, "(1000*2+2)/3 = 667");
}

#[test]
fn test_double_majority_fails_when_owners_condition_not_met_eligible_for_second_vote() {
    // Shares: 700/1000 ≥ 667 (2/3 threshold) → shares met.
    // Owners: 5 in favour out of 9 total → 5*3=15 < 9*2=18 → owners NOT met.
    // for_votes*2 = 1400 > 1000 → Art.26-1 eligible.
    use laruche::domain::vote::SecondVoteType;
    let tally = VoteTally {
        shares_total: 1000,
        shares_present: 900,
        total_owners: 9,
        present_owners: 8,
        favorable_owners: 5,
        for_votes: 700,
        against_votes: 200,
        abstentions: 0,
    };
    let result = tally.compute(&VoteRule::DoubleMajority);
    assert!(!result.passed, "owners condition not met → rejected");
    assert_eq!(
        result.eligible_for_second_vote,
        Some(SecondVoteType::Art26_1),
        "700 > 500 (half of 1000) → eligible for Art.26-1"
    );
}

#[test]
fn test_double_majority_fails_when_shares_not_met_below_half_not_eligible_for_second_vote() {
    // for=490 < 500 (half) → NOT eligible for Art.26-1
    let tally = VoteTally {
        shares_total: 1000,
        shares_present: 900,
        total_owners: 9,
        present_owners: 8,
        favorable_owners: 7,
        for_votes: 490,
        against_votes: 410,
        abstentions: 0,
    };
    let result = tally.compute(&VoteRule::DoubleMajority);
    assert!(!result.passed);
    assert_eq!(result.eligible_for_second_vote, None);
}

// ── Unanimity ─────────────────────────────────────────────────────────────────

#[test]
fn test_unanimity_passes_when_all_shares_vote_in_favor() {
    let tally = VoteTally {
        shares_total: 1000,
        shares_present: 1000,
        total_owners: 10,
        present_owners: 10,
        favorable_owners: 10,
        for_votes: 1000,
        against_votes: 0,
        abstentions: 0,
    };
    let result = tally.compute(&VoteRule::Unanimity);
    assert!(result.passed, "1000/1000 in favour → unanimity");
    assert_eq!(result.threshold, 1000);
    assert_eq!(result.eligible_for_second_vote, None);
}

#[test]
fn test_unanimity_fails_when_one_share_against() {
    let tally = VoteTally {
        shares_total: 1000,
        shares_present: 1000,
        total_owners: 10,
        present_owners: 10,
        favorable_owners: 9,
        for_votes: 999,
        against_votes: 1,
        abstentions: 0,
    };
    let result = tally.compute(&VoteRule::Unanimity);
    assert!(!result.passed, "any opposition breaks unanimity");
    assert_eq!(result.eligible_for_second_vote, None);
}

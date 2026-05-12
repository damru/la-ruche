/// Vote tally and majority calculations for general meeting resolutions.
/// Pure logic — no I/O, fully testable without a DB.
///
/// Legal references:
///   - Art. 24, Loi 10/07/1965: majority of votes present/represented
///   - Art. 25, Loi 10/07/1965: absolute majority of all co-owners
///   - Art. 25-1, Loi 10/07/1965: when art. 25 fails but > 1/3 votes in favour,
///     assembly may immediately re-vote under art. 24
///   - Art. 26, Loi 10/07/1965: double majority (2/3 votes + majority of members)
///   - Art. 26-1, Loi 10/07/1965: when art. 26 fails but > 1/2 votes in favour,
///     assembly may immediately re-vote under art. 25
///   - Art. 26 al. 4: unanimity
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoteRule {
    SimpleMajority,
    AbsoluteMajority,
    DoubleMajority,
    Unanimity,
}

/// Indicates which passerelle article allows an immediate second vote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SecondVoteType {
    /// Art. 25-1: art. 25 rejected but > 1/3 of total shares in favour → re-vote under art. 24
    #[serde(rename = "art_25_1")]
    Art25_1,
    /// Art. 26-1: art. 26 rejected but > 1/2 of total shares in favour → re-vote under art. 25
    #[serde(rename = "art_26_1")]
    Art26_1,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VoteTally {
    /// Total shares of the building
    pub shares_total: i32,
    /// Shares of co-owners present or represented
    pub shares_present: i32,
    /// Total number of co-owners
    pub total_owners: u32,
    /// Number of co-owners present or represented
    pub present_owners: u32,
    /// Number of co-owners voting in favour
    pub favorable_owners: u32,
    /// Shares voting in favour
    pub for_votes: i32,
    /// Shares voting against
    pub against_votes: i32,
    /// Abstentions
    pub abstentions: i32,
}

#[derive(Debug, Serialize)]
pub struct VoteResult {
    pub passed: bool,
    pub for_votes: i32,
    pub against_votes: i32,
    pub abstentions: i32,
    pub shares_present: i32,
    pub shares_total: i32,
    pub shares_counted: i32,
    pub threshold: i32,
    pub eligible_for_second_vote: Option<SecondVoteType>,
}

impl VoteTally {
    /// Compute the result according to the required majority rule
    pub fn compute(&self, rule: &VoteRule) -> VoteResult {
        match rule {
            VoteRule::SimpleMajority => self.simple_majority(),
            VoteRule::AbsoluteMajority => self.absolute_majority(),
            VoteRule::DoubleMajority => self.double_majority(),
            VoteRule::Unanimity => self.unanimity(),
        }
    }

    /// Art. 24 — majority of votes of those present/represented (abstentions don't count)
    fn simple_majority(&self) -> VoteResult {
        let shares_counted = self.for_votes + self.against_votes;
        let threshold = shares_counted / 2 + 1;
        let passed = self.for_votes >= threshold;
        VoteResult {
            passed,
            for_votes: self.for_votes,
            against_votes: self.against_votes,
            abstentions: self.abstentions,
            shares_present: self.shares_present,
            shares_total: self.shares_total,
            shares_counted,
            threshold,
            eligible_for_second_vote: None,
        }
    }

    /// Art. 25 — absolute majority of ALL co-owners.
    /// Art. 25-1: if rejected but for_votes > 1/3 of total shares,
    /// the assembly may immediately re-vote under art. 24.
    fn absolute_majority(&self) -> VoteResult {
        // strictly more than half of total shares
        let threshold = self.shares_total / 2 + 1;
        let passed = self.for_votes >= threshold;
        // art. 25-1: strictly more than one third of total shares
        let eligible_for_second_vote = if !passed && self.for_votes * 3 > self.shares_total {
            Some(SecondVoteType::Art25_1)
        } else {
            None
        };
        VoteResult {
            passed,
            for_votes: self.for_votes,
            against_votes: self.against_votes,
            abstentions: self.abstentions,
            shares_present: self.shares_present,
            shares_total: self.shares_total,
            shares_counted: self.shares_total,
            threshold,
            eligible_for_second_vote,
        }
    }

    /// Art. 26 — double majority:
    ///   2/3 of shares of ALL co-owners + 2/3 of co-owners in favour
    /// Art. 26-1: if rejected but for_votes > 1/2 of total shares,
    ///   the assembly may immediately re-vote under art. 25.
    fn double_majority(&self) -> VoteResult {
        let threshold = (self.shares_total * 2 + 2) / 3;
        let shares_threshold_met = self.for_votes >= threshold;
        let owners_threshold_met = self.favorable_owners * 3 >= self.total_owners * 2;
        let passed = shares_threshold_met && owners_threshold_met;
        // art. 26-1: strictly more than half of total shares in favour
        let eligible_for_second_vote = if !passed && self.for_votes * 2 > self.shares_total {
            Some(SecondVoteType::Art26_1)
        } else {
            None
        };
        VoteResult {
            passed,
            for_votes: self.for_votes,
            against_votes: self.against_votes,
            abstentions: self.abstentions,
            shares_present: self.shares_present,
            shares_total: self.shares_total,
            shares_counted: self.shares_total,
            threshold,
            eligible_for_second_vote,
        }
    }

    /// Unanimity — 100 % of shares
    fn unanimity(&self) -> VoteResult {
        let passed =
            self.against_votes == 0 && self.abstentions == 0 && self.for_votes == self.shares_total;
        VoteResult {
            passed,
            for_votes: self.for_votes,
            against_votes: self.against_votes,
            abstentions: self.abstentions,
            shares_present: self.shares_present,
            shares_total: self.shares_total,
            shares_counted: self.shares_total,
            threshold: self.shares_total,
            eligible_for_second_vote: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tally(total: i32, present: i32, for_v: i32, against: i32) -> VoteTally {
        VoteTally {
            shares_total: total,
            shares_present: present,
            total_owners: 10,
            present_owners: 7,
            favorable_owners: 4,
            for_votes: for_v,
            against_votes: against,
            abstentions: present - for_v - against,
        }
    }

    #[test]
    fn art24_passed() {
        let d = tally(1000, 600, 350, 250);
        assert!(d.compute(&VoteRule::SimpleMajority).passed);
    }

    #[test]
    fn art24_rejected() {
        let d = tally(1000, 600, 299, 301);
        assert!(!d.compute(&VoteRule::SimpleMajority).passed);
    }

    #[test]
    fn art24_reports_counted_shares_separately_from_total_shares() {
        let d = tally(1000, 600, 250, 200);
        let result = d.compute(&VoteRule::SimpleMajority);
        assert_eq!(result.shares_total, 1000);
        assert_eq!(result.shares_counted, 450);
        assert_eq!(result.threshold, 226);
        assert_eq!(result.abstentions, 150);
    }

    #[test]
    fn art25_requires_absolute_majority() {
        // 400/1000 < 500 → rejected even if majority of present
        let d = tally(1000, 700, 400, 300);
        let result = d.compute(&VoteRule::AbsoluteMajority);
        assert!(!result.passed);
        assert_eq!(result.threshold, 501);
    }

    #[test]
    fn art25_1_eligible_when_rejected_above_one_third() {
        // 340/1000 > 333 → eligible for second vote under art. 24
        let d = tally(1000, 700, 340, 360);
        let result = d.compute(&VoteRule::AbsoluteMajority);
        assert!(!result.passed);
        assert_eq!(
            result.eligible_for_second_vote,
            Some(SecondVoteType::Art25_1)
        );
    }

    #[test]
    fn art25_1_not_eligible_when_rejected_at_or_below_one_third() {
        // 333/1000 = exactly 1/3 → not strictly more than 1/3, no second vote
        let d = tally(1000, 700, 333, 367);
        let result = d.compute(&VoteRule::AbsoluteMajority);
        assert!(!result.passed);
        assert_eq!(result.eligible_for_second_vote, None);
    }

    #[test]
    fn art25_1_not_eligible_when_passed() {
        // 501/1000 → passed, no second vote flag
        let d = tally(1000, 700, 501, 199);
        let result = d.compute(&VoteRule::AbsoluteMajority);
        assert!(result.passed);
        assert_eq!(result.eligible_for_second_vote, None);
    }

    #[test]
    fn art24_never_eligible_for_second_vote() {
        let d = tally(1000, 600, 299, 301);
        let result = d.compute(&VoteRule::SimpleMajority);
        assert!(!result.passed);
        assert_eq!(result.eligible_for_second_vote, None);
    }

    #[test]
    fn art26_1_eligible_when_rejected_above_half() {
        // for=5100/9000 > 4500 (50%) but < 6001 (2/3) → rejected at art. 26, eligible for art. 26-1
        let d = VoteTally {
            shares_total: 9000,
            shares_present: 8000,
            total_owners: 10,
            present_owners: 9,
            favorable_owners: 5,
            for_votes: 5100,
            against_votes: 2900,
            abstentions: 0,
        };
        let result = d.compute(&VoteRule::DoubleMajority);
        assert!(!result.passed);
        assert_eq!(
            result.eligible_for_second_vote,
            Some(SecondVoteType::Art26_1)
        );
    }

    #[test]
    fn art26_1_not_eligible_when_rejected_at_or_below_half() {
        // for=4500/9000 = exactly 1/2 → not strictly more than 1/2, no second vote
        let d = VoteTally {
            shares_total: 9000,
            shares_present: 8000,
            total_owners: 10,
            present_owners: 9,
            favorable_owners: 4,
            for_votes: 4500,
            against_votes: 3500,
            abstentions: 0,
        };
        let result = d.compute(&VoteRule::DoubleMajority);
        assert!(!result.passed);
        assert_eq!(result.eligible_for_second_vote, None);
    }

    #[test]
    fn art26_requires_two_thirds_of_shares_and_owners() {
        let d = VoteTally {
            shares_total: 1000,
            shares_present: 900,
            total_owners: 9,
            present_owners: 8,
            favorable_owners: 5,
            for_votes: 700,
            against_votes: 200,
            abstentions: 0,
        };

        assert!(!d.compute(&VoteRule::DoubleMajority).passed);
    }

    #[test]
    fn art26_passes_when_two_thirds_of_shares_and_owners_are_in_favor() {
        let d = VoteTally {
            shares_total: 1000,
            shares_present: 900,
            total_owners: 9,
            present_owners: 8,
            favorable_owners: 6,
            for_votes: 700,
            against_votes: 200,
            abstentions: 0,
        };

        let result = d.compute(&VoteRule::DoubleMajority);
        assert!(result.passed);
        assert_eq!(result.threshold, 667);
    }

    #[test]
    fn unanimity_strict() {
        let d = tally(1000, 1000, 999, 0);
        assert!(!d.compute(&VoteRule::Unanimity).passed);
    }

    // EP-003-S07: archived lots must be excluded from shares_total.
    // The DB layer (compute_active_shares_total) enforces this; these tests
    // document the impact of using the wrong base on majority outcomes.

    #[test]
    fn ep003_s07_art25_passes_with_active_base_only() {
        // Building: 1000 total, 200 from archived lot → active = 800.
        // For votes: 401. With active base: threshold = 401 → passes.
        let d = tally(800, 600, 401, 199);
        let result = d.compute(&VoteRule::AbsoluteMajority);
        assert!(result.passed);
        assert_eq!(result.threshold, 401);
    }

    #[test]
    fn ep003_s07_art25_fails_when_archived_included_in_base() {
        // Same votes but shares_total erroneously includes archived lot (1000).
        // threshold = 501, 401 < 501 → incorrectly rejected.
        let d = tally(1000, 600, 401, 199);
        let result = d.compute(&VoteRule::AbsoluteMajority);
        assert!(!result.passed);
        assert_eq!(result.threshold, 501);
    }

    #[test]
    fn ep003_s07_art26_threshold_shifts_with_active_base() {
        // Active 800 shares: art.26 threshold = (800*2+2)/3 = 534.
        // 540 in favour → passes with correct base.
        let d = VoteTally {
            shares_total: 800,
            shares_present: 700,
            total_owners: 10,
            present_owners: 8,
            favorable_owners: 7,
            for_votes: 540,
            against_votes: 160,
            abstentions: 0,
        };
        let result = d.compute(&VoteRule::DoubleMajority);
        assert!(result.passed);
        assert_eq!(result.threshold, 534);
    }

    #[test]
    fn ep003_s07_art26_1_eligibility_shifts_with_active_base() {
        // Active 800: for_votes = 410 > 800/2 → eligible for art.26-1 re-vote.
        // With inflated 1000 base: 410 <= 500 → NOT eligible (wrong).
        let active_base = tally(800, 700, 410, 290);
        let inflated_base = tally(1000, 700, 410, 290);
        let active_result = active_base.compute(&VoteRule::DoubleMajority);
        let inflated_result = inflated_base.compute(&VoteRule::DoubleMajority);
        assert_eq!(
            active_result.eligible_for_second_vote,
            Some(SecondVoteType::Art26_1)
        );
        assert_eq!(inflated_result.eligible_for_second_vote, None);
    }

    // ── Additional edge-case unit tests ────────────────────────────────────────

    #[test]
    fn unanimity_passes_when_all_shares_vote_in_favor() {
        // 1000/1000 in favour, no against, no abstentions → unanimity
        let d = tally(1000, 1000, 1000, 0);
        let result = d.compute(&VoteRule::Unanimity);
        assert!(result.passed);
        assert_eq!(result.threshold, 1000);
        assert_eq!(result.eligible_for_second_vote, None);
    }

    #[test]
    fn art24_fails_on_exact_tie_between_for_and_against() {
        // 300 for vs 300 against → shares_counted = 600, threshold = 301 → not strictly more
        let d = tally(1000, 600, 300, 300);
        let result = d.compute(&VoteRule::SimpleMajority);
        assert!(!result.passed);
        assert_eq!(result.shares_counted, 600);
        assert_eq!(result.threshold, 301);
        assert_eq!(result.eligible_for_second_vote, None);
    }

    #[test]
    fn art25_passes_at_exact_absolute_majority_threshold() {
        // 501/1000 → threshold = 501, 501 >= 501 → passes
        let d = tally(1000, 800, 501, 299);
        let result = d.compute(&VoteRule::AbsoluteMajority);
        assert!(result.passed);
        assert_eq!(result.threshold, 501);
        assert_eq!(result.eligible_for_second_vote, None);
    }

    #[test]
    fn art26_fails_when_owners_condition_not_met_but_eligible_for_second_vote() {
        // Shares: 700/1000 ≥ 2/3 threshold (667), so shares condition is met.
        // Owners: 5 in favour out of 9 total → 5*3=15 < 9*2=18 → owners condition NOT met.
        // → overall rejected, but for_votes*2 (1400) > shares_total (1000) → Art.26-1 eligible.
        let d = VoteTally {
            shares_total: 1000,
            shares_present: 900,
            total_owners: 9,
            present_owners: 8,
            favorable_owners: 5,
            for_votes: 700,
            against_votes: 200,
            abstentions: 0,
        };
        let result = d.compute(&VoteRule::DoubleMajority);
        assert!(!result.passed, "owners condition not met → rejected");
        assert_eq!(result.threshold, 667);
        assert_eq!(
            result.eligible_for_second_vote,
            Some(SecondVoteType::Art26_1),
            "700 > 500 (half of 1000) → eligible for Art.26-1"
        );
    }

    #[test]
    fn art26_fails_when_only_shares_not_met_below_half_not_eligible_for_second_vote() {
        // for_votes = 490/1000 < 500 (half) → NOT eligible for Art.26-1
        let d = VoteTally {
            shares_total: 1000,
            shares_present: 900,
            total_owners: 9,
            present_owners: 8,
            favorable_owners: 7,
            for_votes: 490,
            against_votes: 410,
            abstentions: 0,
        };
        let result = d.compute(&VoteRule::DoubleMajority);
        assert!(!result.passed);
        assert_eq!(result.eligible_for_second_vote, None);
    }

    #[test]
    fn art25_fails_just_below_absolute_majority_threshold() {
        // 500/1000 → threshold = 501, 500 < 501 → fails
        // 500 * 3 = 1500 > 1000 → eligible for Art.25-1 (> 1/3)
        let d = tally(1000, 800, 500, 300);
        let result = d.compute(&VoteRule::AbsoluteMajority);
        assert!(!result.passed);
        assert_eq!(result.threshold, 501);
        assert_eq!(
            result.eligible_for_second_vote,
            Some(SecondVoteType::Art25_1),
            "500 > 333 (1/3 of 1000) → Art.25-1 applies"
        );
    }
}

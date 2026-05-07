/// Vote tally and majority calculations for general meeting resolutions.
/// Pure logic — no I/O, fully testable without a DB.
///
/// Legal references:
///   - Art. 24, Loi 10/07/1965: majority of votes present/represented
///   - Art. 25, Loi 10/07/1965: absolute majority of all co-owners
///   - Art. 25-1, Loi 10/07/1965: when art. 25 fails but > 1/3 votes in favour,
///     assembly may immediately re-vote under art. 24
///   - Art. 26, Loi 10/07/1965: double majority (2/3 votes + majority of members)
///   - Art. 26 al. 4: unanimity
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoteRule {
    SimpleMajority,
    AbsoluteMajority,
    DoubleMajority,
    Unanimity,
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
    pub eligible_for_second_vote: bool,
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
            eligible_for_second_vote: false,
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
        let eligible_for_second_vote = !passed && self.for_votes * 3 > self.shares_total;
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
    fn double_majority(&self) -> VoteResult {
        let threshold = (self.shares_total * 2 + 2) / 3;
        let shares_threshold_met = self.for_votes >= threshold;
        let owners_threshold_met = self.favorable_owners * 3 >= self.total_owners * 2;
        let passed = shares_threshold_met && owners_threshold_met;
        VoteResult {
            passed,
            for_votes: self.for_votes,
            against_votes: self.against_votes,
            abstentions: self.abstentions,
            shares_present: self.shares_present,
            shares_total: self.shares_total,
            shares_counted: self.shares_total,
            threshold,
            eligible_for_second_vote: false,
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
            eligible_for_second_vote: false,
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
        assert!(result.eligible_for_second_vote);
    }

    #[test]
    fn art25_1_not_eligible_when_rejected_at_or_below_one_third() {
        // 333/1000 = exactly 1/3 → not strictly more than 1/3, no second vote
        let d = tally(1000, 700, 333, 367);
        let result = d.compute(&VoteRule::AbsoluteMajority);
        assert!(!result.passed);
        assert!(!result.eligible_for_second_vote);
    }

    #[test]
    fn art25_1_not_eligible_when_passed() {
        // 501/1000 → passed, no second vote flag
        let d = tally(1000, 700, 501, 199);
        let result = d.compute(&VoteRule::AbsoluteMajority);
        assert!(result.passed);
        assert!(!result.eligible_for_second_vote);
    }

    #[test]
    fn art24_never_eligible_for_second_vote() {
        let d = tally(1000, 600, 299, 301);
        let result = d.compute(&VoteRule::SimpleMajority);
        assert!(!result.passed);
        assert!(!result.eligible_for_second_vote);
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
}

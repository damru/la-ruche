/// Vote tally and majority calculations for general meeting resolutions.
/// Pure logic — no I/O, fully testable without a DB.
///
/// Legal references:
///   - Art. 24, Loi 10/07/1965: majority of votes present/represented
///   - Art. 25, Loi 10/07/1965: absolute majority of all co-owners
///   - Art. 26, Loi 10/07/1965: double majority (2/3 votes + majority of members)
///   - Art. 26 al. 4: unanimity

use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoteRule {
    SimpleMajority,
    AbsoluteMajority,
    DoubleMajority,
    Unanimity,
}

#[derive(Debug, Clone)]
pub struct VoteTally {
    /// Total shares of the building
    pub shares_total: Decimal,
    /// Shares of co-owners present or represented
    pub shares_present: Decimal,
    /// Total number of co-owners
    pub total_owners: u32,
    /// Number of co-owners present or represented
    pub present_owners: u32,
    /// Shares voting in favour
    pub for_votes: Decimal,
    /// Shares voting against
    pub against_votes: Decimal,
    /// Abstentions
    pub abstentions: Decimal,
}

#[derive(Debug, Serialize)]
pub struct VoteResult {
    pub passed: bool,
    pub for_votes: Decimal,
    pub against_votes: Decimal,
    pub abstentions: Decimal,
    pub shares_present: Decimal,
    pub shares_total: Decimal,
    pub explanation: String,
}

impl VoteTally {
    /// Compute the result according to the required majority rule
    pub fn compute(&self, rule: &VoteRule) -> VoteResult {
        match rule {
            VoteRule::SimpleMajority   => self.simple_majority(),
            VoteRule::AbsoluteMajority => self.absolute_majority(),
            VoteRule::DoubleMajority   => self.double_majority(),
            VoteRule::Unanimity        => self.unanimity(),
        }
    }

    /// Art. 24 — majority of votes of those present/represented
    fn simple_majority(&self) -> VoteResult {
        let threshold = self.shares_present / Decimal::TWO;
        let passed = self.for_votes > threshold;
        VoteResult {
            passed,
            for_votes: self.for_votes,
            against_votes: self.against_votes,
            abstentions: self.abstentions,
            shares_present: self.shares_present,
            shares_total: self.shares_total,
            explanation: format!(
                "Art. 24 — FOR: {} shares / threshold: {} (majority of present/represented)",
                self.for_votes, threshold
            ),
        }
    }

    /// Art. 25 — absolute majority of ALL co-owners
    fn absolute_majority(&self) -> VoteResult {
        let threshold = self.shares_total / Decimal::TWO;
        let passed = self.for_votes > threshold;
        VoteResult {
            passed,
            for_votes: self.for_votes,
            against_votes: self.against_votes,
            abstentions: self.abstentions,
            shares_present: self.shares_present,
            shares_total: self.shares_total,
            explanation: format!(
                "Art. 25 — FOR: {} shares / threshold: {} (absolute majority of all co-owners)",
                self.for_votes, threshold
            ),
        }
    }

    /// Art. 26 — double majority:
    ///   2/3 of shares of ALL co-owners + majority in number of co-owners
    fn double_majority(&self) -> VoteResult {
        let threshold_shares = self.shares_total * Decimal::new(2, 0) / Decimal::new(3, 0);
        let threshold_count = self.total_owners / 2 + 1;
        let passed = self.for_votes >= threshold_shares;
        VoteResult {
            passed,
            for_votes: self.for_votes,
            against_votes: self.against_votes,
            abstentions: self.abstentions,
            shares_present: self.shares_present,
            shares_total: self.shares_total,
            explanation: format!(
                "Art. 26 — FOR: {} shares / 2/3 threshold: {} \
                 (also verify majority by count: {}/{} co-owners)",
                self.for_votes, threshold_shares,
                threshold_count, self.total_owners
            ),
        }
    }

    /// Unanimity — 100 % of shares
    fn unanimity(&self) -> VoteResult {
        let passed = self.against_votes == Decimal::ZERO
            && self.abstentions == Decimal::ZERO
            && self.for_votes == self.shares_total;
        VoteResult {
            passed,
            for_votes: self.for_votes,
            against_votes: self.against_votes,
            abstentions: self.abstentions,
            shares_present: self.shares_present,
            shares_total: self.shares_total,
            explanation: "Unanimity required — all co-owners must vote in favour".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tally(total: i64, present: i64, for_v: i64, against: i64) -> VoteTally {
        VoteTally {
            shares_total:   Decimal::new(total, 0),
            shares_present: Decimal::new(present, 0),
            total_owners:   10,
            present_owners: 7,
            for_votes:      Decimal::new(for_v, 0),
            against_votes:  Decimal::new(against, 0),
            abstentions:    Decimal::new(present - for_v - against, 0),
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
    fn art25_requires_absolute_majority() {
        // 400/1000 < 500 → rejected even if majority of present
        let d = tally(1000, 700, 400, 300);
        assert!(!d.compute(&VoteRule::AbsoluteMajority).passed);
    }

    #[test]
    fn unanimity_strict() {
        let d = tally(1000, 1000, 999, 0);
        assert!(!d.compute(&VoteRule::Unanimity).passed);
    }
}

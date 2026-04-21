/// Vote tally and majority calculations for general meeting resolutions.
/// Pure logic — no I/O, fully testable without a DB.
///
/// Legal references:
///   - Art. 24, Loi 10/07/1965: majority of votes present/represented
///   - Art. 25, Loi 10/07/1965: absolute majority of all co-owners
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
    pub explanation: String,
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

    /// Art. 24 — majority of votes of those present/represented
    fn simple_majority(&self) -> VoteResult {
        // strictly more than half of present shares
        let passed = self.for_votes * 2 > self.shares_present;
        VoteResult {
            passed,
            for_votes: self.for_votes,
            against_votes: self.against_votes,
            abstentions: self.abstentions,
            shares_present: self.shares_present,
            shares_total: self.shares_total,
            explanation: format!(
                "Art. 24 — FOR: {} shares / threshold: >{} (majority of present/represented)",
                self.for_votes,
                self.shares_present / 2
            ),
        }
    }

    /// Art. 25 — absolute majority of ALL co-owners
    fn absolute_majority(&self) -> VoteResult {
        // strictly more than half of total shares
        let passed = self.for_votes * 2 > self.shares_total;
        VoteResult {
            passed,
            for_votes: self.for_votes,
            against_votes: self.against_votes,
            abstentions: self.abstentions,
            shares_present: self.shares_present,
            shares_total: self.shares_total,
            explanation: format!(
                "Art. 25 — FOR: {} shares / threshold: >{} (absolute majority of all co-owners)",
                self.for_votes,
                self.shares_total / 2
            ),
        }
    }

    /// Art. 26 — double majority:
    ///   2/3 of shares of ALL co-owners + majority in number of co-owners
    fn double_majority(&self) -> VoteResult {
        // for_votes >= ceil(2/3 * shares_total)
        let passed = self.for_votes * 3 >= self.shares_total * 2;
        let threshold_count = self.total_owners / 2 + 1;
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
                self.for_votes,
                self.shares_total * 2 / 3,
                threshold_count,
                self.total_owners
            ),
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
            explanation: "Unanimity required — all co-owners must vote in favour".into(),
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

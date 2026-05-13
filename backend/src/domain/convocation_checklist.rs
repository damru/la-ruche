//! EP-003-S09 — Regulatory completeness checklist for AG convocation.
//!
//! Pure module: takes a structured [`ConvocationContext`] and produces the list of
//! mandatory checklist items per art. 7–13 of the décret 1967, with their status
//! (✓ satisfied / ✗ missing). No DB calls.
//!
//! Business rule: `ready_to_send` = all blocking items satisfied.
//! Blocking items (strict legal obligation):
//!   - j-21 deadline respected
//!   - at least one `notice` document attached
//!   - non-empty agenda
//!   - at least one resolution
//!   - vote-by-mail form attached (art. 9 al. 4 décret 67-223 v6.0, LEGIARTI000042078632)
//!
//! Advisory items (best practice, non-blocking):
//!   - convocation channel declared
//!   - send date recorded

use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct ConvocationContext {
    pub meeting_date: DateTime<Utc>,
    pub now: DateTime<Utc>,
    pub agenda: Option<String>,
    pub notice_documents_count: usize,
    pub resolutions_count: usize,
    pub convocation_sent_at: Option<DateTime<Utc>>,
    pub convocation_channel: Option<String>,
    pub has_vote_by_mail_form: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum ChecklistSeverity {
    Blocking,
    Advisory,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ChecklistItem {
    pub key: String,
    pub satisfied: bool,
    pub severity: ChecklistSeverity,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConvocationChecklist {
    pub items: Vec<ChecklistItem>,
    pub ready_to_send: bool,
}

pub fn evaluate(ctx: &ConvocationContext) -> ConvocationChecklist {
    let items = vec![
        item(
            "j_minus_21_respected",
            j_minus_21_respected(ctx),
            ChecklistSeverity::Blocking,
        ),
        item(
            "notice_document_attached",
            ctx.notice_documents_count >= 1,
            ChecklistSeverity::Blocking,
        ),
        item(
            "agenda_present",
            ctx.agenda
                .as_deref()
                .map(|a| !a.trim().is_empty())
                .unwrap_or(false),
            ChecklistSeverity::Blocking,
        ),
        item(
            "resolutions_present",
            ctx.resolutions_count >= 1,
            ChecklistSeverity::Blocking,
        ),
        item(
            "vote_by_mail_form_attached",
            ctx.has_vote_by_mail_form,
            ChecklistSeverity::Blocking,
        ),
        item(
            "convocation_channel_declared",
            ctx.convocation_channel.is_some(),
            ChecklistSeverity::Advisory,
        ),
        item(
            "convocation_sent_at_recorded",
            ctx.convocation_sent_at.is_some(),
            ChecklistSeverity::Advisory,
        ),
    ];

    let ready_to_send = items
        .iter()
        .filter(|i| i.severity == ChecklistSeverity::Blocking)
        .all(|i| i.satisfied);

    ConvocationChecklist {
        items,
        ready_to_send,
    }
}

fn item(key: &str, satisfied: bool, severity: ChecklistSeverity) -> ChecklistItem {
    ChecklistItem {
        key: key.to_string(),
        satisfied,
        severity,
    }
}

fn j_minus_21_respected(ctx: &ConvocationContext) -> bool {
    let reference = ctx.convocation_sent_at.unwrap_or(ctx.now);
    let days = (ctx.meeting_date - reference).num_days();
    days >= 21
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> ConvocationContext {
        ConvocationContext {
            meeting_date: DateTime::parse_from_rfc3339("2026-06-15T18:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            now: DateTime::parse_from_rfc3339("2026-05-13T18:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            agenda: Some("Approbation des comptes".to_string()),
            notice_documents_count: 1,
            resolutions_count: 2,
            convocation_sent_at: None,
            convocation_channel: None,
            has_vote_by_mail_form: true,
        }
    }

    fn key(checklist: &ConvocationChecklist, k: &str) -> bool {
        checklist
            .items
            .iter()
            .find(|i| i.key == k)
            .map(|i| i.satisfied)
            .unwrap_or(false)
    }

    #[test]
    fn ready_when_all_blocking_items_satisfied() {
        let c = evaluate(&ctx());
        assert!(c.ready_to_send);
    }

    #[test]
    fn j_minus_21_fails_when_meeting_too_close() {
        let mut c = ctx();
        c.meeting_date = c.now + chrono::Duration::days(20);
        let result = evaluate(&c);
        assert!(!result.ready_to_send);
        assert!(!key(&result, "j_minus_21_respected"));
    }

    #[test]
    fn j_minus_21_passes_at_exactly_21_days() {
        let mut c = ctx();
        c.meeting_date = c.now + chrono::Duration::days(21);
        assert!(key(&evaluate(&c), "j_minus_21_respected"));
    }

    #[test]
    fn missing_notice_blocks() {
        let mut c = ctx();
        c.notice_documents_count = 0;
        let result = evaluate(&c);
        assert!(!result.ready_to_send);
        assert!(!key(&result, "notice_document_attached"));
    }

    #[test]
    fn empty_agenda_blocks() {
        let mut c = ctx();
        c.agenda = Some("   ".to_string());
        let result = evaluate(&c);
        assert!(!result.ready_to_send);
        assert!(!key(&result, "agenda_present"));
    }

    #[test]
    fn no_resolutions_blocks() {
        let mut c = ctx();
        c.resolutions_count = 0;
        let result = evaluate(&c);
        assert!(!result.ready_to_send);
        assert!(!key(&result, "resolutions_present"));
    }

    #[test]
    fn missing_vote_by_mail_form_blocks() {
        let mut c = ctx();
        c.has_vote_by_mail_form = false;
        let result = evaluate(&c);
        assert!(!result.ready_to_send);
        assert!(!key(&result, "vote_by_mail_form_attached"));
    }

    #[test]
    fn advisory_items_dont_block() {
        let result = evaluate(&ctx());
        assert!(result.ready_to_send);
        assert!(!key(&result, "convocation_channel_declared"));
        assert!(!key(&result, "convocation_sent_at_recorded"));
    }

    #[test]
    fn channel_and_sent_at_satisfy_advisory_items() {
        let mut c = ctx();
        c.convocation_channel = Some("lrar".to_string());
        c.convocation_sent_at = Some(c.now);
        let result = evaluate(&c);
        assert!(key(&result, "convocation_channel_declared"));
        assert!(key(&result, "convocation_sent_at_recorded"));
    }

    #[test]
    fn j_minus_21_evaluated_from_sent_at_when_present() {
        let mut c = ctx();
        c.convocation_sent_at = Some(c.meeting_date - chrono::Duration::days(16));
        let result = evaluate(&c);
        assert!(!key(&result, "j_minus_21_respected"));
    }
}

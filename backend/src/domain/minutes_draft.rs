//! Draft procès-verbal (PV) generation for a closed AG.
//!
//! Pure module: takes a structured [`MinutesContext`] and produces formatted Markdown.
//! No DB calls. Lets the syndic download a draft PV for manual signature outside the app.
//!
//! Mandatory PV content per Art. 17 et 17-1A décret n°67-223 (loi n°65-557 du 10/07/1965):
//!   1. Date, time and location of the meeting.
//!   2. Names of the président de séance, secrétaire and scrutateurs.
//!   3. Attendance sheet: présents, représentés, défaillants + their tantièmes.
//!   4. Total tantièmes of présents + représentés (quorum verification).
//!   5. Agenda (ordre du jour).
//!   6. For each resolution: full text submitted to vote, result, vote breakdown.
//!   7. Reservations (réserves) emitted by co-owners during the vote.
//!   8. Opening time and, when available, closing time of the session.
//!   9. Contestation deadline (Art. 42 loi 1965 — 2 months from notification).
//!   10. Individual names of opposants, abstentions and assimilated non-voters per resolution (Art. 17-1A).
//!   11. Proxy holder name for each represented co-owner (mandant → mandataire).
//!   12. Proxy distribution mention if blank proxies were handed out in session.
//!   13. Technical incidents preventing electronic voters from casting their vote.
//!   14. Signature deadline: at end of session or within 8 days (art. 17 al. 1 décret 67-223).
//!   15. Attendance sheet annexe mention.

use chrono::{DateTime, Utc};

/// Full context for rendering the draft PV of a closed AG.
#[derive(Debug, Clone)]
pub struct MinutesContext {
    pub building_name: String,
    pub building_address: String,
    pub meeting_type: String,
    pub meeting_date: DateTime<Utc>,
    pub closing_time: Option<DateTime<Utc>>,
    pub location: Option<String>,
    pub president_name: Option<String>,
    pub secretary_name: Option<String>,
    pub scrutineers: Vec<String>,
    pub agenda: Option<String>,
    pub total_building_shares: Option<i32>,
    /// Art. 17-1A: how blank proxies were distributed during the session (if applicable).
    pub proxy_distribution: Option<String>,
    /// Art. 17-1A: technical incidents preventing electronic voters from casting their vote.
    pub technical_incidents: Vec<String>,
    pub resolutions: Vec<MinutesResolution>,
    pub attendance: Vec<MinutesAttendanceLine>,
    pub president_signed_at: Option<DateTime<Utc>>,
    pub secretary_signed_at: Option<DateTime<Utc>>,
    pub scrutineers_signed: Vec<MinutesSignature>,
}

#[derive(Debug, Clone)]
pub struct MinutesSignature {
    pub name: String,
    pub signed_at: DateTime<Utc>,
}

/// Individual vote position of a co-owner for PV purposes — Art. 17-1A décret n°67-223.
#[derive(Debug, Clone)]
pub struct MinutesVoteDetail {
    pub owner_name: String,
    pub unit_number: String,
    pub shares: i32,
    /// One of: "against" | "abstain" | "non_voter" (assimilé défaillant).
    pub position: String,
}

#[derive(Debug, Clone)]
pub struct MinutesResolution {
    pub number: i32,
    pub title: String,
    pub description: Option<String>,
    pub vote_type: String,
    pub status: String,
    pub for_votes: i32,
    pub against_votes: i32,
    pub abstentions: i32,
    pub threshold: i32,
    pub eligible_for_second_vote: Option<String>,
    pub reservations: Vec<String>,
    /// Individual non-favorable votes (against / abstain / non_voter) — Art. 17-1A.
    pub vote_details: Vec<MinutesVoteDetail>,
}

#[derive(Debug, Clone)]
pub struct MinutesAttendanceLine {
    pub unit_number: String,
    pub owner_name: Option<String>,
    pub mode: String,
    pub shares: i32,
    /// Name of the proxy holder when mode = "proxy" (mandataire) — Art. 17-1A.
    pub representative_name: Option<String>,
}

/// Renders the draft PV as Markdown.
///
/// Covers all mandatory mentions of Art. 17 et 17-1A décret n°67-223:
/// opening/closing time, agenda, attendance quorum (with proxy holder names),
/// resolution texts with individual opposant/abstention/non-voter names,
/// proxy distribution mention, technical incidents, reservations,
/// signature deadline (8 days — Art. 17 décret 1967), and
/// contestation deadline (Art. 42 loi 1965).
pub fn render(ctx: &MinutesContext) -> String {
    let mut out = String::new();

    out.push_str(&format!("# Procès-verbal — {}\n\n", ctx.building_name));
    out.push_str(&format!("**Adresse :** {}\n\n", ctx.building_address));
    out.push_str(&format!(
        "**Type d'assemblée :** {}\n\n",
        humanize_meeting_type(&ctx.meeting_type)
    ));
    out.push_str(&format!(
        "**Date :** {}\n\n",
        ctx.meeting_date.format("%d/%m/%Y")
    ));
    out.push_str(&format!(
        "**Heure d'ouverture :** {}\n\n",
        ctx.meeting_date.format("%H:%M UTC")
    ));
    if let Some(closing) = ctx.closing_time {
        out.push_str(&format!(
            "**Heure de clôture :** {}\n\n",
            closing.format("%H:%M UTC")
        ));
    }
    if let Some(location) = &ctx.location {
        out.push_str(&format!("**Lieu :** {}\n\n", location));
    }
    if let Some(dist) = &ctx.proxy_distribution {
        out.push_str(&format!(
            "**Distribution des mandats en blanc :** {}\n\n",
            humanize_proxy_distribution(dist)
        ));
    }

    out.push_str("## Rôles de séance\n\n");
    out.push_str(&format!(
        "- **Président :** {}\n",
        ctx.president_name.as_deref().unwrap_or("—")
    ));
    out.push_str(&format!(
        "- **Secrétaire :** {}\n",
        ctx.secretary_name.as_deref().unwrap_or("—")
    ));
    if ctx.scrutineers.is_empty() {
        out.push_str("- **Scrutateurs :** —\n");
    } else {
        out.push_str(&format!(
            "- **Scrutateurs :** {}\n",
            ctx.scrutineers.join(", ")
        ));
    }
    out.push('\n');

    out.push_str("## Feuille de présence\n\n");
    if ctx.attendance.is_empty() {
        out.push_str("_Aucune entrée enregistrée._\n\n");
    } else {
        let shares_present_and_represented: i32 = ctx
            .attendance
            .iter()
            .filter(|a| a.mode == "present" || a.mode == "proxy" || a.mode == "correspondence")
            .map(|a| a.shares)
            .sum();

        if let Some(total) = ctx.total_building_shares {
            out.push_str(&format!(
                "**Tantièmes présents + représentés :** {} / {} ({}%)\n\n",
                shares_present_and_represented,
                total,
                if total > 0 {
                    shares_present_and_represented * 100 / total
                } else {
                    0
                }
            ));
        } else {
            out.push_str(&format!(
                "**Tantièmes présents + représentés :** {}\n\n",
                shares_present_and_represented
            ));
        }

        for mode in ["present", "proxy", "correspondence", "absent"] {
            let lines: Vec<&MinutesAttendanceLine> =
                ctx.attendance.iter().filter(|a| a.mode == mode).collect();
            if lines.is_empty() {
                continue;
            }
            out.push_str(&format!("### {}\n\n", humanize_mode(mode)));
            for line in lines {
                let rep_suffix = if line.mode == "proxy" {
                    line.representative_name
                        .as_deref()
                        .map(|r| format!(" — représenté·e par {}", r))
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                out.push_str(&format!(
                    "- Lot {} — {}{}  — {} tantièmes\n",
                    line.unit_number,
                    line.owner_name.as_deref().unwrap_or("—"),
                    rep_suffix,
                    line.shares
                ));
            }
            out.push('\n');
        }
    }

    out.push_str("## Ordre du jour\n\n");
    match &ctx.agenda {
        Some(agenda) if !agenda.trim().is_empty() => {
            out.push_str(agenda.trim());
            out.push_str("\n\n");
        }
        _ => out.push_str("_Ordre du jour non renseigné._\n\n"),
    }

    out.push_str("## Résolutions\n\n");
    if ctx.resolutions.is_empty() {
        out.push_str("_Aucune résolution._\n\n");
    } else {
        for r in &ctx.resolutions {
            out.push_str(&format!("### Résolution n° {} — {}\n\n", r.number, r.title));
            if let Some(desc) = &r.description
                && !desc.trim().is_empty()
            {
                out.push_str(&format!("{}\n\n", desc.trim()));
            }
            out.push_str(&format!(
                "- **Règle de majorité :** {}\n",
                humanize_vote_rule(&r.vote_type)
            ));
            out.push_str(&format!("- **Statut :** {}\n", humanize_status(&r.status)));
            out.push_str(&format!(
                "- **Votes :** Pour = {}, Contre = {}, Abstentions = {}\n",
                r.for_votes, r.against_votes, r.abstentions
            ));
            out.push_str(&format!("- **Seuil légal :** {}\n", r.threshold));
            if let Some(second) = &r.eligible_for_second_vote {
                out.push_str(&format!(
                    "- **Passerelle :** {}\n",
                    humanize_passerelle(second)
                ));
            }
            if !r.reservations.is_empty() {
                out.push_str("- **Réserves :**\n");
                for reservation in &r.reservations {
                    out.push_str(&format!("  - {}\n", reservation));
                }
            }
            let opposants: Vec<&MinutesVoteDetail> = r
                .vote_details
                .iter()
                .filter(|v| v.position == "against")
                .collect();
            let abstentions_detail: Vec<&MinutesVoteDetail> = r
                .vote_details
                .iter()
                .filter(|v| v.position == "abstain")
                .collect();
            let non_voters: Vec<&MinutesVoteDetail> = r
                .vote_details
                .iter()
                .filter(|v| v.position == "non_voter")
                .collect();
            if !opposants.is_empty() {
                out.push_str("- **Opposants :**\n");
                for v in &opposants {
                    out.push_str(&format!(
                        "  - {} (Lot {} — {} tantièmes)\n",
                        v.owner_name, v.unit_number, v.shares
                    ));
                }
            }
            if !abstentions_detail.is_empty() {
                out.push_str("- **Abstentionnistes :**\n");
                for v in &abstentions_detail {
                    out.push_str(&format!(
                        "  - {} (Lot {} — {} tantièmes)\n",
                        v.owner_name, v.unit_number, v.shares
                    ));
                }
            }
            if !non_voters.is_empty() {
                out.push_str("- **Assimilés défaillants (art. 17-1A loi 1965) :**\n");
                for v in &non_voters {
                    out.push_str(&format!(
                        "  - {} (Lot {} — {} tantièmes)\n",
                        v.owner_name, v.unit_number, v.shares
                    ));
                }
            }
            out.push('\n');
        }
    }

    if !ctx.technical_incidents.is_empty() {
        out.push_str("## Incidents techniques\n\n");
        out.push_str(
            "_Les incidents techniques suivants ont empêché des copropriétaires de faire \
            connaître leur vote (visioconférence / vote électronique — Art. 17-1A loi 1965) :_\n\n",
        );
        for incident in &ctx.technical_incidents {
            out.push_str(&format!("- {}\n", incident));
        }
        out.push('\n');
    }

    out.push_str("## Signatures du bureau de séance\n\n");
    out.push_str(&render_signature_line(
        "Président",
        ctx.president_name.as_deref(),
        ctx.president_signed_at,
    ));
    out.push_str(&render_signature_line(
        "Secrétaire",
        ctx.secretary_name.as_deref(),
        ctx.secretary_signed_at,
    ));
    if ctx.scrutineers_signed.is_empty() {
        if ctx.scrutineers.is_empty() {
            out.push_str("- **Scrutateurs :** —\n");
        } else {
            for name in &ctx.scrutineers {
                out.push_str(&render_signature_line("Scrutateur", Some(name), None));
            }
        }
    } else {
        for sig in &ctx.scrutineers_signed {
            out.push_str(&render_signature_line(
                "Scrutateur",
                Some(&sig.name),
                Some(sig.signed_at),
            ));
        }
    }
    out.push('\n');
    out.push_str("_La feuille de présence est annexée au présent procès-verbal._\n\n");

    out.push_str("---\n\n");
    let signature_deadline = ctx.meeting_date + chrono::Duration::days(8);
    out.push_str(&format!(
        "_Le présent procès-verbal est signé à la fin de la séance ou au plus tard le {} \
        (dans les huit jours suivant la tenue de l'assemblée — art. 17 al. 1 décret 67-223)._\n\n",
        signature_deadline.format("%d/%m/%Y")
    ));
    let notification_duty_deadline = ctx.meeting_date + chrono::Duration::days(30);
    out.push_str(&format!(
        "_Obligation du syndic : notifier le présent procès-verbal aux copropriétaires \
        opposants et défaillants au plus tard le {} \
        (dans le mois suivant la tenue de l'assemblée — art. 42 al. 3 loi 65-557). \
        La notification doit mentionner les résultats du vote et reproduire le texte \
        de l'art. 42 al. 2 (art. 18 décret 67-223)._\n\n",
        notification_duty_deadline.format("%d/%m/%Y")
    ));
    out.push_str(
        "_Délai de contestation des opposants et défaillants : 2 mois à compter \
        de la date de notification du procès-verbal (art. 42 loi 65-557)._\n",
    );

    out
}

fn render_signature_line(
    role: &str,
    name: Option<&str>,
    signed_at: Option<DateTime<Utc>>,
) -> String {
    let name = name.unwrap_or("—");
    match signed_at {
        Some(at) => format!(
            "- **{} :** {} — signé le {}\n",
            role,
            name,
            at.format("%d/%m/%Y %H:%M UTC")
        ),
        None => format!("- **{} :** {} — _non signé_\n", role, name),
    }
}

fn humanize_meeting_type(t: &str) -> &str {
    match t {
        "extraordinary" => "Assemblée générale extraordinaire",
        _ => "Assemblée générale annuelle",
    }
}

fn humanize_vote_rule(r: &str) -> &str {
    match r {
        "simple_majority" => "Article 24 — majorité simple",
        "absolute_majority" => "Article 25 — majorité absolue",
        "double_majority" => "Article 26 — double majorité",
        "unanimity" => "Unanimité",
        _ => r,
    }
}

fn humanize_status(s: &str) -> &str {
    match s {
        "passed" => "Adoptée",
        "rejected" => "Rejetée",
        _ => "En attente",
    }
}

fn humanize_passerelle(p: &str) -> &str {
    match p {
        "art_25_1" => "Article 25-1 — re-délibération possible à la majorité simple (art. 24)",
        "art_26_1" => "Article 26-1 — re-délibération possible à la majorité absolue (art. 25)",
        _ => p,
    }
}

fn humanize_proxy_distribution(d: &str) -> &str {
    match d {
        "council_president" => "le président du conseil syndical",
        "council_member" => "un membre du conseil syndical",
        "session_president" => "le président de séance",
        _ => d,
    }
}

fn humanize_mode(m: &str) -> &str {
    match m {
        "present" => "Présents",
        "proxy" => "Représentés (procuration)",
        "correspondence" => "Vote par correspondance",
        "absent" => "Absents",
        _ => m,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ctx() -> MinutesContext {
        MinutesContext {
            building_name: "Résidence Test".to_string(),
            building_address: "1 rue de la République, 75001 Paris".to_string(),
            meeting_type: "annual".to_string(),
            meeting_date: DateTime::parse_from_rfc3339("2026-05-13T18:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            closing_time: None,
            location: Some("Salle commune".to_string()),
            president_name: Some("Jean Dupont".to_string()),
            secretary_name: Some("Marie Martin".to_string()),
            scrutineers: vec!["Paul Durand".to_string(), "Sophie Bernard".to_string()],
            agenda: Some("1. Approbation des comptes 2025\n2. Budget 2026".to_string()),
            total_building_shares: Some(1000),
            proxy_distribution: None,
            technical_incidents: vec![],
            resolutions: vec![MinutesResolution {
                number: 1,
                title: "Approbation des comptes 2025".to_string(),
                description: Some(
                    "L'assemblée approuve les comptes de l'exercice 2025 tels que présentés."
                        .to_string(),
                ),
                vote_type: "simple_majority".to_string(),
                status: "passed".to_string(),
                for_votes: 600,
                against_votes: 200,
                abstentions: 200,
                threshold: 401,
                eligible_for_second_vote: None,
                reservations: vec![],
                vote_details: vec![],
            }],
            attendance: vec![
                MinutesAttendanceLine {
                    unit_number: "1".to_string(),
                    owner_name: Some("Jean Dupont".to_string()),
                    mode: "present".to_string(),
                    shares: 500,
                    representative_name: None,
                },
                MinutesAttendanceLine {
                    unit_number: "2".to_string(),
                    owner_name: Some("Marie Martin".to_string()),
                    mode: "absent".to_string(),
                    shares: 300,
                    representative_name: None,
                },
            ],
            president_signed_at: None,
            secretary_signed_at: None,
            scrutineers_signed: vec![],
        }
    }

    #[test]
    fn draft_includes_building_header() {
        let md = render(&sample_ctx());
        assert!(md.contains("Résidence Test"));
        assert!(md.contains("1 rue de la République, 75001 Paris"));
        assert!(md.contains("Assemblée générale annuelle"));
    }

    #[test]
    fn draft_lists_session_roles() {
        let md = render(&sample_ctx());
        assert!(md.contains("Jean Dupont"));
        assert!(md.contains("Marie Martin"));
        assert!(md.contains("Paul Durand, Sophie Bernard"));
    }

    #[test]
    fn draft_lists_resolutions_with_rule_and_result() {
        let md = render(&sample_ctx());
        assert!(md.contains("Approbation des comptes 2025"));
        assert!(md.contains("Article 24 — majorité simple"));
        assert!(md.contains("Adoptée"));
        assert!(md.contains("Pour = 600"));
        assert!(md.contains("Seuil légal :** 401"));
    }

    #[test]
    fn draft_flags_resolution_eligible_for_passerelle() {
        let mut ctx = sample_ctx();
        ctx.resolutions[0].status = "rejected".to_string();
        ctx.resolutions[0].vote_type = "absolute_majority".to_string();
        ctx.resolutions[0].eligible_for_second_vote = Some("art_25_1".to_string());
        let md = render(&ctx);
        assert!(md.contains("Article 25-1"));
    }

    #[test]
    fn draft_groups_attendance_by_mode() {
        let md = render(&sample_ctx());
        assert!(md.contains("### Présents"));
        assert!(md.contains("### Absents"));
        let present_idx = md.find("### Présents").unwrap();
        let absent_idx = md.find("### Absents").unwrap();
        assert!(present_idx < absent_idx);
    }

    #[test]
    fn draft_handles_empty_resolutions_and_attendance() {
        let mut ctx = sample_ctx();
        ctx.resolutions.clear();
        ctx.attendance.clear();
        let md = render(&ctx);
        assert!(md.contains("Aucune résolution"));
        assert!(md.contains("Aucune entrée enregistrée"));
    }

    #[test]
    fn draft_flags_president_signature_when_present() {
        let mut ctx = sample_ctx();
        ctx.president_signed_at = Some(
            DateTime::parse_from_rfc3339("2026-05-13T18:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
        );
        let md = render(&ctx);
        assert!(md.contains("Président :** Jean Dupont — signé le"));
        assert!(md.contains("Secrétaire :** Marie Martin — _non signé_"));
    }

    #[test]
    fn draft_renders_scrutineers_with_signatures() {
        let mut ctx = sample_ctx();
        let at = DateTime::parse_from_rfc3339("2026-05-13T19:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        ctx.scrutineers_signed = vec![MinutesSignature {
            name: "Paul Durand".to_string(),
            signed_at: at,
        }];
        let md = render(&ctx);
        assert!(md.contains("Scrutateur :** Paul Durand — signé le"));
    }

    #[test]
    fn draft_lists_scrutineers_unsigned_when_no_signatures_recorded() {
        let ctx = sample_ctx();
        let md = render(&ctx);
        assert!(md.contains("Scrutateur :** Paul Durand — _non signé_"));
        assert!(md.contains("Scrutateur :** Sophie Bernard — _non signé_"));
    }

    #[test]
    fn draft_includes_agenda() {
        let md = render(&sample_ctx());
        assert!(md.contains("## Ordre du jour"));
        assert!(md.contains("Approbation des comptes 2025"));
        assert!(md.contains("Budget 2026"));
    }

    #[test]
    fn draft_shows_missing_agenda_notice() {
        let mut ctx = sample_ctx();
        ctx.agenda = None;
        let md = render(&ctx);
        assert!(md.contains("Ordre du jour non renseigné"));
    }

    #[test]
    fn draft_includes_quorum_summary_with_total() {
        let md = render(&sample_ctx());
        // 500 shares present (lot 1), 300 absent (lot 2), total = 1000
        assert!(md.contains("Tantièmes présents + représentés :** 500 / 1000 (50%)"));
    }

    #[test]
    fn draft_includes_opening_time() {
        let md = render(&sample_ctx());
        assert!(md.contains("Heure d'ouverture :** 18:00 UTC"));
    }

    #[test]
    fn draft_includes_closing_time_when_set() {
        let mut ctx = sample_ctx();
        ctx.closing_time = Some(
            DateTime::parse_from_rfc3339("2026-05-13T20:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
        );
        let md = render(&ctx);
        assert!(md.contains("Heure de clôture :** 20:30 UTC"));
    }

    #[test]
    fn draft_includes_resolution_description() {
        let md = render(&sample_ctx());
        assert!(md.contains("L'assemblée approuve les comptes de l'exercice 2025"));
    }

    #[test]
    fn draft_includes_reservations_when_present() {
        let mut ctx = sample_ctx();
        ctx.resolutions[0]
            .reservations
            .push("M. Durand émet une réserve sur le montant des honoraires.".to_string());
        let md = render(&ctx);
        assert!(md.contains("**Réserves :**"));
        assert!(md.contains("M. Durand émet une réserve sur le montant des honoraires."));
    }

    #[test]
    fn draft_includes_contestation_deadline() {
        let md = render(&sample_ctx());
        // Meeting date 13/05/2026 + 60 days = 12/07/2026
        assert!(md.contains("Art. 42 loi 1965"));
        assert!(md.contains("12/07/2026"));
    }

    #[test]
    fn draft_renders_em_dash_when_roles_missing() {
        let mut ctx = sample_ctx();
        ctx.president_name = None;
        ctx.secretary_name = None;
        ctx.scrutineers.clear();
        let md = render(&ctx);
        assert!(md.contains("**Président :** —"));
        assert!(md.contains("**Secrétaire :** —"));
        assert!(md.contains("**Scrutateurs :** —"));
    }

    #[test]
    fn draft_lists_opposants_by_name() {
        let mut ctx = sample_ctx();
        ctx.resolutions[0].vote_details = vec![MinutesVoteDetail {
            owner_name: "Marie Martin".to_string(),
            unit_number: "2".to_string(),
            shares: 300,
            position: "against".to_string(),
        }];
        let md = render(&ctx);
        assert!(md.contains("**Opposants :**"));
        assert!(md.contains("Marie Martin (Lot 2 — 300 tantièmes)"));
    }

    #[test]
    fn draft_lists_abstentions_by_name() {
        let mut ctx = sample_ctx();
        ctx.resolutions[0].vote_details = vec![MinutesVoteDetail {
            owner_name: "Paul Durand".to_string(),
            unit_number: "3".to_string(),
            shares: 200,
            position: "abstain".to_string(),
        }];
        let md = render(&ctx);
        assert!(md.contains("**Abstentionnistes :**"));
        assert!(md.contains("Paul Durand (Lot 3 — 200 tantièmes)"));
    }

    #[test]
    fn draft_lists_non_voters_as_assimile_defaillant() {
        let mut ctx = sample_ctx();
        ctx.resolutions[0].vote_details = vec![MinutesVoteDetail {
            owner_name: "Sophie Bernard".to_string(),
            unit_number: "4".to_string(),
            shares: 100,
            position: "non_voter".to_string(),
        }];
        let md = render(&ctx);
        assert!(md.contains("**Assimilés défaillants (art. 17-1A loi 1965) :**"));
        assert!(md.contains("Sophie Bernard (Lot 4 — 100 tantièmes)"));
    }

    #[test]
    fn draft_includes_proxy_representative_in_attendance() {
        let mut ctx = sample_ctx();
        ctx.attendance[0].mode = "proxy".to_string();
        ctx.attendance[0].representative_name = Some("Paul Durand".to_string());
        let md = render(&ctx);
        assert!(md.contains("représenté·e par Paul Durand"));
    }

    #[test]
    fn draft_includes_proxy_distribution_when_set() {
        let mut ctx = sample_ctx();
        ctx.proxy_distribution = Some("session_president".to_string());
        let md = render(&ctx);
        assert!(md.contains("Distribution des mandats en blanc"));
        assert!(md.contains("le président de séance"));
    }

    #[test]
    fn draft_includes_technical_incidents_when_present() {
        let mut ctx = sample_ctx();
        ctx.technical_incidents = vec!["Lot 3 — coupure réseau à 19h12.".to_string()];
        let md = render(&ctx);
        assert!(md.contains("## Incidents techniques"));
        assert!(md.contains("Lot 3 — coupure réseau à 19h12."));
    }

    #[test]
    fn draft_includes_annexe_note() {
        let md = render(&sample_ctx());
        assert!(md.contains("La feuille de présence est annexée au présent procès-verbal."));
    }

    #[test]
    fn draft_includes_signature_deadline() {
        let md = render(&sample_ctx());
        // Meeting date 13/05/2026 + 8 days = 21/05/2026
        assert!(md.contains("21/05/2026"));
        assert!(md.contains("Art. 17 décret 1967"));
    }
}

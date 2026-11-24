use serde_json::Value;
use uuid::Uuid;

/// Masks the reporter identity on an anonymous report unless the viewer IS
/// the reporter. Applied to every serialized report (list, get, create).
/// Masked fields: author_id, author_first_name, author_last_name -> null.
/// Always injects "is_author" so the reporter's own UI can label it "vous";
/// other viewers get is_author = false without being able to deduce the
/// identity.
pub(crate) fn mask_report_author(report: &mut Value, viewer_id: Uuid) {
    let author_id = report
        .get("author_id")
        .and_then(Value::as_str)
        .map(|s| Uuid::parse_str(s).ok());
    let is_author = matches!(author_id, Some(Some(id)) if id == viewer_id);
    report["is_author"] = Value::Bool(is_author);
    let anonymous = report.get("is_anonymous").and_then(Value::as_bool) == Some(true);
    if anonymous && !is_author {
        report["author_id"] = Value::Null;
        report["author_first_name"] = Value::Null;
        report["author_last_name"] = Value::Null;
    }
}

/// Applies [`mask_report_author`] to every element of a JSON array of reports.
pub(crate) fn mask_report_list(reports: &mut Value, viewer_id: Uuid) {
    if let Some(items) = reports.as_array_mut() {
        for item in items {
            mask_report_author(item, viewer_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn report_json(author: &str, anonymous: bool) -> Value {
        json!({
            "id": "00000000-0000-0000-0000-000000000001",
            "author_id": author,
            "author_first_name": "Ada",
            "author_last_name": "Lovelace",
            "is_anonymous": anonymous,
            "title": "Fuite",
        })
    }

    const AUTHOR: &str = "11111111-1111-1111-1111-111111111111";
    const OTHER: &str = "22222222-2222-2222-2222-222222222222";

    #[test]
    fn anonymous_report_masked_for_other_viewer() {
        let viewer = Uuid::parse_str(OTHER).unwrap();
        let mut v = report_json(AUTHOR, true);
        mask_report_author(&mut v, viewer);
        assert_eq!(v["author_id"], Value::Null);
        assert_eq!(v["author_first_name"], Value::Null);
        assert_eq!(v["author_last_name"], Value::Null);
        assert_eq!(v["is_author"], json!(false));
        assert_eq!(v["is_anonymous"], json!(true));
    }

    #[test]
    fn anonymous_report_visible_to_its_author() {
        let author = Uuid::parse_str(AUTHOR).unwrap();
        let mut v = report_json(AUTHOR, true);
        mask_report_author(&mut v, author);
        assert_eq!(v["author_id"], json!(AUTHOR));
        assert_eq!(v["author_first_name"], json!("Ada"));
        assert_eq!(v["is_author"], json!(true));
    }

    #[test]
    fn non_anonymous_report_never_masked() {
        let _author = Uuid::parse_str(AUTHOR).unwrap();
        let viewer = Uuid::parse_str(OTHER).unwrap();
        let mut v = report_json(AUTHOR, false);
        mask_report_author(&mut v, viewer);
        assert_eq!(v["author_id"], json!(AUTHOR));
        assert_eq!(v["author_first_name"], json!("Ada"));
        assert_eq!(v["is_author"], json!(false));
    }

    #[test]
    fn list_masks_each_element_independently() {
        let _author = Uuid::parse_str(AUTHOR).unwrap();
        let viewer = Uuid::parse_str(OTHER).unwrap();
        let mut v = json!([report_json(AUTHOR, true), report_json(OTHER, false)]);
        mask_report_list(&mut v, viewer);
        assert_eq!(v[0]["author_id"], Value::Null);
        assert_eq!(v[0]["is_author"], json!(false));
        assert_eq!(v[1]["author_id"], json!(OTHER));
        assert_eq!(v[1]["is_author"], json!(true));
    }
}

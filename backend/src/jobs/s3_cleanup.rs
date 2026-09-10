/// Background job: periodic S3 cleanup (EP-005-S09, PRD US-DOC-008,
/// LEGAL §6.1-6.3).
///
/// Two independent passes, run once a day:
///   1. Orphaned S3 objects — no `document` row points to them (e.g. a
///      presigned upload that never got confirmed). Deleted only once older
///      than `ORPHAN_GRACE`, to avoid racing an in-flight upload.
///   2. Deferred purge of soft-deleted documents whose category's legal
///      retention (`domain::retention`) has elapsed — physically deletes the
///      DB row, then the S3 object. Categories with unlimited retention
///      (founding documents, PV d'AG, carnet d'entretien) are never purged.
use std::sync::Arc;

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use tokio::time::{Duration, interval};
use tracing::{error, info, warn};

use crate::{db::AppState, domain::retention};

/// Minimum age of an S3 object with no matching DB row before it is treated
/// as orphaned rather than an in-flight upload (spec EP-005-S09 §5.2: 48h).
const ORPHAN_GRACE: ChronoDuration = ChronoDuration::hours(48);

#[derive(Debug, Default)]
pub struct CleanupReport {
    pub orphans_deleted: usize,
    pub documents_purged: usize,
    pub errors: usize,
}

pub async fn run(state: Arc<AppState>) {
    let mut ticker = interval(Duration::from_secs(86_400));
    loop {
        ticker.tick().await;
        if !state.cfg.s3.cleanup_enabled {
            continue;
        }
        match run_once(&state).await {
            Ok(report) => info!(
                orphans_deleted = report.orphans_deleted,
                documents_purged = report.documents_purged,
                errors = report.errors,
                "S3 cleanup job completed"
            ),
            Err(e) => error!(error = %e, "S3 cleanup job error"),
        }
    }
}

/// Runs both passes once; returns a report instead of looping, for testing.
pub async fn run_once(state: &Arc<AppState>) -> anyhow::Result<CleanupReport> {
    let mut report = CleanupReport::default();
    clean_orphaned_objects(state, &mut report).await?;
    purge_expired_soft_deletes(state, &mut report).await?;
    Ok(report)
}

async fn clean_orphaned_objects(
    state: &Arc<AppState>,
    report: &mut CleanupReport,
) -> anyhow::Result<()> {
    let now = Utc::now();
    let building_ids = crate::db::document::all_building_ids(&state.pool).await?;

    for building_id in building_ids {
        let bucket = state.storage.bucket_name(building_id);
        let known_keys =
            crate::db::document::all_storage_keys_for_building(&state.pool, building_id).await?;

        let objects = match state.storage.list_objects(&bucket).await {
            Ok(objects) => objects,
            Err(e) => {
                warn!(error = %e, bucket = %bucket, "S3 cleanup: could not list objects");
                report.errors += 1;
                continue;
            }
        };

        for (key, last_modified) in objects {
            if known_keys.contains(&key) {
                continue;
            }

            let age_ok = last_modified
                .and_then(|lm| DateTime::from_timestamp(lm.secs(), 0))
                .map(|last_modified| now - last_modified >= ORPHAN_GRACE)
                .unwrap_or(false);
            if !age_ok {
                continue;
            }

            match state.storage.delete_object(&bucket, &key).await {
                Ok(()) => {
                    info!(bucket = %bucket, key = %key, "S3 cleanup: deleted orphaned object");
                    report.orphans_deleted += 1;
                }
                Err(e) => {
                    warn!(error = %e, bucket = %bucket, key = %key, "S3 cleanup: failed to delete orphaned object");
                    report.errors += 1;
                }
            }
        }
    }

    Ok(())
}

async fn purge_expired_soft_deletes(
    state: &Arc<AppState>,
    report: &mut CleanupReport,
) -> anyhow::Result<()> {
    let now = Utc::now();
    let candidates = crate::db::document::soft_deleted_candidates(&state.pool).await?;

    for candidate in candidates {
        if !retention::is_purgeable(&candidate.category, candidate.deleted_at, now) {
            continue;
        }

        // DB delete first — if S3 delete then fails, the object becomes an
        // orphan rattrapé by the next pass-1 run (idempotent).
        if let Err(e) = crate::db::document::purge(&state.pool, candidate.id).await {
            warn!(error = %e, document_id = %candidate.id, "S3 cleanup: failed to purge document row");
            report.errors += 1;
            continue;
        }

        match state
            .storage
            .delete_object(&candidate.bucket, &candidate.storage_path)
            .await
        {
            Ok(()) => {
                info!(
                    document_id = %candidate.id,
                    bucket = %candidate.bucket,
                    key = %candidate.storage_path,
                    category = %candidate.category,
                    "S3 cleanup: purged expired soft-deleted document"
                );
                report.documents_purged += 1;
            }
            Err(e) => {
                warn!(error = %e, document_id = %candidate.id, "S3 cleanup: DB row purged but S3 delete failed");
                report.errors += 1;
            }
        }
    }

    Ok(())
}

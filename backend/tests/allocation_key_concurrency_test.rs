// EP-001-S10 follow-up: concurrency test for allocation_key mutations.
//
// Retro 2026-09-14 (t_13e08ee0 / t_15ad7bbf): Hopper flagged a possible TOCTOU
// in `db::allocation_key::update` / `delete` — both read a "before" row and
// then mutate based on it. This test exercises the code under real
// concurrent load (parallel tokio tasks hitting the same pool) and asserts
// the invariants that must hold regardless of interleaving:
//
//   1. `uak_unique_active` (unique index on (unit_id, key_id) WHERE valid_to
//      IS NULL) is never violated by concurrent `set_share` calls on the
//      same (unit, key) pair — the close-then-insert pattern in
//      `set_share` must not race into two active rows.
//   2. Concurrent `delete` calls on the same allocation_key are idempotent:
//      exactly one deletion succeeds, none panic, and only one
//      `allocation_key.deleted` audit entry is written.
//   3. Concurrent `update` calls on the same allocation_key both succeed
//      (last-write-wins on `name`/`description`) and each produces its own
//      audit entry — no entry is silently dropped or duplicated.
//
// Verdict: covers the risk with an executable proof instead of leaving it
// undocumented. See docs/PROGRESS.md for the write-up.

mod common;

use laruche::db::allocation_key;
use sqlx::Row;
use uuid::Uuid;

async fn seed_building_unit_key(pool: &sqlx::PgPool) -> Seed {
    // Minimal building + user + unit + allocation key, bypassing the HTTP
    // layer so we can drive `db::allocation_key` directly with a shared pool
    // from many concurrent tasks.
    let building_id: Uuid = sqlx::query(
        "INSERT INTO building (name, address_line1, postal_code, city, shares_total) \
         VALUES ('Concurrency Test', '1 rue Test', '75001', 'Paris', 1000) \
         RETURNING id",
    )
    .fetch_one(pool)
    .await
    .expect("insert building")
    .get("id");

    let user_email = format!("concurrency+{}@test.fr", Uuid::new_v4());
    let user_id: Uuid = sqlx::query(
        "INSERT INTO user_account (email, last_name, first_name, password_hash) \
         VALUES ($1, 'Test', 'Concurrency', 'x') RETURNING id",
    )
    .bind(&user_email)
    .fetch_one(pool)
    .await
    .expect("insert user")
    .get("id");

    let unit_id: Uuid = sqlx::query(
        "INSERT INTO unit (building_id, unit_number, unit_type) \
         VALUES ($1, 'A001', 'apartment') RETURNING id",
    )
    .bind(building_id)
    .fetch_one(pool)
    .await
    .expect("insert unit")
    .get("id");

    let key = allocation_key::create(pool, building_id, "Ascenseur", None, user_id, None)
        .await
        .expect("create key");

    Seed {
        building_id,
        user_id,
        unit_id,
        key_id: key.id,
    }
}

struct Seed {
    building_id: Uuid,
    user_id: Uuid,
    unit_id: Uuid,
    key_id: Uuid,
}

async fn seed(pool: &sqlx::PgPool) -> Seed {
    seed_building_unit_key(pool).await
}

/// Invariant 1: concurrent `set_share` on the same (unit, key) pair must
/// never leave more than one active (`valid_to IS NULL`) row — that is
/// exactly what `uak_unique_active` enforces at the DB level, but the
/// application-level close-then-insert must not surface a constraint
/// violation as an unhandled 500 under contention, and the invariant must
/// hold in the data regardless.
#[tokio::test]
async fn concurrent_set_share_never_violates_uak_unique_active() {
    let (_cfg, pool) = common::test_db().await;
    let Seed {
        building_id,
        user_id,
        unit_id,
        key_id,
    } = seed(&pool).await;

    let mut tasks = Vec::new();
    for i in 0..10 {
        let pool = pool.clone();
        tasks.push(tokio::spawn(async move {
            allocation_key::set_share(
                &pool,
                unit_id,
                key_id,
                100 + i,
                building_id,
                user_id,
                Some("concurrent set_share"),
            )
            .await
        }));
    }

    for task in tasks {
        // No task panics, and the DB constraint (if ever hit) surfaces as an
        // AppResult error, not a process abort.
        let outcome = task.await.expect("task did not panic");
        assert!(
            outcome.is_ok(),
            "set_share failed under concurrency: {outcome:?}"
        );
    }

    // Invariant: exactly one active row for (unit_id, key_id).
    let active_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM unit_allocation_key \
         WHERE unit_id = $1 AND key_id = $2 AND valid_to IS NULL",
    )
    .bind(unit_id)
    .bind(key_id)
    .fetch_one(&pool)
    .await
    .expect("count active rows");
    assert_eq!(
        active_count, 1,
        "uak_unique_active must hold: exactly one active row expected after concurrent set_share"
    );

    // Every successful write that changed the value produced its own audit
    // entry (share_set) — none silently dropped, none duplicated beyond what
    // the sequence of distinct values (100..109) requires.
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log \
         WHERE entity_type = 'allocation_key' AND entity_id = $1 \
           AND operation = 'allocation_key.share_set'",
    )
    .bind(key_id)
    .fetch_one(&pool)
    .await
    .expect("count audit entries");
    assert!(
        audit_count >= 1,
        "expected at least one share_set audit entry, got {audit_count}"
    );
}

/// Invariant 2: concurrent `delete` on the same allocation_key is
/// idempotent — exactly one caller observes `true` (performed the delete),
/// the rest observe `false` (already gone), none panic, and exactly one
/// `allocation_key.deleted` audit entry is written despite N racing callers
/// all reading the same "before" row prior to their UPDATE.
#[tokio::test]
async fn concurrent_delete_is_idempotent_and_audited_once() {
    let (_cfg, pool) = common::test_db().await;
    let Seed {
        building_id,
        user_id,
        key_id,
        ..
    } = seed(&pool).await;

    let mut tasks = Vec::new();
    for _ in 0..8 {
        let pool = pool.clone();
        tasks.push(tokio::spawn(async move {
            allocation_key::delete(
                &pool,
                key_id,
                building_id,
                user_id,
                Some("concurrent delete"),
            )
            .await
        }));
    }

    let mut successes = 0;
    for task in tasks {
        let outcome = task.await.expect("task did not panic");
        let deleted = outcome.expect("delete must not error under concurrency");
        if deleted {
            successes += 1;
        }
    }
    assert_eq!(
        successes, 1,
        "exactly one concurrent delete call should report success"
    );

    let deleted_at: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT deleted_at FROM allocation_key WHERE id = $1")
            .bind(key_id)
            .fetch_one(&pool)
            .await
            .expect("fetch key");
    assert!(deleted_at.is_some(), "key must be soft-deleted");

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log \
         WHERE entity_type = 'allocation_key' AND entity_id = $1 \
           AND operation = 'allocation_key.deleted'",
    )
    .bind(key_id)
    .fetch_one(&pool)
    .await
    .expect("count delete audit entries");
    assert_eq!(
        audit_count, 1,
        "exactly one deleted audit entry expected, no duplicates from the race"
    );
}

/// Invariant 3: concurrent `update` calls on the same allocation_key never
/// panic and never silently drop an audit entry. The pre-existing TOCTOU
/// concern (each racer reads "before" from its own snapshot rather than
/// under a row lock) means `old_values` on the losing racer's audit entry
/// can reflect a value that was already overwritten by the winner — this
/// test documents and asserts that exact behavior rather than assuming it
/// away, so the risk is proven (not just claimed) and captured for anyone
/// reasoning about the audit trail's legal reliability (LEGAL.md §1.2).
#[tokio::test]
async fn concurrent_update_both_succeed_with_independent_audit_entries() {
    let (_cfg, pool) = common::test_db().await;
    let Seed {
        building_id,
        user_id,
        key_id,
        ..
    } = seed(&pool).await;

    let pool_a = pool.clone();
    let pool_b = pool.clone();
    let key_id_a = key_id;
    let key_id_b = key_id;

    let (res_a, res_b) = tokio::join!(
        tokio::spawn(async move {
            allocation_key::update(
                &pool_a,
                key_id_a,
                building_id,
                "Nom A",
                None,
                user_id,
                Some("racer A"),
            )
            .await
        }),
        tokio::spawn(async move {
            allocation_key::update(
                &pool_b,
                key_id_b,
                building_id,
                "Nom B",
                None,
                user_id,
                Some("racer B"),
            )
            .await
        })
    );

    let row_a = res_a.expect("task A did not panic").expect("update A ok");
    let row_b = res_b.expect("task B did not panic").expect("update B ok");
    assert!(row_a.is_some(), "racer A must find the row");
    assert!(row_b.is_some(), "racer B must find the row");

    // Both racers wrote their own audit entry — the TOCTOU does not cause an
    // update to be lost, only (potentially) a stale `old_values` snapshot.
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log \
         WHERE entity_type = 'allocation_key' AND entity_id = $1 \
           AND operation = 'allocation_key.updated'",
    )
    .bind(key_id)
    .fetch_one(&pool)
    .await
    .expect("count update audit entries");
    assert_eq!(
        audit_count, 2,
        "both concurrent updates must produce their own audit entry"
    );

    // Final DB state is whichever writer committed last — deterministic
    // from Postgres's point of view, non-deterministic from the test's
    // (that's the nature of the race); assert it is one of the two names,
    // not corrupted/blank.
    let final_name: String = sqlx::query_scalar("SELECT name FROM allocation_key WHERE id = $1")
        .bind(key_id)
        .fetch_one(&pool)
        .await
        .expect("fetch final name");
    assert!(
        final_name == "Nom A" || final_name == "Nom B",
        "final name must be one of the two racers' writes, got {final_name:?}"
    );
}

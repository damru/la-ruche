mod common;

use uuid::Uuid;

#[tokio::test]
async fn archiving_unit_closes_active_allocation_keys() {
    let (_cfg, pool) = common::test_db().await;

    let building_id = Uuid::new_v4();
    let unit_id = Uuid::new_v4();
    let key_id = Uuid::new_v4();
    let unit_number = format!("A-{}", &Uuid::new_v4().simple().to_string()[..8]);
    let key_name = format!("Archive key {}", &Uuid::new_v4().simple().to_string()[..8]);

    sqlx::query(
        r#"INSERT INTO building (id, name, address_line1, postal_code, city, shares_total)
           VALUES ($1, 'Residence Archive Test', '1 rue Archive', '75001', 'Paris', 1000)"#,
    )
    .bind(building_id)
    .execute(&pool)
    .await
    .expect("insert building");

    sqlx::query(
        r#"INSERT INTO unit (id, building_id, unit_number, unit_type)
           VALUES ($1, $2, $3, 'apartment'::unit_type)"#,
    )
    .bind(unit_id)
    .bind(building_id)
    .bind(&unit_number)
    .execute(&pool)
    .await
    .expect("insert unit");

    sqlx::query(
        r#"INSERT INTO allocation_key (id, building_id, name, key_type)
           VALUES ($1, $2, $3, 'special'::allocation_key_type)"#,
    )
    .bind(key_id)
    .bind(building_id)
    .bind(&key_name)
    .execute(&pool)
    .await
    .expect("insert allocation key");

    sqlx::query(
        r#"INSERT INTO unit_allocation_key (unit_id, key_id, share_value)
           VALUES ($1, $2, 125)"#,
    )
    .bind(unit_id)
    .bind(key_id)
    .execute(&pool)
    .await
    .expect("insert unit allocation key");

    laruche::db::unit::archive(&pool, unit_id, building_id)
        .await
        .expect("archive unit");

    let archived: bool = sqlx::query_scalar("SELECT active FROM unit WHERE id = $1")
        .bind(unit_id)
        .fetch_one(&pool)
        .await
        .expect("fetch unit active flag");
    assert!(!archived, "archiving a unit must set active = false");

    let active_allocations: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM unit_allocation_key WHERE unit_id = $1 AND valid_to IS NULL",
    )
    .bind(unit_id)
    .fetch_one(&pool)
    .await
    .expect("count active allocation keys");
    assert_eq!(active_allocations, 0);

    let closed_allocations: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM unit_allocation_key WHERE unit_id = $1 AND valid_to IS NOT NULL",
    )
    .bind(unit_id)
    .fetch_one(&pool)
    .await
    .expect("count closed allocation keys");
    assert_eq!(closed_allocations, 1);
}

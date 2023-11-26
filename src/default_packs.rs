use sqlx::{Pool, Postgres};

use crate::Error;

pub async fn create_default_packs(conn: &Pool<Postgres>) -> Result<(), Error> {
    let _ = sqlx::query!("INSERT INTO packs(id, price, common_chance, rare_chance, epic_chance, legendary_chance, mythic_chance, awakened_chance) VALUES ('pack', 1000, 600, 250, 100, 35, 10, 5)").execute(conn).await;
    Ok(())
}
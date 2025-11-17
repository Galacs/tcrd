use sqlx::{Pool, Postgres};

use crate::Error;


pub async fn create_user_balance(user_id: i32, conn: &Pool<Postgres>) -> Result<(), Error> {
    sqlx::query!("INSERT INTO balances(user_id) VALUES ($1)", user_id.to_string()).execute(conn).await?;
    Ok(())
}

pub async fn create_user_stats(user_id: i32, conn: &Pool<Postgres>) -> Result<(), Error> {
    sqlx::query!("INSERT INTO user_stats(user_id) VALUES ($1)", user_id.to_string()).execute(conn).await?;
    Ok(())
}


// Returns true if user wasn't in the db
pub async fn exists_or_create_user(user_id: i32, conn: &Pool<Postgres>) -> Result<bool, Error> {
    let mut bool = false;
    if sqlx::query!("SELECT * FROM balances WHERE user_id = $1", user_id.to_string()).fetch_one(conn).await.is_err() {
        create_user_balance(user_id, conn).await?;
        bool = true;
    };
    if sqlx::query!("SELECT * FROM user_stats WHERE user_id = $1", user_id.to_string()).fetch_one(conn).await.is_err() {
        create_user_stats(user_id, conn).await?;
        bool = true;
    };
    Ok(bool)
}

use sqlx::{Pool, Sqlite};

use crate::Error;


pub async fn create_user(user_id: i64, conn: &Pool<Sqlite>) -> Result<(), Error> {
    sqlx::query!("INSERT INTO balances(user_id) VALUES ($1)", user_id).execute(conn).await?;
    Ok(())
}

// Returns true if user wasn't in the db
pub async fn exists_or_create_user(user_id: i64, conn: &Pool<Sqlite>) -> Result<bool, Error> {
    let Err(_) = sqlx::query!("SELECT * FROM balances WHERE user_id = $1", user_id).fetch_one(conn).await else {
        return Ok(false);
    };
    create_user(user_id, conn).await?;
    Ok(true)
}
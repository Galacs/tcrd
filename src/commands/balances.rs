use crate::{Context, Error};

/// You can run this every hour to win 200 Belly
#[poise::command(slash_command, prefix_command)]
pub async fn hourly(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    let user_id = ctx.author().id.0 as i64;
    crate::create_user::exists_or_create_user(user_id, conn).await?;
    let Ok(rows) = sqlx::query!("SELECT last_hourly FROM balances WHERE user_id = $1", user_id.to_string()).fetch_one(conn).await else {
        return Ok(());
    };

    let last_hourly = chrono::DateTime::from_timestamp(rows.last_hourly, 0).unwrap();

    let duration = chrono::offset::Utc::now().signed_duration_since(last_hourly).num_seconds();

    if duration < 60*60 {
        ctx.say(format!("Cooldown reached, you must wait {}s", 60*60 - duration)).await?;
    } else {
        ctx.say("You won 200").await?;
        sqlx::query!("UPDATE balances SET balance = balance + 200 WHERE user_id = $1", user_id.to_string()).execute(conn).await?;
        sqlx::query!("UPDATE balances SET last_hourly = extract(epoch from now()) WHERE user_id = $1", user_id.to_string()).execute(conn).await?;
    }
    Ok(())
}

/// You can run this every day to win 1000 Belly
#[poise::command(slash_command, prefix_command)]
pub async fn daily(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    let user_id = ctx.author().id.0 as i64;
    crate::create_user::exists_or_create_user(user_id, conn).await?;
    let Ok(rows) = sqlx::query!("SELECT last_daily FROM balances WHERE user_id = $1", user_id.to_string()).fetch_one(conn).await else {
        return Ok(());
    };

    let last_daily = chrono::DateTime::from_timestamp(rows.last_daily, 0).unwrap();

    let duration = chrono::offset::Utc::now().signed_duration_since(last_daily).num_seconds();

    if duration < 60*60*24 {
        ctx.say(format!("Cooldown reached, you must wait {}s", 60*60*24 - duration)).await?;
    } else {
        ctx.say("You just won 1000").await?;
        sqlx::query!("UPDATE balances SET balance = balance + 1000 WHERE user_id = $1", user_id.to_string()).execute(conn).await?;
        sqlx::query!("UPDATE balances SET last_daily = extract(epoch from now()) WHERE user_id = $1", user_id.to_string()).execute(conn).await?;
    }
    Ok(())
}

/// Gets your balance
#[poise::command(slash_command, prefix_command)]
pub async fn balance(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    let user_id = ctx.author().id.0 as i64;
    crate::create_user::exists_or_create_user(user_id, conn).await?;
    let Ok(row) = sqlx::query!("SELECT balance FROM balances WHERE user_id = $1", user_id.to_string()).fetch_one(conn).await else {
        return Ok(());
    };

    ctx.say(format!("Your current balance is {} Belly", row.balance)).await?;
    Ok(())
}


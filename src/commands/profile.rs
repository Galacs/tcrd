use crate::{Context, Error};


/// Gets your stats
#[poise::command(slash_command, prefix_command)]
pub async fn profile(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    let user_id = ctx.author().id.0 as i64;
    crate::create_user::exists_or_create_user(user_id, conn).await?;
    let stats = sqlx::query!("SELECT user_stats.game_won,user_stats.game_lost,balances.balance FROM user_stats INNER JOIN balances ON user_stats.user_id=balances.user_id WHERE user_stats.user_id=$1", user_id).fetch_one(conn).await?;
    let cards_number = sqlx::query!("SELECT count(user_id) AS count FROM users_cards WHERE user_id=$1", user_id).fetch_one(conn).await?;
    ctx.send(|b| b.embed(|e| {
        e.title(format!("@{}'s stats", ctx.author().tag()))
        .field("", &format!(
            "**Total Wins:** {} games
            **Total Lost:** {} games
            **Total card owned:** {} cards
            **Money:** {} Belly",
            stats.game_won, stats.game_lost, cards_number.count, stats.balance
        ), false)
    })).await?;
    
    Ok(())
}
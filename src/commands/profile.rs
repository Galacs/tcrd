use crate::{Context, Error};


/// Provides the user's statistics, containing their wins, losses, number of cards and current currency
#[poise::command(slash_command, prefix_command)]
pub async fn profile(
    ctx: Context<'_>,
    #[description = "User you want to see the profile of"] player: Option<poise::serenity_prelude::User>,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    let user = player.unwrap_or(ctx.author().clone());
    let user_id = user.id.0 as i64;
    let username = match user.discriminator {
        0000 => user.name,
        _ => user.tag(),
    };
    crate::create_user::exists_or_create_user(user_id, conn).await?;
    let user_id = user_id.to_string();
    let stats = sqlx::query!("SELECT user_stats.game_won,user_stats.game_lost,balances.balance FROM user_stats INNER JOIN balances ON user_stats.user_id=balances.user_id WHERE user_stats.user_id=$1", user_id).fetch_one(conn).await?;
    let cards_number = sqlx::query!("SELECT count(user_id) AS count FROM users_cards WHERE user_id=$1", user_id).fetch_one(conn).await?;
    ctx.send(|b| b.embed(|e| {
        e.title(format!("@{}'s stats", username))
        .field("", &format!(
            "**Total Wins:** {} games
            **Total Lost:** {} games
            **Total card owned:** {} cards
            **Money:** {} Belly",
            stats.game_won, stats.game_lost, cards_number.count.unwrap(), stats.balance
        ), false)
    })).await?;
    
    Ok(())
}
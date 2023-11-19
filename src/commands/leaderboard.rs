use crate::{Context, Error};


/// Get the top 20 players with the most game won
#[poise::command(slash_command, prefix_command)]
pub async fn leaderboards(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    let rows = sqlx::query!("SELECT * FROM user_stats ORDER BY game_won DESC LIMIT 20").fetch_all(conn).await?;
    let mut leader_str = String::new();

    for (id, row) in rows.iter().enumerate() {
        leader_str.push_str(&format!("{}. <@{}>: {} games won, {} games lost\n", id + 1, row.user_id, row.game_won, row.game_lost));
    }

    ctx.send(|b| b.embed(|e| {
        e.title("Top 20")
        .field("", leader_str, false)
    }).allowed_mentions(|m| m.empty_users())).await?;
    Ok(())
}
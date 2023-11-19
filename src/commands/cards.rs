use std::str::FromStr;

use poise::serenity_prelude::CreateEmbed;

use crate::{Context, Error, cards::card::{Card, Rarity, UserCard, Type}, paginate_cards, create_card_embed};

pub fn create_user_card_embed(e: &mut CreateEmbed, card: Card, user_card: UserCard) -> &mut CreateEmbed {
    create_card_embed(e, card)
    .field("", &format!(
        "**Owned:** {}",
        user_card.count
    ), false)
}

/// Lists your own cards
#[poise::command(prefix_command, slash_command)]
pub async fn cards(ctx: Context<'_>) -> Result<(), Error> {
    let conn = &ctx.data().0;
    let user_id = ctx.author().id.0 as i64;
    let rows = sqlx::query!("SELECT *,count(card_id) AS count FROM users_cards INNER JOIN cards ON users_cards.card_id = cards.id WHERE user_id=$1 GROUP BY card_id", user_id).fetch_all(conn).await?;
    if rows.is_empty() {
        ctx.say("Can't find any cards").await?;
        return Ok(());
    }
    // ctx.say("the card was created").await?;
    let rows: Vec<(Card, UserCard)> = rows.iter().map(|row| {
        (Card {
            id: row.id.clone(),
            rarity: Rarity::from_str(&row.rarity).unwrap(),
            kind: Type::from_str(&row.kind).unwrap(),
            description: row.description.clone(),
            hp: row.hp as i32,
            damage: row.damage as i32,
            defense: row.defense as i32,
        }, UserCard { count: row.count as i32 })
    }).collect();

    let (cards, user_cards): (Vec<_>, Vec<_>) = rows.iter().cloned().unzip();

    paginate_cards::paginate(ctx, cards, Some(user_cards)).await?;
    Ok(())
}
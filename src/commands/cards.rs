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
    let user_id = ctx.author().id.0.to_string();
    let rows = sqlx::query!("SELECT *,count(card_id) AS count FROM users_cards INNER JOIN cards ON users_cards.card_id = cards.id WHERE user_id=$1 GROUP BY card_id, user_id, id", user_id).fetch_all(conn).await?;
    if rows.is_empty() {
        ctx.say("Can't find any cards").await?;
        return Ok(());
    }
    // ctx.say("the card was created").await?;
    let rows: Vec<(Card, UserCard)> = rows.iter().map(|row| {
        (Card {
            id: row.id.clone(),
            extension: row.image_extension.clone(),
            rarity: Rarity::from_str(&row.rarity).unwrap(),
            kind: Type::from_str(&row.kind).unwrap(),
            description: row.description.clone(),
            hp: row.hp,
            damage: row.damage,
            defense: row.defense,
        }, UserCard { count: row.count.ok_or("no count").unwrap() })
    }).collect();

    let (cards, user_cards): (Vec<_>, Vec<_>) = rows.iter().cloned().unzip();

    paginate_cards::paginate(ctx, cards, Some(user_cards)).await?;
    Ok(())
}
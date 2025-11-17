use std::str::FromStr;

use poise::serenity_prelude::CreateEmbed;

use crate::{
    cards::card::{Card, Rarity, Type, UserCard},
    create_card_embed, paginate_cards, Context, Error,
};

pub fn create_user_card_embed(
    e: &mut CreateEmbed,
    card: Card,
    user_card: UserCard,
) -> &mut CreateEmbed {
    create_card_embed(e, card).field("", &format!("**Owned:** {}", user_card.count), false)
}

/// Lists your own cards
#[poise::command(prefix_command, slash_command)]
pub async fn cards(
    ctx: Context<'_>,
    #[description = "User you want to see the cards of"] player: Option<
        poise::serenity_prelude::User,
    >,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    let user = player.unwrap_or(ctx.author().clone());
    let user_id = user.id.0 as i32;
    let username = match user.discriminator {
        0000 => user.name,
        _ => user.tag(),
    };
    let rows = sqlx::query!("SELECT *,count(card_id) AS count FROM users_cards INNER JOIN cards ON users_cards.card_id = cards.id WHERE user_id=$1 GROUP BY card_id, user_id, id", user_id.to_string()).fetch_all(conn).await?;
    if rows.is_empty() {
        ctx.say("Can't find any cards").await?;
        return Ok(());
    }

    let rows: Vec<(Card, UserCard)> = rows
        .iter()
        .map(|row| {
            (
                Card {
                    id: row.id.clone(),
                    extension: row.image_extension.clone(),
                    rarity: Rarity::from_str(&row.rarity).unwrap(),
                    kind: Type::from_str(&row.kind).unwrap(),
                    description: row.description.clone(),
                    hp: row.hp,
                    damage: row.damage,
                    defense: row.defense,
                    obtainable: row.obtainable,
                },
                UserCard {
                    count: row.count.ok_or("no count").unwrap() as i32,
                },
            )
        })
        .collect();

    let (cards, user_cards): (Vec<_>, Vec<_>) = rows.iter().cloned().unzip();

    ctx.say(format!("Here are {}'s cards", username)).await?;
    paginate_cards::paginate(ctx, cards, Some(user_cards)).await?;
    Ok(())
}

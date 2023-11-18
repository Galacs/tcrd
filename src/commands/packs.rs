use std::str::FromStr;

use crate::{Context, Error, cards::card::{Rarity, Card}, commands::cards::give_card_to_user};
use rand::Rng;
use sqlx::{Sqlite, Pool};

#[poise::command(slash_command, prefix_command)]
pub async fn pack(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    let user_id = ctx.author().id.0 as i64;
    crate::create_user::exists_or_create_user(user_id, conn).await?;
    let Ok(row) = sqlx::query!("SELECT balance FROM balances WHERE user_id = $1", user_id).fetch_one(conn).await else {
        return Ok(());
    };

    if row.balance < 1000 {
        ctx.say("You do not have enough Belly to buy a pack, you need 1000 Belly").await?;
        return Ok(());
    }

    sqlx::query!("UPDATE balances SET balance = balance - 1000").execute(conn).await?;
    
    let pack = sqlx::query!("SELECT common_chance, rare_chance, epic_chance, legendary_chance, mythic_chance, awakened_chance FROM packs").fetch_one(conn).await?;
    
    let number = rand::thread_rng().gen_range(0..1000);

    async fn get_random_card(conn: &Pool<Sqlite>, rarity: Rarity) -> Result<Card, Error> {
        let rarity_str = rarity.to_string();
        let row = sqlx::query!("SELECT * from cards WHERE rarity=$1 ORDER BY RANDOM() LIMIT 1", rarity_str).fetch_one(conn).await?;
        let card = Card {
            id: row.id,
            rarity: Rarity::from_str(&row.rarity).unwrap(),
            description: row.description,
            hp: row.hp as i32,
            damage: row.damage as i32,
            defense: row.defense as i32,
        };
        Ok(card)
    }

    let card = if (pack.awakened_chance..=pack.mythic_chance).contains(&number) {
        ctx.say("Congratulations you won an Awakened card").await?;
        get_random_card(conn, Rarity::Awakened).await?
    } else if (pack.mythic_chance..=pack.legendary_chance).contains(&number) {
        ctx.say("Congratulations you won a Mythic card").await?;
        get_random_card(conn, Rarity::Mythic).await?
    } else if (pack.legendary_chance..=pack.epic_chance).contains(&number) {
        ctx.say("Congratulations you won an Legendary card").await?;
        get_random_card(conn, Rarity::Legendary).await?
    } else if (pack.epic_chance..=pack.rare_chance).contains(&number) {
        ctx.say("Congratulations you won a Epic card").await?;
        get_random_card(conn, Rarity::Epic).await?
    } else if (pack.rare_chance..=pack.common_chance).contains(&number) {
        ctx.say("Congratulations you won a Rare card").await?;
        get_random_card(conn, Rarity::Rare).await?
    } else {
        ctx.say("Congratulations you won a Common card").await?;
        get_random_card(conn, Rarity::Common).await?
    };

    ctx.send(|b| {
        b.embed(|e| {
            crate::create_card_embed(e, card.clone())
        })
    }).await?;

    if !give_card_to_user(conn, &card.id, ctx.author().id.0).await? {
        ctx.reply("The card wasn't added to your collection as your already have three of them").await?;
    }

    Ok(())
}


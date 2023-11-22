use std::{str::FromStr, path::PathBuf};
use poise::serenity_prelude as serenity;
use futures::{Stream, StreamExt};
use sqlx::{Row, Pool, Sqlite};
use crate::{cards::card::{Rarity, Card, Type, FightCard}, Context, Error, create_card_embed, paginate_cards};


/// Admin commands used to manage cards and debug
#[poise::command(
    prefix_command,
    slash_command,
    hide_in_help,
    owners_only,
    subcommands("create", "get", "list", "delete", "give", "fight", "stats", "balance")
)]
pub async fn manage(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, prefix_command, subcommands("add", "set", "clear"))]
pub async fn balance(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn add(
    ctx: Context<'_>,
    #[description = "Amount"] amount: i32,
    #[description = "User"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    let user = user.unwrap_or(ctx.author().clone());
    let user_id = user.id.0 as i64;
    sqlx::query!("UPDATE balances SET balance = balance + $2 WHERE user_id = $1", user_id, amount).execute(conn).await?;
    ctx.say(format!("Added {} Belly to <@{}>'s balance", amount, user_id)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn set(
    ctx: Context<'_>,
    #[description = "amount"] amount: i32,
    #[description = "User"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    let user = user.unwrap_or(ctx.author().clone());
    let user_id = user.id.0 as i64;
    sqlx::query!("UPDATE balances SET balance = $2 WHERE user_id = $1", user_id, amount).execute(conn).await?;
    ctx.say(format!("Set <@{}>'s balance to {} Belly", user_id, amount)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn clear(
    ctx: Context<'_>,
    #[description = "User"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    let user = user.unwrap_or(ctx.author().clone());
    let user_id = user.id.0 as i64;
    sqlx::query!("UPDATE balances SET balance = 0 WHERE user_id = $1", user_id).execute(conn).await?;
    ctx.say(format!("Cleared <@{}>'s balance", user_id)).await?;
    Ok(())
}

/// Creates new cards
#[allow(clippy::too_many_arguments)]
#[poise::command(slash_command, prefix_command)]
async fn create(
    ctx: Context<'_>,
    #[description = "ID"] id: String,
    #[description = "Rarity"] rarity: Rarity,
    #[description = "Type"] kind: Type,
    #[description = "Description"] description: String,
    #[description = "HP"] hp: i32,
    #[description = "Damage"] damage: i32,
    #[description = "Defense in %"] defense: i32,
    #[description = "Image"] image: serenity::Attachment,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    let filepath = PathBuf::from_str(&image.filename)?;
    let extension = filepath.extension().ok_or("file extension error")?.to_str().ok_or("file extension error")?;
    if (sqlx::query!("INSERT INTO cards(id, image_extension, rarity, kind, description, hp, damage, defense) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)", id, extension, rarity, kind, description, hp, damage, defense).execute(conn).await).is_err() {
        ctx.say("A similar card already exists").await?;
        return Ok(());
    }

    let bucket = &ctx.data().2;
    let image_bytes = reqwest::get(image.url).await?.bytes().await?;
    let mut cursor = std::io::Cursor::new(image_bytes);
    bucket.put_object_stream(&mut cursor, format!("{}.{}", &id, &extension)).await?;

    ctx.say("the card was created").await?;
    let card = Card { id, extension: extension.to_owned(), rarity, kind, description, hp, damage, defense };
    ctx.send(|b| b.embed(|e| create_card_embed(e, card))).await?;
    Ok(())
}

/// Lists all the cards in the database
#[poise::command(slash_command, prefix_command)]
async fn list(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    let rows= sqlx::query!("SELECT * FROM cards").fetch_all(conn).await?;
    if rows.is_empty() {
        ctx.say("Can't find any cards").await?;
        return Ok(());
    }
    // ctx.say("the card was created").await?;
    let cards = rows.iter().map(|row| {
        Card {
            id: row.id.clone(),
            extension: row.image_extension.clone(),
            rarity: Rarity::from_str(&row.rarity).unwrap(),
            kind: Type::from_str(&row.kind).unwrap(),
            description: row.description.clone(),
            hp: row.hp as i32,
            damage: row.damage as i32,
            defense: row.defense as i32,
        }
    }).collect();
    paginate_cards::paginate(ctx, cards, None).await?;
    Ok(())
}

/// Get bot stats
#[poise::command(slash_command, prefix_command)]
async fn stats(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    let users = sqlx::query!("SELECT count(user_id) as count FROM user_stats").fetch_one(conn).await?;
    let cards = sqlx::query!("SELECT count(id) as count FROM cards").fetch_one(conn).await?;
    let player_cards = sqlx::query!("SELECT count(card_id) as count FROM users_cards").fetch_one(conn).await?;
    let game_won = sqlx::query!("SELECT SUM(game_won) AS count FROM user_stats").fetch_one(conn).await?;

    ctx.send(|b| b.embed(|e| {
        e.title("Statistics")
        .field("Number registed users", users.count, false)
        .field("Number of unique cards", cards.count, false)
        .field("Number of cards owned by players", player_cards.count, false)
        .field("Number of fight played", game_won.count.unwrap_or(0), false)
    })).await?;
    Ok(())
}

/// Gets information about a specific card
#[poise::command(slash_command, prefix_command)]
async fn get(
    ctx: Context<'_>,
    #[description = "ID"]
    #[autocomplete = "autocomplete_card_id"]
    id: String,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    let Ok(row) = sqlx::query!("SELECT * FROM cards WHERE id=$1", id).fetch_one(conn).await else {
        ctx.say("Can't find card").await?;
        return Ok(());
    };
    let card = Card {
        id: row.id,
        extension: row.image_extension,
        rarity: Rarity::from_str(&row.rarity).unwrap(),
        kind: Type::from_str(&row.kind).unwrap(),
        description: row.description,
        hp: row.hp as i32,
        damage: row.damage as i32,
        defense: row.defense as i32,
    };
    ctx.send(|b| {
        b.embed(|e| {
            create_card_embed(e, card)
        })
    }).await?;
    Ok(())
}

pub async fn check_card(conn: &Pool<Sqlite>, card_id: &String) -> Result<bool, Error> {
    if (sqlx::query!("SELECT * FROM cards WHERE id=$1", card_id).fetch_one(conn).await).is_ok() {
        return Ok(true);
    };
    Ok(false)
}

pub async fn id_to_fight_card(conn: &Pool<Sqlite>, card_id: &String) -> Result<FightCard, Error> {
    let row = sqlx::query!("SELECT * FROM cards WHERE id=$1", card_id).fetch_one(conn).await?;
    Ok(FightCard {
        id: row.id,
        hp: row.hp as i32,
        defense: row.defense as i32,
        damage: row.damage as i32,
    })
}

/// Tests fighting against arbitrary cards
#[allow(clippy::too_many_arguments)]
#[poise::command(slash_command, prefix_command)]
async fn fight(
    ctx: Context<'_>,
    #[description = "Player A card 1"] #[autocomplete = "autocomplete_card_id"] id1: String,
    #[description = "Player A card 2"] #[autocomplete = "autocomplete_card_id"] id2: Option<String>,
    #[description = "Player A card 3"] #[autocomplete = "autocomplete_card_id"] id3: Option<String>,
    #[description = "Player A card 4"] #[autocomplete = "autocomplete_card_id"] id4: Option<String>,
    #[description = "Player A card 5"] #[autocomplete = "autocomplete_card_id"] id5: Option<String>,

    #[description = "Player B card 1"] #[autocomplete = "autocomplete_card_id"] id6: String,
    #[description = "Player B card 2"] #[autocomplete = "autocomplete_card_id"] id7: Option<String>,
    #[description = "Player B card 3"] #[autocomplete = "autocomplete_card_id"] id8: Option<String>,
    #[description = "Player B card 4"] #[autocomplete = "autocomplete_card_id"] id9: Option<String>,
    #[description = "Player B card 5"] #[autocomplete = "autocomplete_card_id"] id10:Option<String>,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    let id1 = Some(id1);
    let id6 = Some(id6);
    for id in [&id1, &id2, &id3, &id4, &id5, &id6, &id7, &id8, &id9, &id10] {
        let Some(id) = id else {
            continue;
        };
        if !check_card(conn, id).await? {
            ctx.say(format!("{} card is not valid", id)).await?;
            return Ok(())
        }
    }

    let a = vec![id1, id2, id3, id4, id5];
    let player_a_cards_id_iter = a.iter().flatten();
    let b = vec![id6, id7, id8, id9, id10];
    let player_b_cards_id_iter= b.iter().flatten();

    let mut player_a_cards: Vec<FightCard> = Vec::new();
    for id in player_a_cards_id_iter {
        player_a_cards.push(id_to_fight_card(conn, id).await?);
    }

    let mut player_b_cards: Vec<FightCard> = Vec::new();
    for id in player_b_cards_id_iter {
        player_b_cards.push(id_to_fight_card(conn, id).await?);
    }

    crate::commands::fight::fight_two_players(&ctx, 0, &mut player_a_cards, 1, &mut player_b_cards).await?;

    Ok(())
}

pub async fn give_card_to_user(conn: &Pool<Sqlite>, card_id: &String, user_id: u64) -> Result<bool, Error> {
    let card_limit = 3;
    let user_id = user_id as i64;
    if let Ok(rows) = sqlx::query!("SELECT user_id FROM users_cards WHERE user_id=$1 AND card_id=$2", user_id, card_id).fetch_all(conn).await {
        if rows.len() >= card_limit {
            return Ok(false)
        }
    }
    sqlx::query!("INSERT INTO users_cards(user_id, card_id) VALUES ($1, $2)", user_id, card_id).execute(conn).await?;
    Ok(true)
}

/// Give scard to an user or invoker
#[poise::command(slash_command, prefix_command)]
pub async fn give(
    ctx: Context<'_>,
    #[description = "ID"]
    #[autocomplete = "autocomplete_card_id"]
    id: String,
    #[description = "User"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let user = match &user {
        Some(u) => u,
        None => ctx.author()
    };
    let conn = &ctx.data().0;
    let Ok(row) = sqlx::query!("SELECT * FROM cards WHERE id=$1", id).fetch_one(conn).await else {
        ctx.say("Can't find that card").await?;
        return Ok(());
    };

    if !give_card_to_user(conn, &row.id, user.id.0).await? {
        ctx.say("Number of cards exceeded").await?;
        return Ok(());
    }

    let card = Card {
        id: row.id.clone(),
        extension: row.image_extension,
        rarity: Rarity::from_str(&row.rarity).unwrap(),
        kind: Type::from_str(&row.kind).unwrap(),
        description: row.description,
        hp: row.hp as i32,
        damage: row.damage as i32,
        defense: row.defense as i32,
    };
    ctx.send(|b| {
        b.embed(|e| {
            create_card_embed(e, card)
        })
    }).await?;
    ctx.say(format!("Gave {} to {}", row.id, user)).await?;
    Ok(())
}

/// Delete a card from the database
#[poise::command(slash_command, prefix_command)]
async fn delete(
    ctx: Context<'_>,
    #[description = "ID"]
    #[autocomplete = "autocomplete_card_id"]
    id: String,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    if let Ok(row) = sqlx::query!("SELECT image_extension FROM cards WHERE id=$1", id).fetch_one(conn).await {
        let bucket = &ctx.data().2;
        bucket.delete_object(format!("{}.{}", id, row.image_extension)).await?;
    }
    if let Ok(res) = sqlx::query!("DELETE FROM cards WHERE id=$1", id).execute(conn).await {
        if res.rows_affected() < 1 {
            ctx.say("Can't find card").await?;
            return Ok(())
        }
    }
    ctx.reply("Card was deleted").await?;
    Ok(())
}

pub async fn autocomplete_card_id<'a>(
    ctx: Context<'a>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let conn = &ctx.data().0;
    let match_str = format!("%{}%", partial);
    sqlx::query("SELECT id from cards WHERE id LIKE ?").bind(match_str).fetch(conn).map(|s| s.unwrap().try_get("id").unwrap())
}

pub async fn autocomplete_user_card_id<'a>(
    ctx: Context<'a>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let conn = &ctx.data().0;
    let match_str = format!("%{}%", partial);
    sqlx::query("SELECT cards.id FROM users_cards INNER JOIN cards ON users_cards.card_id = cards.id WHERE cards.id LIKE ? AND user_id=? GROUP BY card_id").bind(match_str).bind(ctx.author().id.0 as i64).fetch(conn).map(|s| s.unwrap().try_get("id").unwrap())
}
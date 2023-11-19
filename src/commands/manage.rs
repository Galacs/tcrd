use std::str::FromStr;
use poise::serenity_prelude as serenity;
use futures::{Stream, StreamExt};
use rand::Rng;
use sqlx::{Row, Pool, Sqlite};
use crate::{cards::card::{Rarity, Card, Type, FightCard}, Context, Error, create_card_embed, paginate_cards};


#[poise::command(
    prefix_command,
    slash_command,
    subcommands("create", "get", "list", "delete", "give", "fight")
)]
pub async fn manage(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

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
    #[description = "Defense"] defense: i32,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    if (sqlx::query!("INSERT INTO cards(id, rarity, kind, description, hp, damage, defense) VALUES ($1, $2, $3, $4, $5, $6, $7)", id, rarity, kind, description, hp, damage, defense).execute(conn).await).is_err() {
        ctx.say("A similar card already exists").await?;
        return Ok(());
    }
    ctx.say("the card was created").await?;
    let card = Card { id, rarity, kind, description, hp, damage, defense };
    ctx.send(|b| b.embed(|e| create_card_embed(e, card))).await?;
    Ok(())
}

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

    async fn id_to_fight_card(conn: &Pool<Sqlite>, card_id: &String) -> Result<FightCard, Error> {
        let row = sqlx::query!("SELECT * FROM cards WHERE id=$1", card_id).fetch_one(conn).await?;
        Ok(FightCard {
            id: row.id,
            hp: row.hp as i32,
            defense: row.defense as i32,
            damage: row.damage as i32,
        })
    }

    let mut player_a_cards: Vec<FightCard> = Vec::new();
    for id in player_a_cards_id_iter {
        player_a_cards.push(id_to_fight_card(conn, id).await?);
    }

    let mut player_b_cards: Vec<FightCard> = Vec::new();
    for id in player_b_cards_id_iter {
        player_b_cards.push(id_to_fight_card(conn, id).await?);
    }

    let mut turn_count = 0;
    // Fight happens now
    while !(player_a_cards.is_empty() || player_b_cards.is_empty()) {
        // Choose which card of the two players will fight
        let player_a_index = rand::thread_rng().gen_range(0..player_a_cards.len());
        let player_b_index = rand::thread_rng().gen_range(0..player_b_cards.len());
        let card_a = &mut player_a_cards[player_a_index];
        let card_b = &mut player_b_cards[player_b_index];

        // Which card attacks first ?
        let player_a_attacks = rand::thread_rng().gen_bool(0.5);
        let player_b_dodge = rand::thread_rng().gen_bool((card_b.defense as f32/100.0).into());
        let player_a_dodge = rand::thread_rng().gen_bool((card_a.defense as f32/100.0).into());
        if player_a_attacks {
            // Did player B dodge ?
            if player_b_dodge {
                ctx.say(format!("Player B's {} dodged {}'s attack with a {}% chance", card_b.id, card_a.id, card_b.defense)).await?;
            } else {
                card_b.hp -= card_a.damage;
                if card_b.hp <= 0 {
                    ctx.say(format!("Player A's {} dealt {} damage and killed Player's B {}", card_a.id, card_a.damage, card_b.id)).await?;
                    player_b_cards.remove(player_b_index);
                } else {
                    ctx.say(format!("Player A's {} dealt {} damage to Player's B {}. It now has {} hp", card_a.id, card_a.damage, card_b.id, card_b.hp)).await?;
                }
            }
        } else {
            // Did player A dodge ?
            if player_a_dodge {
                ctx.say(format!("Player A's {} dodged {}'s attack with a {}% chance", card_a.id, card_b.id, card_a.defense)).await?;
            } else {
                card_a.hp -= card_b.damage;
                if card_a.hp <= 0 {
                    ctx.say(format!("Player B's {} dealt {} damage and killed Player's A {}", card_b.id, card_b.damage, card_a.id)).await?;
                    player_a_cards.remove(player_a_index);
                } else {
                    ctx.say(format!("Player B's {} dealt {} damage to Player's A {}. It now has {} hp", card_b.id, card_b.damage, card_a.id, card_a.hp)).await?;
                }
            }
        }

        turn_count += 1;
        ctx.channel_id().broadcast_typing(&ctx.serenity_context().http).await?;
        let sleep_ms = rand::thread_rng().gen_range(250..750);
        tokio::time::sleep(tokio::time::Duration::from_millis(sleep_ms)).await;
    }

    if player_a_cards.is_empty() {
        ctx.say(format!("Player A lost in {} turns", turn_count)).await?;
    } else {
        ctx.say(format!("Player B lost in {} turns", turn_count)).await?;
    }

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

#[poise::command(slash_command, prefix_command)]
async fn delete(
    ctx: Context<'_>,
    #[description = "ID"]
    #[autocomplete = "autocomplete_card_id"]
    id: String,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    if let Ok(res) = sqlx::query!("DELETE FROM cards WHERE id=$1", id).execute(conn).await {
        if res.rows_affected() < 1 {
            ctx.say("Can't find card").await?;
            return Ok(())
        }
    }
    ctx.reply("Card was deleted").await?;
    Ok(())
}

async fn autocomplete_card_id<'a>(
    ctx: Context<'a>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let conn = &ctx.data().0;
    let match_str = format!("%{}%", partial);
    sqlx::query("SELECT id from cards WHERE id LIKE ?").bind(match_str).fetch(conn).map(|s| s.unwrap().try_get("id").unwrap())
}
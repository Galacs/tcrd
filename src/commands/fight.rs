use std::collections::HashMap;

use futures::{TryStreamExt, StreamExt};
use poise::serenity_prelude::User;
use rand::Rng;
use redis::AsyncCommands;
use sqlx::{Pool, Sqlite};
use crate::{cards::card::FightCard, Context, Error, commands::manage::{check_card, id_to_fight_card}};
use crate::commands::manage::autocomplete_card_id;
use serde::{Serialize, Deserialize};

#[poise::command(
    prefix_command,
    slash_command,
    subcommands("player", "accept")
)]
pub async fn fight(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct FightInfo {
    author_id: u64,
    author_name: String,
    author_channel_id: u64,
    author_guild_id: u64,
}

#[poise::command(slash_command, prefix_command)]
async fn player(
    ctx: Context<'_>,
    #[description = "Player to fight"] player: User,
    #[description = "Card 1"] #[autocomplete = "autocomplete_card_id"] card_1: String,
    #[description = "Card 2"] #[autocomplete = "autocomplete_card_id"] card_2: Option<String>,
    #[description = "Card 3"] #[autocomplete = "autocomplete_card_id"] card_3: Option<String>,
    #[description = "Card 4"] #[autocomplete = "autocomplete_card_id"] card_4: Option<String>,
    #[description = "Card 5"] #[autocomplete = "autocomplete_card_id"] card_5: Option<String>,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    // Check if cards are valid
    let card_1 = Some(card_1);
    for card in [&card_1, &card_2, &card_3, &card_4, &card_5] {
        let Some(id) = card else {
            continue;
        };
        if !check_card(conn, id).await? {
            ctx.say(format!("{} card doesn't exist", id)).await?;
            return Ok(())
        }
    }

    let a = vec![card_1, card_2, card_3, card_4, card_5];
    let player_cards_id_iter = a.iter().flatten();
    let mut player_cards: Vec<FightCard> = Vec::new();
    for id in player_cards_id_iter {
        player_cards.push(id_to_fight_card(conn, id).await?);
    }

    // Check if the player owns all the cards
    if !check_cards_ownership(&ctx, conn, player_cards.clone()).await? {
        return Ok(())
    }

    // Get redis connection
    let redis_client = &ctx.data().1;
    let mut redis = redis_client.get_async_connection().await?;
    let serialized = serde_json::to_string(&FightInfo {
        author_id: ctx.author().id.0,
        author_name: ctx.author().name.clone(),
        author_channel_id: ctx.channel_id().0,
        author_guild_id: ctx.guild_id().unwrap_or(poise::serenity_prelude::GuildId(0_u64)).into(),
    })?;

    redis.lpush(format!("user-fight-request-{}-{}", ctx.author().id.0, player.id.0), serialized).await?;
    let mut pubsub = redis.into_pubsub();
    pubsub.subscribe(format!("user-fight-response-{}-{}", ctx.author().id.0, player.id.0)).await?;
    ctx.say(format!("fight request sent to <@{}>", player.id)).await?;

    let msg: String = pubsub.on_message().next().await.unwrap().get_payload()?;

    let mut player_b_cards: Vec<FightCard> = serde_json::from_str(&msg)?;

    ctx.say(format!("<@{}> accepted the fight request, the match will be played in this channel", player.id)).await?;

    let player_a_won = fight_two_players(&ctx, ctx.author().id.0, &mut player_cards, player.id.0, &mut player_b_cards).await?;

    let user_a_id = ctx.author().id.0 as i64;
    let user_b_id = player.id.0 as i64;

    if player_a_won {
        sqlx::query!("UPDATE user_stats SET game_won = game_won + 1 WHERE user_id=$1", user_a_id).execute(conn).await?;
        sqlx::query!("UPDATE user_stats SET game_lost = game_lost + 1 WHERE user_id=$1", user_b_id).execute(conn).await?;
    } else {
        sqlx::query!("UPDATE user_stats SET game_won = game_won + 1 WHERE user_id=$1", user_b_id).execute(conn).await?;
        sqlx::query!("UPDATE user_stats SET game_lost = game_lost + 1 WHERE user_id=$1", user_a_id).execute(conn).await?;        
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn accept(
    ctx: Context<'_>,
    #[description = "Player that sent the request"] player: User,
    #[description = "Card 1"] #[autocomplete = "autocomplete_card_id"] card_1: String,
    #[description = "Card 2"] #[autocomplete = "autocomplete_card_id"] card_2: Option<String>,
    #[description = "Card 3"] #[autocomplete = "autocomplete_card_id"] card_3: Option<String>,
    #[description = "Card 4"] #[autocomplete = "autocomplete_card_id"] card_4: Option<String>,
    #[description = "Card 5"] #[autocomplete = "autocomplete_card_id"] card_5: Option<String>,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    // Check if cards are valid
    let card_1 = Some(card_1);
    for card in [&card_1, &card_2, &card_3, &card_4, &card_5] {
        let Some(id) = card else {
            continue;
        };
        if !check_card(conn, id).await? {
            ctx.say(format!("{} card doesn't exist", id)).await?;
            return Ok(())
        }
    }

    let a = vec![card_1, card_2, card_3, card_4, card_5];
    let player_cards_id_iter = a.iter().flatten();
    let mut player_cards: Vec<FightCard> = Vec::new();
    for id in player_cards_id_iter {
        player_cards.push(id_to_fight_card(conn, id).await?);
    }

    // Check if the player owns all the cards
    if !check_cards_ownership(&ctx, conn, player_cards.clone()).await? {
        return Ok(())
    }

    // Get redis connection
    let redis_client = &ctx.data().1;
    let mut redis = redis_client.get_async_connection().await?;

    let res: String = redis.rpop(format!("user-fight-request-{}-{}", player.id.0, ctx.author().id.0), None).await?;

    let fight: FightInfo = serde_json::from_str(&res)?;

    let serialized = serde_json::to_string(&player_cards)?;
    // Publish reponse
    redis.publish(format!("user-fight-response-{}-{}",  player.id.0, ctx.author().id.0), serialized).await?;

    ctx.say(format!("Fight request from <@{}> accepted, the fight will be played in <#{}>", fight.author_id, fight.author_channel_id)).await?;
    Ok(())
}

pub async fn check_cards_ownership(ctx: &Context<'_>, conn: &Pool<Sqlite>, cards: Vec<FightCard>) -> Result<bool, Error> {
    let author_id = ctx.author().id.0 as i64;
    let mut owned_cards_rows_iter = sqlx::query!("SELECT *,count(card_id) as count from users_cards WHERE user_id=$1 GROUP BY card_id", author_id).fetch(conn);

    let mut actual_map: HashMap<String, u32> = HashMap::new();
    while let Some(card) = owned_cards_rows_iter.try_next().await? {
        actual_map.insert(card.card_id, card.count as u32);
    }

    let mut request_map: HashMap<String, u32> = HashMap::new();
    for card in cards {
        *request_map.entry(card.id).or_default() += 1;
    }

    // Check if player posses enough cards
    for card in request_map {
        if !actual_map.contains_key(&card.0) {
            ctx.say(format!("You don't own {}", card.0)).await?;
            return Ok(false);
        }
        let Some(count) = actual_map.get(&card.0) else {
            return Ok(false);
        };
        if &card.1 > count {
            ctx.say(format!("You don't have enough {} cards, you need {} and you only got {}", card.0, card.1, count)).await?;
            return Ok(false);
        }
    }
    Ok(true)
}

// Returns true if the first player won
pub async fn fight_two_players(
    ctx: &Context<'_>,
    player_a_id: u64,
    player_a_cards: &mut Vec<FightCard>,
    player_b_id: u64,
    player_b_cards: & mut Vec<FightCard>
) -> Result<bool, Error> {
    // Which card attacks first ?
    let mut player_a_attacks = rand::thread_rng().gen_bool(0.5);

    let mut turn_count = 0;
    // Fight happens now
    while !(player_a_cards.is_empty() || player_b_cards.is_empty()) {
        // Choose which card of the two players will fight
        let player_a_index = rand::thread_rng().gen_range(0..player_a_cards.len());
        let player_b_index = rand::thread_rng().gen_range(0..player_b_cards.len());
        let card_a = &mut player_a_cards[player_a_index];
        let card_b = &mut player_b_cards[player_b_index];

        let player_b_dodge = rand::thread_rng().gen_bool((card_b.defense as f32/100.0).into());
        let player_a_dodge = rand::thread_rng().gen_bool((card_a.defense as f32/100.0).into());
        if player_a_attacks {
            // Did player B dodge ?
            if player_b_dodge {
                ctx.say(format!("<@{player_b_id}>'s {} dodged {}'s attack with a {}% chance", card_b.id, card_a.id, card_b.defense)).await?;
            } else {
                card_b.hp -= card_a.damage;
                if card_b.hp <= 0 {
                    ctx.say(format!("<@{player_a_id}>'s {} dealt {} damage and killed <@{player_b_id}>'s {}", card_a.id, card_a.damage, card_b.id)).await?;
                    player_b_cards.remove(player_b_index);
                } else {
                    ctx.say(format!("<@{player_a_id}>'s {} dealt {} damage to <@{player_b_id}>'s {}. It now has {} hp", card_a.id, card_a.damage, card_b.id, card_b.hp)).await?;
                }
            }
        } else {
            // Did player A dodge ?
            if player_a_dodge {
                ctx.say(format!("<@{player_a_id}>'s {} dodged {}'s attack with a {}% chance", card_a.id, card_b.id, card_a.defense)).await?;
            } else {
                card_a.hp -= card_b.damage;
                if card_a.hp <= 0 {
                    ctx.say(format!("<@{player_b_id}>'s {} dealt {} damage and killed <@{player_a_id}>'s {}", card_b.id, card_b.damage, card_a.id)).await?;
                    player_a_cards.remove(player_a_index);
                } else {
                    ctx.say(format!("<@{player_b_id}>'s {} dealt {} damage to <@{player_a_id}>'s {}. It now has {} hp", card_b.id, card_b.damage, card_a.id, card_a.hp)).await?;
                }
            }
        }

        player_a_attacks = !player_a_attacks;

        turn_count += 1;
        ctx.channel_id().broadcast_typing(&ctx.serenity_context().http).await?;
        let sleep_ms = rand::thread_rng().gen_range(250..750);
        tokio::time::sleep(tokio::time::Duration::from_millis(sleep_ms)).await;
    }

    if player_a_cards.is_empty() {
        ctx.say(format!("<@{player_a_id}> lost in {} turns", turn_count)).await?;
        Ok(false)
    } else {
        ctx.say(format!("<@{player_b_id}> lost in {} turns", turn_count)).await?;
        Ok(true)
    }
}
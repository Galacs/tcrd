use std::str::FromStr;

use futures::StreamExt;
use poise::serenity_prelude::User;
use poise::serenity_prelude as serenity;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

use crate::paginate_cards;
use crate::{Context, Error, commands::{manage::{autocomplete_user_card_id, check_card,}, fight::check_cards_ownership}, cards::card::{FightCard, Card, Rarity, Type}};

#[derive(Serialize, Deserialize, Debug)]
struct TradeInfo {
    author_id: u64,
    author_name: String,
    author_channel_id: u64,
    author_guild_id: u64,
    cards: Vec<Card>,
}

pub async fn id_to_card(conn: &Pool<Sqlite>, card_id: &String) -> Result<Card, Error> {
    let row = sqlx::query!("SELECT * FROM cards WHERE id=$1", card_id).fetch_one(conn).await?;
    Ok( Card {
        id: row.id,
        hp: row.hp as i32,
        defense: row.defense as i32,
        damage: row.damage as i32,
        extension: row.image_extension,
        rarity: Rarity::from_str(&row.rarity).unwrap(),
        kind: Type::from_str(&row.kind).unwrap(),
        description: row.description,
    })
}

#[poise::command(
    prefix_command,
    slash_command,
    subcommands("request", "accept")
)]
pub async fn trade(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Command to start card trade
#[poise::command(slash_command, prefix_command)]
pub async fn request(
    ctx: Context<'_>,
    #[description = "Player you want to trade with"] player: User,
    #[description = "Card 1"] #[autocomplete = "autocomplete_user_card_id"] card_1: String,
    #[description = "Card 2"] #[autocomplete = "autocomplete_user_card_id"] card_2: Option<String>,
    #[description = "Card 3"] #[autocomplete = "autocomplete_user_card_id"] card_3: Option<String>,
    #[description = "Card 4"] #[autocomplete = "autocomplete_user_card_id"] card_4: Option<String>,
    #[description = "Card 5"] #[autocomplete = "autocomplete_user_card_id"] card_5: Option<String>,
) -> Result<(), Error> {
    if &player == ctx.author() {
        ctx.say("You can't trade with yourself").await?;
        return Ok(());
    }
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
    let mut player_fight_cards: Vec<FightCard> = Vec::new();
    let mut player_cards: Vec<Card> = Vec::new();
    for id in player_cards_id_iter {
        player_fight_cards.push(id_to_card(conn, id).await?.into());
        player_cards.push(id_to_card(conn, id).await?);
    }

    // Check if the player owns all the cards
    if !check_cards_ownership(&ctx, conn, player_fight_cards.clone()).await? {
        return Ok(())
    }

    // Get redis connection
    let redis_client = &ctx.data().1;
    let mut redis = redis_client.get_async_connection().await?;
    let serialized = serde_json::to_string(&TradeInfo {
        author_id: ctx.author().id.0,
        author_name: ctx.author().name.clone(),
        author_channel_id: ctx.channel_id().0,
        author_guild_id: ctx.guild_id().unwrap_or(poise::serenity_prelude::GuildId(0_u64)).into(),
        cards: player_cards.clone()
    })?;

    redis.lpush(format!("user-trade-request-{}-{}", ctx.author().id.0, player.id.0), serialized).await?;
    let mut pubsub = redis.into_pubsub();
    pubsub.subscribe(format!("user-trade-response-{}-{}", ctx.author().id.0, player.id.0)).await?;
    ctx.say(format!("trade request sent to <@{}>", player.id)).await?;

    let msg: String = pubsub.on_message().next().await.unwrap().get_payload()?;

    let player_b_cards: Vec<Card> = serde_json::from_str(&msg)?;

    ctx.say(format!("<@{}> accepted the trade request with the following cards", player.id)).await?;

    // Player B cards
    paginate_cards::paginate(ctx, player_b_cards, None).await?;

    println!("salut");

    ctx.send(|m| {
        m.content(format!("Do you want to accept a trade request from <@{}>", player.id)).components(|c| {
            c.create_action_row(|ar| {
                ar.create_button(|b| {
                    b.style(serenity::ButtonStyle::Primary)
                        .label("Yes")
                        .custom_id("yes")
                })
            })
        })
    })
    .await?;

    while let Some(mci) = serenity::CollectComponentInteraction::new(ctx)
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(120))
        .filter(move |mci| mci.data.custom_id == "yes")
        .await
    {
        mci.create_interaction_response(ctx, |ir| {
            ir.kind(serenity::InteractionResponseType::DeferredUpdateMessage)
        })
        .await?;
    }


    // Trade logic goes here

    // let player_a_won = fight_two_players(&ctx, ctx.author().id.0, &mut player_cards, player.id.0, &mut player_b_cards).await?;

    // let user_a_id = ctx.author().id.0 as i64;
    // let user_b_id = player.id.0 as i64;
    Ok(())
}


/// Accepts a trade request from another player
#[poise::command(slash_command, prefix_command)]
async fn accept(
    ctx: Context<'_>,
    #[description = "Player that sent the request"] player: User,
    #[description = "Card 1"] #[autocomplete = "autocomplete_user_card_id"] card_1: String,
    #[description = "Card 2"] #[autocomplete = "autocomplete_user_card_id"] card_2: Option<String>,
    #[description = "Card 3"] #[autocomplete = "autocomplete_user_card_id"] card_3: Option<String>,
    #[description = "Card 4"] #[autocomplete = "autocomplete_user_card_id"] card_4: Option<String>,
    #[description = "Card 5"] #[autocomplete = "autocomplete_user_card_id"] card_5: Option<String>,
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
    let mut player_fight_cards: Vec<FightCard> = Vec::new();
    let mut player_cards: Vec<Card> = Vec::new();
    for id in player_cards_id_iter {
        player_fight_cards.push(id_to_card(conn, id).await?.into());
        player_cards.push(id_to_card(conn, id).await?);
    }

    // Check if the player owns all the cards
    if !check_cards_ownership(&ctx, conn, player_fight_cards).await? {
        return Ok(())
    }

    // Get redis connection
    let redis_client = &ctx.data().1;
    let mut redis = redis_client.get_async_connection().await?;

    let res: String = redis.rpop(format!("user-trade-request-{}-{}", player.id.0, ctx.author().id.0), None).await?;

    let trade: TradeInfo = serde_json::from_str(&res)?;

    let serialized = serde_json::to_string(&player_cards)?;
    // Publish reponse
    redis.publish(format!("user-trade-response-{}-{}",  player.id.0, ctx.author().id.0), serialized).await?;

    ctx.say(format!("trade request from <@{}> accepted", trade.author_id)).await?;
    Ok(())
}
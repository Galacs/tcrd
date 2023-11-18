use std::str::FromStr;

use poise::serenity_prelude::{self as serenity, CreateEmbed};
use sqlx::{Pool, Sqlite, SqlitePool};

use crate::cards::card::{Rarity, Card};

mod cards;
mod paginate_cards;

pub struct Data(Pool<Sqlite>);
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(
    prefix_command,
    slash_command,
    subcommands("create", "get", "list", "delete"),
    subcommand_required
)]
pub async fn cards(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn create(
    ctx: Context<'_>,
    #[description = "ID"] id: String,
    #[description = "Rarity"] rarity: Rarity,
    #[description = "Description"] description: String,
    #[description = "HP"] hp: i32,
    #[description = "Damage"] damage: i32,
    #[description = "Defense"] defense: i32,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    if (sqlx::query!("INSERT INTO cards(id, rarity, description, hp, damage, defense) VALUES ($1, $2, $3, $4, $5, $6)", id, rarity, description, hp, damage, defense).execute(conn).await).is_err() {
        ctx.say("A similar card already exists").await?;
        return Ok(());
    }
    ctx.say("the card was created").await?;
    Ok(())
}

pub fn create_card_embed(e: &mut CreateEmbed, card: Card) -> &mut CreateEmbed {
    e.title(card.id.clone())
    .field("", &format!(
        "**ID:** {}
        **Rarity:** {}
        **HP:** {}
        **Damage:** {}
        **Defense:** {}",
        card.id, card.rarity, card.hp, card.damage, card.defense
    ), false)
    .field("**Description**", card.description, false)
}

#[poise::command(slash_command, prefix_command)]
async fn list(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    let Ok(rows) = sqlx::query!("SELECT * FROM cards").fetch_all(conn).await else {
        ctx.say("Can't find any cards").await?;
        return Ok(());
    };
    // ctx.say("the card was created").await?;
    let cards = rows.iter().map(|row| {
        Card {
            id: row.id.clone(),
            rarity: Rarity::from_str(&row.rarity).unwrap(),
            description: row.description.clone(),
            hp: row.hp as i32,
            damage: row.damage as i32,
            defense: row.defense as i32,
        }
    }).collect();
    paginate_cards::paginate(ctx, cards).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn get(
    ctx: Context<'_>,
    #[description = "ID"] id: String,
) -> Result<(), Error> {
    let conn = &ctx.data().0;
    let Ok(row) = sqlx::query!("SELECT * FROM cards WHERE id=$1", id).fetch_one(conn).await else {
        ctx.say("Can't find card").await?;
        return Ok(());
    };
    let card = Card {
        id: row.id,
        rarity: Rarity::from_str(&row.rarity).unwrap(),
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

#[poise::command(slash_command, prefix_command)]
async fn delete(
    ctx: Context<'_>,
    #[description = "ID"] id: String,
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

#[tokio::main]
async fn main() -> Result<(), Error> {
    // DB
    let database_url = "sqlite://tcrd.db";
    let conn = SqlitePool::connect(database_url).await?;
    sqlx::migrate!().run(&conn).await?;


    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![cards()],
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                if let Ok(var) = std::env::var("GUILD_ID") {
                    poise::builtins::register_in_guild(ctx, &framework.options().commands, serenity::GuildId(var.parse().expect("GUILD_ID should be an integer"))).await?;
                }
                else {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                }
                Ok(Data(conn))
            })
        });

    framework.run().await?;
    Ok(())
}
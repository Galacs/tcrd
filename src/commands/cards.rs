use std::str::FromStr;

use crate::{cards::card::{Rarity, Card}, Context, Error, create_card_embed, paginate_cards};


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
    let card = Card { id, rarity, description, hp, damage, defense };
    ctx.send(|b| b.embed(|e| create_card_embed(e, card))).await?;
    Ok(())
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
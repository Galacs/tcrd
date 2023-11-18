use crate::{Context, Error};
use rand::Rng;

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


    if (pack.awakened_chance..=pack.mythic_chance).contains(&number) {
        ctx.say("Congratulations you won an Awakened card").await?;
    } else if (pack.mythic_chance..=pack.legendary_chance).contains(&number) {
        ctx.say("Congratulations you won a Mythic card").await?;
    }
    else if (pack.legendary_chance..=pack.epic_chance).contains(&number) {
        ctx.say("Congratulations you won an Legendary card").await?;
    }
    else if (pack.epic_chance..=pack.rare_chance).contains(&number) {
        ctx.say("Congratulations you won a Epic card").await?;
    }
    else if (pack.rare_chance..=pack.common_chance).contains(&number) {
        ctx.say("Congratulations you won a Rare card").await?;
    } else {
        ctx.say("Congratulations you won a Common card").await?;
    }

    Ok(())
}

